// Settings Page JavaScript
class ExtensionSettings {
  constructor() {
    this.defaultSettings = {
      autoLockTimeout: 900000, // 15 minutes
      lockOnBrowserRestart: true,
      lockOnSystemSleep: true,
      enableBiometric: false,
      biometricTimeout: 900000, // 15 minutes
      enableOfflineCache: false,

      enableDebugMode: false
    };
    
    this.currentSettings = { ...this.defaultSettings };
    this.isLoading = false;
    this.saveTimeout = null;
    
    this.init();
  }

  async init() {
    await this.loadSettings();
    this.setupEventListeners();
    this.updateUI();
    this.checkBiometricSupport();
    await this.updateCacheStats();
  }

  async loadSettings() {
    try {
      const result = await chrome.storage.local.get('extensionSettings');
      if (result.extensionSettings) {
        this.currentSettings = { ...this.defaultSettings, ...result.extensionSettings };
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
      this.showStatus('Failed to load settings', 'error');
    }
  }

  async saveSettings() {
    if (this.isLoading) return;
    
    this.setLoading(true);
    
    try {
      console.log('Saving settings:', this.currentSettings);
      await chrome.storage.local.set({ extensionSettings: this.currentSettings });
      console.log('Settings saved to storage successfully');
      this.showStatus('Settings saved successfully', 'success');
      
      // Notify background script of settings change
      chrome.runtime.sendMessage({
        type: 'SETTINGS_UPDATED',
        settings: this.currentSettings
      });
      
    } catch (error) {
      console.error('Failed to save settings:', error);
      this.showStatus('Failed to save settings', 'error');
    } finally {
      this.setLoading(false);
    }
  }

  setupEventListeners() {
    // Auto-lock timeout
    const autoLockSelect = document.getElementById('autoLockTimeout');
    autoLockSelect.addEventListener('change', (e) => {
      this.currentSettings.autoLockTimeout = parseInt(e.target.value);
      this.debouncedSave();
    });

    // Lock on browser restart
    const lockOnRestartCheckbox = document.getElementById('lockOnBrowserRestart');
    lockOnRestartCheckbox.addEventListener('change', (e) => {
      this.currentSettings.lockOnBrowserRestart = e.target.checked;
      this.debouncedSave();
    });

    // Lock on system sleep
    const lockOnSleepCheckbox = document.getElementById('lockOnSystemSleep');
    lockOnSleepCheckbox.addEventListener('change', (e) => {
      this.currentSettings.lockOnSystemSleep = e.target.checked;
      this.debouncedSave();
    });

    // Biometric authentication
    const biometricCheckbox = document.getElementById('enableBiometric');
    biometricCheckbox.addEventListener('change', (e) => {
      this.currentSettings.enableBiometric = e.target.checked;
      this.updateBiometricUI();
      this.debouncedSave();
    });

    // Biometric timeout
    const biometricTimeoutSelect = document.getElementById('biometricTimeout');
    biometricTimeoutSelect.addEventListener('change', (e) => {
      this.currentSettings.biometricTimeout = parseInt(e.target.value);
      this.debouncedSave();
    });

    // Setup biometric button
    const setupBiometricBtn = document.getElementById('setupBiometric');
    setupBiometricBtn.addEventListener('click', () => {
      this.setupBiometric();
    });

    // Offline cache
    const offlineCacheCheckbox = document.getElementById('enableOfflineCache');
    offlineCacheCheckbox.addEventListener('change', (e) => {
      this.currentSettings.enableOfflineCache = e.target.checked;
      this.debouncedSave();
    });

    // Clear cache button
    const clearCacheBtn = document.getElementById('clearCache');
    clearCacheBtn.addEventListener('click', () => {
      this.clearOfflineCache();
    });



    // Debug mode
    const debugModeCheckbox = document.getElementById('enableDebugMode');
    debugModeCheckbox.addEventListener('change', (e) => {
      this.currentSettings.enableDebugMode = e.target.checked;
      this.debouncedSave();
    });

    // Reset settings button
    const resetBtn = document.getElementById('resetSettings');
    resetBtn.addEventListener('click', () => {
      this.showConfirmModal(
        'Reset Settings',
        'Are you sure you want to reset all settings to their default values? This action cannot be undone.',
        () => this.resetSettings()
      );
    });

    // Save settings button
    const saveBtn = document.getElementById('saveSettings');
    saveBtn.addEventListener('click', () => {
      this.saveSettings();
    });

    // Modal event listeners
    this.setupModalListeners();
  }

  setupModalListeners() {
    const modal = document.getElementById('confirmModal');
    const cancelBtn = document.getElementById('confirmCancel');
    const okBtn = document.getElementById('confirmOk');

    cancelBtn.addEventListener('click', () => {
      this.hideModal();
    });

    okBtn.addEventListener('click', () => {
      if (this.confirmCallback) {
        this.confirmCallback();
      }
      this.hideModal();
    });

    // Close modal on background click
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        this.hideModal();
      }
    });

    // Close modal on Escape key
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && modal.style.display !== 'none') {
        this.hideModal();
      }
    });
  }

  updateUI() {
    // Update form values
    document.getElementById('autoLockTimeout').value = this.currentSettings.autoLockTimeout;
    document.getElementById('lockOnBrowserRestart').checked = this.currentSettings.lockOnBrowserRestart;
    document.getElementById('lockOnSystemSleep').checked = this.currentSettings.lockOnSystemSleep;
    document.getElementById('enableBiometric').checked = this.currentSettings.enableBiometric;
    document.getElementById('biometricTimeout').value = this.currentSettings.biometricTimeout;
    document.getElementById('enableOfflineCache').checked = this.currentSettings.enableOfflineCache;

    document.getElementById('enableDebugMode').checked = this.currentSettings.enableDebugMode;

    this.updateBiometricUI();
  }

  updateBiometricUI() {
    const biometricTimeoutSetting = document.getElementById('biometricTimeoutSetting');
    const setupBiometricSetting = document.getElementById('setupBiometricSetting');
    const enableBiometric = this.currentSettings.enableBiometric;

    if (enableBiometric) {
      biometricTimeoutSetting.style.display = 'flex';
      setupBiometricSetting.style.display = 'flex';
    } else {
      biometricTimeoutSetting.style.display = 'none';
      setupBiometricSetting.style.display = 'none';
    }
  }

  async checkBiometricSupport() {
    try {
      // Check if WebAuthn is supported
      if (!window.PublicKeyCredential) {
        const biometricCheckbox = document.getElementById('enableBiometric');
        const biometricSection = biometricCheckbox.closest('.settings-section');
        
        // Disable biometric options if not supported
        biometricCheckbox.disabled = true;
        biometricSection.style.opacity = '0.5';
        
        const description = biometricSection.querySelector('.section-description');
        description.textContent = 'Biometric authentication is not supported in this browser.';
      }
    } catch (error) {
      console.error('Error checking biometric support:', error);
    }
  }

  async setupBiometric() {
    try {
      this.setLoading(true);
      
      // Check if WebAuthn is available
      if (!window.PublicKeyCredential) {
        throw new Error('WebAuthn is not supported in this browser');
      }

      // Create credential options
      const credentialOptions = {
        publicKey: {
          challenge: new Uint8Array(32),
          rp: {
            name: 'PassQ Extension',
            id: chrome.runtime.id
          },
          user: {
            id: new TextEncoder().encode('passq-user'),
            name: 'PassQ User',
            displayName: 'PassQ User'
          },
          pubKeyCredParams: [{
            type: 'public-key',
            alg: -7 // ES256
          }],
          authenticatorSelection: {
            authenticatorAttachment: 'platform',
            userVerification: 'required'
          },
          timeout: 60000,
          attestation: 'direct'
        }
      };

      // Generate random challenge
      crypto.getRandomValues(credentialOptions.publicKey.challenge);

      // Create credential
      const credential = await navigator.credentials.create(credentialOptions);
      
      if (credential) {
        // Store credential info
        await chrome.storage.local.set({
          biometricCredential: {
            id: credential.id,
            rawId: Array.from(new Uint8Array(credential.rawId)),
            type: credential.type
          }
        });
        
        this.showStatus('Biometric authentication setup successfully', 'success');
      }
    } catch (error) {
      console.error('Biometric setup failed:', error);
      this.showStatus('Biometric setup failed: ' + error.message, 'error');
    } finally {
      this.setLoading(false);
    }
  }

  async clearOfflineCache() {
    this.showConfirmModal(
      'Clear Offline Cache',
      'Are you sure you want to clear all cached data? You will need to be online to access your passwords after this.',
      async () => {
        try {
          this.setLoading(true);
          
          // Clear IndexedDB cache using the new offline cache manager
          const offlineCache = new PassQOfflineCache();
          await offlineCache.init();
          await offlineCache.clearAllData();
          
          // Update cache stats display
          await this.updateCacheStats();
          
          this.showStatus('Offline cache cleared successfully', 'success');
        } catch (error) {
          console.error('Failed to clear cache:', error);
          this.showStatus('Failed to clear cache', 'error');
        } finally {
          this.setLoading(false);
        }
      }
    );
  }

  async updateCacheStats() {
    try {
      const offlineCache = new PassQOfflineCache();
      await offlineCache.init();
      const stats = await offlineCache.getCacheStats();
      
      const cacheStatsElement = document.getElementById('cacheStats');
      if (cacheStatsElement) {
        const lastUpdateText = stats.lastUpdate 
          ? `Last updated: ${stats.lastUpdate.toLocaleString()}`
          : 'Never cached';
        
        cacheStatsElement.innerHTML = `
          <div class="cache-stat-item">
            <span class="cache-stat-label">Cached Passwords:</span>
            <span class="cache-stat-value">${stats.credentialCount}</span>
          </div>
          <div class="cache-stat-item">
            <span class="cache-stat-label">${lastUpdateText}</span>
          </div>
          <div class="cache-stat-item">
            <span class="cache-stat-label">Pending Sync:</span>
            <span class="cache-stat-value">${stats.pendingSyncCount}</span>
          </div>
          <div class="cache-stat-item">
            <span class="cache-stat-label">Status:</span>
            <span class="cache-stat-value ${stats.isOffline ? 'offline' : 'online'}">
              ${stats.isOffline ? '📱 Offline' : '🌐 Online'}
            </span>
          </div>
        `;
      }
    } catch (error) {
      console.error('Failed to update cache stats:', error);
    }
  }

  resetSettings() {
    this.currentSettings = { ...this.defaultSettings };
    this.updateUI();
    this.saveSettings();
  }

  debouncedSave() {
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout);
    }
    
    this.saveTimeout = setTimeout(() => {
      this.saveSettings();
    }, 1000);
  }

  showConfirmModal(title, message, callback) {
    const modal = document.getElementById('confirmModal');
    const titleEl = document.getElementById('confirmTitle');
    const messageEl = document.getElementById('confirmMessage');
    
    titleEl.textContent = title;
    messageEl.textContent = message;
    this.confirmCallback = callback;
    
    modal.style.display = 'flex';
  }

  hideModal() {
    const modal = document.getElementById('confirmModal');
    modal.style.display = 'none';
    this.confirmCallback = null;
  }

  showStatus(message, type = 'info') {
    const statusEl = document.getElementById('saveStatus');
    statusEl.textContent = message;
    statusEl.className = `save-status ${type}`;
    
    // Clear status after 3 seconds
    setTimeout(() => {
      statusEl.textContent = '';
      statusEl.className = 'save-status';
    }, 3000);
  }

  setLoading(loading) {
    this.isLoading = loading;
    const container = document.querySelector('.settings-container');
    
    if (loading) {
      container.classList.add('loading');
    } else {
      container.classList.remove('loading');
    }
  }
}

// Initialize settings when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  new ExtensionSettings();
});

// Handle messages from background script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'SETTINGS_REQUEST') {
    // Send current settings to requesting script
    sendResponse({ settings: window.extensionSettings?.currentSettings });
  }
});

// Export for potential use by other scripts
window.ExtensionSettings = ExtensionSettings;