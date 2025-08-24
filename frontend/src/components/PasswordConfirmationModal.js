import React, { useState } from 'react';
import { Icons } from './Icons';

function PasswordConfirmationModal({ isOpen, onClose, onConfirm, title = "Confirm Password", message = "Please enter your password to continue" }) {
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!password.trim()) {
      setError('Password is required');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      await onConfirm(password);
      // Reset form on success
      setPassword('');
      setShowPassword(false);
      onClose();
    } catch (err) {
      setError(err.message || 'Invalid password. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleClose = () => {
    setPassword('');
    setShowPassword(false);
    setError('');
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="cartoon-border bg-white w-full max-w-md">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-black/20">
          <h2 className="text-lg font-bold flex items-center gap-2">
            <Icons.Shield size={20} className="text-red-500" />
            {title}
          </h2>
          <button
            onClick={handleClose}
            className="p-1 hover:bg-gray-100 rounded transition-colors"
            disabled={isLoading}
          >
            <Icons.X size={20} />
          </button>
        </div>

        {/* Content */}
        <form onSubmit={handleSubmit} className="p-4">
          <div className="mb-4">
            <div className="flex items-center gap-2 mb-3 text-amber-600">
              <Icons.AlertTriangle size={18} />
              <span className="text-sm font-medium">Security Warning</span>
            </div>
            <p className="text-gray-700 text-sm mb-4">
              {message}
            </p>
            <p className="text-xs text-gray-500 mb-4">
              Your password will be verified to ensure secure access to your data.
            </p>
          </div>

          {/* Password Input */}
          <div className="mb-4">
            <label htmlFor="password" className="block text-sm font-medium text-gray-700 mb-2">
              Password
            </label>
            <div className="relative">
              <input
                type={showPassword ? 'text' : 'password'}
                id="password"
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  setError(''); // Clear error when user types
                }}
                className="cartoon-input w-full pr-10"
                placeholder="Enter your password"
                disabled={isLoading}
                autoFocus
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-2 top-1/2 transform -translate-y-1/2 p-1 hover:bg-gray-100 rounded transition-colors"
                disabled={isLoading}
              >
                {showPassword ? <Icons.EyeOff size={16} /> : <Icons.Eye size={16} />}
              </button>
            </div>
            {error && (
              <p className="text-red-500 text-xs mt-1 flex items-center gap-1">
                <Icons.AlertCircle size={12} />
                {error}
              </p>
            )}
          </div>

          {/* Action Buttons */}
          <div className="flex gap-2 justify-end">
            <button
              type="button"
              onClick={handleClose}
              className="cartoon-btn"
              disabled={isLoading}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="cartoon-btn bg-red-500 hover:bg-red-600 text-white flex items-center gap-2"
              disabled={isLoading || !password.trim()}
            >
              {isLoading ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></div>
                  Verifying...
                </>
              ) : (
                <>
                  <Icons.Download size={16} />
                  Confirm & Export
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default PasswordConfirmationModal;