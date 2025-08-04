// Background script for PassQ Firefox Extension

// Extension state
let isLoggedIn = false;
let passqApiUrl = 'http://localhost:8080'; // Backend API URL
let passwords = [];

// Listen for messages from content scripts and popup
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  switch (message.type) {
    case 'GET_LOGIN_STATUS':
      sendResponse({ isLoggedIn });
      break;
      
    case 'LOGIN':
      handleLogin(message.credentials)
        .then(result => sendResponse(result))
        .catch(error => sendResponse({ success: false, error: error.message }));
      return true; // Keep message channel open for async response
      
    case 'LOGOUT':
      handleLogout();
      sendResponse({ success: true });
      break;
      
    case 'GET_PASSWORDS':
      if (isLoggedIn) {
        fetchPasswords()
          .then(result => sendResponse(result))
          .catch(error => sendResponse({ success: false, error: error.message }));
      } else {
        sendResponse({ success: false, error: 'Not logged in' });
      }
      return true;
      
    case 'FIND_CREDENTIALS':
      if (isLoggedIn) {
        const credentials = findCredentialsForDomain(message.domain);
        sendResponse({ success: true, credentials });
      } else {
        sendResponse({ success: false, error: 'Not logged in' });
      }
      break;
      
    case 'AUTOFILL_CREDENTIALS':
      if (isLoggedIn && message.credentials) {
        // Send credentials to content script for autofill
        browser.tabs.sendMessage(sender.tab.id, {
          type: 'FILL_CREDENTIALS',
          credentials: message.credentials
        });
        sendResponse({ success: true });
      } else {
        sendResponse({ success: false, error: 'Invalid request' });
      }
      break;
      
    default:
      sendResponse({ success: false, error: 'Unknown message type' });
  }
});

// Handle login to PassQ backend
async function handleLogin(credentials) {
  try {
    const response = await fetch(`${passqApiUrl}/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials)
    });
    
    if (response.ok) {
      const data = await response.json();
      isLoggedIn = true;
      
      // Store auth token securely
      await browser.storage.local.set({
        authToken: data.token,
        isLoggedIn: true
      });
      
      // Fetch initial passwords
      await fetchPasswords();
      
      return { success: true };
    } else {
      throw new Error('Login failed');
    }
  } catch (error) {
    console.error('Login error:', error);
    throw error;
  }
}

// Handle logout
async function handleLogout() {
  isLoggedIn = false;
  passwords = [];
  
  // Clear stored data
  await browser.storage.local.clear();
}

// Fetch passwords from PassQ backend
async function fetchPasswords() {
  try {
    const storage = await browser.storage.local.get(['authToken']);
    if (!storage.authToken) {
      throw new Error('No auth token');
    }
    
    const response = await fetch(`${passqApiUrl}/passwords`, {
      headers: {
        'Authorization': `Bearer ${storage.authToken}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (response.ok) {
      const data = await response.json();
      passwords = data.passwords || [];
      return { success: true, passwords };
    } else {
      throw new Error('Failed to fetch passwords');
    }
  } catch (error) {
    console.error('Fetch passwords error:', error);
    throw error;
  }
}

// Find credentials for a specific domain
function findCredentialsForDomain(domain) {
  if (!domain || !passwords.length) {
    return [];
  }
  
  // Clean domain (remove www, protocols, etc.)
  const cleanDomain = domain.replace(/^(https?:\/\/)?(www\.)?/, '').split('/')[0];
  
  return passwords.filter(password => {
    const passwordDomain = password.website.replace(/^(https?:\/\/)?(www\.)?/, '').split('/')[0];
    return passwordDomain.includes(cleanDomain) || cleanDomain.includes(passwordDomain);
  });
}

// Initialize extension on startup
browser.runtime.onStartup.addListener(async () => {
  const storage = await browser.storage.local.get(['isLoggedIn', 'authToken']);
  if (storage.isLoggedIn && storage.authToken) {
    isLoggedIn = true;
    try {
      await fetchPasswords();
    } catch (error) {
      console.error('Failed to restore session:', error);
      await handleLogout();
    }
  }
});

// Handle extension installation
browser.runtime.onInstalled.addListener((details) => {
  if (details.reason === 'install') {
    // Open PassQ welcome page
    browser.tabs.create({
      url: 'http://localhost:80'
    });
  }
});