// PassQ Chrome Extension Popup Script

class PassQPopup {
  constructor() {
    this.apiUrl = null;
    this.currentTab = null;
    this.credentials = [];
    this.filteredCredentials = [];
    this.crypto = new PassQCrypto();
    this.domSanitizer = new PassQDOMSanitizer();
    
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
      const result = await chrome.storage.local.get(['passqServerUrl']);
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
      
      await chrome.storage.local.set({ passqServerUrl: serverUrl });
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
      const tabs = await chrome.tabs.query({ active: true, currentWindow: true });
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
         console.log('Favicon failed, trying globe placeholder');
          // Security: Safe DOM clearing
          while (siteFavicon.firstChild) {
            siteFavicon.removeChild(siteFavicon.firstChild);
          }
          const globeImg = this.domSanitizer.createSafeElement('img', {
            'src': 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAxNiAxNiIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KICA8Y2lyY2xlIGN4PSI4IiBjeT0iOCIgcj0iNyIgZmlsbD0id2hpdGUiIHN0cm9rZT0iIzAwMCIgc3Ryb2tlLXdpZHRoPSIyIi8+CiAgPHBhdGggZD0iTTEgOGgxNCIgc3Ryb2tlPSIjMDAwIiBzdHJva2Utd2lkdGg9IjEuNSIvPgogIDxwYXRoIGQ9Ik04IDFjLTIuNSAzLTIuNSA5IDAgMTQiIHN0cm9rZT0iIzAwMCIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiLz4KICA8cGF0aCBkPSJNOCAxYzIuNSAzIDIuNSA5IDAgMTQiIHN0cm9rZT0iIzAwMCIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiLz4KICA8cGF0aCBkPSJNMyA0LjVjMyAwIDcgMCAxMCAwIiBzdHJva2U9IiMwMDAiIHN0cm9rZS13aWR0aD0iMS4yIiBmaWxsPSJub25lIi8+CiAgPHBhdGggZD0iTTMgMTEuNWMzIDAgNyAwIDEwIDAiIHN0cm9rZT0iIzAwMCIgc3Ryb2tlLXdpZHRoPSIxLjIiIGZpbGw9Im5vbmUiLz4KPC9zdmc+',
            'alt': 'Globe'
          });
          globeImg.style.width = '100%';
          globeImg.style.height = '100%';
          globeImg.onerror = () => {
            console.log('Globe placeholder failed, using emoji');
            // Security: Safe DOM clearing and text content setting
            while (siteFavicon.firstChild) {
              siteFavicon.removeChild(siteFavicon.firstChild);
            }
            this.domSanitizer.safeSetTextContent(siteFavicon, '🌐');
            siteFavicon.style.fontSize = '16px';
            siteFavicon.style.textAlign = 'center';
            siteFavicon.style.lineHeight = '24px';
          };
          siteFavicon.appendChild(globeImg);
       };
      // Security: Safe DOM clearing
      while (siteFavicon.firstChild) {
        siteFavicon.removeChild(siteFavicon.firstChild);
      }
      siteFavicon.appendChild(img);
    }
  }

  async checkLoginStatus() {
    try {
      const token = await this.crypto.retrieveToken();
      if (token) {
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
      const token = await this.crypto.retrieveToken();
      if (!token) {
        this.showLoginForm();
        return;
      }

      const response = await fetch(`${this.apiUrl}/passwords`, {
        headers: {
          'Authorization': `Bearer ${token}`,
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
    // Security: Safe DOM clearing
    while (credentialsList.firstChild) {
      credentialsList.removeChild(credentialsList.firstChild);
    }
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
      const img = document.createElement('img');
      img.src = `https://www.google.com/s2/favicons?domain=${domain}`;
      img.style.width = '100%';
      img.style.height = '100%';
      img.style.borderRadius = '4px';
      img.onerror = () => {
          console.log('Credential favicon failed, trying globe placeholder');
           // Security: Safe DOM clearing
           while (favicon.firstChild) {
             favicon.removeChild(favicon.firstChild);
           }
           const globeImg = this.domSanitizer.createSafeElement('img', {
             'src': 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAxNiAxNiIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KICA8Y2lyY2xlIGN4PSI4IiBjeT0iOCIgcj0iNyIgZmlsbD0id2hpdGUiIHN0cm9rZT0iIzAwMCIgc3Ryb2tlLXdpZHRoPSIyIi8+CiAgPHBhdGggZD0iTTEgOGgxNCIgc3Ryb2tlPSIjMDAwIiBzdHJva2Utd2lkdGg9IjEuNSIvPgogIDxwYXRoIGQ9Ik04IDFjLTIuNSAzLTIuNSA5IDAgMTQiIHN0cm9rZT0iIzAwMCIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiLz4KICA8cGF0aCBkPSJNOCAxYzIuNSAzIDIuNSA5IDAgMTQiIHN0cm9rZT0iIzAwMCIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiLz4KICA8cGF0aCBkPSJNMyA0LjVjMyAwIDcgMCAxMCAwIiBzdHJva2U9IiMwMDAiIHN0cm9rZS13aWR0aD0iMS4yIiBmaWxsPSJub25lIi8+CiAgPHBhdGggZD0iTTMgMTEuNWMzIDAgNyAwIDEwIDAiIHN0cm9rZT0iIzAwMCIgc3Ryb2tlLXdpZHRoPSIxLjIiIGZpbGw9Im5vbmUiLz4KPC9zdmc+',
             'alt': 'Globe'
           });
           globeImg.style.width = '100%';
           globeImg.style.height = '100%';
           globeImg.onerror = () => {
             console.log('Globe placeholder failed, using emoji');
             // Security: Safe DOM clearing and text content setting
             while (favicon.firstChild) {
               favicon.removeChild(favicon.firstChild);
             }
             this.domSanitizer.safeSetTextContent(favicon, '🌐');
             favicon.style.fontSize = '12px';
             favicon.style.textAlign = 'center';
             favicon.style.lineHeight = '24px';
           };
           favicon.appendChild(globeImg);
        };
      // Security: Safe DOM clearing
      while (favicon.firstChild) {
        favicon.removeChild(favicon.firstChild);
      }
      favicon.appendChild(img);
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
      await chrome.tabs.sendMessage(this.currentTab.id, {
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
      const token = await this.crypto.retrieveToken();
      if (!token) {
        this.showToast('Not authenticated', 'error');
        return;
      }

      const response = await fetch(`${this.apiUrl}/passwords/${this.currentEditingCredentialId}`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${token}`,
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
      const token = await this.crypto.retrieveToken();
      if (!token) {
        this.showToast('Not authenticated', 'error');
        return;
      }

      const response = await fetch(`${this.apiUrl}/passwords/${this.currentEditingCredentialId}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
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
      // Security: Safe DOM manipulation for toggle button
      while (toggleBtn.firstChild) {
        toggleBtn.removeChild(toggleBtn.firstChild);
      }
      const eyeOffIcon = this.domSanitizer.createSafeElement('span');
      this.domSanitizer.safeSetTextContent(eyeOffIcon, '🙈');
      toggleBtn.appendChild(eyeOffIcon);
    } else {
      passwordInput.type = 'password';
      // Security: Safe DOM manipulation for toggle button
      while (toggleBtn.firstChild) {
        toggleBtn.removeChild(toggleBtn.firstChild);
      }
      const eyeIcon = this.domSanitizer.createSafeElement('span');
      this.domSanitizer.safeSetTextContent(eyeIcon, '👁️');
      toggleBtn.appendChild(eyeIcon);
    }
  }

  openVault() {
    if (this.apiUrl) {
      chrome.tabs.create({ url: this.apiUrl });
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
          chrome.tabs.create({ url: this.apiUrl });
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
        await this.crypto.storeToken(data.data.access_token); // Store the access token
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
      await this.crypto.removeToken();
      await this.crypto.clearCryptoData();
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