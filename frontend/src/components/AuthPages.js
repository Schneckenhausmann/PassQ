import React, { useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import Login from './Login';
import Registration from './Registration';
import ForgotPassword from './ForgotPassword';
import ResetPassword from './ResetPassword';
import Logo from './Logo';

function AuthPages({ onLoginSuccess }) {
  const [showLogin, setShowLogin] = useState(true);
  const location = useLocation();
  const navigate = useNavigate();

  // Extract token from URL for password reset
  const urlParams = new URLSearchParams(location.search);
  const resetToken = urlParams.get('token');

  const handleBackToLogin = () => {
    setShowLogin(true);
    navigate('/');
  };

  const handleResetSuccess = () => {
    alert('Password reset successful! Please log in with your new password.');
    handleBackToLogin();
  };

  const renderAuthComponent = () => {
    if (location.pathname === '/forgot-password') {
      return <ForgotPassword onBack={handleBackToLogin} />;
    }
    
    if (location.pathname === '/reset-password') {
      return (
        <ResetPassword 
          token={resetToken} 
          onSuccess={handleResetSuccess}
          onBack={handleBackToLogin}
        />
      );
    }
    
    // Default to login/registration flow
    if (showLogin) {
      return (
        <Login 
          onSwitch={() => setShowLogin(false)} 
          onLoginSuccess={onLoginSuccess} 
        />
      );
    } else {
      return (
        <Registration 
          onSwitch={() => setShowLogin(true)}
          onRegistrationSuccess={() => setShowLogin(true)}
        />
      );
    }
  };

  return (
    <div className="min-h-screen bg-white text-black flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="flex items-center justify-center gap-3 mb-4">
            <Logo size={64} />
            <h1 className="text-4xl font-extrabold tracking-widest select-none">PassQ</h1>
          </div>
          <p className="text-sm text-gray-600 uppercase tracking-wide">Secure Password Manager</p>
        </div>
        <div className="cartoon-border cartoon-shadow bg-white p-8 rounded-lg">
          {renderAuthComponent()}
        </div>
      </div>
    </div>
  );
}

export default AuthPages;