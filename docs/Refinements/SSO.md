# Single Sign-On (SSO) Implementation Plan

## Overview

This document outlines the implementation plan for adding Microsoft 365 and Google Single Sign-On (SSO) authentication to PassQ. This will provide users with convenient OAuth-based login options while maintaining the existing username/password authentication system.

## 🎯 Goals

- **Seamless Integration**: Add SSO without disrupting existing authentication
- **User Choice**: Provide multiple authentication methods (local, MS365, Google)
- **Security**: Maintain PassQ's high security standards with OAuth 2.0
- **User Experience**: Clean, intuitive login interface with distinct provider buttons
- **Data Mapping**: Properly map SSO user data to PassQ user accounts

## 🏗️ Technical Architecture

### Authentication Flow

```
1. User clicks "Sign in with Microsoft" or "Sign in with Google"
2. Frontend redirects to OAuth provider authorization URL
3. User authenticates with provider (MS365/Google)
4. Provider redirects back with authorization code
5. Backend exchanges code for access token
6. Backend retrieves user profile from provider
7. Backend creates/updates user account in PassQ
8. Backend issues PassQ JWT token
9. Frontend receives JWT and proceeds to dashboard
```

### Database Schema Changes

#### New Table: `oauth_accounts`
```sql
CREATE TABLE oauth_accounts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL, -- 'microsoft' or 'google'
    provider_user_id VARCHAR(255) NOT NULL, -- OAuth provider's user ID
    email VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    avatar_url VARCHAR(500),
    access_token_hash VARCHAR(255), -- Hashed for security
    refresh_token_hash VARCHAR(255), -- Hashed for security
    token_expires_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(provider, provider_user_id),
    UNIQUE(provider, email)
);
```

#### Update `users` Table
```sql
ALTER TABLE users ADD COLUMN auth_method VARCHAR(20) DEFAULT 'local';
-- Values: 'local', 'microsoft', 'google', 'hybrid'
ALTER TABLE users ADD COLUMN is_sso_user BOOLEAN DEFAULT FALSE;
ALTER TABLE users ADD COLUMN sso_display_name VARCHAR(255);
ALTER TABLE users ADD COLUMN sso_avatar_url VARCHAR(500);
```

## 🔧 Backend Implementation

### 1. Dependencies (Cargo.toml)
```toml
[dependencies]
# Existing dependencies...
oauth2 = "4.4"
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"
url = "2.4"
base64 = "0.21"
```

### 2. Environment Variables
```bash
# Microsoft 365 OAuth
MICROSOFT_CLIENT_ID=your-microsoft-app-id
MICROSOFT_CLIENT_SECRET=your-microsoft-app-secret
MICROSOFT_REDIRECT_URI=http://localhost:8080/auth/microsoft/callback

# Google OAuth
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
GOOGLE_REDIRECT_URI=http://localhost:8080/auth/google/callback

# OAuth Configuration
OAUTH_STATE_SECRET=your-oauth-state-secret-32-chars
```

### 3. New Backend Modules

#### `src/oauth.rs`
```rust
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, AuthUrl, TokenUrl, Client
};

pub struct OAuthConfig {
    pub microsoft: MicrosoftConfig,
    pub google: GoogleConfig,
}

pub struct MicrosoftConfig {
    pub client: Client,
    pub auth_url: String,
    pub token_url: String,
}

pub struct GoogleConfig {
    pub client: Client,
    pub auth_url: String,
    pub token_url: String,
}

// OAuth provider implementations
pub async fn get_microsoft_auth_url(config: &MicrosoftConfig) -> (String, String);
pub async fn get_google_auth_url(config: &GoogleConfig) -> (String, String);
pub async fn exchange_microsoft_code(config: &MicrosoftConfig, code: String) -> Result<TokenResponse>;
pub async fn exchange_google_code(config: &GoogleConfig, code: String) -> Result<TokenResponse>;
pub async fn get_microsoft_user_info(access_token: &str) -> Result<UserInfo>;
pub async fn get_google_user_info(access_token: &str) -> Result<UserInfo>;
```

#### `src/sso_auth.rs`
```rust
use crate::models::{User, OAuthAccount};
use crate::oauth::{UserInfo, OAuthConfig};

pub async fn handle_sso_login(
    provider: &str,
    user_info: UserInfo,
    pool: &PgPool
) -> Result<User> {
    // 1. Check if OAuth account exists
    // 2. If exists, return associated user
    // 3. If not, check if user with email exists
    // 4. If user exists, link OAuth account
    // 5. If no user, create new user with OAuth account
    // 6. Update last login and OAuth token info
}

pub async fn link_oauth_account(
    user_id: i32,
    provider: &str,
    user_info: UserInfo,
    pool: &PgPool
) -> Result<OAuthAccount>;

pub async fn unlink_oauth_account(
    user_id: i32,
    provider: &str,
    pool: &PgPool
) -> Result<()>;
```

### 4. API Endpoints

#### Authentication Endpoints
```rust
// GET /auth/microsoft/login
pub async fn microsoft_login() -> HttpResponse {
    // Generate auth URL and redirect
}

// GET /auth/google/login  
pub async fn google_login() -> HttpResponse {
    // Generate auth URL and redirect
}

// GET /auth/microsoft/callback
pub async fn microsoft_callback(
    query: web::Query<CallbackQuery>
) -> HttpResponse {
    // Handle Microsoft OAuth callback
}

// GET /auth/google/callback
pub async fn google_callback(
    query: web::Query<CallbackQuery>
) -> HttpResponse {
    // Handle Google OAuth callback
}

// POST /auth/sso/link
pub async fn link_sso_account(
    req: HttpRequest,
    json: web::Json<LinkSSORequest>
) -> HttpResponse {
    // Link SSO account to existing user
}

// DELETE /auth/sso/unlink/{provider}
pub async fn unlink_sso_account(
    req: HttpRequest,
    path: web::Path<String>
) -> HttpResponse {
    // Unlink SSO account from user
}
```

#### User Management Endpoints
```rust
// GET /api/user/sso-accounts
pub async fn get_user_sso_accounts(
    req: HttpRequest
) -> HttpResponse {
    // Return linked SSO accounts for user
}

// GET /api/user/profile
// Update existing endpoint to include SSO info
```

## 🎨 Frontend Implementation

### 1. Dependencies (package.json)
```json
{
  "dependencies": {
    "@microsoft/msal-browser": "^3.0.0",
    "google-auth-library": "^9.0.0"
  }
}
```

### 2. Login Component Updates

#### `src/components/Login.js`
```jsx
import React, { useState } from 'react';
import { MicrosoftLoginButton } from './MicrosoftLoginButton';
import { GoogleLoginButton } from './GoogleLoginButton';

const Login = () => {
  const [loginMethod, setLoginMethod] = useState('local');

  return (
    <div className="login-container">
      <div className="login-card">
        <h1>Welcome to PassQ</h1>
        
        {/* SSO Login Buttons */}
        <div className="sso-login-section">
          <h3>Quick Sign In</h3>
          <div className="sso-buttons">
            <MicrosoftLoginButton />
            <GoogleLoginButton />
          </div>
          <div className="divider">
            <span>or</span>
          </div>
        </div>

        {/* Traditional Login Form */}
        <div className="traditional-login">
          <h3>Sign in with PassQ Account</h3>
          {/* Existing login form */}
        </div>
      </div>
    </div>
  );
};
```

#### `src/components/MicrosoftLoginButton.js`
```jsx
import React from 'react';
import { microsoftAuthService } from '../services/microsoftAuth';

const MicrosoftLoginButton = () => {
  const handleMicrosoftLogin = async () => {
    try {
      await microsoftAuthService.login();
    } catch (error) {
      console.error('Microsoft login failed:', error);
    }
  };

  return (
    <button 
      className="sso-button microsoft-button"
      onClick={handleMicrosoftLogin}
    >
      <img src="/icons/microsoft-logo.svg" alt="Microsoft" />
      <span>Continue with Microsoft</span>
    </button>
  );
};
```

#### `src/components/GoogleLoginButton.js`
```jsx
import React from 'react';
import { googleAuthService } from '../services/googleAuth';

const GoogleLoginButton = () => {
  const handleGoogleLogin = async () => {
    try {
      await googleAuthService.login();
    } catch (error) {
      console.error('Google login failed:', error);
    }
  };

  return (
    <button 
      className="sso-button google-button"
      onClick={handleGoogleLogin}
    >
      <img src="/icons/google-logo.svg" alt="Google" />
      <span>Continue with Google</span>
    </button>
  );
};
```

### 3. Authentication Services

#### `src/services/microsoftAuth.js`
```javascript
class MicrosoftAuthService {
  constructor() {
    this.baseURL = process.env.REACT_APP_API_URL || 'http://localhost:8080';
  }

  async login() {
    // Redirect to backend Microsoft OAuth endpoint
    window.location.href = `${this.baseURL}/auth/microsoft/login`;
  }

  async handleCallback(code, state) {
    // Handle OAuth callback (if needed)
  }
}

export const microsoftAuthService = new MicrosoftAuthService();
```

#### `src/services/googleAuth.js`
```javascript
class GoogleAuthService {
  constructor() {
    this.baseURL = process.env.REACT_APP_API_URL || 'http://localhost:8080';
  }

  async login() {
    // Redirect to backend Google OAuth endpoint
    window.location.href = `${this.baseURL}/auth/google/login`;
  }

  async handleCallback(code, state) {
    // Handle OAuth callback (if needed)
  }
}

export const googleAuthService = new GoogleAuthService();
```

### 4. CSS Styling

#### SSO Button Styles
```css
.sso-login-section {
  margin-bottom: 2rem;
}

.sso-buttons {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.sso-button {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  padding: 0.875rem 1.5rem;
  border: 3px solid #000;
  border-radius: 8px;
  background: #fff;
  color: #000;
  font-weight: bold;
  font-size: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 4px 4px 0px #000;
}

.sso-button:hover {
  transform: translate(-2px, -2px);
  box-shadow: 6px 6px 0px #000;
}

.sso-button:active {
  transform: translate(0, 0);
  box-shadow: 2px 2px 0px #000;
}

.microsoft-button {
  background: #0078d4;
  color: white;
}

.google-button {
  background: #4285f4;
  color: white;
}

.sso-button img {
  width: 20px;
  height: 20px;
}

.divider {
  text-align: center;
  position: relative;
  margin: 1.5rem 0;
}

.divider::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 2px;
  background: #000;
}

.divider span {
  background: #fff;
  padding: 0 1rem;
  font-weight: bold;
  text-transform: uppercase;
}
```

## 🔐 Security Considerations

### 1. Token Security
- **Hash Storage**: Store OAuth tokens as hashes, not plaintext
- **Token Rotation**: Implement refresh token rotation
- **Expiration**: Respect provider token expiration times
- **Revocation**: Handle token revocation gracefully

### 2. State Validation
- **CSRF Protection**: Use cryptographically secure state parameters
- **State Verification**: Validate state parameter on callback
- **Nonce Handling**: Implement proper nonce validation for OIDC

### 3. User Data Protection
- **Minimal Scope**: Request only necessary OAuth scopes
- **Data Encryption**: Encrypt sensitive OAuth data at rest
- **Audit Logging**: Log all SSO authentication events
- **Privacy Compliance**: Handle user data per privacy regulations

### 4. Account Linking Security
- **Email Verification**: Verify email ownership before linking
- **Existing Account Protection**: Prevent unauthorized account takeover
- **Multi-Factor**: Maintain MFA requirements for linked accounts

## 📋 Implementation Phases

### Phase 1: Backend Foundation (Week 1-2)
- [ ] Set up OAuth dependencies and configuration
- [ ] Create database migrations for OAuth tables
- [ ] Implement OAuth service modules
- [ ] Create basic OAuth endpoints
- [ ] Add environment variable configuration

### Phase 2: Microsoft 365 Integration (Week 2-3)
- [ ] Implement Microsoft OAuth flow
- [ ] Create Microsoft-specific endpoints
- [ ] Add Microsoft user profile mapping
- [ ] Test Microsoft authentication flow
- [ ] Handle Microsoft-specific edge cases

### Phase 3: Google Integration (Week 3-4)
- [ ] Implement Google OAuth flow
- [ ] Create Google-specific endpoints
- [ ] Add Google user profile mapping
- [ ] Test Google authentication flow
- [ ] Handle Google-specific edge cases

### Phase 4: Frontend Implementation (Week 4-5)
- [ ] Create SSO login buttons
- [ ] Implement authentication services
- [ ] Update login page design
- [ ] Add OAuth callback handling
- [ ] Implement error handling and user feedback

### Phase 5: Account Management (Week 5-6)
- [ ] Add account linking functionality
- [ ] Create SSO account management UI
- [ ] Implement account unlinking
- [ ] Add user profile SSO information
- [ ] Test account linking scenarios

### Phase 6: Security & Testing (Week 6-7)
- [ ] Security audit and penetration testing
- [ ] Implement comprehensive error handling
- [ ] Add audit logging for SSO events
- [ ] Performance testing and optimization
- [ ] Documentation and user guides

### Phase 7: Production Deployment (Week 7-8)
- [ ] Production environment configuration
- [ ] SSL/TLS certificate setup
- [ ] OAuth app registration with providers
- [ ] Production testing and validation
- [ ] User migration and rollout plan

## 🧪 Testing Strategy

### Unit Tests
- OAuth token exchange functions
- User profile mapping logic
- Account linking/unlinking operations
- Security validation functions

### Integration Tests
- Complete OAuth flows (Microsoft & Google)
- Account creation and linking scenarios
- Error handling and edge cases
- Security boundary testing

### End-to-End Tests
- Full user authentication journeys
- Cross-browser compatibility
- Mobile responsiveness
- Performance under load

## 📊 Success Metrics

### User Experience
- **Login Success Rate**: >95% successful SSO logins
- **Login Time**: <10 seconds for complete SSO flow
- **User Adoption**: >30% of new users choose SSO
- **Error Rate**: <2% authentication failures

### Security
- **Zero Security Incidents**: No OAuth-related security breaches
- **Token Security**: All tokens properly encrypted and rotated
- **Audit Compliance**: 100% of SSO events logged
- **Privacy Compliance**: GDPR/CCPA compliant data handling

### Technical
- **Uptime**: >99.9% availability for SSO endpoints
- **Performance**: <500ms response time for OAuth callbacks
- **Scalability**: Support for 10,000+ concurrent SSO users
- **Maintainability**: Clean, documented, testable code

## 🔄 Future Enhancements

### Additional Providers
- **Apple ID**: iOS/macOS user convenience
- **GitHub**: Developer-focused authentication
- **LinkedIn**: Professional network integration
- **SAML**: Enterprise SSO support

### Advanced Features
- **Just-in-Time Provisioning**: Automatic user creation
- **Group Mapping**: Map OAuth groups to PassQ roles
- **Session Management**: Advanced session control
- **Conditional Access**: Risk-based authentication

### Enterprise Features
- **Admin Console**: SSO configuration management
- **Audit Dashboard**: SSO usage analytics
- **Compliance Reports**: Security and usage reporting
- **API Integration**: Programmatic SSO management

## 📚 Resources

### Documentation
- [Microsoft Identity Platform](https://docs.microsoft.com/en-us/azure/active-directory/develop/)
- [Google OAuth 2.0](https://developers.google.com/identity/protocols/oauth2)
- [OAuth 2.0 Security Best Practices](https://tools.ietf.org/html/draft-ietf-oauth-security-topics)
- [OpenID Connect Core](https://openid.net/specs/openid-connect-core-1_0.html)

### Libraries
- [oauth2-rs](https://docs.rs/oauth2/) - Rust OAuth 2.0 client
- [reqwest](https://docs.rs/reqwest/) - HTTP client for API calls
- [MSAL.js](https://github.com/AzureAD/microsoft-authentication-library-for-js) - Microsoft authentication
- [Google Auth Library](https://github.com/googleapis/google-auth-library-nodejs) - Google authentication

---

**Note**: This implementation plan prioritizes security, user experience, and maintainability. All OAuth implementations should follow industry best practices and undergo thorough security review before production deployment.