import React, { useState, useEffect } from 'react';
import { Icons } from './Icons';
import { sanitizeInput, sanitizeURL, sanitizeName, sanitizeNotes } from '../utils/sanitization';

function EditEntryModal({ isOpen, onClose, onSave, entry, folders, onAddFolder }) {
  const [formData, setFormData] = useState({
    website: '',
    username: '',
    password: '',
    notes: '',
    otp_secret: '',
    folder_id: 'root'
  });
  const [showPassword, setShowPassword] = useState(false);
  const [showNewFolderInput, setShowNewFolderInput] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [attachments, setAttachments] = useState([]);

  useEffect(() => {
    if (entry) {
      setFormData({
        website: entry.website || '',
        username: entry.username || '',
        password: entry.password || '',
        notes: entry.notes || '',
        otp_secret: entry.otp_secret || '',
        folder_id: entry.folder_id || 'root'
      });
      setAttachments(entry.attachments || []);
    }
  }, [entry]);

  const handleSubmit = (e) => {
    e.preventDefault();
    if (formData.website && formData.username && formData.password) {
      onSave(entry.id, {
        ...formData,
        attachments: attachments
      });
      onClose();
    }
  };

  const handleChange = (e) => {
    let value = e.target.value;
    const fieldName = e.target.name;
    
    // If it's the otp_secret field and contains otpauth://, extract the secret
    if (fieldName === 'otp_secret' && value.startsWith('otpauth://')) {
      try {
        const url = new URL(value);
        const secret = url.searchParams.get('secret');
        if (secret) {
          value = secret;
        }
      } catch (error) {
        console.warn('Invalid otpauth URL:', error);
      }
    }
    
    // Sanitize input based on field type
    switch (fieldName) {
      case 'website':
        value = sanitizeURL(value);
        break;
      case 'username':
        value = sanitizeInput(value, { maxLength: 255 });
        break;
      case 'password':
        // Don't sanitize passwords as they may contain special characters
        break;
      case 'notes':
        value = sanitizeNotes(value);
        break;
      case 'otp_secret':
        value = sanitizeInput(value, { maxLength: 100 });
        break;
      default:
        value = sanitizeInput(value);
        break;
    }
    
    setFormData({
      ...formData,
      [fieldName]: value
    });
  };

  const generatePassword = () => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*';
    let password = '';
    for (let i = 0; i < 16; i++) {
      password += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    setFormData(prevData => ({ ...prevData, password }));
  };

  const generateOTPSecret = () => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    let secret = '';
    for (let i = 0; i < 32; i++) {
      secret += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    setFormData(prevData => ({ ...prevData, otp_secret: secret }));
  };

  const handleCreateFolder = () => {
    if (newFolderName.trim()) {
      onAddFolder(newFolderName.trim(), formData.folder_id === 'root' ? null : formData.folder_id);
      setNewFolderName('');
      setShowNewFolderInput(false);
    }
  };

  const handleFileUpload = (e) => {
    const files = Array.from(e.target.files);
    const newAttachments = files.map(file => ({
      id: Date.now() + Math.random(),
      name: file.name,
      size: file.size,
      type: file.type,
      data: null // In a real app, you'd convert to base64 or upload to server
    }));
    setAttachments([...attachments, ...newAttachments]);
  };

  const removeAttachment = (id) => {
    setAttachments(attachments.filter(att => att.id !== id));
  };

  if (!isOpen || !entry) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 backdrop-blur-sm" onClick={onClose}>
      <div className="relative bg-white rounded-xl shadow-2xl cartoon-shadow p-6 w-full max-w-md mx-auto cartoon-border" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-xl font-bold">Edit Entry</h3>
          <button className="text-black/60 hover:text-black transition" onClick={onClose}>
            <Icons.Close size={20} />
          </button>
        </div>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="edit-website" className="block text-sm font-medium mb-1">Website</label>
            <input id="edit-website" type="text" name="website" placeholder="example.com" value={formData.website} onChange={handleChange} required className="w-full border-2 border-black rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-black/30" />
          </div>
          <div>
            <label htmlFor="edit-username" className="block text-sm font-medium mb-1">Username/Email</label>
            <input id="edit-username" type="text" name="username" placeholder="your@email.com" value={formData.username} onChange={handleChange} required className="w-full border-2 border-black rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-black/30" />
          </div>
          <div>
            <label htmlFor="edit-folder" className="block text-sm font-medium mb-1">Folder</label>
            <div className="flex items-center gap-2">
              <select id="edit-folder" name="folder_id" value={formData.folder_id} onChange={handleChange} className="flex-1 border-2 border-black rounded px-3 py-2">
                <option value="root">All Items</option>
                {folders.filter(f => f.id !== 'root').map(folder => (
                  <option key={folder.id} value={folder.id}>{folder.name}</option>
                ))}
              </select>
              <button type="button" className="cartoon-btn cartoon-btn-primary px-2 py-1" onClick={() => setShowNewFolderInput(!showNewFolderInput)} title="Create new folder"><Icons.Plus size={16} /></button>
            </div>
            {showNewFolderInput && (
              <div className="flex items-center gap-2 mt-2">
                <input type="text" placeholder="Folder name" value={newFolderName} onChange={(e) => setNewFolderName(e.target.value)} className="flex-1 border-2 border-black rounded px-2 py-1" />
                <button type="button" className="cartoon-btn cartoon-btn-primary px-2 py-1" onClick={handleCreateFolder}>Create</button>
                <button type="button" className="cartoon-btn px-2 py-1" onClick={() => setShowNewFolderInput(false)}>Cancel</button>
              </div>
            )}
          </div>
          <div>
            <label htmlFor="edit-password" className="block text-sm font-medium mb-1">Password</label>
            <div className="flex items-center gap-2">
              <input id="edit-password" type={showPassword ? 'text' : 'password'} name="password" placeholder="Enter password" value={formData.password} onChange={handleChange} required className="flex-1 border-2 border-black rounded px-3 py-2" />
              <button type="button" className="cartoon-btn px-2 py-1" onClick={generatePassword} title="Generate strong password"><Icons.Dice size={16} /></button>
              <button type="button" className="cartoon-btn px-2 py-1" onClick={() => setShowPassword(!showPassword)}>{showPassword ? <Icons.EyeOff size={16} /> : <Icons.Eye size={16} />}</button>
            </div>
          </div>
          <div>
            <label htmlFor="edit-otp_secret" className="block text-sm font-medium mb-1">OTP Secret (Optional)</label>
            <div className="flex items-center gap-2">
              <input 
                id="edit-otp_secret" 
                type="text" 
                name="otp_secret" 
                placeholder="Enter OTP secret or otpauth:// URL" 
                value={formData.otp_secret} 
                onChange={handleChange} 
                className="flex-1 border-2 border-black rounded px-3 py-2" 
              />
              <button 
                type="button" 
                className="cartoon-btn px-2 py-1" 
                onClick={generateOTPSecret} 
                title="Generate OTP secret"
              >
                <Icons.Dice size={16} />
              </button>
            </div>
            <p className="text-xs text-gray-600 mt-1">
              Supports: Base32 secrets, otpauth:// URLs, or Google Authenticator links
            </p>
          </div>
          <div>
            <label htmlFor="edit-notes" className="block text-sm font-medium mb-1">Notes</label>
            <textarea id="edit-notes" name="notes" placeholder="Notes (optional)" value={formData.notes} onChange={handleChange} className="w-full border-2 border-black rounded px-3 py-2 min-h-[60px]" />
          </div>
          <div className="flex justify-end gap-2 mt-6">
            <button type="button" className="cartoon-btn" onClick={onClose}>Cancel</button>
            <button type="submit" className="cartoon-btn cartoon-btn-primary">Save Changes</button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default EditEntryModal;