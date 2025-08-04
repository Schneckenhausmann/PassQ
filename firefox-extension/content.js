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
    const response = await browser.runtime.sendMessage({ type: 'GET_LOGIN_STATUS' });
    this.isActive = response.isLoggedIn;

    if (this.isActive) {
      this.setupAutofill();
    }

    // Listen for messages from background script
    browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
      switch (message.type) {
        case 'FILL_CREDENTIALS':
          this.fillCredentials(message.credentials);
          break;
        case 'LOGIN_STATUS_CHANGED':
          this.isActive = message.isLoggedIn;
          if (this.isActive) {
            this.setupAutofill();
          } else {
            this.cleanup();
          }
          break;
      }
    });
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
    const usernameRect = usernameField.getBoundingClientRect();
    const passwordRect = passwordField.getBoundingClientRect();
    
    // Check if username field comes before password field (vertically or horizontally)
    return usernameRect.top <= passwordRect.top || 
           (usernameRect.top === passwordRect.top && usernameRect.left < passwordRect.left);
  }

  addAutofillButton(usernameField, passwordField) {
    // Create autofill button
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'passq-autofill-btn';
    button.innerHTML = `
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2" stroke="currentColor" stroke-width="2"/>
        <circle cx="12" cy="16" r="1" stroke="currentColor" stroke-width="2"/>
        <path d="M7 11V7C7 5.67392 7.52678 4.40215 8.46447 3.46447C9.40215 2.52678 10.6739 2 12 2C13.3261 2 14.5979 2.52678 15.5355 3.46447C16.4732 4.40215 17 5.67392 17 7V11" stroke="currentColor" stroke-width="2"/>
      </svg>
    `;
    button.title = 'Fill with PassQ';
    
    // Position button
    this.positionAutofillButton(button, usernameField);
    
    // Add click handler
    button.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.showCredentialSelector(usernameField, passwordField);
    });
    
    document.body.appendChild(button);
    
    // Store reference for cleanup
    usernameField.passqButton = button;
  }

  positionAutofillButton(button, field) {
    const rect = field.getBoundingClientRect();
    const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
    const scrollLeft = window.pageXOffset || document.documentElement.scrollLeft;
    
    button.style.position = 'absolute';
    button.style.top = (rect.top + scrollTop + (rect.height - 20) / 2) + 'px';
    button.style.left = (rect.right + scrollLeft - 25) + 'px';
    button.style.zIndex = '10000';
  }

  async onFieldFocus(usernameField, passwordField) {
    // Get credentials for current domain
    const domain = window.location.hostname;
    const response = await browser.runtime.sendMessage({
      type: 'FIND_CREDENTIALS',
      domain: domain
    });
    
    if (response.success && response.credentials.length > 0) {
      this.credentials = response.credentials;
      
      // Show subtle indication that autofill is available
      if (usernameField.passqButton) {
        usernameField.passqButton.classList.add('passq-available');
      }
    }
  }

  async showCredentialSelector(usernameField, passwordField) {
    if (this.credentials.length === 0) {
      // No credentials found, show message
      this.showMessage('No credentials found for this site');
      return;
    }
    
    if (this.credentials.length === 1) {
      // Auto-fill single credential
      this.fillCredentials(this.credentials[0]);
      return;
    }
    
    // Show credential selector overlay
    this.showCredentialOverlay(usernameField, passwordField);
  }

  showCredentialOverlay(usernameField, passwordField) {
    // Remove existing overlay
    this.hideCredentialOverlay();
    
    const overlay = document.createElement('div');
    overlay.className = 'passq-credential-overlay';
    overlay.innerHTML = `
      <div class="passq-overlay-header">
        <span>Choose account for ${window.location.hostname}</span>
        <button class="passq-close-btn" type="button">×</button>
      </div>
      <div class="passq-credential-list">
        ${this.credentials.map((cred, index) => `
          <div class="passq-credential-item" data-index="${index}">
            <div class="passq-credential-info">
              <div class="passq-credential-website">${cred.website}</div>
              <div class="passq-credential-username">${cred.username}</div>
            </div>
          </div>
        `).join('')}
      </div>
    `;
    
    // Position overlay
    const rect = usernameField.getBoundingClientRect();
    const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
    const scrollLeft = window.pageXOffset || document.documentElement.scrollLeft;
    
    overlay.style.position = 'absolute';
    overlay.style.top = (rect.bottom + scrollTop + 5) + 'px';
    overlay.style.left = (rect.left + scrollLeft) + 'px';
    overlay.style.zIndex = '10001';
    
    document.body.appendChild(overlay);
    this.currentOverlay = overlay;
    
    // Add event listeners
    overlay.querySelector('.passq-close-btn').addEventListener('click', () => {
      this.hideCredentialOverlay();
    });
    
    overlay.querySelectorAll('.passq-credential-item').forEach((item, index) => {
      item.addEventListener('click', () => {
        this.fillCredentials(this.credentials[index]);
        this.hideCredentialOverlay();
      });
    });
    
    // Close on outside click
    setTimeout(() => {
      document.addEventListener('click', this.handleOutsideClick.bind(this));
    }, 100);
  }

  hideCredentialOverlay() {
    if (this.currentOverlay) {
      this.currentOverlay.remove();
      this.currentOverlay = null;
      document.removeEventListener('click', this.handleOutsideClick.bind(this));
    }
  }

  handleOutsideClick(event) {
    if (this.currentOverlay && !this.currentOverlay.contains(event.target)) {
      this.hideCredentialOverlay();
    }
  }

  fillCredentials(credentials) {
    // Find the form fields
    const passwordFields = document.querySelectorAll('input[type="password"]');
    
    for (const passwordField of passwordFields) {
      const form = passwordField.closest('form') || passwordField.parentElement;
      const usernameField = this.findUsernameField(form, passwordField);
      
      if (usernameField) {
        // Fill username
        this.fillField(usernameField, credentials.username);
        
        // Fill password
        this.fillField(passwordField, credentials.password);
        
        // Show success message
        this.showMessage('Credentials filled successfully');
        
        break; // Fill only the first matching form
      }
    }
  }

  fillField(field, value) {
    // Set value
    field.value = value;
    
    // Trigger events to ensure the page recognizes the change
    const events = ['input', 'change', 'keyup', 'blur'];
    events.forEach(eventType => {
      const event = new Event(eventType, { bubbles: true });
      field.dispatchEvent(event);
    });
  }

  showMessage(text) {
    // Remove existing message
    const existing = document.querySelector('.passq-message');
    if (existing) existing.remove();
    
    const message = document.createElement('div');
    message.className = 'passq-message';
    message.textContent = text;
    document.body.appendChild(message);
    
    // Auto-remove after 3 seconds
    setTimeout(() => {
      if (message.parentNode) {
        message.remove();
      }
    }, 3000);
  }

  handleKeyboardShortcut(event) {
    // Ctrl+Shift+P (or Cmd+Shift+P on Mac) to trigger autofill
    if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'P') {
      event.preventDefault();
      
      // Find focused field or first password field
      const activeElement = document.activeElement;
      let targetField = null;
      
      if (activeElement && (activeElement.type === 'password' || activeElement.type === 'text' || activeElement.type === 'email')) {
        targetField = activeElement;
      } else {
        targetField = document.querySelector('input[type="password"]');
      }
      
      if (targetField) {
        const form = targetField.closest('form') || targetField.parentElement;
        const passwordField = targetField.type === 'password' ? targetField : form.querySelector('input[type="password"]');
        const usernameField = this.findUsernameField(form, passwordField);
        
        if (usernameField && passwordField) {
          this.onFieldFocus(usernameField, passwordField).then(() => {
            this.showCredentialSelector(usernameField, passwordField);
          });
        }
      }
    }
  }

  setupMutationObserver() {
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            // Check if new password fields were added
            const passwordFields = node.querySelectorAll ? node.querySelectorAll('input[type="password"]') : [];
            if (passwordFields.length > 0 || (node.type === 'password')) {
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
    
    this.mutationObserver = observer;
  }

  cleanup() {
    // Remove all PassQ elements
    document.querySelectorAll('.passq-autofill-btn, .passq-credential-overlay, .passq-message').forEach(el => el.remove());
    
    // Remove event listeners
    document.removeEventListener('keydown', this.handleKeyboardShortcut.bind(this));
    
    // Disconnect mutation observer
    if (this.mutationObserver) {
      this.mutationObserver.disconnect();
    }
    
    // Clear processed flags
    document.querySelectorAll('[data-passq-processed]').forEach(el => {
      el.removeAttribute('data-passq-processed');
    });
  }
}

// Initialize autofill when page loads
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    new PassQAutofill();
  });
} else {
  new PassQAutofill();
}