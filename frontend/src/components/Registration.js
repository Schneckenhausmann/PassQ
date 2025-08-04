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
      <div className="registration-container">
        <h2>Registration Successful!</h2>
        <p>You will be redirected to the login page shortly...</p>
      </div>
    );
  }

  return (
    <div className="auth-form">
      <h2>Register</h2>
      {error && <div className="error">{error}</div>}
      
      <form onSubmit={handleRegister}>
        <div className="form-group">
          <label htmlFor="reg-username">Username</label>
          <input
            id="reg-username"
            type="text"
            placeholder="Choose a username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            disabled={success}
            required
          />
        </div>
        
        <div className="form-group">
          <label htmlFor="reg-password">Password</label>
          <input
            id="reg-password"
            type="password"
            placeholder="Create a strong password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={success}
            required
          />
        </div>
        
        <div className="form-group">
          <label htmlFor="reg-confirm-password">Confirm Password</label>
          <input
            id="reg-confirm-password"
            type="password"
            placeholder="Confirm your password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            disabled={success}
            required
          />
        </div>
        
        <button type="submit" className="btn-primary" disabled={success}>
          Register
        </button>
      </form>
      <div className="switch-link">
        Already have an account?
        <button type="button" onClick={onSwitch}>
          Login
        </button>
      </div>
    </div>
  );
}

export default Registration;