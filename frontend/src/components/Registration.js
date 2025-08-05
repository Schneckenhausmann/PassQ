import React, { useState } from 'react';
import axios from 'axios';

function Registration({ onSwitch, onRegistrationSuccess }) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [success, setSuccess] = useState(false);

  const handleRegister = async (e) => {
    e.preventDefault();
    
    // Validation
    if (!username || !password) {
      setError('Username and password are required');
      return;
    }

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    if (password.length < 8) {
      setError('Password must be at least 8 characters long');
      return;
    }

    try {
      setError('');
      
      const response = await axios.post('/register', {
        username,
        password
      });

      if (response.data.success) {
        setSuccess(true);
        // Call the success callback to switch to login
        setTimeout(() => {
          if (onRegistrationSuccess) {
            onRegistrationSuccess();
          }
        }, 2000);
      } else {
        setError(response.data.message || 'Registration failed');
      }
    } catch (error) {
      console.error('Registration error:', error);
      setError(error.response?.data?.message || 'Registration failed. Please try again.');
    }
  };

  if (success) {
    return (
      <div className="text-center space-y-4">
        <h2 className="text-xl font-bold uppercase tracking-wider text-green-700">Registration Successful!</h2>
        <p className="text-sm text-gray-600">You will be redirected to the login page shortly...</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-bold text-center uppercase tracking-wider">Register</h2>
      {error && <div className="cartoon-border bg-red-100 text-red-800 p-3 rounded text-sm font-medium">{error}</div>}
      
      <form onSubmit={handleRegister} className="space-y-4">
        <div className="space-y-2">
          <label htmlFor="reg-username" className="block text-sm font-bold uppercase tracking-wide">Username</label>
          <input
            id="reg-username"
            type="text"
            placeholder="Choose a username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            disabled={success}
            required
            className="w-full cartoon-border cartoon-shadow px-3 py-2 rounded bg-white text-black placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-black disabled:opacity-50"
          />
        </div>
        
        <div className="space-y-2">
          <label htmlFor="reg-password" className="block text-sm font-bold uppercase tracking-wide">Password</label>
          <input
            id="reg-password"
            type="password"
            placeholder="Create a strong password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={success}
            required
            className="w-full cartoon-border cartoon-shadow px-3 py-2 rounded bg-white text-black placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-black disabled:opacity-50"
          />
        </div>
        
        <div className="space-y-2">
          <label htmlFor="reg-confirm-password" className="block text-sm font-bold uppercase tracking-wide">Confirm Password</label>
          <input
            id="reg-confirm-password"
            type="password"
            placeholder="Confirm your password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            disabled={success}
            required
            className="w-full cartoon-border cartoon-shadow px-3 py-2 rounded bg-white text-black placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-black disabled:opacity-50"
          />
        </div>
        
        <button type="submit" className="w-full cartoon-btn cartoon-btn-primary py-3 text-sm font-bold uppercase tracking-wider" disabled={success}>
          Register
        </button>
      </form>
      <div className="text-center text-sm">
        <span className="text-gray-600">Already have an account? </span>
        <button type="button" onClick={onSwitch} className="font-bold text-black hover:underline uppercase tracking-wide">
          Login
        </button>
      </div>
    </div>
  );
}

export default Registration;