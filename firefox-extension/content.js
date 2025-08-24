// Content script for PassQ Firefox Extension
// Handles autofill functionality on web pages

class PassQAutofill {
  constructor() {
    this.isActive = false;
    this.credentials = [];
    this.overlayVisible = false;
    this.init();
  }

  async init() {
    // Check if user is logged in
    try {
      const response = await browser.runtime.sendMessage({ action: 'checkLoginStatus' });
      this.isActive = response.success && response.isLoggedIn;
    } catch (error) {
      console.error('Error checking login status:', error);
      this.isActive = false;
    }

    if (this.isActive) {
      this.setupAutofill();
    }

    // Listen for messages from background script and popup
    browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
      this.handleMessage(message, sender, sendResponse);
      return true;
    });
  }

  handleMessage(message, sender, sendResponse) {
    switch (message.action) {
      case 'autofill':
        this.fillCredentials(message.credential);
        sendResponse({ success: true });
        break;
      case 'loginStatusChanged':
        this.isActive = message.isLoggedIn;
        if (this.isActive) {
          this.setupAutofill();
        } else {
          this.cleanup();
        }
        sendResponse({ success: true });
        break;
      default:
        sendResponse({ success: false, error: 'Unknown action' });
    }
  }

  setupAutofill() {
    // Find login forms on the page
    this.findLoginForms();
    
    // Set up observers for dynamically added forms
    this.setupMutationObserver();
    
    // Add keyboard shortcut listener
    document.addEventListener('keydown', this.handleKeyboardShortcut.bind(this));
  }

  findLoginForms() {
    // Find password fields
    const passwordFields = document.querySelectorAll('input[type="password"]');
    
    passwordFields.forEach(passwordField => {
      const form = passwordField.closest('form') || passwordField.parentElement;
      if (form && !form.hasAttribute('data-passq-processed')) {
        this.processLoginForm(form, passwordField);
        form.setAttribute('data-passq-processed', 'true');
      }
    });
  }

  processLoginForm(form, passwordField) {
    // Find username field (email, text, or tel input before password)
    const usernameField = this.findUsernameField(form, passwordField);
    
    if (usernameField) {
      // Add PassQ autofill button
      this.addAutofillButton(usernameField, passwordField);
      
      // Add focus listeners
      usernameField.addEventListener('focus', () => this.onFieldFocus(usernameField, passwordField));
      passwordField.addEventListener('focus', () => this.onFieldFocus(usernameField, passwordField));
    }
  }

  findUsernameField(form, passwordField) {
    // Look for common username field patterns
    const selectors = [
      'input[type="email"]',
      'input[type="text"][name*="user"]',
      'input[type="text"][name*="email"]',
      'input[type="text"][name*="login"]',
      'input[type="text"][id*="user"]',
      'input[type="text"][id*="email"]',
      'input[type="text"][id*="login"]',
      'input[type="tel"]'
    ];

    for (const selector of selectors) {
      const field = form.querySelector(selector);
      if (field && this.isBeforePassword(field, passwordField)) {
        return field;
      }
    }

    // Fallback: find any text input before password
    const textInputs = form.querySelectorAll('input[type="text"]');
    for (const input of textInputs) {
      if (this.isBeforePassword(input, passwordField)) {
        return input;
      }
    }

    return null;
  }

  isBeforePassword(usernameField, passwordField) {
    // Check if username field comes before password field in DOM
    const position = usernameField.compareDocumentPosition(passwordField);
    return position & Node.DOCUMENT_POSITION_FOLLOWING;
  }

  addAutofillButton(usernameField, passwordField) {
    // Create PassQ autofill button
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'passq-autofill-btn';
    button.innerHTML = '🔑';
    button.title = 'Fill with PassQ';
    button.style.cssText = `
      position: absolute;
      right: 8px;
      top: 50%;
      transform: translateY(-50%);
      background: #3b82f6;
      color: white;
      border: none;
      border-radius: 4px;
      width: 24px;
      height: 24px;
      font-size: 12px;
      cursor: pointer;
      z-index: 10000;
      display: none;
    `;

    // Position button relative to username field
    usernameField.style.position = 'relative';
    usernameField.parentElement.style.position = 'relative';
    usernameField.parentElement.appendChild(button);

    // Show button on field focus
    const showButton = () => {
      button.style.display = 'block';
      this.positionAutofillButton(button, usernameField);
    };
    
    const hideButton = () => {
      setTimeout(() => {
        if (!button.matches(':hover')) {
          button.style.display = 'none';
        }
      }, 200);
    };

    usernameField.addEventListener('focus', showButton);
    usernameField.addEventListener('blur', hideButton);
    passwordField.addEventListener('focus', showButton);
    passwordField.addEventListener('blur', hideButton);

    // Handle button click
    button.addEventListener('click', async (e) => {
      e.preventDefault();
      e.stopPropagation();
      await this.showCredentialSelector(usernameField, passwordField);
    });
  }

  positionAutofillButton(button, field) {
    const rect = field.getBoundingClientRect();
    button.style.right = '8px';
    button.style.top = '50%';
  }

  async onFieldFocus(usernameField, passwordField) {
    if (!this.isActive) return;

    // Get current domain
    const domain = window.location.hostname;
    
    try {
      // Find credentials for current domain
      const response = await browser.runtime.sendMessage({
        action: 'findCredentials',
        domain: domain
      });

      if (response.success && response.credentials.length > 0) {
        this.credentials = response.credentials;
        // Auto-fill if only one credential
        if (response.credentials.length === 1) {
          this.fillCredentials(response.credentials[0]);
        }
      }
    } catch (error) {
      console.error('Error finding credentials:', error);
    }
  }

  async showCredentialSelector(usernameField, passwordField) {
    if (!this.isActive) return;

    const domain = window.location.hostname;
    
    try {
      const response = await browser.runtime.sendMessage({
        action: 'findCredentials',
        domain: domain
      });

      if (response.success && response.credentials.length > 0) {
        this.credentials = response.credentials;
        this.showCredentialOverlay(usernameField, passwordField);
      } else {
        this.showMessage('No credentials found for this site');
      }
    } catch (error) {
      console.error('Error finding credentials:', error);
      this.showMessage('Error loading credentials');
    }
  }

  showCredentialOverlay(usernameField, passwordField) {
    // Remove existing overlay
    this.hideCredentialOverlay();

    const overlay = document.createElement('div');
    overlay.className = 'passq-credential-overlay';
    overlay.style.cssText = `
      position: fixed;
      background: white;
      border: 1px solid #d1d5db;
      border-radius: 8px;
      box-shadow: 0 10px 25px rgba(0, 0, 0, 0.15);
      z-index: 10001;
      max-width: 300px;
      max-height: 200px;
      overflow-y: auto;
    `;

    // Position overlay near the username field
    const rect = usernameField.getBoundingClientRect();
    overlay.style.left = `${rect.left}px`;
    overlay.style.top = `${rect.bottom + 5}px`;

    // Create credential list
    this.credentials.forEach((credential, index) => {
      const item = document.createElement('div');
      item.className = 'passq-credential-item';
      item.style.cssText = `
        padding: 12px;
        border-bottom: 1px solid #f3f4f6;
        cursor: pointer;
        display: flex;
        flex-direction: column;
        gap: 4px;
      `;

      const title = document.createElement('div');
      title.style.cssText = 'font-weight: 500; color: #1f2937;';
      title.textContent = credential.title || this.extractDomain(credential.website || '');

      const username = document.createElement('div');
      username.style.cssText = 'font-size: 14px; color: #6b7280;';
      username.textContent = credential.username || '';

      item.appendChild(title);
      item.appendChild(username);

      // Hover effects
      item.addEventListener('mouseenter', () => {
        item.style.backgroundColor = '#f9fafb';
      });
      
      item.addEventListener('mouseleave', () => {
        item.style.backgroundColor = 'transparent';
      });

      // Click handler
      item.addEventListener('click', () => {
        this.fillCredentials(credential);
        this.hideCredentialOverlay();
      });

      overlay.appendChild(item);
    });

    document.body.appendChild(overlay);
    this.overlayVisible = true;

    // Close overlay when clicking outside
    setTimeout(() => {
      document.addEventListener('click', this.handleOutsideClick.bind(this));
    }, 100);
  }

  hideCredentialOverlay() {
    const overlay = document.querySelector('.passq-credential-overlay');
    if (overlay) {
      overlay.remove();
      this.overlayVisible = false;
      document.removeEventListener('click', this.handleOutsideClick.bind(this));
    }
  }

  handleOutsideClick(event) {
    if (!event.target.closest('.passq-credential-overlay') && !event.target.closest('.passq-autofill-btn')) {
      this.hideCredentialOverlay();
    }
  }

  fillCredentials(credential) {
    // Find username and password fields
    const passwordFields = document.querySelectorAll('input[type="password"]');
    
    passwordFields.forEach(passwordField => {
      const form = passwordField.closest('form') || passwordField.parentElement;
      const usernameField = this.findUsernameField(form, passwordField);
      
      if (usernameField && credential.username) {
        this.fillField(usernameField, credential.username);
      }
      
      if (credential.password) {
        this.fillField(passwordField, credential.password);
      }
    });

    this.showMessage('Credentials filled successfully');
  }

  fillField(field, value) {
    // Set value
    field.value = value;
    
    // Trigger events to notify the page
    const events = ['input', 'change', 'keyup'];
    events.forEach(eventType => {
      const event = new Event(eventType, { bubbles: true });
      field.dispatchEvent(event);
    });
    
    // Focus the field briefly to ensure it's recognized
    field.focus();
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
    // Remove existing message
    const existingMessage = document.querySelector('.passq-message');
    if (existingMessage) {
      existingMessage.remove();
    }

    const message = document.createElement('div');
    message.className = 'passq-message';
    message.textContent = text;
    message.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: #10b981;
      color: white;
      padding: 12px 16px;
      border-radius: 6px;
      z-index: 10002;
      font-size: 14px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    `;

    document.body.appendChild(message);

    // Auto-remove after 3 seconds
    setTimeout(() => {
      if (message.parentElement) {
        message.remove();
      }
    }, 3000);
  }

  handleKeyboardShortcut(event) {
    // Ctrl+Shift+F (or Cmd+Shift+F on Mac) to trigger autofill
    if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'F') {
      event.preventDefault();
      
      // Find focused field or first password field
      let targetField = document.activeElement;
      if (!targetField || (targetField.type !== 'password' && targetField.type !== 'text' && targetField.type !== 'email')) {
        targetField = document.querySelector('input[type="password"]');
      }
      
      if (targetField) {
        const form = targetField.closest('form') || targetField.parentElement;
        const passwordField = form.querySelector('input[type="password"]');
        const usernameField = this.findUsernameField(form, passwordField);
        
        if (usernameField && passwordField) {
          this.showCredentialSelector(usernameField, passwordField);
        }
      }
    }
  }

  setupMutationObserver() {
    // Watch for dynamically added forms
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            // Check if the added node contains password fields
            const passwordFields = node.querySelectorAll ? 
              node.querySelectorAll('input[type="password"]') : [];
            
            if (passwordFields.length > 0) {
              // Process new forms
              setTimeout(() => this.findLoginForms(), 100);
            }
          }
        });
      });
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true
    });
  }

  cleanup() {
    // Remove all PassQ elements
    const elements = document.querySelectorAll('.passq-autofill-btn, .passq-credential-overlay, .passq-message');
    elements.forEach(el => el.remove());
    
    // Remove event listeners
    document.removeEventListener('keydown', this.handleKeyboardShortcut.bind(this));
    document.removeEventListener('click', this.handleOutsideClick.bind(this));
    
    // Reset state
    this.isActive = false;
    this.credentials = [];
    this.overlayVisible = false;
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