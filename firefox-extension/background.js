// PassQ Firefox Extension Background Script

class PassQBackground {
  constructor() {
    this.apiUrl = null;
    this.authToken = null;
    this.crypto = new PassQCrypto();
    this.init();
  }

  async init() {
    await this.loadConfig();
    this.setupMessageListeners();
  }

  async loadConfig() {
    try {
      const result = await browser.storage.local.get(['passqServerUrl']);
      this.apiUrl = result.passqServerUrl;
      this.authToken = await this.crypto.retrieveToken();
    } catch (error) {
      console.error('Error loading config:', error);
    }
  }

  setupMessageListeners() {
    browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
      this.handleMessage(message, sender, sendResponse);
      return true; // Keep the message channel open for async response
    });

    // Listen for storage changes to update config
    browser.storage.onChanged.addListener(async (changes, namespace) => {
      if (namespace === 'local') {
        if (changes.passqServerUrl) {
          this.apiUrl = changes.passqServerUrl.newValue;
        }
        if (changes.encryptedAuthToken) {
          this.authToken = await this.crypto.retrieveToken();
        }
      }
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
      
      await browser.tabs.sendMessage(tabId, {
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
}

// Initialize background script
new PassQBackground();