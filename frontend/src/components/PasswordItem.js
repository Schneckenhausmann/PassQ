import React, { useState } from 'react';
import { Icons } from './Icons';
import { passwordAPI } from '../services/api';

function PasswordItem({ id, website, username, password, notes, otp_secret, attachments, onEdit, onDelete, onShare, folderId, isShared = false }) {
  const [showPassword, setShowPassword] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [otpCode, setOtpCode] = useState(null);
  const [otpLoading, setOtpLoading] = useState(false);
  const [showDetails, setShowDetails] = useState(false);
  const [copied, setCopied] = useState(false);
  const [copyAnim, setCopyAnim] = useState(false);

  // Debug logging
  console.log('PasswordItem props:', { id, website, username, password: password ? '[REDACTED]' : 'UNDEFINED/NULL', passwordType: typeof password, passwordLength: password?.length });

  const handleDragStart = (e) => {
    setIsDragging(true);
    e.dataTransfer.setData('text/plain', JSON.stringify({ id, website, username, password, notes, otp_secret, attachments, folderId }));
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragEnd = () => {
    setIsDragging(false);
  };

  const handleCopyPassword = () => {
    if (typeof password === 'string' && password.length > 0) {
      navigator.clipboard.writeText(password);
      setCopyAnim(true);
      setTimeout(() => setCopyAnim(false), 700);
    }
  };

  const onToggleShowPassword = () => {
    setShowPassword((prev) => !prev);
  };

  const generateOTP = async () => {
    if (!otp_secret) return;
    setOtpLoading(true);
    try {
      const response = await passwordAPI.generateOTP(id);
      if (response.data.success) {
        setOtpCode(response.data.data.otp_code);
        setTimeout(() => setOtpCode(null), 30000);
      }
    } catch (error) {
      console.error('Failed to generate OTP:', error);
    } finally {
      setOtpLoading(false);
    }
  };

  // Extract domain for favicon
  const extractDomain = (url) => {
    try {
      if (!url) return '';
      // Remove protocol if present
      let domain = url.replace(/^https?:\/\//, '');
      // Remove www. if present
      domain = domain.replace(/^www\./, '');
      // Remove path and query parameters
      domain = domain.split('/')[0].split('?')[0];
      return domain;
    } catch {
      return url || '';
    }
  };

  const domain = extractDomain(website);
  const faviconUrl = domain ? `https://www.google.com/s2/favicons?domain=${domain}&sz=32` : null;

  // Ensure we have a valid password to display
  const displayPassword = password && typeof password === 'string' ? password : '';
  const maskedPassword = displayPassword ? '*'.repeat(displayPassword.length) : '';

  return (
    <div className="flex items-center justify-between gap-4 p-4 mb-3 bg-white rounded-xl cartoon-border shadow cartoon-shadow">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-3 mb-1">
          {faviconUrl && (
            <div className="w-5 h-5 flex-shrink-0 flex items-center justify-center bg-gray-100 rounded border">
              <img 
                src={faviconUrl} 
                alt="" 
                className="w-4 h-4 rounded" 
                onError={(e) => {
                  e.target.style.display = 'none';
                  e.target.parentElement.innerHTML = '🌐';
                  e.target.parentElement.className = 'w-5 h-5 flex-shrink-0 flex items-center justify-center bg-gray-100 rounded border text-xs';
                }}
              />
            </div>
          )}
          <div className="font-semibold text-lg truncate">{website}</div>
        </div>
        <div className="text-sm text-black/60 truncate">{username}</div>
        <div className="mt-1">
          <span className="font-mono tracking-wider text-base select-all text-black">
            {showPassword ? displayPassword : maskedPassword}
            {!displayPassword && <span className="text-red-500 text-xs ml-2">[NO PASSWORD DATA]</span>}
          </span>
        </div>
        <div className="mt-1 flex items-center gap-2">
          <button className="cartoon-btn px-2 py-1" onClick={onToggleShowPassword} title={showPassword ? 'Hide password' : 'Show password'}>
            {showPassword ? <Icons.EyeOff size={16} /> : <Icons.Eye size={16} />}
          </button>
          <button className={"cartoon-btn px-2 py-1 min-w-[36px] min-h-[36px] relative overflow-hidden flex items-center justify-center"} onClick={handleCopyPassword} title="Copy password">
            <span className={copyAnim ? "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 animate-float-up" : "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 transition-all"}>
              <Icons.Copy size={16} />
            </span>
            <span className={copyAnim ? "absolute left-1/2 bottom-0 -translate-x-1/2 translate-y-full animate-float-in" : "opacity-0"}>
              <Icons.Copy size={16} />
            </span>
          </button>
        </div>
      </div>
      <div className="flex flex-col gap-2 items-end">
        <button className="cartoon-btn px-2 py-1" onClick={() => onEdit && onEdit(id)} title="Edit entry"><Icons.Edit size={16} /></button>
        <button className="cartoon-btn px-2 py-1" onClick={() => onShare && onShare(id)} title="Share entry"><Icons.Share size={16} /></button>
        <button className="cartoon-btn px-2 py-1" onClick={() => onDelete && onDelete(id)} title="Delete entry"><Icons.Trash size={16} /></button>
      </div>
    </div>
  );
}

export default PasswordItem;

/* Add to the bottom of the file (or in a CSS/JSX style block if using Tailwind's arbitrary values):
@keyframes floatUp {
  0% { transform: translate(-50%, -50%) scale(1); opacity: 1; }
  80% { transform: translate(-50%, -120%) scale(1.2); opacity: 1; }
  100% { transform: translate(-50%, -200%) scale(0.8); opacity: 0; }
}
@keyframes floatIn {
  0% { transform: translate(-50%, 100%) scale(0.8); opacity: 0; }
  100% { transform: translate(-50%, -50%) scale(1); opacity: 1; }
}
.animate-float-up {
  animation: floatUp 0.7s cubic-bezier(0.4,0,0.2,1) forwards;
}
.animate-float-in {
  animation: floatIn 0.7s cubic-bezier(0.4,0,0.2,1) forwards;
}
*/