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