// PassQ Chrome Extension Sync Manager
// Handles bidirectional synchronization between local cache and server

class PassQSyncManager {
  constructor() {
    this.offlineCache = null;
    this.apiUrl = null;
    this.authToken = null;
    this.crypto = new PassQCrypto();
    this.syncInProgress = false;
    this.syncInterval = null;
    this.conflictResolutionStrategy = 'server_wins'; // 'server_wins', 'client_wins', 'merge'
  }

  async init() {
    await this.loadConfig();
    this.offlineCache = new PassQOfflineCache();
    await this.offlineCache.init();
    
    // Start periodic sync if user is online and authenticated
    if (navigator.onLine && this.authToken) {
      this.startPeriodicSync();
    }
    
    // Listen for online/offline events (only in popup/content script context)
    if (typeof window !== 'undefined') {
      window.addEventListener('online', () => this.handleOnlineEvent());
      window.addEventListener('offline', () => this.handleOfflineEvent());
    }
  }

  async loadConfig() {
    try {
      const result = await chrome.storage.local.get(['passqServerUrl']);
      this.apiUrl = result.passqServerUrl;
      this.authToken = await this.crypto.retrieveToken();
    } catch (error) {
      console.error('Error loading sync config:', error);
    }
  }

  async handleOnlineEvent() {
    console.log('PassQ Sync: Back online, starting sync...');
    await this.performFullSync();
    this.startPeriodicSync();
  }

  handleOfflineEvent() {
    console.log('PassQ Sync: Gone offline, stopping periodic sync');
    this.stopPeriodicSync();
  }

  startPeriodicSync() {
    this.stopPeriodicSync(); // Clear any existing interval
    
    // Sync every 5 minutes when online
    this.syncInterval = setInterval(async () => {
      if (await this.isOnline() && this.authToken) {
        await this.performIncrementalSync();
      }
    }, 5 * 60 * 1000);
  }

  stopPeriodicSync() {
    if (this.syncInterval) {
      clearInterval(this.syncInterval);
      this.syncInterval = null;
    }
  }

  async performFullSync() {
    if (this.syncInProgress) {
      console.log('Sync already in progress, skipping...');
      return { success: false, message: 'Sync already in progress' };
    }

    this.syncInProgress = true;
    
    try {
      console.log('Starting full bidirectional sync...');
      
      // Step 1: Upload pending local changes
      await this.uploadPendingChanges();
      
      // Step 2: Download latest server data
      await this.downloadServerData();
      
      // Step 3: Resolve any conflicts
      await this.resolveConflicts();
      
      // Step 4: Update sync metadata
      await this.updateSyncMetadata();
      
      console.log('Full sync completed successfully');
      return { success: true, message: 'Full sync completed' };
      
    } catch (error) {
      console.error('Full sync failed:', error);
      return { success: false, message: error.message };
    } finally {
      this.syncInProgress = false;
    }
  }

  async performIncrementalSync() {
    if (this.syncInProgress) {
      return { success: false, message: 'Sync already in progress' };
    }

    this.syncInProgress = true;
    
    try {
      // Check if there are pending changes or if server has updates
      const pendingItems = await this.offlineCache.getPendingSyncItems();
      const lastSyncTime = await this.getLastSyncTime();
      
      if (pendingItems.length === 0 && !await this.hasServerUpdates(lastSyncTime)) {
        return { success: true, message: 'No changes to sync' };
      }
      
      // Perform incremental sync
      await this.uploadPendingChanges();
      await this.downloadServerUpdates(lastSyncTime);
      await this.updateSyncMetadata();
      
      return { success: true, message: 'Incremental sync completed' };
      
    } catch (error) {
      console.error('Incremental sync failed:', error);
      return { success: false, message: error.message };
    } finally {
      this.syncInProgress = false;
    }
  }

  async uploadPendingChanges() {
    const pendingItems = await this.offlineCache.getPendingSyncItems();
    
    if (pendingItems.length === 0) {
      console.log('No pending changes to upload');
      return;
    }
    
    console.log(`Uploading ${pendingItems.length} pending changes...`);
    
    for (const item of pendingItems) {
      try {
        await this.uploadSyncItem(item);
        await this.offlineCache.markSyncItemCompleted(item.id);
      } catch (error) {
        console.error(`Failed to upload sync item ${item.id}:`, error);
        // Continue with other items, don't fail the entire sync
      }
    }
  }

  async uploadSyncItem(syncItem) {
    if (!this.apiUrl || !this.authToken) {
      throw new Error('Not authenticated or server not configured');
    }
    
    const { operation, credentialId, data } = syncItem;
    
    switch (operation) {
      case 'create':
        await this.createCredentialOnServer(data);
        break;
      case 'update':
        await this.updateCredentialOnServer(credentialId, data);
        break;
      case 'delete':
        await this.deleteCredentialOnServer(credentialId);
        break;
      default:
        throw new Error(`Unknown sync operation: ${operation}`);
    }
  }

  async createCredentialOnServer(credentialData) {
    const response = await fetch(`${this.apiUrl}/passwords`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(credentialData)
    });
    
    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Failed to create credential on server');
    }
    
    return await response.json();
  }

  async updateCredentialOnServer(credentialId, credentialData) {
    const response = await fetch(`${this.apiUrl}/passwords/${credentialId}`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(credentialData)
    });
    
    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Failed to update credential on server');
    }
    
    return await response.json();
  }

  async deleteCredentialOnServer(credentialId) {
    const response = await fetch(`${this.apiUrl}/passwords/${credentialId}`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Failed to delete credential on server');
    }
  }

  async downloadServerData() {
    if (!this.apiUrl || !this.authToken) {
      throw new Error('Not authenticated or server not configured');
    }
    
    console.log('Downloading latest server data...');
    
    const response = await fetch(`${this.apiUrl}/passwords`, {
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      if (response.status === 401) {
        throw new Error('Authentication expired');
      }
      throw new Error('Failed to fetch server data');
    }
    
    const result = await response.json();
    const serverCredentials = result.data || [];
    
    // Cache the server data
    await this.offlineCache.cacheCredentials(serverCredentials);
    
    console.log(`Downloaded and cached ${serverCredentials.length} credentials`);
  }

  async downloadServerUpdates(lastSyncTime) {
    // For incremental sync, we would ideally have a server endpoint that returns
    // only items modified since lastSyncTime. For now, we'll download all data.
    await this.downloadServerData();
  }

  async hasServerUpdates(lastSyncTime) {
    // This would check if server has updates since lastSyncTime
    // For now, we'll assume there might be updates if it's been more than 10 minutes
    const tenMinutesAgo = Date.now() - (10 * 60 * 1000);
    return lastSyncTime < tenMinutesAgo;
  }

  async resolveConflicts() {
    // Conflict resolution logic would go here
    // For now, we'll use the configured strategy (default: server_wins)
    console.log(`Using conflict resolution strategy: ${this.conflictResolutionStrategy}`);
    
    // In a real implementation, we would:
    // 1. Identify conflicting items (same ID, different modification times)
    // 2. Apply the resolution strategy
    // 3. Update the cache accordingly
  }

  async updateSyncMetadata() {
    const metadata = {
      lastSyncTime: Date.now(),
      lastSyncType: 'full',
      syncVersion: 1
    };
    
    await this.offlineCache.updateMetadata('sync', metadata);
  }

  async getLastSyncTime() {
    const metadata = await this.offlineCache.getMetadata('sync');
    return metadata?.lastSyncTime || 0;
  }

  async forceSyncCredential(credentialId, operation = 'update', data = null) {
    // Force sync a specific credential immediately
    try {
      const syncItem = {
        id: `${operation}_${credentialId}_${Date.now()}`,
        credentialId,
        operation,
        data,
        timestamp: Date.now(),
        synced: false
      };
      
      await this.offlineCache.addSyncItem(syncItem);
      
      // If online, try to sync immediately
      if (navigator.onLine && this.authToken) {
        await this.uploadSyncItem(syncItem);
        await this.offlineCache.markSyncItemCompleted(syncItem.id);
      }
      
      return { success: true };
    } catch (error) {
      console.error('Force sync failed:', error);
      return { success: false, error: error.message };
    }
  }

  async getSyncStatus() {
    try {
      let pendingItems = [];
      let lastSyncTime = null;
      
      if (this.offlineCache) {
        try {
          pendingItems = await this.offlineCache.getPendingSyncItems();
          lastSyncTime = await this.getLastSyncTime();
        } catch (error) {
          console.warn('Error getting sync data:', error);
        }
      }
      
      return {
        isOnline: navigator.onLine,
        isAuthenticated: !!this.authToken,
        isSyncing: this.syncInProgress,
        syncInProgress: this.syncInProgress,
        pendingItems: pendingItems,
        pendingSyncCount: pendingItems.length,
        lastSyncTime,
        periodicSyncActive: !!this.syncInterval
      };
    } catch (error) {
      console.error('Error in getSyncStatus:', error);
      return {
        isOnline: navigator.onLine,
        isAuthenticated: !!this.authToken,
        isSyncing: false,
        syncInProgress: false,
        pendingItems: [],
        pendingSyncCount: 0,
        lastSyncTime: null,
        periodicSyncActive: false
      };
    }
  }

  // Helper method to check online status in service worker context
  async isOnline() {
    // In service worker context, we don't have navigator.onLine
    // Instead, we can try a simple network request to check connectivity
    if (typeof navigator !== 'undefined' && 'onLine' in navigator) {
      return navigator.onLine;
    }
    
    // Fallback for service worker: try to make a simple request
    try {
      const response = await fetch(this.apiUrl + '/health', {
        method: 'HEAD',
        cache: 'no-cache',
        signal: AbortSignal.timeout(5000) // 5 second timeout
      });
      return response.ok;
    } catch (error) {
      console.log('Network check failed, assuming offline:', error.message);
      return false;
    }
  }

  destroy() {
    this.stopPeriodicSync();
    if (typeof window !== 'undefined') {
      window.removeEventListener('online', this.handleOnlineEvent);
      window.removeEventListener('offline', this.handleOfflineEvent);
    }
  }
}

// Export for both service worker and regular context
if (typeof self !== 'undefined') {
  self.PassQSyncManager = PassQSyncManager;
}
if (typeof window !== 'undefined') {
  window.PassQSyncManager = PassQSyncManager;
}