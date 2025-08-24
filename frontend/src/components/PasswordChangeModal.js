import React, { useState } from 'react';
import { Icons } from './Icons';
import { authAPI } from '../services/api';

function PasswordChangeModal({ isOpen, onClose }) {
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showCurrentPassword, setShowCurrentPassword] = useState(false);
  const [showNewPassword, setShowNewPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    
    if (newPassword !== confirmPassword) {
      setError('New passwords do not match');
      return;
    }
    
    if (newPassword.length < 6) {
      setError('New password must be at least 6 characters long');
      return;
    }
    
    setLoading(true);
    
    try {
      await authAPI.changePassword({
        current_password: currentPassword,
        new_password: newPassword
      });
      
      setSuccess(true);
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      
      // Close modal after 2 seconds
      setTimeout(() => {
        setSuccess(false);
        onClose();
      }, 2000);
    } catch (err) {
      setError(err.response?.data?.error || 'Failed to change password');
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading) {
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setError('');
      setSuccess(false);
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white cartoon-border shadow-2xl cartoon-shadow rounded-lg w-full max-w-md">
        <div className="flex items-center justify-between p-4 border-b border-black/20">
          <h2 className="text-lg font-bold">Change Password</h2>
          <button 
            onClick={handleClose} 
            className="cartoon-btn p-1"
            disabled={loading}
          >
            <Icons.X size={16} />
          </button>
        </div>
        
        <div className="p-4">
          {success ? (
            <div className="text-center py-8">
              <div className="text-green-600 mb-2">
                <Icons.Check size={48} className="mx-auto" />
              </div>
              <p className="text-green-600 font-semibold">Password changed successfully!</p>
            </div>
          ) : (
            <form onSubmit={handleSubmit} className="space-y-4">
              {error && (
                <div className="bg-red-50 border border-red-200 text-red-700 px-3 py-2 rounded text-sm">
                  {error}
                </div>
              )}
              
              <div>
                <label className="block text-sm font-medium mb-1">Current Password</label>
                <div className="relative">
                  <input
                    type={showCurrentPassword ? 'text' : 'password'}
                    value={currentPassword}
                    onChange={(e) => setCurrentPassword(e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 pr-10"
                    required
                    disabled={loading}
                  />
                  <button
                    type="button"
                    onClick={() => setShowCurrentPassword(!showCurrentPassword)}
                    className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-500 hover:text-gray-700"
                    disabled={loading}
                  >
                    {showCurrentPassword ? <Icons.EyeOff size={16} /> : <Icons.Eye size={16} />}
                  </button>
                </div>
              </div>
              
              <div>
                <label className="block text-sm font-medium mb-1">New Password</label>
                <div className="relative">
                  <input
                    type={showNewPassword ? 'text' : 'password'}
                    value={newPassword}
                    onChange={(e) => setNewPassword(e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 pr-10"
                    required
                    disabled={loading}
                    minLength={6}
                  />
                  <button
                    type="button"
                    onClick={() => setShowNewPassword(!showNewPassword)}
                    className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-500 hover:text-gray-700"
                    disabled={loading}
                  >
                    {showNewPassword ? <Icons.EyeOff size={16} /> : <Icons.Eye size={16} />}
                  </button>
                </div>
              </div>
              
              <div>
                <label className="block text-sm font-medium mb-1">Confirm New Password</label>
                <div className="relative">
                  <input
                    type={showConfirmPassword ? 'text' : 'password'}
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 pr-10"
                    required
                    disabled={loading}
                    minLength={6}
                  />
                  <button
                    type="button"
                    onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-500 hover:text-gray-700"
                    disabled={loading}
                  >
                    {showConfirmPassword ? <Icons.EyeOff size={16} /> : <Icons.Eye size={16} />}
                  </button>
                </div>
              </div>
              
              <div className="flex gap-2 pt-2">
                <button
                  type="button"
                  onClick={handleClose}
                  className="flex-1 cartoon-btn"
                  disabled={loading}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="flex-1 cartoon-btn cartoon-btn-primary"
                  disabled={loading || !currentPassword || !newPassword || !confirmPassword}
                >
                  {loading ? 'Changing...' : 'Change Password'}
                </button>
              </div>
            </form>
          )}
        </div>
      </div>
    </div>
  );
}

export default PasswordChangeModal;