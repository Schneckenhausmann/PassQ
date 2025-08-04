// PassQ Firefox Extension Popup Script

class PassQPopup {
  constructor() {
    this.apiUrl = 'http://localhost:5000';
    this.currentTab = null;
    this.credentials = [];
    this.filteredCredentials = [];
    
    this.init();
  }

  async init() {
    await this.getCurrentTab();
    await this.checkLoginStatus();
    this.setupEventListeners();
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
    const siteName = siteInfo.querySelector('.site-name');
    const siteUrl = siteInfo.querySelector('.site-url');
    const siteFavicon = siteInfo.querySelector('.site-favicon');
    
    siteName.textContent = domain;
    siteUrl.textContent = url.origin;
    
    // Set favicon
    const faviconUrl = `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
    const img = document.createElement('img');
    img.src = faviconUrl;
    img.style.width = '100%';
    img.style.height = '100%';
    img.style.borderRadius = '4px';
    img.onerror = () => {
      siteFavicon.innerHTML = '🔒';
    };
    siteFavicon.innerHTML = '';
    siteFavicon.appendChild(img);
  }

  async checkLoginStatus() {
    try {
      const result = await browser.runtime.sendMessage({ action: 'checkLogin' });
      
      if (result.success && result.isLoggedIn) {
        await this.showMainContent();
      } else {
        this.showLoginForm();
      }
    } catch (error) {
      console.error('Error checking login status:', error);
      this.showErrorState('Connection failed');
    }
  }

  showLoginForm() {
    document.getElementById('loadingState').style.display = 'none';
    document.getElementById('loginForm').style.display = 'block';
    document.getElementById('mainContent').style.display = 'none';
    document.getElementById('errorState').style.display = 'none';
  }

  async showMainContent() {
    document.getElementById('loadingState').style.display = 'none';
    document.getElementById('loginForm').style.display = 'none';
    document.getElementById('mainContent').style.display = 'flex';
    document.getElementById('errorState').style.display = 'none';
    
    await this.loadCredentials();
  }

  showErrorState(message) {
    document.getElementById('loadingState').style.display = 'none';
    document.getElementById('loginForm').style.display = 'none';
    document.getElementById('mainContent').style.display = 'none';
    document.getElementById('errorState').style.display = 'flex';
    document.getElementById('errorMessage').textContent = message;
  }

  showLoadingState() {
    document.getElementById('loadingState').style.display = 'flex';
    document.getElementById('loginForm').style.display = 'none';
    document.getElementById('mainContent').style.display = 'none';
    document.getElementById('errorState').style.display = 'none';
  }

  async loadCredentials() {
    try {
      const result = await browser.runtime.sendMessage({ action: 'getPasswords' });
      
      if (result.success) {
        this.credentials = result.passwords || [];
        this.filteredCredentials = [...this.credentials];
        this.renderCredentials();
      } else {
        console.error('Failed to load credentials:', result.error);
      }
    } catch (error) {
      console.error('Error loading credentials:', error);
    }
  }

  renderCredentials() {
    const credentialsList = document.getElementById('credentialsList');
    credentialsList.innerHTML = '';
    
    if (this.filteredCredentials.length === 0) {
      credentialsList.innerHTML = `
        <div style="padding: 20px; text-align: center; color: #9ca3af;">
          <p>No credentials found</p>
        </div>
      `;
      return;
    }
    
    this.filteredCredentials.forEach(credential => {
      const item = this.createCredentialItem(credential);
      credentialsList.appendChild(item);
    });
  }

  createCredentialItem(credential) {
    const item = document.createElement('div');
    item.className = 'credential-item';
    
    const domain = this.extractDomain(credential.website);
    const faviconUrl = `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
    
    item.innerHTML = `
      <div class="credential-favicon">
        <img src="${faviconUrl}" style="width: 100%; height: 100%; border-radius: 3px;" 
             onerror="this.style.display='none'; this.parentElement.innerHTML='🔒';">
      </div>
      <div class="credential-info">
        <div class="credential-website">${credential.website}</div>
        <div class="credential-username">${credential.username}</div>
      </div>
    `;
    
    item.addEventListener('click', () => {
      this.autofillCredential(credential);
    });
    
    return item;
  }

  extractDomain(website) {
    try {
      const url = website.startsWith('http') ? website : `https://${website}`;
      return new URL(url).hostname;
    } catch {
      return website;
    }
  }

  async autofillCredential(credential) {
    try {
      const result = await browser.runtime.sendMessage({
        action: 'autofill',
        credential: credential
      });
      
      if (result.success) {
        window.close();
      } else {
        console.error('Autofill failed:', result.error);
      }
    } catch (error) {
      console.error('Error during autofill:', error);
    }
  }

  filterCredentials(searchTerm) {
    if (!searchTerm) {
      this.filteredCredentials = [...this.credentials];
    } else {
      const term = searchTerm.toLowerCase();
      this.filteredCredentials = this.credentials.filter(credential => 
        credential.website.toLowerCase().includes(term) ||
        credential.username.toLowerCase().includes(term)
      );
    }
    this.renderCredentials();
  }

  setupEventListeners() {
    // Login form
    const loginForm = document.getElementById('loginFormElement');
    loginForm.addEventListener('submit', async (e) => {
      e.preventDefault();
      await this.handleLogin();
    });

    // Search
    const searchInput = document.getElementById('searchInput');
    searchInput.addEventListener('input', (e) => {
      this.filterCredentials(e.target.value);
    });

    // Autofill button
    const autofillBtn = document.getElementById('autofillBtn');
    autofillBtn.addEventListener('click', async () => {
      await this.handleCurrentSiteAutofill();
    });

    // Open vault
    const openVaultBtn = document.getElementById('openVault');
    openVaultBtn.addEventListener('click', () => {
      browser.tabs.create({ url: 'http://localhost:3000' });
      window.close();
    });

    // Open PassQ link
    const openPassQLink = document.getElementById('openPassQ');
    openPassQLink.addEventListener('click', (e) => {
      e.preventDefault();
      browser.tabs.create({ url: 'http://localhost:3000' });
      window.close();
    });

    // Logout
    const logoutBtn = document.getElementById('logoutBtn');
    logoutBtn.addEventListener('click', async () => {
      await this.handleLogout();
    });

    // Retry button
    const retryBtn = document.getElementById('retryBtn');
    retryBtn.addEventListener('click', async () => {
      this.showLoadingState();
      await this.checkLoginStatus();
    });
  }

  async handleLogin() {
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    
    if (!username || !password) {
      return;
    }
    
    this.showLoadingState();
    
    try {
      const result = await browser.runtime.sendMessage({
        action: 'login',
        username: username,
        password: password
      });
      
      if (result.success) {
        await this.showMainContent();
      } else {
        this.showErrorState(result.error || 'Login failed');
      }
    } catch (error) {
      console.error('Login error:', error);
      this.showErrorState('Connection failed');
    }
  }

  async handleCurrentSiteAutofill() {
    if (!this.currentTab) return;
    
    const domain = new URL(this.currentTab.url).hostname;
    const matchingCredentials = this.credentials.filter(credential => {
      const credentialDomain = this.extractDomain(credential.website);
      return credentialDomain === domain;
    });
    
    if (matchingCredentials.length === 1) {
      await this.autofillCredential(matchingCredentials[0]);
    } else if (matchingCredentials.length > 1) {
      // Show selection in the credentials list
      this.filteredCredentials = matchingCredentials;
      this.renderCredentials();
      document.getElementById('searchInput').value = domain;
    } else {
      console.log('No matching credentials found for current site');
    }
  }

  async handleLogout() {
    try {
      await browser.runtime.sendMessage({ action: 'logout' });
      this.showLoginForm();
      
      // Clear form
      document.getElementById('username').value = '';
      document.getElementById('password').value = '';
    } catch (error) {
      console.error('Logout error:', error);
    }
  }
}

// Initialize popup when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  new PassQPopup();
});