// PassQ Chrome Extension Offline Cache Manager
// Provides IndexedDB-based caching for offline functionality

class PassQOfflineCache {
  constructor() {
    this.dbName = 'PassQOfflineDB';
    this.dbVersion = 1;
    this.db = null;
    this.stores = {
      credentials: 'credentials',
      metadata: 'metadata',
      sync: 'sync'
    };
  }

  async init() {
    return new Promise((resolve, reject) => {
      // Check if IndexedDB is available (works in both service worker and regular context)
      const idb = typeof self !== 'undefined' ? self.indexedDB : (typeof window !== 'undefined' ? window.indexedDB : null);
      if (!idb) {
        console.warn('IndexedDB not available, offline functionality disabled');
        reject(new Error('IndexedDB not supported'));
        return;
      }

      const request = idb.open(this.dbName, this.dbVersion);
      
      request.onerror = () => {
        console.error('Failed to open IndexedDB:', request.error);
        reject(request.error);
      };
      
      request.onsuccess = () => {
        this.db = request.result;
        console.log('IndexedDB initialized successfully');
        resolve(this.db);
      };
      
      request.onupgradeneeded = (event) => {
        const db = event.target.result;
        
        // Create credentials store
        if (!db.objectStoreNames.contains(this.stores.credentials)) {
          const credentialsStore = db.createObjectStore(this.stores.credentials, {
            keyPath: 'id',
            autoIncrement: true
          });
          credentialsStore.createIndex('website', 'website', { unique: false });
          credentialsStore.createIndex('username', 'username', { unique: false });
          credentialsStore.createIndex('lastModified', 'lastModified', { unique: false });
        }
        
        // Create metadata store
        if (!db.objectStoreNames.contains(this.stores.metadata)) {
          db.createObjectStore(this.stores.metadata, { keyPath: 'key' });
        }
        
        // Create sync store for tracking changes
        if (!db.objectStoreNames.contains(this.stores.sync)) {
          const syncStore = db.createObjectStore(this.stores.sync, {
            keyPath: 'id',
            autoIncrement: true
          });
          syncStore.createIndex('action', 'action', { unique: false });
          syncStore.createIndex('timestamp', 'timestamp', { unique: false });
          syncStore.createIndex('synced', 'synced', { unique: false });
        }
      };
    });
  }

  async cacheCredentials(credentials) {
    if (!this.db) {
      await this.init();
    }
    
    try {
      // Clear existing credentials first
      await this.clearCredentials();
      
      // Create a new transaction for adding credentials
      const transaction = this.db.transaction([this.stores.credentials], 'readwrite');
      const store = transaction.objectStore(this.stores.credentials);
      
      // Cache new credentials with timestamp
      const promises = credentials.map(credential => {
        const cachedCredential = {
          ...credential,
          cachedAt: Date.now(),
          lastModified: credential.lastModified || Date.now()
        };
        return this.putToStore(store, cachedCredential);
      });
      
      await Promise.all(promises);
      
      // Update metadata
      await this.setMetadata('lastCacheUpdate', Date.now());
      await this.setMetadata('credentialCount', credentials.length);
      
      console.log(`Cached ${credentials.length} credentials offline`);
      return true;
    } catch (error) {
      console.error('Failed to cache credentials:', error);
      throw error;
    }
  }

  async getCachedCredentials() {
    try {
      if (!this.db) {
        await this.init();
      }
      
      if (!this.db) {
        console.warn('Database not initialized, returning empty credentials');
        return [];
      }
      
      const transaction = this.db.transaction([this.stores.credentials], 'readonly');
      const store = transaction.objectStore(this.stores.credentials);
      
      return new Promise((resolve, reject) => {
        const request = store.getAll();
        
        request.onsuccess = () => {
          resolve(request.result || []);
        };
        
        request.onerror = () => {
          console.error('Error getting cached credentials:', request.error);
          reject(request.error);
        };
      });
    } catch (error) {
      console.error('Failed to get cached credentials:', error);
      return [];
    }
  }

  async searchCachedCredentials(query) {
    const credentials = await this.getCachedCredentials();
    
    if (!query || query.trim() === '') {
      return credentials;
    }
    
    const searchTerm = query.toLowerCase();
    return credentials.filter(credential => {
      return (
        (credential.website && credential.website.toLowerCase().includes(searchTerm)) ||
        (credential.username && credential.username.toLowerCase().includes(searchTerm)) ||
        (credential.title && credential.title.toLowerCase().includes(searchTerm))
      );
    });
  }

  async findCredentialsByDomain(domain) {
    const credentials = await this.getCachedCredentials();
    
    return credentials.filter(credential => {
      if (!credential.website) return false;
      
      try {
        const credentialDomain = new URL(
          credential.website.startsWith('http') 
            ? credential.website 
            : `https://${credential.website}`
        ).hostname;
        
        return credentialDomain.includes(domain) || domain.includes(credentialDomain);
      } catch {
        return credential.website.toLowerCase().includes(domain.toLowerCase());
      }
    });
  }

  async addCredential(credential) {
    if (!this.db) {
      await this.init();
    }
    
    const transaction = this.db.transaction([this.stores.credentials, this.stores.sync], 'readwrite');
    const credentialsStore = transaction.objectStore(this.stores.credentials);
    const syncStore = transaction.objectStore(this.stores.sync);
    
    try {
      const credentialWithTimestamp = {
        ...credential,
        cachedAt: Date.now(),
        lastModified: Date.now()
      };
      
      const result = await this.addToStore(credentialsStore, credentialWithTimestamp);
      
      // Track for sync
      await this.addToStore(syncStore, {
        action: 'create',
        credentialId: result,
        data: credentialWithTimestamp,
        timestamp: Date.now(),
        synced: false
      });
      
      return result;
    } catch (error) {
      console.error('Failed to add credential to cache:', error);
      throw error;
    }
  }

  async updateCredential(id, updates) {
    if (!this.db) {
      await this.init();
    }
    
    const transaction = this.db.transaction([this.stores.credentials, this.stores.sync], 'readwrite');
    const credentialsStore = transaction.objectStore(this.stores.credentials);
    const syncStore = transaction.objectStore(this.stores.sync);
    
    try {
      const existing = await this.getFromStore(credentialsStore, id);
      if (!existing) {
        throw new Error('Credential not found');
      }
      
      const updated = {
        ...existing,
        ...updates,
        lastModified: Date.now()
      };
      
      await this.putToStore(credentialsStore, updated);
      
      // Track for sync
      await this.addToStore(syncStore, {
        action: 'update',
        credentialId: id,
        data: updated,
        timestamp: Date.now(),
        synced: false
      });
      
      return updated;
    } catch (error) {
      console.error('Failed to update credential in cache:', error);
      throw error;
    }
  }

  async deleteCredential(id) {
    if (!this.db) {
      await this.init();
    }
    
    const transaction = this.db.transaction([this.stores.credentials, this.stores.sync], 'readwrite');
    const credentialsStore = transaction.objectStore(this.stores.credentials);
    const syncStore = transaction.objectStore(this.stores.sync);
    
    try {
      const existing = await this.getFromStore(credentialsStore, id);
      if (!existing) {
        throw new Error('Credential not found');
      }
      
      await this.deleteFromStore(credentialsStore, id);
      
      // Track for sync
      await this.addToStore(syncStore, {
        action: 'delete',
        credentialId: id,
        data: existing,
        timestamp: Date.now(),
        synced: false
      });
      
      return true;
    } catch (error) {
      console.error('Failed to delete credential from cache:', error);
      throw error;
    }
  }

  async getPendingSyncItems() {
    try {
      if (!this.db) {
        await this.init();
      }
      
      if (!this.db) {
        console.warn('Database not initialized for pending sync items');
        return [];
      }
      const transaction = this.db.transaction([this.stores.sync], 'readonly');
      const store = transaction.objectStore(this.stores.sync);
      
      // Check if the index exists
      if (!store.indexNames.contains('synced')) {
        console.warn('Synced index not found, returning empty array');
        return [];
      }
      
      const index = store.index('synced');
      
      return new Promise((resolve, reject) => {
        const request = index.getAll(false);
        
        request.onsuccess = () => {
          resolve(request.result || []);
        };
        
        request.onerror = () => {
          console.error('Error getting pending sync items:', request.error);
          resolve([]); // Return empty array instead of rejecting
        };
      });
    } catch (error) {
      console.error('Error in getPendingSyncItems:', error);
      return [];
    }
  }

  async markSyncItemCompleted(syncId) {
    if (!this.db) {
      await this.init();
    }
    
    const transaction = this.db.transaction([this.stores.sync], 'readwrite');
    const store = transaction.objectStore(this.stores.sync);
    
    try {
      const item = await this.getFromStore(store, syncId);
      if (item) {
        item.synced = true;
        item.syncedAt = Date.now();
        await this.putToStore(store, item);
      }
    } catch (error) {
      console.error('Failed to mark sync item as completed:', error);
      throw error;
    }
  }

  async clearCredentials() {
    if (!this.db) {
      await this.init();
    }
    
    const transaction = this.db.transaction([this.stores.credentials], 'readwrite');
    const store = transaction.objectStore(this.stores.credentials);
    
    return new Promise((resolve, reject) => {
      const request = store.clear();
      
      request.onsuccess = () => {
        resolve();
      };
      
      request.onerror = () => {
        reject(request.error);
      };
    });
  }

  async clearAllData() {
    if (!this.db) {
      await this.init();
    }
    
    const transaction = this.db.transaction(
      [this.stores.credentials, this.stores.metadata, this.stores.sync], 
      'readwrite'
    );
    
    try {
      await Promise.all([
        this.clearStore(transaction.objectStore(this.stores.credentials)),
        this.clearStore(transaction.objectStore(this.stores.metadata)),
        this.clearStore(transaction.objectStore(this.stores.sync))
      ]);
      
      console.log('All offline data cleared');
    } catch (error) {
      console.error('Failed to clear offline data:', error);
      throw error;
    }
  }

  async getMetadata(key) {
    try {
      if (!this.db) {
        await this.init();
      }
      
      if (!this.db) {
        console.warn('Database not initialized for metadata');
        return null;
      }
      
      const transaction = this.db.transaction([this.stores.metadata], 'readonly');
      const store = transaction.objectStore(this.stores.metadata);
      
      const result = await this.getFromStore(store, key);
      return result ? result.value : null;
    } catch (error) {
      console.error('Failed to get metadata:', error);
      return null;
    }
  }

  async setMetadata(key, value) {
    if (!this.db) {
      await this.init();
    }
    
    const transaction = this.db.transaction([this.stores.metadata], 'readwrite');
    const store = transaction.objectStore(this.stores.metadata);
    
    await this.putToStore(store, { key, value, timestamp: Date.now() });
  }

  async isOfflineMode() {
    try {
      // Check if we have cached credentials and no network
      const credentials = await this.getCachedCredentials();
      const hasCache = credentials.length > 0;
      const isOnline = navigator.onLine;
      
      return hasCache && !isOnline;
    } catch {
      return false;
    }
  }

  async getCacheStats() {
    try {
      // Ensure database is initialized
      if (!this.db) {
        try {
          await this.init();
        } catch (initError) {
          console.warn('Failed to initialize database for cache stats:', initError);
          return {
            credentialCount: 0,
            lastUpdate: null,
            pendingSyncCount: 0,
            isOffline: true
          };
        }
      }

      // Check if database is still not available
      if (!this.db) {
        console.warn('Database not available for cache stats');
        return {
          credentialCount: 0,
          lastUpdate: null,
          pendingSyncCount: 0,
          isOffline: true
        };
      }

      // Get stats with individual error handling
      let credentials = [];
      let lastUpdate = null;
      let pendingSync = [];
      let isOffline = true;

      try {
        credentials = await this.getCachedCredentials() || [];
      } catch (error) {
        console.warn('Failed to get cached credentials for stats:', error);
      }

      try {
        lastUpdate = await this.getMetadata('lastCacheUpdate');
      } catch (error) {
        console.warn('Failed to get last update metadata:', error);
      }

      try {
        pendingSync = await this.getPendingSyncItems() || [];
      } catch (error) {
        console.warn('Failed to get pending sync items:', error);
      }

      try {
        isOffline = await this.isOfflineMode();
      } catch (error) {
        console.warn('Failed to check offline mode:', error);
      }
      
      return {
        credentialCount: credentials.length,
        lastUpdate: lastUpdate ? new Date(lastUpdate) : null,
        pendingSyncCount: pendingSync.length,
        isOffline: isOffline
      };
    } catch (error) {
      console.error('Failed to get cache stats:', error);
      return {
        credentialCount: 0,
        lastUpdate: null,
        pendingSyncCount: 0,
        isOffline: true
      };
    }
  }

  // Helper methods for IndexedDB operations
  addToStore(store, data) {
    return new Promise((resolve, reject) => {
      const request = store.add(data);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  getFromStore(store, key) {
    return new Promise((resolve, reject) => {
      const request = store.get(key);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  putToStore(store, data) {
    return new Promise((resolve, reject) => {
      const request = store.put(data);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  deleteFromStore(store, key) {
    return new Promise((resolve, reject) => {
      const request = store.delete(key);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  clearStore(store) {
    return new Promise((resolve, reject) => {
      const request = store.clear();
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }
}

// Export for use in other scripts
// Export for both service worker and regular context
if (typeof self !== 'undefined') {
  self.PassQOfflineCache = PassQOfflineCache;
}
if (typeof window !== 'undefined') {
  window.PassQOfflineCache = PassQOfflineCache;
}