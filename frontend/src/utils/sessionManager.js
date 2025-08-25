class SessionManager {
  constructor() {
    this.timeout = null;
    this.warningTimeout = null;
    this.isLocked = false;
    this.onLockCallback = null;
    this.onWarningCallback = null;
    this.activityEvents = ['mousedown', 'mousemove', 'keypress', 'scroll', 'touchstart', 'click'];
    
    // Load settings
    this.loadSettings();
    
    // Listen for settings changes
    window.addEventListener('autoLockSettingsChanged', (event) => {
      this.loadSettings();
      this.resetTimer();
    });
    
    // Listen for page visibility changes
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) {
        this.pauseTimer();
      } else {
        this.resumeTimer();
      }
    });
    
    // Listen for beforeunload to handle page refresh
    window.addEventListener('beforeunload', () => {
      if (this.lockOnRefresh) {
        this.lockSession();
      }
    });
    
    this.bindActivityListeners();
  }
  
  loadSettings() {
    this.autoLockTimeout = localStorage.getItem('autoLockTimeout') || '15';
    this.lockOnRefresh = localStorage.getItem('lockOnRefresh') !== 'false';
  }
  
  bindActivityListeners() {
    this.activityEvents.forEach(event => {
      document.addEventListener(event, this.handleActivity.bind(this), true);
    });
  }
  
  unbindActivityListeners() {
    this.activityEvents.forEach(event => {
      document.removeEventListener(event, this.handleActivity.bind(this), true);
    });
  }
  
  handleActivity() {
    if (!this.isLocked && this.autoLockTimeout !== 'never') {
      this.resetTimer();
    }
  }
  
  resetTimer() {
    this.clearTimers();
    
    if (this.autoLockTimeout === 'never') {
      return;
    }
    
    const timeoutMs = parseInt(this.autoLockTimeout) * 60 * 1000; // Convert minutes to milliseconds
    const warningMs = timeoutMs - 60000; // Show warning 1 minute before lock
    
    // Set warning timer (1 minute before lock)
    if (warningMs > 0) {
      this.warningTimeout = setTimeout(() => {
        if (this.onWarningCallback) {
          this.onWarningCallback();
        }
      }, warningMs);
    }
    
    // Set lock timer
    this.timeout = setTimeout(() => {
      this.lockSession();
    }, timeoutMs);
  }
  
  clearTimers() {
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
    if (this.warningTimeout) {
      clearTimeout(this.warningTimeout);
      this.warningTimeout = null;
    }
  }
  
  pauseTimer() {
    // Store remaining time when pausing
    if (this.timeout) {
      this.pausedAt = Date.now();
    }
  }
  
  resumeTimer() {
    // Resume timer with remaining time
    if (this.pausedAt && this.timeout) {
      const elapsed = Date.now() - this.pausedAt;
      const timeoutMs = parseInt(this.autoLockTimeout) * 60 * 1000;
      const remaining = timeoutMs - elapsed;
      
      if (remaining > 0) {
        this.clearTimers();
        this.timeout = setTimeout(() => {
          this.lockSession();
        }, remaining);
      } else {
        this.lockSession();
      }
    }
    this.pausedAt = null;
  }
  
  lockSession() {
    if (this.isLocked) return;
    
    this.isLocked = true;
    this.clearTimers();
    
    // Clear sensitive data from memory
    this.clearSensitiveData();
    
    if (this.onLockCallback) {
      this.onLockCallback();
    }
  }
  
  unlockSession() {
    this.isLocked = false;
    this.resetTimer();
  }
  
  clearSensitiveData() {
    // Clear any cached credentials or sensitive data
    // This will be expanded as we add more sensitive data storage
    sessionStorage.clear();
  }
  
  setLockCallback(callback) {
    this.onLockCallback = callback;
  }
  
  setWarningCallback(callback) {
    this.onWarningCallback = callback;
  }
  
  destroy() {
    this.clearTimers();
    this.unbindActivityListeners();
    window.removeEventListener('autoLockSettingsChanged', this.loadSettings);
    document.removeEventListener('visibilitychange', this.pauseTimer);
    window.removeEventListener('beforeunload', this.lockSession);
  }
  
  // Manual lock function
  manualLock() {
    this.lockSession();
  }
  
  // Get remaining time until lock (in seconds)
  getRemainingTime() {
    if (this.autoLockTimeout === 'never' || !this.timeout) {
      return null;
    }
    
    const timeoutMs = parseInt(this.autoLockTimeout) * 60 * 1000;
    const elapsed = Date.now() - (this.lastActivity || Date.now());
    const remaining = Math.max(0, timeoutMs - elapsed);
    
    return Math.floor(remaining / 1000);
  }
}

// Create singleton instance
const sessionManager = new SessionManager();

export default sessionManager;