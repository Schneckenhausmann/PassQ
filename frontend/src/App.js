import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router } from 'react-router-dom';
import Login from './components/Login';
import Registration from './components/Registration';
import Dashboard from './components/Dashboard';
import Logo from './components/Logo';
import { Icons } from './components/Icons';
import PasswordChangeModal from './components/PasswordChangeModal';
import './App.css';

function App() {
  const [showLogin, setShowLogin] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [username, setUsername] = useState('');
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);

  useEffect(() => {
    const token = localStorage.getItem('authToken');
    const storedUsername = localStorage.getItem('username');
    if (token && storedUsername) {
      setIsAuthenticated(true);
      setUsername(storedUsername);
    }
  }, []);

  const handleLoginSuccess = (token, user) => {
    localStorage.setItem('authToken', token);
    localStorage.setItem('username', user);
    setIsAuthenticated(true);
    setUsername(user);
  };

  const handleLogout = () => {
    localStorage.removeItem('authToken');
    localStorage.removeItem('username');
    setIsAuthenticated(false);
    setUsername('');
    setShowLogin(true);
  };

  const handleRegistrationSuccess = () => {
    setShowLogin(true);
  };

  if (isAuthenticated) {
    return (
      <Router>
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
      </Router>
    );
  }

  return (
    <Router>
      <div className="min-h-screen bg-white text-black flex flex-col items-center justify-center">
        <header className="flex flex-col items-center gap-2 py-6">
          <Logo size={60} />
          <h1 className="text-3xl font-extrabold tracking-widest select-none">PassQ</h1>
        </header>
        <div className="w-full max-w-xs md:max-w-sm cartoon-border cartoon-shadow bg-white rounded-lg p-6 mt-2">
          {showLogin ? (
            <Login onSwitch={() => setShowLogin(false)} onLoginSuccess={handleLoginSuccess} />
          ) : (
            <Registration onSwitch={() => setShowLogin(true)} onRegistrationSuccess={handleRegistrationSuccess} />
          )}
        </div>
      </div>
    </Router>
  );
}

export default App;