import DOMPurify from 'dompurify';

/**
 * Sanitize HTML content to prevent XSS attacks
 * @param {string} dirty - The potentially unsafe HTML string
 * @param {Object} options - DOMPurify configuration options
 * @returns {string} - Sanitized HTML string
 */
export const sanitizeHTML = (dirty, options = {}) => {
  if (typeof dirty !== 'string') return '';
  
  const defaultOptions = {
    ALLOWED_TAGS: ['b', 'i', 'em', 'strong', 'u', 'br', 'p'],
    ALLOWED_ATTR: [],
    KEEP_CONTENT: true,
    ...options
  };
  
  return DOMPurify.sanitize(dirty, defaultOptions);
};

/**
 * Sanitize plain text input to prevent injection attacks
 * @param {string} input - The user input string
 * @param {Object} options - Sanitization options
 * @returns {string} - Sanitized string
 */
export const sanitizeInput = (input, options = {}) => {
  if (typeof input !== 'string') return '';
  
  const {
    maxLength = 1000,
    allowNewlines = false,
    trimWhitespace = true
  } = options;
  
  let sanitized = input;
  
  // Remove control characters except newlines (if allowed)
  if (allowNewlines) {
    sanitized = sanitized.replace(/[\x00-\x09\x0B\x0C\x0E-\x1F\x7F]/g, '');
  } else {
    sanitized = sanitized.replace(/[\x00-\x1F\x7F]/g, '');
  }
  
  // Trim whitespace if requested
  if (trimWhitespace) {
    sanitized = sanitized.trim();
  }
  
  // Limit length
  sanitized = sanitized.slice(0, maxLength);
  
  return sanitized;
};

/**
 * Sanitize URL input to prevent malicious redirects
 * @param {string} url - The URL string to sanitize
 * @returns {string} - Sanitized URL or empty string if invalid
 */
export const sanitizeURL = (url) => {
  if (typeof url !== 'string') return '';
  
  try {
    const urlObj = new URL(url);
    
    // Only allow http and https protocols
    if (!['http:', 'https:'].includes(urlObj.protocol)) {
      return '';
    }
    
    return urlObj.toString();
  } catch {
    // If URL is invalid, try to construct a valid one
    const cleaned = url.trim().toLowerCase();
    
    // If it looks like a domain, prepend https://
    if (/^[a-z0-9.-]+\.[a-z]{2,}$/i.test(cleaned)) {
      return `https://${cleaned}`;
    }
    
    return '';
  }
};

/**
 * Sanitize username input
 * @param {string} username - The username string
 * @returns {string} - Sanitized username
 */
export const sanitizeUsername = (username) => {
  if (typeof username !== 'string') return '';
  
  return username
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._@-]/g, '') // Only allow alphanumeric, dots, underscores, @ and hyphens
    .slice(0, 100); // Limit length
};

/**
 * Sanitize folder/entry names
 * @param {string} name - The name string
 * @returns {string} - Sanitized name
 */
export const sanitizeName = (name) => {
  if (typeof name !== 'string') return '';
  
  return name
    .trim()
    .replace(/[<>:"/\\|?*\x00-\x1F]/g, '') // Remove filesystem-unsafe characters
    .slice(0, 255); // Limit to filesystem-safe length
};

/**
 * Sanitize notes/text content that may contain user formatting
 * @param {string} notes - The notes content
 * @returns {string} - Sanitized notes
 */
export const sanitizeNotes = (notes) => {
  if (typeof notes !== 'string') return '';
  
  return sanitizeInput(notes, {
    maxLength: 10000,
    allowNewlines: true,
    trimWhitespace: false
  });
};

/**
 * Validate and sanitize search queries
 * @param {string} query - The search query
 * @returns {string} - Sanitized search query
 */
export const sanitizeSearchQuery = (query) => {
  if (typeof query !== 'string') return '';
  
  return query
    .trim()
    .replace(/[<>"'&]/g, '') // Remove HTML/script injection characters
    .replace(/[\x00-\x1F\x7F]/g, '') // Remove control characters
    .slice(0, 100); // Limit length for performance
};