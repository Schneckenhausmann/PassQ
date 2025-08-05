import React, { useState, useCallback, useMemo } from 'react';
import { Icons } from './Icons';

// Utility function to sanitize search input and prevent XSS
const sanitizeSearchInput = (input) => {
  if (typeof input !== 'string') return '';
  
  // Remove potentially dangerous characters and limit length
  return input
    .replace(/[<>"'&]/g, '') // Remove HTML/script injection characters
    .replace(/[\x00-\x1F\x7F]/g, '') // Remove control characters
    .trim()
    .slice(0, 100); // Limit to 100 characters for performance
};

// Debounce hook for performance optimization
const useDebounce = (callback, delay) => {
  const [debounceTimer, setDebounceTimer] = useState(null);
  
  const debouncedCallback = useCallback((...args) => {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    
    const newTimer = setTimeout(() => {
      callback(...args);
    }, delay);
    
    setDebounceTimer(newTimer);
  }, [callback, delay, debounceTimer]);
  
  return debouncedCallback;
};

function SearchBar({ passwords, onFilteredResults, placeholder = "Search passwords..." }) {
  const [searchTerm, setSearchTerm] = useState('');
  const [isFocused, setIsFocused] = useState(false);
  
  // Memoized search function for performance
  const performSearch = useMemo(() => {
    return (term) => {
      if (!term || term.length === 0) {
        return passwords;
      }
      
      const sanitizedTerm = sanitizeSearchInput(term).toLowerCase();
      
      if (sanitizedTerm.length === 0) {
        return passwords;
      }
      
      // Perform case-insensitive search across multiple fields
      return passwords.filter(password => {
        const website = (password.website || '').toLowerCase();
        const username = (password.username || '').toLowerCase();
        const notes = (password.notes || '').toLowerCase();
        
        return website.includes(sanitizedTerm) || 
               username.includes(sanitizedTerm) || 
               notes.includes(sanitizedTerm);
      });
    };
  }, [passwords]);
  
  // Debounced search to improve performance
  const debouncedSearch = useDebounce((term) => {
    const results = performSearch(term);
    onFilteredResults(results, term);
  }, 300); // 300ms delay
  
  const handleInputChange = (e) => {
    const value = e.target.value;
    const sanitizedValue = sanitizeSearchInput(value);
    
    setSearchTerm(sanitizedValue);
    debouncedSearch(sanitizedValue);
  };
  
  const handleClearSearch = () => {
    setSearchTerm('');
    onFilteredResults(passwords, '');
  };
  
  const handleKeyDown = (e) => {
    // Prevent potential script injection through key events
    if (e.key === 'Enter') {
      e.preventDefault();
      const results = performSearch(searchTerm);
      onFilteredResults(results, searchTerm);
    }
    
    if (e.key === 'Escape') {
      handleClearSearch();
    }
  };
  
  return (
    <div className="w-full max-w-2xl mb-6">
      <div className="relative">
        <div className={`cartoon-border cartoon-shadow bg-white transition-all duration-200 ${
          isFocused ? 'shadow-lg' : ''
        }`}>
          <div className="flex items-center px-4 py-3">
            {/* Search Icon */}
            <Icons.Search 
              size={20} 
              className="text-black/60 mr-3 flex-shrink-0" 
            />
            
            {/* Search Input */}
            <input
              type="text"
              value={searchTerm}
              onChange={handleInputChange}
              onKeyDown={handleKeyDown}
              onFocus={() => setIsFocused(true)}
              onBlur={() => setIsFocused(false)}
              placeholder={placeholder}
              className="flex-1 bg-transparent text-black placeholder-black/50 font-medium focus:outline-none text-sm"
              maxLength={100} // Additional security measure
              autoComplete="off" // Prevent autocomplete for security
              spellCheck="false" // Disable spellcheck for passwords
            />
            
            {/* Clear Button */}
            {searchTerm && (
              <button
                onClick={handleClearSearch}
                className="ml-2 p-1 hover:bg-black/10 rounded transition-colors duration-150 flex-shrink-0"
                title="Clear search"
                type="button"
              >
                <Icons.X size={16} className="text-black/60" />
              </button>
            )}
          </div>
        </div>
        
        {/* Search Results Count */}
        {searchTerm && (
          <div className="mt-2 text-xs text-black/60 font-medium">
            {performSearch(searchTerm).length} result{performSearch(searchTerm).length !== 1 ? 's' : ''} found
          </div>
        )}
      </div>
    </div>
  );
}

export default SearchBar;