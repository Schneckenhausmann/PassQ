# Frontend Documentation

The frontend is implemented in React with a UI similar to Bitwarden. The application provides user authentication, registration, and dashboard functionality.

## Architecture Overview

The frontend follows React best practices with component-based architecture. It implements basic authentication flows and state management, though there are opportunities for security enhancements.

## Components

### App.js
Main application component that sets up routing and state management.
- Implements React Router for navigation
- Manages authentication state switching between login and registration
- Handles JWT token storage in localStorage after successful login
- Manages authentication state with token persistence
- Basic component structure with conditional rendering

### Login.js
User authentication component with master password input.
- Simple login form with username and password fields
- Basic error handling through console logging
- Axios integration for backend API communication
- Calls onLoginSuccess callback with JWT token and username
- No input validation on client side

### Registration.js
User registration component with comprehensive form handling.
- Input validation for username and password fields
- Password confirmation matching
- Client-side validation (8+ character minimum)
- Error state management for user feedback
- Axios integration for backend registration

### Dashboard.js
Main dashboard component displaying password management interface.
- Full API integration for real-time data management
- Integration with multiple child components (PasswordItem, FolderTree, ShareModal, SearchBar, AddEntryModal, EditEntryModal)
- State management for sharing functionality and search operations
- Complete CRUD operations for passwords and folders
- CSV import/export functionality with download and file upload capabilities
- Basic component structure with conditional rendering

### PasswordItem.js
Component for displaying individual password entries with favicon.
- Mock implementation for demonstration purposes

### FolderTree.js
Component for displaying and managing folder structure with subfolders.
- Mock implementation for demonstration purposes

## CSV Import/Export Functionality

The application includes comprehensive CSV import/export capabilities for password data migration and backup.

### Export Features
- **Secure export with password confirmation**: Export requires user to re-enter their master password for verification
- **One-click export**: Export all passwords and folders to CSV format after password confirmation
- **Standard format**: Compatible with common password managers (Bitwarden, LastPass, etc.)
- **Automatic download**: CSV file automatically downloads with timestamp
- **Enhanced security**: Export requires both JWT authentication and password re-verification

### Import Features
- **File upload interface**: Drag-and-drop or click to select CSV files
- **Multi-format support**: Automatically detects and supports CSV exports from multiple password managers
- **Format validation**: Validates CSV structure before processing
- **Automatic folder creation**: Creates folders during import if they don't exist
- **Bulk import**: Processes multiple password entries in a single operation
- **Error handling**: Provides clear feedback for malformed data or import failures

### Supported CSV Formats
The import functionality automatically detects and supports the following formats:

#### PassQ Format (Default Export)
- `name`: Website or service name
- `url`: Website URL
- `username`: Login username or email
- `password`: Password
- `notes`: Additional notes or comments
- `folder`: Folder name for organization

#### Bitwarden Format
- `folder`, `favorite`, `type`, `name`, `notes`, `fields`, `reprompt`, `login_uri`, `login_username`, `login_password`, `login_totp`

#### LastPass Format
- `url`, `username`, `password`, `extra`, `name`, `grouping`, `fav`

#### 1Password Format
- `Title`, `Website`, `Username`, `Password`, `One-time password`, `Favorite status`, `Archived status`, `Tags`, `Notes`

**Note**: Format detection is automatic based on column headers. No manual format selection is required.

### Security Considerations
- All CSV operations require valid JWT authentication
- **CSV export requires password re-verification**: Users must confirm their master password before exporting data
- Exported data maintains encryption for sensitive fields
- Import validation prevents malicious data injection
- File processing happens client-side before server transmission
- Password confirmation modal prevents unauthorized data exports

### ShareModal.js
Modal component for sharing folders or entries with other users.
- Mock implementation for demonstration purposes

### SearchBar.js
Real-time search functionality with cartoon styling.
- Provides instant search across password entries
- Modern UI with responsive design

### Icons.js
Comprehensive icon library providing consistent UI elements.
- Centralized icon management for the application
- Consistent styling and theming

### NewLogo.js
Updated logo component with enhanced retro styling.
- Modern branding implementation
- Responsive logo design

### AddEntryModal.js
Modal for adding new password entries with OTP support.
- Complete form handling for new entries
- OTP integration capabilities

### EditEntryModal.js
Modal for editing existing password entries.
- Full CRUD operations for password management
- Form validation and error handling

## State Management
Uses React's built-in useState hook for local component state:
- Authentication state management
- Form validation states
- Modal open/close states
- Mock data management

## Security Considerations

### Input Validation (Partial Implementation)
**Strengths:**
- Registration component includes basic client-side validation
- Password length validation (8+ characters)
- Username format validation

**Areas for Improvement:**
- Login component lacks client-side validation
- No sanitization of user inputs
- Missing comprehensive input validation patterns

### Error Handling (Basic Implementation)
**Current State:**
- Console.error logging for errors
- Basic error state management in components
- Generic error messages

**Recommendations:**
- Implement user-friendly error messages
- Add proper error boundaries
- Sanitize error information before display

### Data Security (Partially Implemented)
**Current State:**
- JWT token storage in localStorage (functional but not ideal)
- Token-based authentication for protected endpoints
- CORS properly configured for API communication
- Mock data implementation in Dashboard

**Recommendations:**
- Migrate from localStorage to secure HttpOnly cookies for token storage
- Add CSRF protection mechanisms
- Implement proper data sanitization
- Add token refresh mechanisms

### XSS Prevention (Needs Enhancement)
**Current State:**
- No explicit XSS protection measures
- Direct rendering of user input

**Recommendations:**
- Implement output encoding for dynamic content
- Add Content Security Policy headers
- Use React's built-in security features

## API Integration

### Backend Communication
- Uses Axios library for HTTP requests
- Base URL: `http://localhost:8080` (proxied through nginx on port 80)
- POST endpoints for `/login` and `/registration`
- Protected endpoints require JWT authentication via Authorization header
- CORS properly configured for cross-origin requests

### Authentication Flow
- Login endpoint returns JWT token in response.data.data
- Token stored in localStorage as 'authToken'
- Username stored in localStorage for session persistence
- Protected API calls include Authorization: Bearer <token> header

### Error Handling
- Basic try-catch blocks in API calls
- Console.error logging for debugging
- Generic error messages for users
- 401 Unauthorized responses handled for invalid/expired tokens

## Styling

### Retro Cartoon Design System
The frontend features a retro cartoon-style design with thick black borders and clean aesthetics:

**Design Philosophy:**
- Cartoon-style aesthetics with thick black borders (3px)
- Clean black and white theme with selective accent colors
- Smooth cartoon-style animations and transitions
- Responsive design for all screen sizes

**Key Visual Features:**
- **Thick Black Borders**: Consistent 3px black borders across all UI components
- **Cartoon Styling**: Bold, uppercase typography with wide letter spacing
- **Clean Contrast**: High contrast black and white design for optimal readability
- **Consistent Color Scheme**: CSS custom properties for maintainable theming
- **Interactive Elements**: Hover effects and smooth transitions

**CSS Architecture:**
- CSS custom properties (variables) for consistent theming
- Modular component-based styling
- Responsive breakpoints for mobile and tablet devices
- Smooth scrollbar styling for WebKit browsers
- Loading animations and shimmer effects

**Color Palette:**
- Primary: Modern blue tones (#3b82f6, #2563eb)
- Success: Fresh green (#10b981)
- Danger: Vibrant red (#ef4444)
- Background: Light gradients with glassmorphism (darkened by 30% for better contrast)
- Text: High contrast dark colors (#1a1a1a) for optimal readability
- Placeholder Text: Medium gray (#666666) for appropriate contrast

**Recent Accessibility Improvements:**
- Enhanced text contrast by converting all white text to dark colors
- Improved readability with proper color contrast ratios
- Fixed low-opacity text elements for better visibility
- Optimized background gradients for improved text legibility

**Interactive Components:**
- Buttons with glassmorphism and hover animations
- Form inputs with focus states and smooth transitions
- Cards and containers with subtle shadows and borders
- Navigation elements with modern styling

## Component Structure

```
App.js (Main)
├── Login.js
└── Registration.js
     │
Dashboard.js
├── PasswordItem.js
├── FolderTree.js
├── ShareModal.js
├── SearchBar.js
├── AddEntryModal.js
├── EditEntryModal.js
├── Icons.js
└── NewLogo.js
```

## Security Recommendations

### Input Validation Enhancement
- Implement comprehensive client-side validation for all forms
- Add input sanitization functions
- Use schema validation libraries (e.g., Zod)
- Implement regex patterns for username and password validation

### Secure Token Storage
- Currently using localStorage for JWT token storage (functional but not ideal)
- Implement migration to secure HttpOnly cookies
- Implement proper token expiration handling
- Add CSRF protection mechanisms
- Add automatic token refresh mechanisms

### XSS Prevention
- Implement output encoding for all dynamic content
- Add Content Security Policy headers
- Use React's built-in security features
- Sanitize all user inputs before rendering

### Error Handling Improvement
- Implement proper error boundaries
- Add user-friendly error messages
- Maintain detailed server-side logging
- Sanitize error information before client display

### Security Headers
- Implement Content Security Policy (CSP)
- Add X-XSS-Protection header
- Include X-Content-Type-Options header
- Set secure cookies flag

## UI/UX Improvements (Recently Implemented)

### Modern Design Implementation
- **Liquid Glass Design**: Implemented glassmorphism effects throughout the application
- **Light Theme**: Migrated from dark theme to modern light theme with better accessibility
- **Responsive Design**: Added comprehensive responsive breakpoints for mobile and tablet
- **Animation System**: Implemented smooth transitions and hover effects
- **Typography**: Upgraded to Inter font family for improved readability
- **Color System**: Established consistent color palette using CSS custom properties

### Enhanced User Experience
- **Loading States**: Added shimmer animations for better perceived performance
- **Interactive Feedback**: Implemented hover and focus states for all interactive elements
- **Visual Hierarchy**: Improved content organization with modern spacing and typography
- **Accessibility**: Enhanced contrast ratios and focus indicators
- **Cross-browser Compatibility**: Optimized styling for modern browsers

## Future Enhancements

### Authentication Flow
- JWT token-based authentication implemented
- Token storage in localStorage with username persistence
- Implement proper authentication context
- Add token refresh mechanisms
- Implement logout functionality
- Add session timeout handling

### Data Management
- Replace mock data with real API integration
- Implement proper data encryption
- Add backup and restore functionality

### Security Features
- Implement multi-factor authentication
- Add biometric authentication support
- Implement advanced sharing mechanisms
- Add audit logging for security events

### Design System Evolution
- Add dark mode toggle functionality
- Implement theme customization options
- Add more animation micro-interactions
- Enhance mobile-first responsive design
- Add accessibility features (screen reader support, keyboard navigation)

## Dependencies

### Core Dependencies
- `react`: UI library
- `react-router-dom`: Routing functionality
- `axios`: HTTP client for API communication

### Development Dependencies
- Standard React development tooling
- CSS modules support

## Security Testing Recommendations

### Static Analysis
- Implement security-focused code reviews
- Use ESLint with security plugins
- Regular dependency vulnerability scanning

### Dynamic Analysis
- Implement penetration testing for frontend vulnerabilities
- Add automated security scanning in CI/CD pipeline
- Test for common frontend vulnerabilities (XSS, CSRF)

### User Acceptance Testing
- Security-focused user testing scenarios
- Vulnerability reporting mechanisms
- Regular security assessments

## Best Practices Implemented

### Component Structure
- Modular component architecture
- Proper separation of concerns
- Reusable component design

### State Management
- Local state management with useState hook
- Conditional rendering for different states
- Basic error handling mechanisms

### Code Organization
- Clear component naming conventions
- Logical file structure
- Consistent coding patterns

## Areas for Security Improvement

### High Priority
1. Implement comprehensive input validation and sanitization
2. Add secure token storage mechanisms
3. Implement XSS prevention measures

### Medium Priority
1. Enhance error handling with proper boundaries
2. Add security headers implementation
3. Implement proper authentication context

### Low Priority
1. Add advanced authentication features (MFA, biometrics)
2. Implement comprehensive audit logging
3. Add performance monitoring and optimization

## References

- OWASP Frontend Security Guidelines: https://cheatsheetseries.owasp.org/cheatsheets/Frontend_Security_Cheat_Sheet.html
- React Security Best Practices: https://reactjs.org/docs/security.html
- OWASP Top 10: https://owasp.org/www-project-top-ten/