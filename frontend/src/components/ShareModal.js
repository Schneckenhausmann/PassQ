import React, { useState } from 'react';
import { Icons } from './Icons';
import { passwordAPI, folderAPI } from '../services/api';
import { sanitizeUsername } from '../utils/sanitization';

function ShareModal({ isOpen, onClose, itemType, itemId, onSuccess }) {
  const [recipient, setRecipient] = useState('');
  const [shareType, setShareType] = useState('view'); // 'view' or 'edit'
  const [expirationDays, setExpirationDays] = useState('7');

  if (!isOpen) return null;

  const handleShare = async () => {
    if (!recipient.trim()) {
      alert('Please enter a recipient username');
      return;
    }

    if (!itemId || !itemType) {
      alert('Invalid item to share');
      return;
    }

    try {
      // Calculate expiration days
      let expirationDaysValue = null;
      if (expirationDays !== 'never') {
        expirationDaysValue = parseInt(expirationDays);
      }

      const shareData = {
        recipient_username: recipient.trim(),
        permission_level: shareType,
        expiration_days: expirationDaysValue
      };

      let response;
      if (itemType === 'password') {
        response = await passwordAPI.share(itemId, shareData);
      } else if (itemType === 'folder') {
        response = await folderAPI.share(itemId, shareData);
      } else {
        throw new Error('Invalid item type');
      }

      if (response.data.success) {
        alert(`${itemType.charAt(0).toUpperCase() + itemType.slice(1)} shared successfully!`);
        setRecipient('');
        setShareType('view');
        setExpirationDays('7');
        if (onSuccess) {
          onSuccess();
        } else {
          onClose();
        }
      } else {
        alert(response.data.message || 'Failed to share item');
      }
    } catch (error) {
      console.error('Error sharing item:', error);
      alert(error.response?.data?.message || 'An error occurred while sharing. Please try again.');
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-white rounded-lg shadow-2xl cartoon-shadow p-6 w-full max-w-md mx-4 cartoon-border" onClick={(e) => e.stopPropagation()}>
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold">Share Item</h3>
          <button onClick={onClose} className="text-gray-500 hover:text-gray-700">
            <Icons.X size={20} />
          </button>
        </div>
        
        <form onSubmit={(e) => { e.preventDefault(); handleShare(); }}>
          <div className="mb-4">
            <label htmlFor="recipient" className="block text-sm font-medium text-gray-700 mb-1">Recipient Username/Email</label>
            <input
              id="recipient"
              type="text"
              placeholder="Enter username or email address"
              value={recipient}
              onChange={(e) => setRecipient(sanitizeUsername(e.target.value))}
              className="w-full px-3 py-2 border border-black border-2 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          
          <div className="mb-4">
            <label htmlFor="shareType" className="block text-sm font-medium text-gray-700 mb-1">Permission Level</label>
            <select
              id="shareType"
              value={shareType}
              onChange={(e) => setShareType(e.target.value)}
              className="w-full px-3 py-2 border border-black border-2 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="view">View Only</option>
              <option value="edit">Can Edit</option>
            </select>
          </div>
          
          <div className="mb-6">
            <label htmlFor="expiration" className="block text-sm font-medium text-gray-700 mb-1">Access Expires</label>
            <select
              id="expiration"
              value={expirationDays}
              onChange={(e) => setExpirationDays(e.target.value)}
              className="w-full px-3 py-2 border border-black border-2 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="1">1 Day</option>
              <option value="7">1 Week</option>
              <option value="30">1 Month</option>
              <option value="90">3 Months</option>
              <option value="never">Never</option>
            </select>
          </div>
          
          <div className="flex gap-2">
            <button type="submit" className="cartoon-btn flex-1 flex items-center justify-center gap-2" disabled={!recipient.trim()}>
              <Icons.Share size={16} />
              Share
            </button>
            <button type="button" className="cartoon-btn flex-1" onClick={onClose}>
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default ShareModal;