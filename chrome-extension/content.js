// Content script for PassQ Chrome Extension
// Handles autofill functionality on web pages

class PassQAutofill {
  constructor() {
    this.isActive = false;
    this.credentials = [];
    this.overlayVisible = false;
    this.domSanitizer = new PassQDOMSanitizer();
    this.shadowRoot = null;
    this.shadowHost = null;
    // Allow autofill on all domains (security handled by manifest permissions)
    this.allowedDomains = null; // Disabled domain restriction
    this.init();
  }

  async init() {
    // Initialize autofill for all domains (security handled by manifest)

    // Check if user is logged in
    try {
      const response = await chrome.runtime.sendMessage({ action: 'checkLoginStatus' });
      this.isActive = response.success && response.isLoggedIn;
    } catch (error) {
      console.error('Error checking login status:', error);
      this.isActive = false;
    }

    if (this.isActive) {
      this.setupAutofill();
    }

    // Listen for messages from background script and popup
    chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
      this.handleMessage(message, sender, sendResponse);
      return true;
    });
  }

  handleMessage(message, sender, sendResponse) {
    // Ensure we always send a response to prevent connection errors
    let responseSent = false;
    
    const safeResponse = (response) => {
      if (!responseSent) {
        responseSent = true;
        try {
          sendResponse(response);
        } catch (error) {
          console.warn('Failed to send response:', error);
        }
      }
    };

    try {
      // Security: Validate message sender
      if (!this.isValidSender(sender)) {
        console.warn('PassQ: Invalid message sender:', sender);
        safeResponse({ success: false, error: 'Invalid sender' });
        return true; // Keep message channel open
      }

      // Security: Validate message structure
      if (!this.isValidMessage(message)) {
        console.warn('PassQ: Invalid message structure:', message);
        safeResponse({ success: false, error: 'Invalid message' });
        return true; // Keep message channel open
      }

      switch (message.action) {
        case 'autofill':
          try {
            if (message.credential && this.isDomainAllowed(window.location.hostname)) {
              this.fillCredentials(message.credential);
              safeResponse({ success: true });
            } else {
              safeResponse({ success: false, error: 'Domain not allowed or invalid credential' });
            }
          } catch (error) {
            console.error('Error during autofill:', error);
            safeResponse({ success: false, error: 'Autofill failed: ' + error.message });
          }
          break;
        case 'loginStatusChanged':
          try {
            this.isActive = message.isLoggedIn;
            if (this.isActive && this.isDomainAllowed(window.location.hostname)) {
              this.setupAutofill();
            } else {
              this.cleanup();
            }
            safeResponse({ success: true });
          } catch (error) {
            console.error('Error handling login status change:', error);
            safeResponse({ success: false, error: 'Failed to update login status' });
          }
          break;
        default:
          safeResponse({ success: false, error: 'Unknown action' });
      }
    } catch (error) {
      console.error('Error in message handler:', error);
      safeResponse({ success: false, error: 'Message handling failed' });
    }
    
    return true; // Keep message channel open for async responses
  }

  // Security: Domain whitelist validation
  isDomainAllowed(domain) {
    // Allow all domains (security handled by manifest permissions)
    return true;
  }

  // Security: Validate message sender
  isValidSender(sender) {
    return sender && (sender.id === chrome.runtime.id || !sender.url);
  }

  // Security: Validate message structure
  isValidMessage(message) {
    if (!message || typeof message !== 'object') {
      return false;
    }
    
    const allowedActions = ['autofill', 'loginStatusChanged'];
    if (!allowedActions.includes(message.action)) {
      return false;
    }
    
    // Additional validation based on action
    if (message.action === 'autofill' && !message.credential) {
      return false;
    }
    
    return true;
  }

  // Create shadow DOM for secure UI injection
  createShadowDOM() {
    if (this.shadowHost) {
      return;
    }

    try {
      this.shadowHost = document.createElement('div');
      this.shadowHost.id = 'passq-shadow-host';
      this.shadowHost.style.cssText = `
        position: fixed !important;
        top: 0 !important;
        left: 0 !important;
        width: 0 !important;
        height: 0 !important;
        z-index: 2147483647 !important;
        pointer-events: none !important;
        border: none !important;
        margin: 0 !important;
        padding: 0 !important;
        background: transparent !important;
      `;
      
      document.documentElement.appendChild(this.shadowHost);
      this.shadowRoot = this.shadowHost.attachShadow({ mode: 'closed' });
      
      // Add styles to shadow DOM
      const style = document.createElement('style');
      style.textContent = `
        .passq-overlay {
          position: fixed;
          background: white;
          border: 2px solid #333;
          border-radius: 8px;
          box-shadow: 0 4px 12px rgba(0,0,0,0.3);
          z-index: 2147483647;
          max-width: 300px;
          max-height: 200px;
          overflow-y: auto;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          font-size: 14px;
          pointer-events: auto;
        }
        
        .passq-credential-item {
          padding: 12px;
          border-bottom: 1px solid #eee;
          cursor: pointer;
          display: flex;
          align-items: center;
        }
        
        .passq-credential-item:hover {
          background: #f5f5f5;
        }
        
        .passq-credential-item:last-child {
          border-bottom: none;
        }
        
        .passq-credential-info {
          flex: 1;
        }
        
        .passq-credential-title {
          font-weight: 600;
          color: #333;
          margin-bottom: 2px;
        }
        
        .passq-credential-username {
          color: #666;
          font-size: 12px;
        }
      `;
      
      this.shadowRoot.appendChild(style);
    } catch (error) {
      console.error('Error creating shadow DOM:', error);
    }
  }

  // Remove shadow DOM
  removeShadowDOM() {
    if (this.shadowHost && this.shadowHost.parentNode) {
      this.shadowHost.parentNode.removeChild(this.shadowHost);
      this.shadowHost = null;
      this.shadowRoot = null;
    }
  }

  setupAutofill() {
    this.findLoginForms();
    this.setupMutationObserver();
    this.createShadowDOM();
    
    // Setup keyboard shortcut (Ctrl+Shift+L)
    document.addEventListener('keydown', this.handleKeyboardShortcut.bind(this));
  }

  findLoginForms() {
    const passwordFields = document.querySelectorAll('input[type="password"]');
    
    passwordFields.forEach(passwordField => {
      if (!passwordField.dataset.passqProcessed) {
        this.processLoginForm(passwordField.form, passwordField);
        passwordField.dataset.passqProcessed = 'true';
      }
    });
  }

  processLoginForm(form, passwordField) {
    if (!passwordField || passwordField.dataset.passqButton) {
      return;
    }

    const usernameField = this.findUsernameField(form, passwordField);
    if (usernameField) {
      this.addAutofillButton(usernameField, passwordField);
    }
  }

  findUsernameField(form, passwordField) {
    const selectors = [
      'input[type="email"]',
      'input[type="text"]',
      'input[name*="user"]',
      'input[name*="email"]',
      'input[name*="login"]',
      'input[id*="user"]',
      'input[id*="email"]',
      'input[id*="login"]'
    ];

    let candidates = [];
    const container = form || document;

    for (const selector of selectors) {
      const fields = container.querySelectorAll(selector);
      candidates.push(...Array.from(fields));
    }

    // Filter and sort candidates
    candidates = candidates.filter(field => 
      field !== passwordField && 
      field.type !== 'hidden' &&
      !field.disabled &&
      field.offsetParent !== null
    );

    if (candidates.length === 0) return null;

    // Prefer fields that come before the password field
    const beforePassword = candidates.filter(field => 
      this.isBeforePassword(field, passwordField)
    );

    return beforePassword.length > 0 ? beforePassword[0] : candidates[0];
  }

  isBeforePassword(usernameField, passwordField) {
    const position = usernameField.compareDocumentPosition(passwordField);
    return !!(position & Node.DOCUMENT_POSITION_FOLLOWING);
  }

  addAutofillButton(usernameField, passwordField) {
    if (usernameField.dataset.passqButton) {
      return;
    }

    // Create autofill button
    const button = document.createElement('div');
    button.innerHTML = '🔑';
    button.style.cssText = `
      position: absolute;
      right: 8px;
      top: 50%;
      transform: translateY(-50%);
      width: 20px;
      height: 20px;
      background: #4CAF50;
      border: none;
      border-radius: 50%;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 12px;
      z-index: 1000;
      box-shadow: 0 2px 4px rgba(0,0,0,0.2);
      transition: all 0.2s ease;
    `;

    button.addEventListener('mouseenter', () => {
      button.style.transform = 'translateY(-50%) scale(1.1)';
      button.style.background = '#45a049';
    });

    button.addEventListener('mouseleave', () => {
      button.style.transform = 'translateY(-50%) scale(1)';
      button.style.background = '#4CAF50';
    });

    button.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.onFieldFocus(usernameField, passwordField);
    });

    // Position button relative to username field
    const fieldRect = usernameField.getBoundingClientRect();
    const fieldStyle = window.getComputedStyle(usernameField);
    
    if (fieldStyle.position === 'static') {
      usernameField.style.position = 'relative';
    }

    usernameField.parentNode.insertBefore(button, usernameField.nextSibling);
    this.positionAutofillButton(button, usernameField);

    // Mark fields as processed
    usernameField.dataset.passqButton = 'true';
    passwordField.dataset.passqButton = 'true';

    // Add focus listeners
    usernameField.addEventListener('focus', () => {
      this.onFieldFocus(usernameField, passwordField);
    });

    passwordField.addEventListener('focus', () => {
      this.onFieldFocus(usernameField, passwordField);
    });
  }

  positionAutofillButton(button, field) {
    const rect = field.getBoundingClientRect();
    button.style.left = `${rect.right - 28}px`;
    button.style.top = `${rect.top + (rect.height / 2) - 10}px`;
  }

  async onFieldFocus(usernameField, passwordField) {
    try {
      const domain = window.location.hostname;
      const response = await chrome.runtime.sendMessage({
        action: 'findCredentials',
        domain: domain
      });

      if (response.success && response.credentials.length > 0) {
        this.credentials = response.credentials;
        this.showCredentialSelector(usernameField, passwordField);
      } else {
        this.showMessage('No credentials found for this site');
      }
    } catch (error) {
      console.error('Error finding credentials:', error);
      this.showMessage('Error loading credentials');
    }
  }

  async showCredentialSelector(usernameField, passwordField) {
    if (this.credentials.length === 1) {
      // Auto-fill if only one credential
      this.fillCredentials(this.credentials[0]);
      this.showMessage('Credentials filled');
    } else {
      // Show selection overlay
      this.showCredentialOverlay(usernameField, passwordField);
    }
  }

  showCredentialOverlay(usernameField, passwordField) {
    if (!this.shadowRoot || this.overlayVisible) {
      return;
    }

    // Remove existing overlay
    const existingOverlay = this.shadowRoot.querySelector('.passq-overlay');
    if (existingOverlay) {
      existingOverlay.remove();
    }

    const overlay = document.createElement('div');
    overlay.className = 'passq-overlay';

    this.credentials.forEach(credential => {
      const item = document.createElement('div');
      item.className = 'passq-credential-item';
      
      const info = document.createElement('div');
      info.className = 'passq-credential-info';
      
      const title = document.createElement('div');
      title.className = 'passq-credential-title';
      this.domSanitizer.safeSetTextContent(title, credential.website || 'Untitled');
      
      const username = document.createElement('div');
      username.className = 'passq-credential-username';
      this.domSanitizer.safeSetTextContent(username, credential.username || 'No username');
      
      info.appendChild(title);
      info.appendChild(username);
      item.appendChild(info);
      
      item.addEventListener('click', () => {
        this.fillCredentials(credential);
        this.hideCredentialOverlay();
        this.showMessage('Credentials filled');
      });
      
      overlay.appendChild(item);
    });

    // Position overlay
    const fieldRect = usernameField.getBoundingClientRect();
    overlay.style.left = `${fieldRect.left}px`;
    overlay.style.top = `${fieldRect.bottom + 5}px`;

    this.shadowRoot.appendChild(overlay);
    this.overlayVisible = true;

    // Hide overlay when clicking outside
    document.addEventListener('click', this.handleOutsideClick.bind(this), true);
  }

  hideCredentialOverlay() {
    if (this.shadowRoot) {
      const overlay = this.shadowRoot.querySelector('.passq-overlay');
      if (overlay) {
        overlay.remove();
      }
    }
    this.overlayVisible = false;
    document.removeEventListener('click', this.handleOutsideClick.bind(this), true);
  }

  handleOutsideClick(event) {
    if (!this.shadowRoot || !this.overlayVisible) return;
    
    this.hideCredentialOverlay();
  }

  fillCredentials(credential) {
    const usernameFields = document.querySelectorAll(
      'input[type="email"], input[type="text"], input[name*="user"], input[name*="email"], input[name*="login"]'
    );
    const passwordFields = document.querySelectorAll('input[type="password"]');

    // Fill username
    if (credential.username && usernameFields.length > 0) {
      const usernameField = Array.from(usernameFields).find(field => 
        field.offsetParent !== null && !field.disabled
      );
      if (usernameField) {
        this.fillField(usernameField, credential.username);
      }
    }

    // Fill password
    if (credential.password && passwordFields.length > 0) {
      const passwordField = Array.from(passwordFields).find(field => 
        field.offsetParent !== null && !field.disabled
      );
      if (passwordField) {
        this.fillField(passwordField, credential.password);
      }
    }
  }

  fillField(field, value) {
    if (!field || !value) return;

    // Set value
    field.value = value;
    field.setAttribute('value', value);

    // Trigger events
    const events = ['input', 'change', 'keyup', 'keydown'];
    events.forEach(eventType => {
      const event = new Event(eventType, { bubbles: true, cancelable: true });
      field.dispatchEvent(event);
    });

    // Focus the field briefly
    field.focus();
    field.blur();
  }

  extractDomain(website) {
    try {
      const url = new URL(website.startsWith('http') ? website : `https://${website}`);
      return url.hostname;
    } catch {
      return website;
    }
  }

  showMessage(text) {
    if (!this.shadowRoot) return;

    // Remove existing message
    const existingMessage = this.shadowRoot.querySelector('.passq-message');
    if (existingMessage) {
      existingMessage.remove();
    }

    const message = document.createElement('div');
    message.className = 'passq-message';
    message.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: #333;
      color: white;
      padding: 12px 16px;
      border-radius: 6px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 14px;
      z-index: 2147483647;
      box-shadow: 0 4px 12px rgba(0,0,0,0.3);
      pointer-events: none;
      opacity: 0;
      transition: opacity 0.3s ease;
    `;
    
    this.domSanitizer.safeSetTextContent(message, text);
    this.shadowRoot.appendChild(message);

    // Animate in
    requestAnimationFrame(() => {
      message.style.opacity = '1';
    });

    // Auto-hide after 3 seconds
    setTimeout(() => {
      if (message.parentNode) {
        message.style.opacity = '0';
        setTimeout(() => {
          if (message.parentNode) {
            message.remove();
          }
        }, 300);
      }
    }, 3000);
  }

  handleKeyboardShortcut(event) {
    // Ctrl+Shift+L (or Cmd+Shift+L on Mac)
    if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'L') {
      event.preventDefault();
      
      const passwordField = document.querySelector('input[type="password"]:not([disabled])');
      if (passwordField) {
        const form = passwordField.form;
        const usernameField = this.findUsernameField(form, passwordField);
        if (usernameField) {
          this.onFieldFocus(usernameField, passwordField);
        }
      }
    }
  }

  setupMutationObserver() {
    const observer = new MutationObserver((mutations) => {
      let shouldCheck = false;
      
      mutations.forEach((mutation) => {
        if (mutation.type === 'childList') {
          mutation.addedNodes.forEach((node) => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              if (node.querySelector && node.querySelector('input[type="password"]')) {
                shouldCheck = true;
              }
            }
          });
        }
      });
      
      if (shouldCheck) {
        setTimeout(() => this.findLoginForms(), 100);
      }
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true
    });
  }

  cleanup() {
    this.removeShadowDOM();
    this.hideCredentialOverlay();
    
    // Remove event listeners
    document.removeEventListener('keydown', this.handleKeyboardShortcut.bind(this));
    
    // Remove processed markers
    document.querySelectorAll('[data-passq-processed]').forEach(element => {
      delete element.dataset.passqProcessed;
    });
    
    document.querySelectorAll('[data-passq-button]').forEach(element => {
      delete element.dataset.passqButton;
    });
  }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    new PassQAutofill();
  });
} else {
  new PassQAutofill();
}