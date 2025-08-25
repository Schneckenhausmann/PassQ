import React, { useState, useEffect } from 'react';
import { Icons } from './Icons';
import { authAPI } from '../services/api';
import PasswordChangeModal from './PasswordChangeModal';

function AccountSettingsModal({ isOpen, onClose }) {
  const [activeTab, setActiveTab] = useState('general');
  const [linkedAccounts, setLinkedAccounts] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [isPasswordModalOpen, setIsPasswordModalOpen] = useState(false);
  
  // Auto-lock settings state
  const [autoLockTimeout, setAutoLockTimeout] = useState('15');
  const [lockOnRefresh, setLockOnRefresh] = useState(true);

  useEffect(() => {
    if (isOpen) {
      loadLinkedAccounts();
      loadAutoLockSettings();
    }
  }, [isOpen]);

  const loadAutoLockSettings = () => {
    const savedTimeout = localStorage.getItem('autoLockTimeout') || '15';
    const savedLockOnRefresh = localStorage.getItem('lockOnRefresh') !== 'false';
    setAutoLockTimeout(savedTimeout);
    setLockOnRefresh(savedLockOnRefresh);
  };

  const saveAutoLockSettings = () => {
    localStorage.setItem('autoLockTimeout', autoLockTimeout);
    localStorage.setItem('lockOnRefresh', lockOnRefresh.toString());
    setSuccess('Auto-lock settings saved successfully');
    
    // Dispatch custom event to notify other components
    window.dispatchEvent(new CustomEvent('autoLockSettingsChanged', {
      detail: { timeout: autoLockTimeout, lockOnRefresh }
    }));
  };

  const loadLinkedAccounts = async () => {
    try {
      setLoading(true);
      setError(''); // Clear any previous errors
      console.log('Loading linked accounts...');
      const response = await authAPI.getLinkedAccounts();
      console.log('Linked accounts response:', response);
      setLinkedAccounts(response.data || []);
    } catch (err) {
      console.error('Error loading linked accounts:', err);
      console.error('Error details - status:', err.status, 'message:', err.message);
      // Handle authentication errors gracefully
      if (err.status === 401 || err.message?.includes('401') || err.message?.includes('Unauthorized')) {
        setError('Authentication required. Please refresh the page and try again.');
        console.log('Setting auth error message');
      } else {
        setError('Failed to load linked accounts: ' + (err.message || 'Unknown error'));
        console.log('Setting generic error message');
      }
      // Set empty array to prevent rendering issues
      setLinkedAccounts([]);
    } finally {
      setLoading(false);
      console.log('Loading finished');
    }
  };

  const handleLinkAccount = async (provider) => {
    try {
      setError('');
      setSuccess('');
      await authAPI.ssoLogin(provider, true); // true for linking mode
    } catch (err) {
      setError(`Failed to link ${provider} account`);
    }
  };

  const handleUnlinkAccount = async (provider) => {
    try {
      setError('');
      setSuccess('');
      setLoading(true);
      
      // Find the account ID for this provider
      const account = linkedAccounts.find(acc => acc.provider === provider);
      if (!account) {
        throw new Error('Account not found');
      }
      
      await authAPI.unlinkAccount(account.id);
      setSuccess(`${provider} account unlinked successfully`);
      await loadLinkedAccounts();
    } catch (err) {
      setError(`Failed to unlink ${provider} account`);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setError('');
      setSuccess('');
      setActiveTab('general');
      onClose();
    }
  };

  const isAccountLinked = (provider) => {
    return linkedAccounts.some(account => account.provider === provider);
  };

  if (!isOpen) return null;

  // Add error boundary to prevent white screen
  try {
    return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white cartoon-border shadow-2xl cartoon-shadow rounded-lg w-full max-w-2xl max-h-[90vh] overflow-hidden">
        <div className="flex items-center justify-between p-4 border-b border-black/20">
          <h2 className="text-lg font-bold">Account Settings</h2>
          <button 
            onClick={handleClose} 
            className="cartoon-btn p-1"
            disabled={loading}
          >
            <Icons.X size={16} />
          </button>
        </div>
        
        <div className="flex h-[500px]">
          {/* Sidebar */}
          <div className="w-48 border-r border-black/20 bg-gray-50">
            <nav className="p-4">
              <button
                onClick={() => setActiveTab('general')}
                className={`w-full text-left px-3 py-2 rounded mb-2 transition-colors ${
                  activeTab === 'general' 
                    ? 'bg-blue-100 text-blue-700 font-semibold' 
                    : 'hover:bg-gray-100'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Icons.User size={16} />
                  General
                </div>
              </button>
              <button
                onClick={() => setActiveTab('security')}
                className={`w-full text-left px-3 py-2 rounded mb-2 transition-colors ${
                  activeTab === 'security' 
                    ? 'bg-blue-100 text-blue-700 font-semibold' 
                    : 'hover:bg-gray-100'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Icons.Shield size={16} />
                  Security
                </div>
              </button>
              <button
                onClick={() => setActiveTab('accounts')}
                className={`w-full text-left px-3 py-2 rounded transition-colors ${
                  activeTab === 'accounts' 
                    ? 'bg-blue-100 text-blue-700 font-semibold' 
                    : 'hover:bg-gray-100'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Icons.Link size={16} />
                  Linked Accounts
                </div>
              </button>
            </nav>
          </div>

          {/* Content */}
          <div className="flex-1 p-6 overflow-y-auto">
            {error && (
              <div className="bg-red-50 border border-red-200 text-red-700 px-3 py-2 rounded text-sm mb-4">
                {error}
              </div>
            )}
            
            {success && (
              <div className="bg-green-50 border border-green-200 text-green-700 px-3 py-2 rounded text-sm mb-4">
                {success}
              </div>
            )}

            {activeTab === 'general' && (
              <div>
                <h3 className="text-lg font-semibold mb-4">General Settings</h3>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-1">Username</label>
                    <input
                      type="text"
                      value={localStorage.getItem('username') || ''}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50"
                      disabled
                    />
                    <p className="text-xs text-gray-500 mt-1">Username cannot be changed</p>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'security' && (
              <div>
                <h3 className="text-lg font-semibold mb-4">Security Settings</h3>
                <div className="space-y-4">
                  {/* Auto-lock Settings */}
                  <div className="border border-gray-200 rounded-lg p-4">
                    <h4 className="font-medium mb-3">Vault Auto-Lock</h4>
                    
                    <div className="space-y-4">
                      <div>
                        <label className="block text-sm font-medium mb-2">Auto-lock timeout</label>
                        <select
                          value={autoLockTimeout}
                          onChange={(e) => setAutoLockTimeout(e.target.value)}
                          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                          <option value="5">5 minutes</option>
                          <option value="15">15 minutes</option>
                          <option value="30">30 minutes</option>
                          <option value="60">1 hour</option>
                          <option value="120">2 hours</option>
                          <option value="240">4 hours</option>
                          <option value="480">8 hours</option>
                          <option value="never">Never</option>
                        </select>
                        <p className="text-xs text-gray-500 mt-1">Vault will automatically lock after this period of inactivity</p>
                      </div>
                      
                      <div className="flex items-center">
                        <input
                          type="checkbox"
                          id="lockOnRefresh"
                          checked={lockOnRefresh}
                          onChange={(e) => setLockOnRefresh(e.target.checked)}
                          className="mr-2 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                        />
                        <label htmlFor="lockOnRefresh" className="text-sm font-medium">
                          Lock vault on page refresh
                        </label>
                      </div>
                      <p className="text-xs text-gray-500">When enabled, any page refresh or navigation requires re-authentication</p>
                      
                      <button
                        onClick={saveAutoLockSettings}
                        className="cartoon-btn cartoon-btn-primary"
                      >
                        Save Auto-lock Settings
                      </button>
                    </div>
                  </div>
                  
                  {/* Password Settings */}
                  <div className="border border-gray-200 rounded-lg p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <h4 className="font-medium">Password</h4>
                        <p className="text-sm text-gray-600">Change your account password</p>
                      </div>
                      <button
                        onClick={() => setIsPasswordModalOpen(true)}
                        className="cartoon-btn cartoon-btn-primary"
                      >
                        Change Password
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'accounts' && (
              <div>
                <h3 className="text-lg font-semibold mb-4">Linked Accounts</h3>
                <p className="text-sm text-gray-600 mb-6">
                  Link your social accounts to enable single sign-on and easier access to your vault.
                </p>
                
                {loading && (
                  <div className="flex items-center justify-center py-8">
                    <div className="text-sm text-gray-500">Loading linked accounts...</div>
                  </div>
                )}
                
                {!loading && (
                  <div className="space-y-4">
                    {/* Microsoft Account */}
                    <div className="border border-gray-200 rounded-lg p-4">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <div className="w-10 h-10 bg-blue-600 rounded-lg flex items-center justify-center">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="white">
                              <path d="M11.4 24H0V12.6h11.4V24zM24 24H12.6V12.6H24V24zM11.4 11.4H0V0h11.4v11.4zM24 11.4H12.6V0H24v11.4z"/>
                            </svg>
                          </div>
                          <div>
                            <h4 className="font-medium">Microsoft</h4>
                            <p className="text-sm text-gray-600">
                              {isAccountLinked('microsoft') 
                                ? 'Connected to your Microsoft account' 
                                : 'Connect your Microsoft account for easy sign-in'
                              }
                            </p>
                          </div>
                        </div>
                        <div>
                          {isAccountLinked('microsoft') ? (
                            <button
                              onClick={() => handleUnlinkAccount('microsoft')}
                              className="cartoon-btn text-red-600 border-red-200 hover:bg-red-50"
                              disabled={loading}
                            >
                              Unlink
                            </button>
                          ) : (
                            <button
                              onClick={() => handleLinkAccount('microsoft')}
                              className="cartoon-btn cartoon-btn-primary"
                              disabled={loading}
                            >
                              Link Account
                            </button>
                          )}
                        </div>
                      </div>
                    </div>

                    {/* Google Account */}
                    <div className="border border-gray-200 rounded-lg p-4">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <div className="w-10 h-10 bg-white border border-gray-300 rounded-lg flex items-center justify-center">
                            <svg width="20" height="20" viewBox="0 0 24 24">
                              <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                              <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                              <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                              <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                            </svg>
                          </div>
                          <div>
                            <h4 className="font-medium">Google</h4>
                            <p className="text-sm text-gray-600">
                              {isAccountLinked('google') 
                                ? 'Connected to your Google account' 
                                : 'Connect your Google account for easy sign-in'
                              }
                            </p>
                          </div>
                        </div>
                        <div>
                          {isAccountLinked('google') ? (
                            <button
                              onClick={() => handleUnlinkAccount('google')}
                              className="cartoon-btn text-red-600 border-red-200 hover:bg-red-50"
                              disabled={loading}
                            >
                              Unlink
                            </button>
                          ) : (
                            <button
                              onClick={() => handleLinkAccount('google')}
                              className="cartoon-btn cartoon-btn-primary"
                              disabled={loading}
                            >
                              Link Account
                            </button>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
      
      <PasswordChangeModal 
        isOpen={isPasswordModalOpen} 
        onClose={() => setIsPasswordModalOpen(false)} 
      />
    </div>
  );
  } catch (error) {
    console.error('Error rendering AccountSettingsModal:', error);
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
        <div className="bg-white cartoon-border shadow-2xl cartoon-shadow rounded-lg w-full max-w-md p-6">
          <h2 className="text-lg font-bold mb-4">Settings Unavailable</h2>
          <p className="text-gray-600 mb-4">There was an error loading the settings page. Please refresh and try again.</p>
          <button onClick={onClose} className="cartoon-btn cartoon-btn-primary w-full">
            Close
          </button>
        </div>
      </div>
    );
  }
}

export default AccountSettingsModal;