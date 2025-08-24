import React, { useState } from 'react';
import { authAPI } from '../services/api';

function ForgotPassword({ onBack }) {
  const [email, setEmail] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [step, setStep] = useState('request'); // 'request' or 'success'

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    if (!email) {
      setError('Email is required');
      return;
    }

    setIsLoading(true);
    setError('');
    setMessage('');

    try {
      await authAPI.requestPasswordReset(email);
      setStep('success');
      setMessage('Password reset instructions have been sent to your email.');
    } catch (error) {
      console.error('Password reset request failed:', error);
      setError(error.message || 'Failed to send reset email. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  if (step === 'success') {
    return (
      <div className="space-y-4">
        <h2 className="text-xl font-bold text-center uppercase tracking-wider">Check Your Email</h2>
        <div className="cartoon-border bg-green-100 text-green-800 p-4 rounded text-sm font-medium text-center">
          <p className="mb-2">📧 Password reset instructions sent!</p>
          <p>{message}</p>
        </div>
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

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-bold text-center uppercase tracking-wider">Reset Password</h2>
      <p className="text-sm text-gray-600 text-center">
        Enter your username/email and we'll send you instructions to reset your password.
      </p>
      
      {error && (
        <div className="cartoon-border bg-red-100 text-red-800 p-3 rounded text-sm font-medium">
          {error}
        </div>
      )}
      
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <label htmlFor="email" className="block text-sm font-bold uppercase tracking-wide">
            Username/Email
          </label>
          <input
            id="email"
            type="text"
            placeholder="Enter your username or email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
            disabled={isLoading}
            className="w-full cartoon-border cartoon-shadow px-3 py-2 rounded bg-white text-black placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-black disabled:opacity-50"
          />
        </div>
        
        <button 
          type="submit" 
          className="w-full cartoon-btn cartoon-btn-primary py-3 text-sm font-bold uppercase tracking-wider" 
          disabled={isLoading}
        >
          {isLoading ? 'Sending...' : 'Send Reset Instructions'}
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

export default ForgotPassword;