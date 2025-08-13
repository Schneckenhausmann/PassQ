# PassQ Firefox Extension

A Firefox extension for PassQ password manager that provides seamless autofill functionality, similar to Bitwarden.

## Features

- 🔐 **Secure Autofill**: Automatically fill login forms with your PassQ credentials
- 🌐 **Website Detection**: Smart detection of login forms and matching credentials
- 🔍 **Quick Search**: Search through your passwords directly from the extension popup
- 🎨 **Modern UI**: Clean, dark-themed interface matching PassQ's design
- 🚀 **Fast Access**: Quick access to your vault and password management
- 🔒 **Secure Communication**: Encrypted communication with your local PassQ instance

## Installation

### Development Installation

1. **Clone or Download**: Ensure you have the `firefox-extension` folder from your PassQ project

2. **Open Firefox Developer Tools**:
   - Open Firefox
   - Navigate to `about:debugging`
   - Click "This Firefox" in the left sidebar

3. **Load Extension**:
   - Click "Load Temporary Add-on"
   - Navigate to the `firefox-extension` folder
   - Select the `manifest.json` file

4. **Verify Installation**:
   - The PassQ extension should appear in your extensions list
   - You should see the PassQ icon in your browser toolbar

### Production Installation

For production use, the extension would need to be packaged and submitted to Mozilla Add-ons store.

## Usage

### Initial Setup

1. **Start PassQ Backend**: Ensure your PassQ backend is running on `http://localhost:8080`
2. **Start PassQ Frontend**: Ensure your PassQ frontend is running on `http://localhost:80` (or `http://localhost:3000` for development)
3. **Click Extension Icon**: Click the PassQ icon in your Firefox toolbar
4. **Login**: Enter your PassQ credentials in the popup

### Autofill Functionality

1. **Automatic Detection**: The extension automatically detects login forms on websites
2. **Autofill Buttons**: Look for small blue PassQ buttons next to username/password fields
3. **Click to Fill**: Click the autofill button to see available credentials
4. **Select Credential**: Choose the appropriate credential to fill the form

### Extension Popup

- **Current Site**: Shows credentials available for the current website
- **Search**: Search through all your passwords
- **Quick Autofill**: One-click autofill for the current site
- **Open Vault**: Direct link to your PassQ web interface

### Keyboard Shortcuts

- `Ctrl+Shift+L` (or `Cmd+Shift+L` on Mac): Trigger autofill on the current page

## Configuration

### Backend URL

By default, the extension connects to `http://localhost:5000`. To change this:

1. Edit `background.js`
2. Update the `apiUrl` variable
3. Reload the extension

### Security

- The extension only works with HTTPS websites (except localhost)
- All communication with the backend is secured
- Passwords are never stored in the extension itself
- Session tokens are managed securely

## Development

### File Structure

```
firefox-extension/
├── manifest.json          # Extension manifest
├── background.js          # Background script for API communication
├── content.js            # Content script for autofill functionality
├── content.css           # Styles for autofill elements
├── popup.html            # Extension popup interface
├── popup.js              # Popup functionality
├── popup.css             # Popup styles
├── icons/                # Extension icons
│   ├── icon-48.svg
│   └── icon-96.svg
└── README.md             # This file
```

### Key Components

1. **Background Script** (`background.js`):
   - Handles API communication with PassQ backend
   - Manages authentication state
   - Provides password fetching functionality

2. **Content Script** (`content.js`):
   - Detects login forms on web pages
   - Adds autofill buttons to input fields
   - Handles form filling and user interactions

3. **Popup** (`popup.html`, `popup.js`, `popup.css`):
   - Provides the main extension interface
   - Handles login, search, and quick autofill
   - Shows current site credentials

### Testing

1. **Load Extension**: Follow installation steps above
2. **Test Login**: Try logging into the extension
3. **Test Autofill**: Visit a website with a login form
4. **Check Console**: Use browser developer tools to check for errors

### Debugging

- **Background Script**: Check `about:debugging` → Extension → Inspect
- **Content Script**: Use regular page developer tools
- **Popup**: Right-click extension icon → Inspect Popup

## Troubleshooting

### Common Issues

1. **Extension Not Loading**:
   - Check that `manifest.json` is valid
   - Ensure all referenced files exist
   - Check Firefox console for errors

2. **Login Fails**:
   - Verify PassQ backend is running on correct port
   - Check network connectivity
   - Verify credentials are correct

3. **Autofill Not Working**:
   - Check if website uses standard login forms
   - Verify content script is injected
   - Check browser console for JavaScript errors

4. **No Credentials Shown**:
   - Ensure you're logged into the extension
   - Check that passwords exist in your PassQ vault
   - Verify API communication is working

### Support

For issues and support:
1. Check the browser console for error messages
2. Verify your PassQ backend is running and accessible
3. Ensure you're using a supported Firefox version (60+)

## Security Considerations

- Only install this extension from trusted sources
- The extension requires access to all websites to detect login forms
- Passwords are fetched securely from your local PassQ instance
- No data is sent to external servers
- Always keep your PassQ backend secure and up-to-date

## License

This extension is part of the PassQ password manager project.