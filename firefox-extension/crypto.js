// PassQ Firefox Extension Crypto Utilities

class PassQCrypto {
  constructor() {
    this.keyName = 'passq-extension-key';
    this.algorithm = {
      name: 'AES-GCM',
      length: 256
    };
  }

  /**
   * Generate or retrieve the encryption key
   */
  async getOrCreateKey() {
    try {
      // Try to get existing key from storage
      const result = await browser.storage.local.get([this.keyName]);
      
      if (result[this.keyName]) {
        // Import the stored key
        const keyData = new Uint8Array(result[this.keyName]);
        return await crypto.subtle.importKey(
          'raw',
          keyData,
          this.algorithm,
          false,
          ['encrypt', 'decrypt']
        );
      } else {
        // Generate new key
        const key = await crypto.subtle.generateKey(
          this.algorithm,
          true,
          ['encrypt', 'decrypt']
        );
        
        // Export and store the key
        const keyData = await crypto.subtle.exportKey('raw', key);
        await browser.storage.local.set({
          [this.keyName]: Array.from(new Uint8Array(keyData))
        });
        
        return key;
      }
    } catch (error) {
      console.error('Error managing encryption key:', error);
      throw new Error('Failed to manage encryption key');
    }
  }

  /**
   * Encrypt a token
   */
  async encryptToken(token) {
    try {
      if (!token) return null;
      
      const key = await this.getOrCreateKey();
      const encoder = new TextEncoder();
      const data = encoder.encode(token);
      
      // Generate random IV
      const iv = crypto.getRandomValues(new Uint8Array(12));
      
      // Encrypt the token
      const encrypted = await crypto.subtle.encrypt(
        {
          name: 'AES-GCM',
          iv: iv
        },
        key,
        data
      );
      
      // Combine IV and encrypted data
      const combined = new Uint8Array(iv.length + encrypted.byteLength);
      combined.set(iv);
      combined.set(new Uint8Array(encrypted), iv.length);
      
      // Convert to base64 for storage
      return btoa(String.fromCharCode(...combined));
    } catch (error) {
      console.error('Error encrypting token:', error);
      throw new Error('Failed to encrypt token');
    }
  }

  /**
   * Decrypt a token
   */
  async decryptToken(encryptedToken) {
    try {
      if (!encryptedToken) return null;
      
      const key = await this.getOrCreateKey();
      
      // Convert from base64
      const combined = new Uint8Array(
        atob(encryptedToken).split('').map(char => char.charCodeAt(0))
      );
      
      // Extract IV and encrypted data
      const iv = combined.slice(0, 12);
      const encrypted = combined.slice(12);
      
      // Decrypt the token
      const decrypted = await crypto.subtle.decrypt(
        {
          name: 'AES-GCM',
          iv: iv
        },
        key,
        encrypted
      );
      
      // Convert back to string
      const decoder = new TextDecoder();
      return decoder.decode(decrypted);
    } catch (error) {
      console.error('Error decrypting token:', error);
      // If decryption fails, the token might be corrupted or using old format
      return null;
    }
  }

  /**
   * Clear all crypto data (for logout)
   */
  async clearCryptoData() {
    try {
      await browser.storage.local.remove([this.keyName]);
    } catch (error) {
      console.error('Error clearing crypto data:', error);
    }
  }

  /**
   * Securely store an encrypted token
   */
  async storeToken(token) {
    try {
      if (!token) {
        await browser.storage.local.remove(['encryptedAuthToken']);
        return;
      }
      
      const encrypted = await this.encryptToken(token);
      await browser.storage.local.set({ encryptedAuthToken: encrypted });
    } catch (error) {
      console.error('Error storing token:', error);
      throw new Error('Failed to store token securely');
    }
  }

  /**
   * Retrieve and decrypt a stored token
   */
  async retrieveToken() {
    try {
      const result = await browser.storage.local.get(['encryptedAuthToken']);
      if (!result.encryptedAuthToken) {
        return null;
      }
      
      return await this.decryptToken(result.encryptedAuthToken);
    } catch (error) {
      console.error('Error retrieving token:', error);
      // If retrieval fails, clear the corrupted token
      await browser.storage.local.remove(['encryptedAuthToken']);
      return null;
    }
  }

  /**
   * Remove stored token
   */
  async removeToken() {
    try {
      await browser.storage.local.remove(['encryptedAuthToken']);
    } catch (error) {
      console.error('Error removing token:', error);
    }
  }
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
  module.exports = PassQCrypto;
} else {
  // Make available globally for extension scripts
  window.PassQCrypto = PassQCrypto;
}