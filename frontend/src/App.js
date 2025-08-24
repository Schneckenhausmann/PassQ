import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Dashboard from './components/Dashboard';
import Logo from './components/Logo';
import { Icons } from './components/Icons';
import PasswordChangeModal from './components/PasswordChangeModal';
import './App.css';

import AuthPages from './components/AuthPages';

function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [username, setUsername] = useState('');
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);

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
          }
        }
      } catch (error) {
        console.log('Authentication check failed:', error);
        // User is not authenticated, which is fine
      }
    };
    
    checkAuthStatus();
  }, []);

  const handleLoginSuccess = (user) => {
    // No need to store token in localStorage anymore - it's in HttpOnly cookie
    setIsAuthenticated(true);
    setUsername(user);
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
    setIsAuthenticated(false);
    setUsername('');
  };

  return (
    <Router>
      {isAuthenticated ? (
        <div className="h-screen bg-white text-black flex flex-col">
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
                <span>Change password</span>
              </button>
              <button className="cartoon-btn cartoon-shadow px-3 py-1 text-xs md:text-sm" onClick={handleLogout}>
                Logout
              </button>
            </div>
          </header>
          <main className="flex-1 w-full">
            <Dashboard />
          </main>
          <PasswordChangeModal 
            isOpen={isSettingsModalOpen} 
            onClose={() => setIsSettingsModalOpen(false)} 
          />
        </div>
      ) : (
        <Routes>
          <Route path="/forgot-password" element={<AuthPages onLoginSuccess={handleLoginSuccess} />} />
          <Route path="/reset-password" element={<AuthPages onLoginSuccess={handleLoginSuccess} />} />
          <Route path="/*" element={<AuthPages onLoginSuccess={handleLoginSuccess} />} />
        </Routes>
      )}
    </Router>
  );
}

export default App;