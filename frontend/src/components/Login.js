import React, { useState } from 'react';
import { authAPI } from '../services/api';

function Login({ onSwitch, onLoginSuccess }) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleLogin = async (e) => {
    e.preventDefault();
    
    if (!username || !password) {
      setError('Username/email and password are required');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      const data = await authAPI.login({
        username,
        password
      });
      
      if (data.success) {
        console.log('Login successful:', data);
        // Call the success callback to update parent state
        if (onLoginSuccess) {
          onLoginSuccess(username); // No token needed, it's in HttpOnly cookie
        }
      } else {
        setError(data.message || 'Login failed');
      }
    } catch (error) {
      console.error('Login failed:', error);
      setError(error.message || 'Login failed. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-bold text-center uppercase tracking-wider">Login</h2>
      {error && <div className="cartoon-border bg-red-100 text-red-800 p-3 rounded text-sm font-medium">{error}</div>}
      <form onSubmit={handleLogin} className="space-y-4">
        <div className="space-y-2">
          <label htmlFor="username" className="block text-sm font-bold uppercase tracking-wide">Username or Email</label>
          <input
            id="username"
            type="text"
            placeholder="Enter your username or email"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
            disabled={isLoading}
            className="w-full cartoon-border cartoon-shadow px-3 py-2 rounded bg-white text-black placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-black disabled:opacity-50"
          />
        </div>
        <div className="space-y-2">
          <label htmlFor="password" className="block text-sm font-bold uppercase tracking-wide">Password</label>
          <input
            id="password"
            type="password"
            placeholder="Enter your password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            disabled={isLoading}
            className="w-full cartoon-border cartoon-shadow px-3 py-2 rounded bg-white text-black placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-black disabled:opacity-50"
          />
        </div>
        <button type="submit" className="w-full cartoon-btn cartoon-btn-primary py-3 text-sm font-bold uppercase tracking-wider" disabled={isLoading}>
          {isLoading ? 'Logging in...' : 'Login'}
        </button>
      </form>
      {/* SSO Login Options */}
      <div className="space-y-3">
        <div className="text-center text-sm font-bold uppercase tracking-wide text-gray-600">
          Or continue with
        </div>
        <div className="space-y-2">
          <button
            type="button"
            onClick={() => authAPI.ssoLogin('microsoft')}
            disabled={isLoading}
            className="w-full cartoon-border cartoon-shadow px-4 py-3 rounded bg-blue-600 text-white font-bold uppercase tracking-wider hover:bg-blue-700 disabled:opacity-50 flex items-center justify-center space-x-2"
          >
            <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
              <path d="M11.4 24H0V12.6h11.4V24zM24 24H12.6V12.6H24V24zM11.4 11.4H0V0h11.4v11.4zM24 11.4H12.6V0H24v11.4z"/>
            </svg>
            <span>Microsoft 365</span>
          </button>
          <button
            type="button"
            onClick={() => authAPI.ssoLogin('google')}
            disabled={isLoading}
            className="w-full cartoon-border cartoon-shadow px-4 py-3 rounded bg-red-600 text-white font-bold uppercase tracking-wider hover:bg-red-700 disabled:opacity-50 flex items-center justify-center space-x-2"
          >
            <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
              <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
              <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
              <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
              <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
            </svg>
            <span>Google</span>
          </button>
        </div>
      </div>

      <div className="text-center text-sm space-y-2">
        <div>
          <span className="text-gray-600">Don't have an account? </span>
          <button type="button" onClick={onSwitch} className="font-bold text-black hover:underline uppercase tracking-wide">
            Register
          </button>
        </div>
        <div>
          <button type="button" onClick={() => window.location.href = '/forgot-password'} className="font-bold text-black hover:underline uppercase tracking-wide text-xs">
            Forgot Password?
          </button>
        </div>
      </div>
    </div>
  );
}

export default Login;