import React, { useEffect, useState } from 'react';
import { authAPI } from '../services/api';

function SSOCallback() {
  const [status, setStatus] = useState('processing');
  const [error, setError] = useState('');

  useEffect(() => {
    const handleCallback = async () => {
      try {
        const urlParams = new URLSearchParams(window.location.search);
        const code = urlParams.get('code');
        const state = urlParams.get('state');
        const provider = window.location.pathname.split('/').pop(); // Extract provider from URL

        if (!code) {
          throw new Error('Authorization code not found');
        }

        // Call the backend callback endpoint
        const response = await authAPI.ssoCallback(provider, code, state);
        
        if (response.success) {
          setStatus('success');
          // Redirect to main app after successful authentication
          setTimeout(() => {
            window.location.href = '/';
          }, 2000);
        } else {
          throw new Error(response.message || 'Authentication failed');
        }
      } catch (error) {
        console.error('SSO callback error:', error);
        setError(error.message || 'Authentication failed');
        setStatus('error');
      }
    };

    handleCallback();
  }, []);

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center p-4">
      <div className="cartoon-border cartoon-shadow bg-white p-8 rounded-lg max-w-md w-full text-center">
        {status === 'processing' && (
          <div className="space-y-4">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-black mx-auto"></div>
            <h2 className="text-xl font-bold uppercase tracking-wider">Authenticating...</h2>
            <p className="text-gray-600">Please wait while we complete your sign-in.</p>
          </div>
        )}
        
        {status === 'success' && (
          <div className="space-y-4">
            <div className="text-green-600">
              <svg className="w-12 h-12 mx-auto" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
              </svg>
            </div>
            <h2 className="text-xl font-bold uppercase tracking-wider text-green-600">Success!</h2>
            <p className="text-gray-600">Authentication successful. Redirecting...</p>
          </div>
        )}
        
        {status === 'error' && (
          <div className="space-y-4">
            <div className="text-red-600">
              <svg className="w-12 h-12 mx-auto" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
            </div>
            <h2 className="text-xl font-bold uppercase tracking-wider text-red-600">Authentication Failed</h2>
            <p className="text-red-600 text-sm">{error}</p>
            <button 
              onClick={() => window.location.href = '/'}
              className="cartoon-btn cartoon-btn-primary py-2 px-4 text-sm font-bold uppercase tracking-wider"
            >
              Back to Login
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export default SSOCallback;