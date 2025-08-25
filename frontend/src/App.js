import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Dashboard from './components/Dashboard';
import Logo from './components/Logo';
import { Icons } from './components/Icons';
import AccountSettingsModal from './components/AccountSettingsModal';
import sessionManager from './utils/sessionManager';
import './App.css';

import AuthPages from './components/AuthPages';
import SSOCallback from './components/SSOCallback';

function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [username, setUsername] = useState('');
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);
  const [showLockWarning, setShowLockWarning] = useState(false);

  useEffect(() => {
    // Check authentication status by making a request to a protected endpoint
    const checkAuthStatus = async () => {
      try {
        const response = await fetch('/auth/verify', {
          method: 'GET',
          credentials: 'include', // Include cookies in the request
          headers: {
            'Content-Type': 'application/json',
          },
        });
        
        if (response.ok) {
          const data = await response.json();
          if (data.success && data.data) {
            setIsAuthenticated(true);
            setUsername(data.data.username);
            
            // Initialize session manager for authenticated users
            sessionManager.setLockCallback(() => {
              handleLogout();
            });
            
            sessionManager.setWarningCallback(() => {
              setShowLockWarning(true);
            });
            
            sessionManager.unlockSession();
          }
        }
      } catch (error) {
        console.log('Authentication check failed:', error);
        // User is not authenticated, which is fine
      }
    };
    
    checkAuthStatus();
    
    // Cleanup session manager on unmount
    return () => {
      sessionManager.destroy();
    };
  }, []);

  const handleLoginSuccess = (user) => {
    // No need to store token in localStorage anymore - it's in HttpOnly cookie
    setIsAuthenticated(true);
    setUsername(user);
    
    // Initialize session manager for newly authenticated users
    sessionManager.setLockCallback(() => {
      handleLogout();
    });
    
    sessionManager.setWarningCallback(() => {
      setShowLockWarning(true);
    });
    
    sessionManager.unlockSession();
  };

  const handleLogout = async () => {
    // Call logout endpoint to clear the HttpOnly cookie
    try {
      await fetch('/logout', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });
    } catch (error) {
      console.log('Logout request failed:', error);
    }
    
    // Clear session manager
    sessionManager.destroy();
    setIsAuthenticated(false);
    setUsername('');
    setShowLockWarning(false);
  };
  
  const dismissWarning = () => {
    setShowLockWarning(false);
    // Reset the timer to give user more time
    sessionManager.resetTimer();
  };

  return (
    <Router>
      {isAuthenticated ? (
        <div className="h-screen bg-white text-black flex flex-col">
          {/* Auto-lock warning notification */}
          {showLockWarning && (
            <div className="fixed top-4 right-4 bg-yellow-100 border border-yellow-400 text-yellow-700 px-4 py-3 rounded shadow-lg z-50">
              <div className="flex items-center">
                <div className="mr-3">
                  <Icons.AlertTriangle size={24} className="text-yellow-600" />
                </div>
                <div className="flex-1">
                  <p className="font-medium">
                    Your vault will automatically lock in 1 minute due to inactivity.
                  </p>
                  <div className="flex space-x-2 mt-2">
                    <button
                      onClick={() => {
                        sessionManager.resetTimer();
                        setShowLockWarning(false);
                      }}
                      className="bg-yellow-500 hover:bg-yellow-600 text-white px-3 py-1 rounded text-sm"
                    >
                      Stay Active
                    </button>
                    <button
                      onClick={() => {
                        sessionManager.lockSession();
                        setShowLockWarning(false);
                      }}
                      className="bg-gray-500 hover:bg-gray-600 text-white px-3 py-1 rounded text-sm"
                    >
                      Lock Now
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}
          
          <header className="flex items-center justify-between px-4 py-3 md:px-8 md:py-4 border-b-4 border-black sticky top-0 z-50 bg-white">
            <div className="flex items-center gap-3">
              <Logo size={44} />
              <h1 className="text-2xl font-extrabold tracking-widest select-none">PassQ</h1>
            </div>
            <div className="flex items-center gap-3">
              <span className="uppercase text-xs md:text-sm">Welcome, {username}!</span>
              <button 
                className="cartoon-btn cartoon-shadow px-3 py-1 text-xs md:text-sm flex items-center gap-1" 
                onClick={() => setIsSettingsModalOpen(true)}
              >
                <Icons.Settings size={14} />
                <span>Settings</span>
              </button>
              <button className="cartoon-btn cartoon-shadow px-3 py-1 text-xs md:text-sm" onClick={handleLogout}>
                Logout
              </button>
            </div>
          </header>
          <main className="flex-1 w-full">
            <Dashboard />
          </main>
          <AccountSettingsModal 
            isOpen={isSettingsModalOpen} 
            onClose={() => setIsSettingsModalOpen(false)} 
          />
        </div>
      ) : (
        <Routes>
          <Route path="/auth/sso/microsoft/callback" element={<SSOCallback />} />
          <Route path="/auth/sso/google/callback" element={<SSOCallback />} />
          <Route path="/forgot-password" element={<AuthPages onLoginSuccess={handleLoginSuccess} />} />
          <Route path="/reset-password" element={<AuthPages onLoginSuccess={handleLoginSuccess} />} />
          <Route path="/*" element={<AuthPages onLoginSuccess={handleLoginSuccess} />} />
        </Routes>
      )}
    </Router>
  );
}

export default App;