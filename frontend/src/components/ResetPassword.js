import React, { useState, useEffect } from 'react';
import { authAPI } from '../services/api';

function ResetPassword({ token, onSuccess, onBack }) {
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);

  useEffect(() => {
    if (!token) {
      setError('Invalid or missing reset token');
    }
  }, [token]);

  const validatePassword = (pwd) => {
    if (pwd.length < 8) {
      return 'Password must be at least 8 characters long';
    }
    if (!/(?=.*[a-z])/.test(pwd)) {
      return 'Password must contain at least one lowercase letter';
    }
    if (!/(?=.*[A-Z])/.test(pwd)) {
      return 'Password must contain at least one uppercase letter';
    }
    if (!/(?=.*\d)/.test(pwd)) {
      return 'Password must contain at least one number';
    }
    return null;
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    if (!password || !confirmPassword) {
      setError('Both password fields are required');
      return;
    }

    const passwordError = validatePassword(password);
    if (passwordError) {
      setError(passwordError);
      return;
    }

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    if (!token) {
      setError('Invalid reset token');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      await authAPI.confirmPasswordReset(token, password);
      onSuccess && onSuccess();
    } catch (error) {
      console.error('Password reset failed:', error);
      setError(error.message || 'Failed to reset password. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-bold text-center uppercase tracking-wider">Set New Password</h2>
      <p className="text-sm text-gray-600 text-center">
        Enter your new password below.
      </p>
      
      {error && (
        <div className="cartoon-border bg-red-100 text-red-800 p-3 rounded text-sm font-medium">
          {error}
        </div>
      )}
      
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <label htmlFor="password" className="block text-sm font-bold uppercase tracking-wide">
            New Password
          </label>
          <div className="relative">
            <input
              id="password"
              type={showPassword ? 'text' : 'password'}
              placeholder="Enter new password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              disabled={isLoading}
              className="w-full cartoon-border cartoon-shadow px-3 py-2 pr-10 rounded bg-white text-black placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-black disabled:opacity-50"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-500 hover:text-black"
              disabled={isLoading}
            >
              {showPassword ? '🙈' : '👁️'}
            </button>
          </div>
          <div className="text-xs text-gray-600 space-y-1">
            <p>Password must contain:</p>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>At least 8 characters</li>
              <li>One uppercase letter</li>
              <li>One lowercase letter</li>
              <li>One number</li>
            </ul>
          </div>
        </div>
        
        <div className="space-y-2">
          <label htmlFor="confirmPassword" className="block text-sm font-bold uppercase tracking-wide">
            Confirm New Password
          </label>
          <div className="relative">
            <input
              id="confirmPassword"
              type={showConfirmPassword ? 'text' : 'password'}
              placeholder="Confirm new password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              required
              disabled={isLoading}
              className="w-full cartoon-border cartoon-shadow px-3 py-2 pr-10 rounded bg-white text-black placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-black disabled:opacity-50"
            />
            <button
              type="button"
              onClick={() => setShowConfirmPassword(!showConfirmPassword)}
              className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-500 hover:text-black"
              disabled={isLoading}
            >
              {showConfirmPassword ? '🙈' : '👁️'}
            </button>
          </div>
        </div>
        
        <button 
          type="submit" 
          className="w-full cartoon-btn cartoon-btn-primary py-3 text-sm font-bold uppercase tracking-wider" 
          disabled={isLoading || !token}
        >
          {isLoading ? 'Resetting...' : 'Reset Password'}
        </button>
      </form>
      
      <div className="text-center text-sm">
        <button 
          type="button" 
          onClick={onBack} 
          className="font-bold text-black hover:underline uppercase tracking-wide"
        >
          ← Back to Login
        </button>
      </div>
    </div>
  );
}

export default ResetPassword;