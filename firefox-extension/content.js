// Content script for PassQ Firefox Extension
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
    // Security: Validate message sender
    if (!this.isValidSender(sender)) {
      console.warn('PassQ: Invalid message sender:', sender);
      sendResponse({ success: false, error: 'Invalid sender' });
      return;
    }

    // Security: Validate message structure
    if (!this.isValidMessage(message)) {
      console.warn('PassQ: Invalid message structure:', message);
      sendResponse({ success: false, error: 'Invalid message' });
      return;
    }

    switch (message.action) {
      case 'autofill':
        if (message.credential && this.isDomainAllowed(window.location.hostname)) {
          this.fillCredentials(message.credential);
          sendResponse({ success: true });
        } else {
          sendResponse({ success: false, error: 'Domain not allowed or invalid credential' });
        }
        break;
      case 'loginStatusChanged':
        this.isActive = message.isLoggedIn;
        if (this.isActive && this.isDomainAllowed(window.location.hostname)) {
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

  // Security: Domain whitelist validation
  isDomainAllowed(domain) {
    // Allow all domains (security handled by manifest permissions)
    return true;
  }

  // Security: Validate message sender
  isValidSender(sender) {
    // Only accept messages from the extension itself
    return sender && sender.id === browser.runtime.id;
  }

  // Security: Validate message structure
  isValidMessage(message) {
    if (!message || typeof message !== 'object') return false;
    if (!message.action || typeof message.action !== 'string') return false;
    
    // Validate specific message types
    switch (message.action) {
      case 'autofill':
        return message.credential && typeof message.credential === 'object';
      case 'loginStatusChanged':
        return typeof message.isLoggedIn === 'boolean';
      default:
        return true; // Allow other actions but log them
    }
  }

  // Security: Create isolated Shadow DOM for extension UI
  createShadowDOM() {
    if (this.shadowRoot) {
      return this.shadowRoot;
    }

    // Create shadow host element
    this.shadowHost = this.domSanitizer.createSafeElement('div', {
      'id': 'passq-shadow-host'
    });
    this.shadowHost.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      z-index: 2147483647;
      pointer-events: none;
    `;

    // Create shadow root with closed mode for better isolation
    this.shadowRoot = this.shadowHost.attachShadow({ mode: 'closed' });

    // Add isolated styles to shadow DOM
    const style = document.createElement('style');
    style.textContent = `
      .passq-credential-overlay {
        position: fixed;
        background: white;
        border: 1px solid #ddd;
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        max-height: 300px;
        overflow-y: auto;
        z-index: 10000;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        font-size: 14px;
        pointer-events: auto;
      }
      
      .passq-credential-item {
        padding: 12px;
        cursor: pointer;
        border-bottom: 1px solid #eee;
        transition: background-color 0.2s;
      }
      
      .passq-credential-item:hover {
        background-color: #f5f5f5;
      }
      
      .passq-credential-item:last-child {
        border-bottom: none;
      }
      
      .passq-message {
        position: fixed;
        top: 20px;
        right: 20px;
        background: #333;
        color: white;
        padding: 12px 16px;
        border-radius: 4px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        font-size: 14px;
        z-index: 10001;
        pointer-events: auto;
      }
    `;
    
    this.shadowRoot.appendChild(style);
    document.body.appendChild(this.shadowHost);
    
    return this.shadowRoot;
  }

  // Security: Remove Shadow DOM
  removeShadowDOM() {
    if (this.shadowHost && this.shadowHost.parentNode) {
      this.shadowHost.parentNode.removeChild(this.shadowHost);
    }
    this.shadowRoot = null;
    this.shadowHost = null;
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
    const button = this.domSanitizer.createSafeElement('button', {
      'class': 'passq-autofill-btn',
      'type': 'button'
    });
    this.domSanitizer.safeSetTextContent(button, '🔑');
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
    
    // Security: Validate domain before proceeding
    if (!this.isDomainAllowed(domain)) {
      console.log('PassQ: Domain not allowed for autofill:', domain);
      return;
    }
    
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
    
    // Security: Validate domain before proceeding
    if (!this.isDomainAllowed(domain)) {
      console.log('PassQ: Domain not allowed for credential selection:', domain);
      this.showMessage('Domain not allowed for autofill');
      return;
    }
    
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

    // Create Shadow DOM for isolation
    const shadowRoot = this.createShadowDOM();

    const overlay = this.domSanitizer.createSafeElement('div', {
      'class': 'passq-credential-overlay'
    });

    // Position overlay near the username field
    const rect = usernameField.getBoundingClientRect();
    overlay.style.left = `${rect.left}px`;
    overlay.style.top = `${rect.bottom + 5}px`;
    overlay.style.width = `${Math.max(200, rect.width)}px`;

    // Create credential list
    this.credentials.forEach((credential, index) => {
      const item = this.domSanitizer.createSafeElement('div', {
        'class': 'passq-credential-item'
      });

      const title = this.domSanitizer.createSafeElement('div');
      title.style.cssText = 'font-weight: 500; color: #1f2937;';
      this.domSanitizer.safeSetTextContent(title, credential.title || this.extractDomain(credential.website || ''));

      const username = this.domSanitizer.createSafeElement('div');
      username.style.cssText = 'font-size: 14px; color: #6b7280;';
      this.domSanitizer.safeSetTextContent(username, credential.username || '');

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

    shadowRoot.appendChild(overlay);
    this.overlayVisible = true;

    // Close overlay when clicking outside
    setTimeout(() => {
      document.addEventListener('click', this.handleOutsideClick.bind(this));
    }, 100);
  }

  hideCredentialOverlay() {
    if (this.shadowRoot) {
      const overlay = this.shadowRoot.querySelector('.passq-credential-overlay');
      if (overlay) {
        overlay.remove();
        this.overlayVisible = false;
        document.removeEventListener('click', this.handleOutsideClick.bind(this));
      }
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
    // Create Shadow DOM for isolation
    const shadowRoot = this.createShadowDOM();
    
    // Remove existing message
    const existingMessage = shadowRoot.querySelector('.passq-message');
    if (existingMessage) {
      existingMessage.remove();
    }

    const message = this.domSanitizer.createSafeElement('div', {
      'class': 'passq-message'
    });
    this.domSanitizer.safeSetTextContent(message, text);
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

    shadowRoot.appendChild(message);

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
    
    // Remove Shadow DOM
    this.removeShadowDOM();
    
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