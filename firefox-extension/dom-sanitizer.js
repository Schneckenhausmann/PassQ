// DOM Sanitization utility for PassQ Firefox Extension
// Lightweight alternative to DOMPurify for extension security

class PassQDOMSanitizer {
  constructor() {
    // Define allowed HTML tags and attributes
    this.allowedTags = new Set([
      'div', 'span', 'p', 'br', 'strong', 'em', 'b', 'i',
      'ul', 'ol', 'li', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'button', 'input', 'label', 'form', 'select', 'option'
    ]);
    
    this.allowedAttributes = new Set([
      'class', 'id', 'type', 'value', 'placeholder', 'disabled',
      'readonly', 'required', 'name', 'for', 'title', 'alt'
    ]);
    
    // Dangerous attributes that should never be allowed
    this.dangerousAttributes = new Set([
      'onclick', 'onload', 'onerror', 'onmouseover', 'onfocus',
      'onblur', 'onchange', 'onsubmit', 'href', 'src', 'action',
      'formaction', 'javascript:', 'data:', 'vbscript:', 'file:'
    ]);
  }

  // Sanitize HTML string
  sanitizeHTML(html) {
    if (!html || typeof html !== 'string') {
      return '';
    }

    // Create a temporary DOM element to parse HTML
    const tempDiv = document.createElement('div');
    tempDiv.innerHTML = html;

    // Recursively sanitize all elements
    this.sanitizeElement(tempDiv);

    return tempDiv.innerHTML;
  }

  // Sanitize a DOM element and its children
  sanitizeElement(element) {
    if (!element || !element.childNodes) {
      return;
    }

    // Process all child nodes
    const childNodes = Array.from(element.childNodes);
    
    for (const node of childNodes) {
      if (node.nodeType === Node.ELEMENT_NODE) {
        // Check if tag is allowed
        if (!this.allowedTags.has(node.tagName.toLowerCase())) {
          // Remove disallowed element but keep its text content
          const textNode = document.createTextNode(node.textContent || '');
          node.parentNode.replaceChild(textNode, node);
          continue;
        }

        // Sanitize attributes
        this.sanitizeAttributes(node);

        // Recursively sanitize children
        this.sanitizeElement(node);
      } else if (node.nodeType === Node.TEXT_NODE) {
        // Text nodes are safe, but ensure no script injection
        const textContent = node.textContent || '';
        if (this.containsScriptInjection(textContent)) {
          node.textContent = this.escapeHTML(textContent);
        }
      } else {
        // Remove other node types (comments, etc.)
        node.parentNode.removeChild(node);
      }
    }
  }

  // Sanitize element attributes
  sanitizeAttributes(element) {
    const attributes = Array.from(element.attributes);
    
    for (const attr of attributes) {
      const attrName = attr.name.toLowerCase();
      const attrValue = attr.value;

      // Remove dangerous attributes
      if (this.dangerousAttributes.has(attrName) || 
          this.containsScriptInjection(attrValue)) {
        element.removeAttribute(attr.name);
        continue;
      }

      // Remove attributes not in allowlist
      if (!this.allowedAttributes.has(attrName)) {
        element.removeAttribute(attr.name);
        continue;
      }

      // Sanitize attribute values
      element.setAttribute(attr.name, this.sanitizeAttributeValue(attrValue));
    }
  }

  // Check if text contains potential script injection
  containsScriptInjection(text) {
    if (!text || typeof text !== 'string') {
      return false;
    }

    const dangerousPatterns = [
      /<script[^>]*>/i,
      /javascript:/i,
      /vbscript:/i,
      /data:text\/html/i,
      /on\w+\s*=/i,
      /<iframe[^>]*>/i,
      /<object[^>]*>/i,
      /<embed[^>]*>/i,
      /<link[^>]*>/i,
      /<meta[^>]*>/i
    ];

    return dangerousPatterns.some(pattern => pattern.test(text));
  }

  // Sanitize attribute values
  sanitizeAttributeValue(value) {
    if (!value || typeof value !== 'string') {
      return '';
    }

    // Remove potential XSS vectors
    return value
      .replace(/javascript:/gi, '')
      .replace(/vbscript:/gi, '')
      .replace(/data:/gi, '')
      .replace(/file:/gi, '')
      .replace(/on\w+\s*=/gi, '')
      .trim();
  }

  // Escape HTML entities
  escapeHTML(text) {
    if (!text || typeof text !== 'string') {
      return '';
    }

    const entityMap = {
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      '"': '&quot;',
      "'": '&#39;',
      '/': '&#x2F;'
    };

    return text.replace(/[&<>"'\/]/g, (char) => entityMap[char]);
  }

  // Safely set innerHTML with sanitization
  safeSetInnerHTML(element, html) {
    if (!element || !html) {
      return;
    }

    const sanitizedHTML = this.sanitizeHTML(html);
    element.innerHTML = sanitizedHTML;
  }

  // Safely set textContent (always safe)
  safeSetTextContent(element, text) {
    if (!element) {
      return;
    }

    element.textContent = text || '';
  }

  // Create a safe DOM element
  createSafeElement(tagName, attributes = {}, textContent = '') {
    if (!this.allowedTags.has(tagName.toLowerCase())) {
      throw new Error(`Tag '${tagName}' is not allowed`);
    }

    const element = document.createElement(tagName);
    
    // Set safe attributes
    for (const [name, value] of Object.entries(attributes)) {
      if (this.allowedAttributes.has(name.toLowerCase()) && 
          !this.dangerousAttributes.has(name.toLowerCase())) {
        element.setAttribute(name, this.sanitizeAttributeValue(value));
      }
    }

    // Set safe text content
    if (textContent) {
      this.safeSetTextContent(element, textContent);
    }

    return element;
  }
}

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
  module.exports = PassQDOMSanitizer;
} else if (typeof window !== 'undefined') {
  window.PassQDOMSanitizer = PassQDOMSanitizer;
}