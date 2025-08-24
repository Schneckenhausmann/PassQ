// CSRF token management utility

class CSRFManager {
  constructor() {
    this.token = null;
    this.tokenExpiry = null;
  }

  // Get CSRF token from backend
  async getToken() {
    try {
      // Check if we have a valid cached token
      if (this.token && this.tokenExpiry && Date.now() < this.tokenExpiry) {
        return this.token;
      }

      const response = await fetch('/auth/csrf-token', {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error('Failed to fetch CSRF token');
      }

      const data = await response.json();
      if (data.success && data.data && data.data.csrf_token) {
        this.token = data.data.csrf_token;
        // Cache token for 30 minutes
        this.tokenExpiry = Date.now() + (30 * 60 * 1000);
        return this.token;
      } else {
        throw new Error('Invalid CSRF token response');
      }
    } catch (error) {
      console.error('Error fetching CSRF token:', error);
      throw error;
    }
  }

  // Clear cached token
  clearToken() {
    this.token = null;
    this.tokenExpiry = null;
  }

  // Get headers with CSRF token
  async getHeaders(additionalHeaders = {}) {
    const token = await this.getToken();
    return {
      'Content-Type': 'application/json',
      'X-CSRF-Token': token,
      ...additionalHeaders,
    };
  }
}

// Create singleton instance
const csrfManager = new CSRFManager();

export default csrfManager;

// Helper function for making authenticated requests with CSRF protection
export const makeAuthenticatedRequest = async (url, options = {}) => {
  try {
    const headers = await csrfManager.getHeaders(options.headers || {});
    
    const response = await fetch(url, {
      ...options,
      credentials: 'include',
      headers,
    });

    // If we get a 403 (CSRF token invalid), clear token and retry once
    if (response.status === 403) {
      csrfManager.clearToken();
      const newHeaders = await csrfManager.getHeaders(options.headers || {});
      
      return fetch(url, {
        ...options,
        credentials: 'include',
        headers: newHeaders,
      });
    }

    return response;
  } catch (error) {
    console.error('Error making authenticated request:', error);
    throw error;
  }
};