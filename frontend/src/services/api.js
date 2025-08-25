import { makeAuthenticatedRequest } from '../utils/csrf';

const API_BASE_URL = process.env.REACT_APP_API_URL || '';

// Helper function to make API requests with CSRF protection
const makeRequest = async (endpoint, options = {}) => {
  const url = `${API_BASE_URL}${endpoint}`;
  
  const response = await makeAuthenticatedRequest(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });

  // Handle auth errors
  if (response.status === 401) {
    window.location.href = '/';
    throw new Error('Unauthorized');
  }

  if (!response.ok) {
    const errorData = await response.json().catch(() => ({ message: 'Request failed' }));
    throw new Error(errorData.message || 'Request failed');
  }

  return response.json();
};

// Auth API
export const authAPI = {
  register: (userData) => makeRequest('/register', { method: 'POST', body: JSON.stringify(userData) }),
  login: (credentials) => makeRequest('/login', { method: 'POST', body: JSON.stringify(credentials) }),
  changePassword: (passwordData) => makeRequest('/auth/change-password', { method: 'PUT', body: JSON.stringify(passwordData) }),
  requestPasswordReset: (email) => makeRequest('/auth/password-reset/request', { method: 'POST', body: JSON.stringify({ email }) }),
  confirmPasswordReset: (token, newPassword) => makeRequest('/auth/password-reset/confirm', { method: 'POST', body: JSON.stringify({ token, new_password: newPassword }) }),
  // SSO Authentication
  ssoLogin: async (provider) => {
    try {
      // Get OAuth authorization URL from backend
      const response = await makeRequest(`/auth/oauth/${provider}/url`, {
        method: 'GET'
      });
      
      if (response.success && response.data) {
        // Redirect to OAuth provider
        window.location.href = response.data.auth_url;
      } else {
        throw new Error('Failed to get OAuth URL');
      }
    } catch (error) {
      console.error('SSO login error:', error);
      throw error;
    }
  },

  ssoCallback: async (code, state, provider) => {
    const response = await makeRequest(`/auth/oauth/${provider}/callback`, {
      method: 'POST',
      body: JSON.stringify({ code, state })
    });
    return response;
  },

  getLinkedAccounts: async () => {
    const response = await makeRequest('/auth/oauth/accounts', {
      method: 'GET'
    });
    return response;
  },

  unlinkAccount: async (accountId) => {
    const response = await makeRequest(`/auth/oauth/accounts/${accountId}`, {
      method: 'DELETE'
    });
    return response;
  }
};

// Password API
export const passwordAPI = {
  getAll: () => makeRequest('/passwords', { method: 'GET' }),
  create: (passwordData) => makeRequest('/passwords', { method: 'POST', body: JSON.stringify(passwordData) }),
  update: (id, passwordData) => makeRequest(`/passwords/${id}`, { method: 'PUT', body: JSON.stringify(passwordData) }),
  move: (id, moveData) => makeRequest(`/passwords/${id}/move`, { method: 'PUT', body: JSON.stringify(moveData) }),
  delete: (id) => makeRequest(`/passwords/${id}`, { method: 'DELETE' }),
  generateOTP: (id) => makeRequest(`/passwords/${id}/otp`, { method: 'GET' }),
  share: (id, shareData) => makeRequest(`/passwords/${id}/share`, { method: 'POST', body: JSON.stringify(shareData) }),
  getSharedPasswords: () => makeRequest('/shared/passwords', { method: 'GET' }),
};

// Share API
export const shareAPI = {
  getSharedItems: () => makeRequest('/shared', { method: 'GET' }),
  removeShare: (shareId) => makeRequest(`/shares/${shareId}`, { method: 'DELETE' }),
};

// Folder API
export const folderAPI = {
  getAll: () => makeRequest('/folders', { method: 'GET' }),
  create: (folderData) => makeRequest('/folders', { method: 'POST', body: JSON.stringify(folderData) }),
  update: (id, folderData) => makeRequest(`/folders/${id}`, { method: 'PUT', body: JSON.stringify(folderData) }),
  delete: (id) => makeRequest(`/folders/${id}`, { method: 'DELETE' }),
  share: (id, shareData) => makeRequest(`/folders/${id}/share`, { method: 'POST', body: JSON.stringify(shareData) }),
};

// CSV API
export const csvAPI = {
  export: async (password) => {
    try {
      const url = `${API_BASE_URL}/export/csv`;
      const response = await makeAuthenticatedRequest(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'text/csv',
        },
        body: JSON.stringify({ password }),
      });
      
      if (!response.ok) {
        // Handle error responses that come as blobs
        if (response.headers.get('content-type')?.includes('application/json')) {
          const errorData = await response.json();
          throw new Error(errorData.message || 'Export failed');
        } else {
          throw new Error('Export failed');
        }
      }
      
      return response;
    } catch (error) {
      throw error;
    }
  },
  import: (csvData) => makeRequest('/import/csv', { method: 'POST', body: JSON.stringify({ csv_data: csvData }) }),
};

// Export makeRequest for direct use if needed
export { makeRequest };