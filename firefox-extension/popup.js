// PassQ Firefox Extension Popup Script

class PassQPopup {
  constructor() {
    this.apiUrl = null;
    this.currentTab = null;
    this.credentials = [];
    this.filteredCredentials = [];
    
    this.init();
  }

  async init() {
    await this.loadServerConfig();
    await this.getCurrentTab();
    
    if (!this.apiUrl) {
      this.showServerConfig();
    } else {
      await this.checkLoginStatus();
    }
    
    this.setupEventListeners();
  }

  async loadServerConfig() {
    try {
      const result = await browser.storage.local.get(['passqServerUrl']);
      if (result.passqServerUrl) {
        this.apiUrl = result.passqServerUrl;
        this.updateServerDisplay();
      }
    } catch (error) {
      console.error('Error loading server config:', error);
    }
  }

  async saveServerConfig(serverUrl) {
    try {
      // Ensure URL has protocol
      if (!serverUrl.startsWith('http://') && !serverUrl.startsWith('https://')) {
        serverUrl = 'https://' + serverUrl;
      }
      
      // Remove trailing slash
      serverUrl = serverUrl.replace(/\/$/, '');
      
      await browser.storage.local.set({ passqServerUrl: serverUrl });
      this.apiUrl = serverUrl;
      this.updateServerDisplay();
    } catch (error) {
      console.error('Error saving server config:', error);
      throw error;
    }
  }

  updateServerDisplay() {
    const currentServerElement = document.getElementById('currentServer');
    if (currentServerElement && this.apiUrl) {
      try {
        const url = new URL(this.apiUrl);
        currentServerElement.textContent = url.host;
      } catch {
        currentServerElement.textContent = this.apiUrl;
      }
    }
  }

  showServerConfig() {
    document.getElementById('serverConfig').style.display = 'block';
    document.getElementById('loginForm').style.display = 'none';
    document.getElementById('mainContent').style.display = 'none';
    document.getElementById('actionBar').style.display = 'none';
    document.getElementById('logoutBtn').style.display = 'none';
    document.getElementById('loadingState').style.display = 'none';
    document.getElementById('errorState').style.display = 'none';
  }

  async getCurrentTab() {
    try {
      const tabs = await browser.tabs.query({ active: true, currentWindow: true });
      this.currentTab = tabs[0];
      this.updateCurrentSiteInfo();
    } catch (error) {
      console.error('Error getting current tab:', error);
    }
  }

  updateCurrentSiteInfo() {
    if (!this.currentTab) return;
    
    const url = new URL(this.currentTab.url);
    const domain = url.hostname;
    
    const siteInfo = document.querySelector('.site-info');
    if (!siteInfo) return;
    
    const siteName = siteInfo.querySelector('.site-name');
    const siteUrl = siteInfo.querySelector('.site-url');
    const siteFavicon = siteInfo.querySelector('.site-favicon');
    
    if (siteName) siteName.textContent = domain;
    if (siteUrl) siteUrl.textContent = url.origin;
    
    // Set favicon
    if (siteFavicon) {
      const faviconUrl = `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
      const img = document.createElement('img');
      img.src = faviconUrl;
      img.style.width = '100%';
      img.style.height = '100%';
      img.style.borderRadius = '4px';
      img.onerror = () => {
        siteFavicon.innerHTML = '🌐';
      };
      siteFavicon.innerHTML = '';
      siteFavicon.appendChild(img);
    }
  }

  async checkLoginStatus() {
    try {
      const result = await browser.storage.local.get(['authToken']);
      if (result.authToken) {
        await this.showMainContent();
      } else {
        this.showLoginForm();
      }
    } catch (error) {
      console.error('Error checking login status:', error);
      this.showLoginForm();
    }
  }

  showLoginForm() {
    document.getElementById('serverConfig').style.display = 'none';
    document.getElementById('loginForm').style.display = 'block';
    document.getElementById('mainContent').style.display = 'none';
    document.getElementById('actionBar').style.display = 'none';
    document.getElementById('logoutBtn').style.display = 'none';
    document.getElementById('loadingState').style.display = 'none';
    document.getElementById('errorState').style.display = 'none';
  }

  async showMainContent() {
    document.getElementById('serverConfig').style.display = 'none';
    document.getElementById('loginForm').style.display = 'none';
    document.getElementById('mainContent').style.display = 'block';
    document.getElementById('actionBar').style.display = 'flex';
    document.getElementById('logoutBtn').style.display = 'block';
    document.getElementById('loadingState').style.display = 'none';
    document.getElementById('errorState').style.display = 'none';
    
    await this.loadCredentials();
  }

  showErrorState(message) {
    document.getElementById('serverConfig').style.display = 'none';
    document.getElementById('loginForm').style.display = 'none';
    document.getElementById('mainContent').style.display = 'none';
    document.getElementById('actionBar').style.display = 'none';
    document.getElementById('logoutBtn').style.display = 'none';
    document.getElementById('loadingState').style.display = 'none';
    document.getElementById('errorState').style.display = 'block';
    document.getElementById('errorMessage').textContent = message;
  }

  showLoadingState() {
    document.getElementById('serverConfig').style.display = 'none';
    document.getElementById('loginForm').style.display = 'none';
    document.getElementById('mainContent').style.display = 'none';
    document.getElementById('actionBar').style.display = 'none';
    document.getElementById('logoutBtn').style.display = 'none';
    document.getElementById('loadingState').style.display = 'block';
    document.getElementById('errorState').style.display = 'none';
  }

  async loadCredentials() {
    try {
      const result = await browser.storage.local.get(['authToken']);
      if (!result.authToken) {
        this.showLoginForm();
        return;
      }

      const response = await fetch(`${this.apiUrl}/passwords`, {
        headers: {
          'Authorization': `Bearer ${result.authToken}`,
          'Content-Type': 'application/json'
        }
      });

      if (response.ok) {
        const result = await response.json();
        this.credentials = result.data; // Backend returns passwords in 'data' field
        this.filteredCredentials = [...this.credentials];
        this.renderCredentials();
      } else {
        throw new Error('Failed to load credentials');
      }
    } catch (error) {
      console.error('Error loading credentials:', error);
      this.showErrorState('Failed to load passwords');
    }
  }

  renderCredentials() {
    const credentialsList = document.getElementById('credentialsList');
    if (!credentialsList) return;
    
    // Clear existing items but keep the template
    const template = document.getElementById('credentialTemplate');
    credentialsList.innerHTML = '';
    if (template) {
      credentialsList.appendChild(template);
    }
    
    if (this.filteredCredentials.length === 0) {
      const noCredentials = document.createElement('div');
      noCredentials.className = 'no-credentials';
      noCredentials.textContent = 'No passwords found';
      credentialsList.appendChild(noCredentials);
      return;
    }
    
    this.filteredCredentials.forEach(credential => {
      const item = this.createCredentialItem(credential);
      credentialsList.appendChild(item);
    });
  }

  createCredentialItem(credential) {
    const template = document.getElementById('credentialTemplate');
    const item = template.cloneNode(true);
    item.style.display = 'flex';
    item.id = `credential-${credential.id}`;
    
    const domain = this.extractDomain(credential.website || '');
    
    // Set favicon
    const favicon = item.querySelector('.site-favicon');
    if (favicon) {
      favicon.style.backgroundImage = `url(https://www.google.com/s2/favicons?domain=${domain})`;
    }
    
    // Set credential info
    const title = item.querySelector('.credential-title');
    const username = item.querySelector('.credential-username');
    const url = item.querySelector('.credential-url');
    
    if (title) title.textContent = credential.title || domain;
    if (username) username.textContent = credential.username || '';
    if (url) url.textContent = domain;
    
    // Set up action buttons
    const copyUsernameBtn = item.querySelector('.copy-username');
    const copyPasswordBtn = item.querySelector('.copy-password');
    const editBtn = item.querySelector('.edit-credential');
    
    if (copyUsernameBtn) {
      copyUsernameBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        this.copyToClipboard(credential.username || '', 'Username copied!');
      });
    }
    
    if (copyPasswordBtn) {
      copyPasswordBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        this.copyToClipboard(credential.password || '', 'Password copied!');
      });
    }
    
    if (editBtn) {
      editBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        this.editCredential(credential);
      });
    }
    
    // Make the whole item clickable for autofill
    item.addEventListener('click', () => this.autofillCredential(credential));
    
    return item;
  }

  extractDomain(website) {
    try {
      const url = new URL(website.startsWith('http') ? website : `https://${website}`);
      return url.hostname;
    } catch {
      return website;
    }
  }

  async autofillCredential(credential) {
    try {
      await browser.tabs.sendMessage(this.currentTab.id, {
        action: 'autofill',
        credential: credential
      });
      window.close();
    } catch (error) {
      console.error('Error autofilling credential:', error);
    }
  }

  async copyToClipboard(text, message = 'Copied!') {
    try {
      await navigator.clipboard.writeText(text);
      this.showToast(message);
    } catch (error) {
      console.error('Error copying to clipboard:', error);
      this.showToast('Failed to copy', 'error');
    }
  }

  showToast(message, type = 'success') {
    // Create toast element
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    
    // Add to popup
    document.body.appendChild(toast);
    
    // Trigger animation
    setTimeout(() => {
      toast.classList.add('show');
    }, 10);
    
    // Remove after 2 seconds
    setTimeout(() => {
      toast.classList.remove('show');
      setTimeout(() => {
        if (toast.parentNode) {
          toast.parentNode.removeChild(toast);
        }
      }, 300);
    }, 2000);
  }

  editCredential(credential) {
    if (!credential) {
      this.showToast('Credential not found', 'error');
      return;
    }
    
    this.openCredentialEditor(credential);
  }
  
  openCredentialEditor(credential) {
    const editor = document.getElementById('credentialEditor');
    const mainContent = document.querySelector('.popup-content');
    
    // Populate form fields
    document.getElementById('editTitle').value = credential.title || '';
    document.getElementById('editUsername').value = credential.username || '';
    document.getElementById('editPassword').value = credential.password || '';
    document.getElementById('editUrl').value = credential.website || '';
    
    // Store current credential ID for saving
    this.currentEditingCredentialId = credential.id;
    
    // Show editor and hide main content
    editor.style.display = 'block';
    mainContent.style.display = 'none';
  }
  
  closeCredentialEditor() {
    const editor = document.getElementById('credentialEditor');
    const mainContent = document.querySelector('.popup-content');
    
    // Hide editor and show main content
    editor.style.display = 'none';
    mainContent.style.display = 'block';
    
    // Clear form fields
    document.getElementById('editTitle').value = '';
    document.getElementById('editUsername').value = '';
    document.getElementById('editPassword').value = '';
    document.getElementById('editUrl').value = '';
    
    this.currentEditingCredentialId = null;
  }
  
  async saveCredential() {
    const title = document.getElementById('editTitle').value.trim();
    const username = document.getElementById('editUsername').value.trim();
    const password = document.getElementById('editPassword').value;
    const url = document.getElementById('editUrl').value.trim();
    
    if (!title || !username || !password) {
      this.showToast('Please fill in all required fields', 'error');
      return;
    }
    
    try {
      const result = await browser.storage.local.get(['authToken']);
      if (!result.authToken) {
        this.showToast('Not authenticated', 'error');
        return;
      }

      const response = await fetch(`${this.apiUrl}/passwords/${this.currentEditingCredentialId}`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${result.authToken}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          title,
          username,
          password,
          website: url
        })
      });
      
      if (response.ok) {
        await this.loadCredentials();
        this.closeCredentialEditor();
        this.showToast('Credential updated successfully!', 'success');
      } else {
        throw new Error('Failed to update credential');
      }
    } catch (error) {
      console.error('Error saving credential:', error);
      this.showToast('Failed to save credential', 'error');
    }
  }
  
  async deleteCredential() {
    if (!confirm('Are you sure you want to delete this credential?')) {
      return;
    }
    
    try {
      const result = await browser.storage.local.get(['authToken']);
      if (!result.authToken) {
        this.showToast('Not authenticated', 'error');
        return;
      }

      const response = await fetch(`${this.apiUrl}/passwords/${this.currentEditingCredentialId}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${result.authToken}`,
          'Content-Type': 'application/json'
        }
      });
      
      if (response.ok) {
        await this.loadCredentials();
        this.closeCredentialEditor();
        this.showToast('Credential deleted successfully!', 'success');
      } else {
        throw new Error('Failed to delete credential');
      }
    } catch (error) {
      console.error('Error deleting credential:', error);
      this.showToast('Failed to delete credential', 'error');
    }
  }
  
  togglePasswordVisibility() {
    const passwordInput = document.getElementById('editPassword');
    const toggleBtn = document.getElementById('togglePassword');
    
    if (passwordInput.type === 'password') {
      passwordInput.type = 'text';
      toggleBtn.innerHTML = `
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20C7 20 2.73 16.39 1 12A18.45 18.45 0 0 1 5.06 5.06L17.94 17.94Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M1 1L23 23" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M10.59 10.59A2 2 0 0 0 13.41 13.41" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      `;
    } else {
      passwordInput.type = 'password';
      toggleBtn.innerHTML = `
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <path d="M1 12S5 4 12 4S23 12 23 12S19 20 12 20S1 12 1 12Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
        </svg>
      `;
    }
  }

  openVault() {
    if (this.apiUrl) {
      browser.tabs.create({ url: this.apiUrl });
      window.close();
    }
  }

  filterCredentials(searchTerm) {
    if (!searchTerm) {
      this.filteredCredentials = [...this.credentials];
    } else {
      const term = searchTerm.toLowerCase();
      this.filteredCredentials = this.credentials.filter(cred => 
        (cred.title && cred.title.toLowerCase().includes(term)) ||
        (cred.username && cred.username.toLowerCase().includes(term)) ||
        (cred.website && cred.website.toLowerCase().includes(term))
      );
    }
    this.renderCredentials();
  }

  setupEventListeners() {
    // Server configuration form
    const serverConfigForm = document.getElementById('serverConfigForm');
    if (serverConfigForm) {
      serverConfigForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        const serverUrl = document.getElementById('serverUrl').value.trim();
        
        if (!serverUrl) return;
        
        try {
          await this.saveServerConfig(serverUrl);
          await this.checkLoginStatus();
        } catch (error) {
          this.showErrorState('Invalid server URL');
        }
      });
    }



    // Login form
    const loginForm = document.getElementById('loginFormElement');
    if (loginForm) {
      loginForm.addEventListener('submit', (e) => {
        e.preventDefault();
        this.handleLogin();
      });
    }

    // Search input
    const searchInput = document.getElementById('searchInput');
    if (searchInput) {
      searchInput.addEventListener('input', (e) => {
        this.filterCredentials(e.target.value);
      });
    }

    // Current site autofill
    const autofillBtn = document.getElementById('autofillBtn');
    if (autofillBtn) {
      autofillBtn.addEventListener('click', () => {
        this.handleCurrentSiteAutofill();
      });
    }

    // Open vault
    const openVaultBtn = document.getElementById('openVault');
    if (openVaultBtn) {
      openVaultBtn.addEventListener('click', () => {
        this.openVault();
      });
    }

    // Add password
    const addPasswordBtn = document.getElementById('addPasswordBtn');
    if (addPasswordBtn) {
      addPasswordBtn.addEventListener('click', () => {
        this.openVault();
      });
    }

    // Open PassQ link
    const openPassQLink = document.getElementById('openPassQ');
    if (openPassQLink) {
      openPassQLink.addEventListener('click', (e) => {
        e.preventDefault();
        if (this.apiUrl) {
          browser.tabs.create({ url: this.apiUrl });
        }
      });
    }

    // Logout
    const logoutBtn = document.getElementById('logoutBtn');
    if (logoutBtn) {
      logoutBtn.addEventListener('click', () => {
        this.handleLogout();
      });
    }

    // Retry button
    const retryBtn = document.getElementById('retryBtn');
    if (retryBtn) {
      retryBtn.addEventListener('click', () => {
        this.init();
      });
    }

    // Credential editor event listeners
    const saveCredentialBtn = document.getElementById('saveCredential');
    if (saveCredentialBtn) {
      saveCredentialBtn.addEventListener('click', () => {
        this.saveCredential();
      });
    }

    const deleteCredentialBtn = document.getElementById('deleteCredential');
    if (deleteCredentialBtn) {
      deleteCredentialBtn.addEventListener('click', () => {
        this.deleteCredential();
      });
    }

    const cancelEditBtn = document.getElementById('cancelEdit');
    if (cancelEditBtn) {
      cancelEditBtn.addEventListener('click', () => {
        this.closeCredentialEditor();
      });
    }

    const togglePasswordBtn = document.getElementById('togglePassword');
    if (togglePasswordBtn) {
      togglePasswordBtn.addEventListener('click', () => {
        this.togglePasswordVisibility();
      });
    }

    const closeEditorBtn = document.getElementById('closeEditor');
    if (closeEditorBtn) {
      closeEditorBtn.addEventListener('click', () => {
        this.closeCredentialEditor();
      });
    }
  }

  async handleLogin() {
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    
    if (!username || !password) return;
    
    this.showLoadingState();
    
    try {
      const response = await fetch(`${this.apiUrl}/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ username, password })
      });
      
      if (response.ok) {
        const data = await response.json();
        await browser.storage.local.set({ authToken: data.data }); // Backend returns token in 'data' field
        await this.showMainContent();
      } else {
        const error = await response.json();
        this.showErrorState(error.message || 'Login failed');
      }
    } catch (error) {
      console.error('Login error:', error);
      this.showErrorState('Connection failed. Please check your server URL.');
    }
  }

  async handleCurrentSiteAutofill() {
    if (!this.currentTab) return;
    
    const url = new URL(this.currentTab.url);
    const domain = url.hostname;
    
    const matchingCredentials = this.credentials.filter(cred => {
      const credDomain = this.extractDomain(cred.website || '');
      return credDomain.includes(domain) || domain.includes(credDomain);
    });
    
    if (matchingCredentials.length > 0) {
      await this.autofillCredential(matchingCredentials[0]);
    }
  }

  async handleLogout() {
    try {
      await browser.storage.local.remove(['authToken']);
      this.showLoginForm();
    } catch (error) {
      console.error('Logout error:', error);
    }
  }
}

// Initialize popup when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  new PassQPopup();
});