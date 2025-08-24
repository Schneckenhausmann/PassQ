# Firefox Extension Documentation

The PassQ Firefox extension provides seamless browser integration for password autofill and credential management, similar to popular password managers like Bitwarden.

## Architecture Overview

The extension follows Firefox's WebExtension API standards with a modular architecture:

- **Background Script**: Handles API communication and data management
- **Content Script**: Detects forms and provides autofill functionality
- **Popup Interface**: Provides user interaction and credential search
- **Secure Storage**: Manages authentication tokens and temporary data

## Core Components

### Background Script (`background.js`)
Manages the extension's core functionality and API communication.

**Key Features:**
- JWT token management and storage
- API communication with PassQ backend
- Password data caching and synchronization
- Domain-based credential matching
- Authentication state management

**API Integration:**
- Connects to PassQ backend at `http://localhost:8080`
- Handles login/logout operations
- Fetches and caches password data
- Implements secure token storage

### Content Script (`content.js`)
Injected into web pages to detect forms and provide autofill functionality.

**Functionality:**
- Automatic login form detection
- Password field identification
- Credential injection and autofill
- UI overlay for credential selection
- Real-time form monitoring

### Popup Interface (`popup.html/js/css`)
Provides the main user interface accessible from the browser toolbar.

**Features:**
- User authentication interface with cartoon/comic book styling
- Password search and selection with full-width search bar
- Quick access to credential management with improved spacing
- Action buttons (copy username, copy password, edit) with rounded corners and black shadows
- Extension settings and configuration
- Comic book themed design with black borders and full-width separator lines

## Security Implementation

### Authentication Security
- **JWT Token Storage**: Secure browser storage for authentication tokens
- **Token Expiration**: Automatic handling of expired tokens
- **Secure Communication**: HTTPS support for production environments
- **Domain Validation**: Ensures credentials are only used on matching domains

### Data Protection
- **No Local Password Storage**: Passwords are fetched from backend on demand
- **Encrypted Communication**: All API calls use secure protocols
- **Memory Management**: Sensitive data cleared after use
- **Permission Model**: Minimal required permissions for security

### Browser Security
- **Content Security Policy**: Prevents XSS attacks
- **Secure Origins**: Validates website origins before autofill
- **Sandboxed Execution**: Extension runs in isolated environment

## API Integration

### Backend Communication
The extension communicates with the PassQ backend using RESTful APIs:

```javascript
// Authentication
POST /login
{
  "username": "string",
  "password": "string"
}

// Password Retrieval
GET /passwords
Authorization: Bearer <jwt_token>

// Response Format
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "website": "example.com",
      "username": "user@example.com",
      "password": "encrypted_password"
    }
  ]
}
```

### Error Handling
- **Network Failures**: Graceful degradation when backend is unavailable
- **Authentication Errors**: Automatic re-authentication prompts
- **API Errors**: User-friendly error messages
- **Timeout Handling**: Configurable request timeouts

## Form Detection Algorithm

### Login Form Identification
The extension uses sophisticated algorithms to detect login forms:

1. **Field Detection**: Identifies username and password fields
2. **Form Validation**: Ensures forms are actual login forms
3. **Dynamic Content**: Handles single-page applications and dynamic forms
4. **Multiple Forms**: Manages pages with multiple login forms

### Autofill Strategy
- **Domain Matching**: Matches saved credentials to current domain
- **Fuzzy Matching**: Handles subdomains and URL variations
- **User Confirmation**: Requires user approval for credential injection
- **Multiple Accounts**: Handles multiple accounts for the same domain

## Installation and Setup

### Development Installation
1. Open Firefox and navigate to `about:debugging`
2. Click "This Firefox" in the left sidebar
3. Click "Load Temporary Add-on"
4. Select `manifest.json` from the `firefox-extension` directory

### Configuration
- **Backend URL**: Configure PassQ backend endpoint
- **Auto-login**: Enable/disable automatic login detection
- **Security Settings**: Configure security preferences

## File Structure

```
firefox-extension/
├── manifest.json          # Extension configuration
├── background.js          # Background script
├── content.js            # Content script for form detection
├── popup.html            # Extension popup interface
├── popup.js              # Popup functionality
├── popup.css             # Popup styling with comic book theme
├── content.css           # Injected styles
├── icons/                # Extension icons (SVG format)
│   ├── globe-placeholder.svg
│   ├── icon-48.svg
│   └── icon-96.svg
└── README.md             # Extension documentation
```

## Browser Compatibility

### Firefox Support
- **Manifest V2**: Full support for current Firefox versions
- **WebExtension API**: Uses standard WebExtension APIs
- **Permissions**: Minimal required permissions

### Future Compatibility
- **Manifest V3**: Migration planned for future Firefox versions
- **Chrome Support**: Potential Chrome extension development
- **Cross-browser**: Standardized WebExtension APIs

## Development Guidelines

### Code Standards
- **ES6+ JavaScript**: Modern JavaScript features
- **Error Handling**: Comprehensive error handling
- **Documentation**: Inline code documentation
- **Security First**: Security-focused development practices

### Testing Strategy
- **Manual Testing**: Test on various websites
- **Security Testing**: Validate security implementations
- **Performance Testing**: Ensure minimal performance impact
- **Compatibility Testing**: Test across Firefox versions

## Performance Considerations

### Optimization Strategies
- **Lazy Loading**: Load resources only when needed
- **Caching**: Cache frequently accessed data
- **Minimal DOM Manipulation**: Efficient DOM operations
- **Background Processing**: Use background scripts for heavy operations

### Resource Management
- **Memory Usage**: Minimize memory footprint
- **Network Requests**: Optimize API calls
- **CPU Usage**: Efficient algorithms and processing

## Future Enhancements

### Planned Features
- **In-popup Editor**: Implement credential editing directly in the popup
- **SVG Icons**: Replace emoji icons with SVG icons from the web version
- **Password Generation**: Integrate password generation
- **Secure Notes**: Support for secure note autofill
- **Multi-factor Authentication**: TOTP integration
- **Bitwarden-like Features**: Identity storage, card storage, and secure notes
- **Offline Mode**: Limited offline functionality

### Technical Improvements
- **Manifest V3**: Migration to Manifest V3
- **Enhanced Security**: Additional security measures
- **Performance Optimization**: Further performance improvements
- **Cross-browser Support**: Chrome and Edge compatibility

## Troubleshooting

### Common Issues

1. **Extension Not Loading**
   - Verify `manifest.json` syntax
   - Check Firefox developer console for errors
   - Ensure all required files are present

2. **Authentication Failures**
   - Verify PassQ backend is running on correct port
   - Check network connectivity
   - Validate credentials

3. **Autofill Not Working**
   - Check if forms are properly detected
   - Verify domain matching
   - Check browser console for JavaScript errors

4. **Performance Issues**
   - Monitor memory usage in Firefox Task Manager
   - Check for excessive API calls
   - Verify efficient DOM operations

### Debug Mode
Enable debug logging by:
1. Opening Firefox Developer Tools
2. Navigating to the Console tab
3. Looking for PassQ extension log messages
4. Checking for error messages and warnings

## Security Best Practices

### Development Security
- **Code Review**: Regular security code reviews
- **Dependency Management**: Keep dependencies updated
- **Vulnerability Scanning**: Regular security scans
- **Secure Coding**: Follow secure coding practices

### User Security
- **Permission Awareness**: Educate users about permissions
- **Regular Updates**: Encourage regular extension updates
- **Secure Configuration**: Provide secure default settings
- **Privacy Protection**: Minimize data collection and storage