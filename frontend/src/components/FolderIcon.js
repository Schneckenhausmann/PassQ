import React from 'react';

const FolderIcon = ({ isOpen = false, className = "" }) => {
  if (isOpen) {
    // Open folder icon
    return (
      <svg 
        className={`w-4 h-4 ${className}`} 
        viewBox="0 0 24 24" 
        fill="none" 
        xmlns="http://www.w3.org/2000/svg"
      >
        <path 
          d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H12L10 5H5C3.89543 5 3 5.89543 3 7Z" 
          stroke="currentColor" 
          strokeWidth="1.5" 
          fill="none"
        />
        <path 
          d="M3 7H21" 
          stroke="currentColor" 
          strokeWidth="1.5"
        />
      </svg>
    );
  }
  
  // Closed folder icon
  return (
    <svg 
      className={`w-4 h-4 ${className}`} 
      viewBox="0 0 24 24" 
      fill="none" 
      xmlns="http://www.w3.org/2000/svg"
    >
      <path 
        d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H12L10 5H5C3.89543 5 3 5.89543 3 7Z" 
        stroke="currentColor" 
        strokeWidth="1.5" 
        fill="currentColor"
        fillOpacity="0.1"
      />
    </svg>
  );
};

export default FolderIcon;