// PassQ Chrome Extension Background Script (Service Worker)

// Import the crypto module using importScripts
importScripts('crypto.js');
importScripts('offline-cache.js');
importScripts('sync-manager.js');

try {
  console.log('PassQCrypto successfully loaded:', typeof PassQCrypto);

  class PassQBackground {
    constructor() {
      this.apiUrl = null;
      this.authToken = null;
      this.crypto = new PassQCrypto();
      this.syncManager = null;
    }

    async init() {
      await this.loadConfig();
      this.setupMessageListeners();
      await this.initializeAutoLock();
      await this.initializeSyncManager();
    }

    async initializeAutoLock() {
      try {
        const result = await chrome.storage.local.get('extensionSettings');
        const settings = result.extensionSettings;
        
        if (settings && settings.autoLockTimeout > 0) {
          this.setupAutoLockTimer(settings.autoLockTimeout);
        }
      } catch (error) {
        console.error('Error initializing auto-lock:', error);
      }
    }

    async initializeSyncManager() {
      try {
        this.syncManager = new PassQSyncManager();
        await this.syncManager.init();
        console.log('Sync manager initialized successfully');
      } catch (error) {
        console.warn('Sync manager initialization failed, offline functionality disabled:', error.message);
        // Extension continues to work without offline sync
        this.syncManager = null;
      }
    }

  async loadConfig() {
    try {
      const result = await chrome.storage.local.get(['passqServerUrl']);
      this.apiUrl = result.passqServerUrl;
      this.authToken = await this.crypto.retrieveToken();
    } catch (error) {
      console.error('Error loading config:', error);
    }
  }

  setupMessageListeners() {
    chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
      // Reset auto-lock timer on any user activity
      this.resetAutoLockTimer();
      
      this.handleMessage(message, sender, sendResponse);
      return true; // Keep the message channel open for async response
    });

    // Listen for storage changes to update config
    chrome.storage.onChanged.addListener(async (changes, namespace) => {
      if (namespace === 'local') {
        if (changes.passqServerUrl) {
          this.apiUrl = changes.passqServerUrl.newValue;
        }
        if (changes.encryptedAuthToken) {
          this.authToken = await this.crypto.retrieveToken();
        }
        if (changes.extensionSettings) {
          // Update auto-lock timer when settings change
          const settings = changes.extensionSettings.newValue;
          if (settings && settings.autoLockTimeout > 0) {
            this.setupAutoLockTimer(settings.autoLockTimeout);
          } else {
            this.clearAutoLockTimer();
          }
        }
      }
    });

    // Listen for tab activation to reset auto-lock timer
    chrome.tabs.onActivated.addListener(() => {
      this.resetAutoLockTimer();
    });

    // Listen for window focus changes
    chrome.windows.onFocusChanged.addListener(() => {
      this.resetAutoLockTimer();
    });
  }

  async handleMessage(message, sender, sendResponse) {
    try {
      switch (message.action) {
        case 'checkLoginStatus':
          await this.handleCheckLoginStatus(sendResponse);
          break;
        case 'login':
          await this.handleLogin(message, sendResponse);
          break;
        case 'logout':
          await this.handleLogout(sendResponse);
          break;
        case 'getPasswords':
          await this.handleGetPasswords(sendResponse);
          break;
        case 'findCredentials':
          await this.handleFindCredentials(message, sendResponse);
          break;
        case 'autofillCredentials':
          await this.handleAutofillCredentials(message, sender, sendResponse);
          break;
        case 'SETTINGS_UPDATED':
          await this.handleSettingsUpdated(message, sendResponse);
          break;
        case 'CLEAR_OFFLINE_CACHE':
          await this.handleClearOfflineCache(sendResponse);
          break;
        case 'SETTINGS_REQUEST':
          await this.handleSettingsRequest(sendResponse);
          break;
        case 'SYNC_NOW':
          await this.handleSyncNow(sendResponse);
          break;
        case 'SYNC_STATUS':
          await this.handleSyncStatus(sendResponse);
          break;
        case 'FORCE_SYNC_CREDENTIAL':
          await this.handleForceSyncCredential(message, sendResponse);
          break;
        default:
          sendResponse({ success: false, error: 'Unknown action' });
      }
    } catch (error) {
      console.error('Error handling message:', error);
      sendResponse({ success: false, error: error.message });
    }
  }

  async handleCheckLoginStatus(sendResponse) {
    try {
      await this.loadConfig();
      const isLoggedIn = !!(this.authToken && this.apiUrl);
      sendResponse({ success: true, isLoggedIn });
    } catch (error) {
      sendResponse({ success: false, error: error.message });
    }
  }

  async handleLogin(message, sendResponse) {
    try {
      if (!this.apiUrl) {
        sendResponse({ success: false, error: 'Server URL not configured' });
        return;
      }

      const response = await fetch(`${this.apiUrl}/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          username: message.username,
          password: message.password
        })
      });

      if (response.ok) {
        const data = await response.json();
        this.authToken = data.data; // Backend returns token in 'data' field
        await this.crypto.storeToken(data.data);
        sendResponse({ success: true, token: data.data });
      } else {
        const error = await response.json();
        sendResponse({ success: false, error: error.message || 'Login failed' });
      }
    } catch (error) {
      console.error('Login error:', error);
      sendResponse({ success: false, error: 'Connection failed' });
    }
  }

  async handleLogout(sendResponse) {
    try {
      this.authToken = null;
      await this.crypto.removeToken();
      await this.crypto.clearCryptoData();
      sendResponse({ success: true });
    } catch (error) {
      sendResponse({ success: false, error: error.message });
    }
  }

  async handleGetPasswords(sendResponse) {
    try {
      if (!this.authToken || !this.apiUrl) {
        sendResponse({ success: false, error: 'Not authenticated or server not configured' });
        return;
      }

      const response = await fetch(`${this.apiUrl}/passwords`, {
        headers: {
          'Authorization': `Bearer ${this.authToken}`,
          'Content-Type': 'application/json'
        }
      });

      if (response.ok) {
        const result = await response.json();
        sendResponse({ success: true, passwords: result.data }); // Backend returns passwords in 'data' field
      } else if (response.status === 401) {
        // Token expired, clear it
        this.authToken = null;
        await this.crypto.removeToken();
        sendResponse({ success: false, error: 'Authentication expired' });
      } else {
        sendResponse({ success: false, error: 'Failed to fetch passwords' });
      }
    } catch (error) {
      console.error('Get passwords error:', error);
      sendResponse({ success: false, error: 'Connection failed' });
    }
  }

  async handleFindCredentials(message, sendResponse) {
    try {
      if (!this.authToken || !this.apiUrl) {
        sendResponse({ success: false, error: 'Not authenticated or server not configured' });
        return;
      }

      const response = await fetch(`${this.apiUrl}/passwords`, {
        headers: {
          'Authorization': `Bearer ${this.authToken}`,
          'Content-Type': 'application/json'
        }
      });

      if (response.ok) {
        const result = await response.json();
        const passwords = result.data; // Backend returns passwords in 'data' field
        const domain = message.domain;
        
        const matchingCredentials = passwords.filter(cred => {
          const credDomain = this.extractDomain(cred.website || '');
          return credDomain.includes(domain) || domain.includes(credDomain);
        });

        sendResponse({ success: true, credentials: matchingCredentials });
      } else {
        sendResponse({ success: false, error: 'Failed to fetch credentials' });
      }
    } catch (error) {
      console.error('Find credentials error:', error);
      sendResponse({ success: false, error: 'Connection failed' });
    }
  }

  async handleAutofillCredentials(message, sender, sendResponse) {
    try {
      const tabId = sender.tab.id;
      
      await chrome.tabs.sendMessage(tabId, {
        action: 'autofill',
        credential: message.credential
      });
      
      sendResponse({ success: true });
    } catch (error) {
      console.error('Autofill error:', error);
      sendResponse({ success: false, error: 'Autofill failed' });
    }
  }

  extractDomain(website) {
    try {
      const url = new URL(website.startsWith('http') ? website : `https://${website}`);
      return url.hostname;
    } catch {
      return website;
    }
  }

  async handleSettingsUpdated(message, sendResponse) {
    try {
      // Store the updated settings
      await chrome.storage.local.set({ extensionSettings: message.settings });
      
      // Initialize auto-lock timer if enabled
      if (message.settings.autoLockTimeout > 0) {
        this.setupAutoLockTimer(message.settings.autoLockTimeout);
      } else {
        this.clearAutoLockTimer();
      }
      
      sendResponse({ success: true });
    } catch (error) {
      console.error('Settings update error:', error);
      sendResponse({ success: false, error: 'Failed to update settings' });
    }
  }

  async handleClearOfflineCache(sendResponse) {
    try {
      // Clear IndexedDB cache (placeholder for future implementation)
      // This would clear the offline credential cache
      console.log('Clearing offline cache...');
      
      // For now, just clear any cached data in chrome.storage
      await chrome.storage.local.remove(['offlineCredentials', 'lastSync']);
      
      sendResponse({ success: true });
    } catch (error) {
      console.error('Clear cache error:', error);
      sendResponse({ success: false, error: 'Failed to clear cache' });
    }
  }

  async handleSettingsRequest(sendResponse) {
    try {
      const result = await chrome.storage.local.get('extensionSettings');
      sendResponse({ success: true, settings: result.extensionSettings || {} });
    } catch (error) {
      console.error('Settings request error:', error);
      sendResponse({ success: false, error: 'Failed to get settings' });
    }
  }

  async handleSyncNow(sendResponse) {
    try {
      if (!this.syncManager) {
        sendResponse({ success: false, error: 'Sync manager not initialized' });
        return;
      }
      
      const result = await this.syncManager.performFullSync();
      sendResponse(result);
    } catch (error) {
      console.error('Sync now error:', error);
      sendResponse({ success: false, error: error.message });
    }
  }

  async handleSyncStatus(sendResponse) {
    try {
      if (!this.syncManager) {
        sendResponse({ success: false, error: 'Sync manager not initialized' });
        return;
      }
      
      const status = await this.syncManager.getSyncStatus();
      sendResponse({ success: true, status });
    } catch (error) {
      console.error('Sync status error:', error);
      sendResponse({ success: false, error: error.message });
    }
  }

  async handleForceSyncCredential(message, sendResponse) {
    try {
      if (!this.syncManager) {
        sendResponse({ success: false, error: 'Sync manager not initialized' });
        return;
      }
      
      const { credentialId, operation, data } = message;
      const result = await this.syncManager.forceSyncCredential(credentialId, operation, data);
      sendResponse(result);
    } catch (error) {
      console.error('Force sync credential error:', error);
      sendResponse({ success: false, error: error.message });
    }
  }

  setupAutoLockTimer(timeout) {
    // Clear existing timer
    this.clearAutoLockTimer();
    
    // Set new timer
    this.autoLockTimer = setTimeout(async () => {
      try {
        // Auto-lock by clearing the auth token
        await this.crypto.removeToken();
        await this.crypto.clearCryptoData();
        this.authToken = null;
        
        // Notify all tabs about the lock
        const tabs = await chrome.tabs.query({});
        tabs.forEach(tab => {
          chrome.tabs.sendMessage(tab.id, { action: 'sessionLocked' }).catch(() => {
            // Ignore errors for tabs that don't have content script
          });
        });
        
        console.log('Extension auto-locked due to inactivity');
      } catch (error) {
        console.error('Auto-lock error:', error);
      }
    }, timeout);
  }

  clearAutoLockTimer() {
    if (this.autoLockTimer) {
      clearTimeout(this.autoLockTimer);
      this.autoLockTimer = null;
    }
  }

  resetAutoLockTimer() {
    // Reset the timer when user activity is detected
    chrome.storage.local.get('extensionSettings').then(result => {
      const settings = result.extensionSettings;
      if (settings && settings.autoLockTimeout > 0) {
        this.setupAutoLockTimer(settings.autoLockTimeout);
      }
    });
  }
}

// Initialize background script
const background = new PassQBackground();
background.init();

} catch (error) {
  console.error('Service worker error:', error);
}