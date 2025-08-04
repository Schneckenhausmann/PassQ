import React, { useState } from 'react';
import { Icons } from './Icons';
import { passwordAPI, folderAPI } from '../services/api';

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
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>Share Item</h3>
          <button className="close-btn" onClick={onClose}>
            <Icons.X size={20} />
          </button>
        </div>
        
        <form className="share-form" onSubmit={(e) => { e.preventDefault(); handleShare(); }}>
          <div className="form-group">
            <label htmlFor="recipient">Recipient Username/Email</label>
            <input
              id="recipient"
              type="text"
              placeholder="Enter username or email address"
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
              required
            />
          </div>
          
          <div className="form-group">
            <label htmlFor="shareType">Permission Level</label>
            <select
              id="shareType"
              value={shareType}
              onChange={(e) => setShareType(e.target.value)}
            >
              <option value="view">View Only</option>
              <option value="edit">Can Edit</option>
            </select>
          </div>
          
          <div className="form-group">
            <label htmlFor="expiration">Access Expires</label>
            <select
              id="expiration"
              value={expirationDays}
              onChange={(e) => setExpirationDays(e.target.value)}
            >
              <option value="1">1 Day</option>
              <option value="7">1 Week</option>
              <option value="30">1 Month</option>
              <option value="90">3 Months</option>
              <option value="never">Never</option>
            </select>
          </div>
          
          <div className="modal-actions">
            <button type="submit" className="btn-primary" disabled={!recipient.trim()}>
              <Icons.Share size={16} />
              Share
            </button>
            <button type="button" className="btn-secondary" onClick={onClose}>
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default ShareModal;