# PassQ - Secure Password Manager

PassQ is a comprehensive, secure password manager with modern features and a distinctive retro cartoon-style design.

## 🚀 Features

### Core Functionality
- 🔐 **Multi-user Support**: Complete user isolation with secure authentication
- 🛡️ **Strong Encryption**: AES-256-GCM encryption with proper nonce handling
- 🔑 **Multi-Factor Authentication**: TOTP support for enhanced security
- 📁 **Folder Organization**: Create folders and subfolders for password organization
- 🤝 **Sharing System**: Share folders or individual entries with other users
- 🌐 **Favicon Fetching**: Automatic website icon retrieval for visual identification

### Data Management
- 📤 **CSV Export/Import**: Secure password export with confirmation and import from popular managers
- 🔄 **Data Portability**: Full backup and restore capabilities for seamless migration
- 🔍 **Advanced Search**: Real-time search across all password entries

### Email & Communication
- 📧 **Password Reset Emails**: Secure password reset via email with modern design
- 📮 **SMTP Integration**: Configurable SMTP with TLS support for production deployment
- 🎨 **Branded Email Templates**: Professional email design matching PassQ's aesthetic

### Browser Integration
- 🦊 **Firefox Extension**: Seamless autofill functionality with comic book styling
- 🔄 **Auto-detection**: Smart login form detection and credential matching
- ⚡ **Quick Access**: One-click autofill and password management

### Design & UX
- 🎨 **Retro Cartoon Design**: Distinctive thick black borders and comic book aesthetics
- 📱 **Responsive Design**: Optimized for desktop, tablet, and mobile devices
- 🖤 **Clean Theme**: Black and white design with selective accent colors
- ✨ **Smooth Animations**: Cartoon-style transitions and hover effects

## 🏗️ Architecture

### Technology Stack
- **Backend**: Rust with Actix-web framework
- **Frontend**: React with modern hooks and responsive design
- **Database**: PostgreSQL with Diesel ORM
- **Extension**: Firefox WebExtension API
- **Deployment**: Docker Compose with production-ready configuration

### Project Structure
```
passq/
├── backend/              # Rust backend server
│   ├── src/
│   │   ├── main.rs      # Entry point and API endpoints
│   │   ├── auth.rs      # JWT authentication and user management
│   │   ├── crypto.rs    # AES-256-GCM encryption
│   │   ├── db.rs        # Database connection and queries
│   │   ├── email.rs     # SMTP email service and templates
│   │   ├── mfa.rs       # Multi-factor authentication (TOTP)
│   │   ├── models.rs    # Database models and relationships
│   │   └── schema.rs    # Database schema definitions
│   ├── migrations/      # Database migration files
│   ├── Cargo.toml      # Rust dependencies
│   └── Dockerfile      # Backend container configuration
├── frontend/            # React frontend application
│   ├── src/
│   │   ├── components/  # React components with cartoon styling
│   │   ├── services/    # API communication services
│   │   ├── utils/       # Utility functions and helpers
│   │   ├── App.js       # Main application component
│   │   └── index.js     # Application entry point
│   ├── package.json    # Node.js dependencies
│   └── Dockerfile      # Frontend container configuration
├── firefox-extension/   # Browser extension for autofill
│   ├── manifest.json   # Extension manifest and permissions
│   ├── background.js   # Background script for API communication
│   ├── content.js      # Content script for form detection
│   ├── popup.html      # Extension popup interface
│   ├── popup.js        # Popup functionality and styling
│   └── icons/          # Extension icons and assets
├── secrets/             # Production secrets management
│   ├── postgres_password.txt.example
│   ├── jwt_secret.txt.example
│   └── encryption_key.txt.example
├── scripts/             # Deployment and security scripts
│   ├── generate_secrets.sh    # Secure secrets generation
│   └── scan_vulnerabilities.sh # Docker security scanning
├── docs/               # Additional documentation
├── docker-compose.yml  # Development environment setup
├── docker-compose.production.yml # Production deployment with secrets
├── .env.example        # Environment variables template
└── README.md           # This comprehensive documentation
```

## 🚀 Quick Start

### Prerequisites
- Docker and Docker Compose
- Git

### Development Setup

1. **Clone the Repository**
   ```bash
   git clone <repository-url>
   cd passq
   ```

2. **Environment Configuration**
   ```bash
   # Copy and customize environment file
   cp .env.example .env
   
   # Edit .env file with secure values:
   # - Generate strong JWT_SECRET (minimum 32 characters)
   # - Generate secure ENCRYPTION_KEY (exactly 32 characters)
   # - Set strong POSTGRES_PASSWORD
   # - Configure AUDIT_SECRET for security logging
   # - Set up OAuth credentials (Microsoft/Google)
   ```

3. **Start Development Environment**
   ```bash
   docker-compose up --build
   ```

4. **Access the Application**
   - **Web Interface**: http://localhost:80
   - **Backend API**: http://localhost:8080
   - **Email Testing (MailHog)**: http://localhost:8025
   - **Database**: localhost:5432

### Firefox Extension Setup

1. **Development Installation**
   - Open Firefox and navigate to `about:debugging`
   - Click "This Firefox" → "Load Temporary Add-on"
   - Select `firefox-extension/manifest.json`

2. **Usage**
   - Click the PassQ icon in the toolbar
   - Login with your PassQ credentials
   - Visit websites with login forms for automatic detection
   - Use autofill buttons or keyboard shortcut `Ctrl+Shift+L`

## 🔧 Configuration

### Environment Variables

All environment variables are now consolidated in a single `.env` file in the root directory:

```bash
# Database Configuration
POSTGRES_PASSWORD=your-secure-database-password
DATABASE_URL=postgres://passq:${POSTGRES_PASSWORD}@db:5432/passq

# Security Keys (Generate strong random values!)
JWT_SECRET=your-super-secure-jwt-secret-minimum-32-chars
ENCRYPTION_KEY=your-32-character-hex-encryption-key
AUDIT_SECRET=your-audit-logging-secret-key

# OAuth Configuration (Optional)
MICROSOFT_CLIENT_ID=your-microsoft-client-id
MICROSOFT_CLIENT_SECRET=your-microsoft-client-secret
MICROSOFT_REDIRECT_URI=http://localhost:3000/auth/microsoft/callback

GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
GOOGLE_REDIRECT_URI=http://localhost:3000/auth/google/callback

# SMTP Configuration (Development)
SMTP_HOST=mailhog
SMTP_PORT=1025
SMTP_USERNAME=""
SMTP_PASSWORD=""
SMTP_FROM_EMAIL=noreply@passq.local
SMTP_FROM_NAME=PassQ Password Manager
```

### Production SMTP Configuration

For production deployment, replace the MailHog settings with real SMTP provider:

#### Gmail Example
```bash
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=your-email@gmail.com
SMTP_FROM_NAME=Your Company Name
```

#### AWS SES Example
```bash
SMTP_HOST=email-smtp.us-east-1.amazonaws.com
SMTP_PORT=587
SMTP_USERNAME=your-ses-smtp-username
SMTP_PASSWORD=your-ses-smtp-password
SMTP_FROM_EMAIL=noreply@yourdomain.com
SMTP_FROM_NAME=Your Company Name
```

#### SendGrid Example
```bash
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
SMTP_FROM_EMAIL=noreply@yourdomain.com
SMTP_FROM_NAME=Your Company Name
```

**Production Security Notes:**
- Always use port 587 (STARTTLS) or 465 (SSL/TLS)
- Never use port 25 or unencrypted connections
- The backend automatically enforces TLS for secure ports
- Use environment variables or secrets management for credentials
- For production database, change `sslmode=disable` to `sslmode=require`

## 🛡️ Security Features

### Encryption & Authentication
- **AES-256-GCM Encryption**: Military-grade encryption with proper nonce handling
- **JWT Authentication**: Stateless authentication with secure token validation
- **bcrypt Password Hashing**: Secure user password storage with salt
- **User Authorization**: Complete data isolation between users
- **Input Validation**: Comprehensive sanitization and validation

### Database Security
- **User-Scoped Access**: All data queries enforce user ownership
- **Foreign Key Constraints**: Database-level data integrity
- **Migration System**: Versioned database schema management
- **Connection Pooling**: Efficient and secure database connections

### Communication Security
- **HTTPS Enforcement**: Secure communication in production
- **CORS Configuration**: Proper cross-origin request handling
- **TLS Email Delivery**: Encrypted email transmission
- **Secure Headers**: Security headers for web application

### Docker Security
- **Pinned Image Versions**: All Docker images use specific version tags
- **Non-Root Users**: Backend and frontend containers run as non-root users
- **Security Constraints**: `no-new-privileges`, `read-only` filesystems where possible
- **Port Binding**: Services bound to localhost only for security
- **Resource Limits**: Memory and CPU limits to prevent resource exhaustion
- **Health Checks**: Container health monitoring for reliability
- **Vulnerability Scanning**: Automated security scanning with Trivy

### Secrets Management
- **Docker Secrets**: Production deployment uses Docker secrets for sensitive data
- **Environment Isolation**: Separate configurations for development and production
- **Secure Generation**: Automated scripts for generating cryptographically secure secrets
- **Example Templates**: Secure examples for all required secrets

## 📊 Database Schema

The application uses PostgreSQL with the following main tables:

- **`users`**: User accounts with hashed passwords and email addresses
- **`folders`**: User-owned folders for organizing passwords
- **`passwords`**: Encrypted password entries linked to users and folders
- **`shares`**: Folder/password sharing between users
- **`audit_logs`**: Security audit trail for user actions
- **`login_history`**: User login tracking and security monitoring
- **`password_reset_tokens`**: Secure password reset token management

### Database Management

```bash
# Run database migrations
docker-compose exec backend diesel migration run

# Check migration status
docker-compose exec backend diesel migration list

# Revert last migration
docker-compose exec backend diesel migration revert
```

## 🔒 Security Tools

### Secrets Generation

For production deployment, use the automated secrets generation script:

```bash
# Generate all required secrets
./scripts/generate_secrets.sh

# The script will create:
# - secrets/postgres_password.txt
# - secrets/jwt_secret.txt
# - secrets/encryption_key.txt
# - secrets/audit_secret.txt
# - Optionally: OAuth and SMTP secrets
```

### Vulnerability Scanning

Scan Docker images for security vulnerabilities:

```bash
# Scan all images for vulnerabilities
./scripts/scan_vulnerabilities.sh

# The script will:
# - Install Trivy if not present
# - Scan all PassQ Docker images
# - Generate detailed security reports
# - Highlight critical and high severity issues
```

### Production Secrets Management

The `secrets/` directory contains example files for production secrets:

- `postgres_password.txt.example` - Database password template
- `jwt_secret.txt.example` - JWT signing key template  
- `encryption_key.txt.example` - Data encryption key template

For production deployment:
1. Copy example files without `.example` extension
2. Fill with secure, randomly generated values
3. Use with `docker-compose.production.yml`
4. Ensure proper file permissions (600)

## 🎨 Design System

### Visual Identity
- **Retro Cartoon Aesthetic**: Thick 3px black borders throughout
- **Color Scheme**: Clean black and white with selective accent colors
- **Typography**: Bold, uppercase styling with wide letter spacing
- **Shadows**: Solid black shadows for depth and comic book feel
- **Borders**: Consistent 3px black borders on all interactive elements

### Component Styling
- **Buttons**: Cartoon-style with shadow effects and hover animations
- **Forms**: Enhanced input fields with focus states and validation
- **Cards**: Rounded corners with black borders and shadow effects
- **Navigation**: Sticky elements with clean separation
- **Icons**: Custom SVG icons matching the retro aesthetic

### Responsive Design
- **Mobile-First**: Optimized for mobile devices with touch-friendly interfaces
- **Tablet Support**: Enhanced layouts for tablet viewing
- **Desktop Experience**: Full-featured interface with advanced functionality
- **Cross-Browser**: Tested across modern browsers for consistency

## 🔄 Recent Updates

### Docker Security & Infrastructure (Latest)
- ✅ **Docker Security Hardening**: Pinned image versions, non-root users, security constraints
- ✅ **Vulnerability Scanning**: Automated Trivy-based security scanning script
- ✅ **Secrets Management**: Docker secrets for production, automated generation scripts
- ✅ **Environment Consolidation**: Single `.env` file for simplified configuration
- ✅ **Production Deployment**: Dedicated `docker-compose.production.yml` with security best practices
- ✅ **OAuth Integration**: Microsoft and Google OAuth support with proper redirect URIs

### Email System
- ✅ **SMTP Integration**: Complete email service with TLS support
- ✅ **Password Reset**: Secure email-based password reset functionality
- ✅ **Email Templates**: Modern, branded email design matching PassQ aesthetic
- ✅ **Production Configuration**: Comprehensive SMTP setup documentation
- ✅ **MailHog Integration**: Local email testing during development

### Authentication Enhancements
- ✅ **Username/Email Login**: Support for both username and email authentication
- ✅ **Enhanced Security**: Improved token validation and user session management
- ✅ **Password Reset Tokens**: Secure token-based password reset system

### Firefox Extension Improvements
- ✅ **Comic Book Styling**: Updated extension popup with cartoon theme
- ✅ **Rounded Action Buttons**: Enhanced button design with 8px border-radius
- ✅ **Enhanced Shadows**: Solid black shadows for stronger visual impact
- ✅ **Improved Spacing**: Better readability with optimized spacing
- ✅ **Full-Width Separators**: Clean visual separation in popup interface
- ✅ **Better Button Layout**: Improved action button spacing and alignment

### Data Management Features
- ✅ **CSV Export with Confirmation**: Secure password export with user verification
- ✅ **CSV Import Support**: Import from Bitwarden, LastPass, 1Password
- ✅ **Data Portability**: Complete backup and restore capabilities

### UI/UX Improvements
- ✅ **Advanced Layout System**: Eliminated page scrollbars with sticky navigation
- ✅ **Enhanced Sidebar**: Double-width sidebar for improved folder navigation
- ✅ **Professional Scrolling**: Clean, optimized scroll behavior
- ✅ **Retro Design Implementation**: Comprehensive cartoon-style aesthetic
- ✅ **Animation System**: Smooth transitions and hover effects
- ✅ **Mobile Responsiveness**: Complete mobile and tablet optimization

### Security Updates
- ✅ **Encryption Vulnerability Fix**: Resolved nonce handling in AES-256-GCM
- ✅ **User Authorization**: Complete endpoint authorization implementation
- ✅ **Database Security**: Added user_id foreign keys for data ownership
- ✅ **Enhanced Logging**: Security audit trail and error handling
- ✅ **Environment Security**: Configurable secrets management

## 🚀 Production Deployment

### Pre-Deployment Checklist

1. **Security Configuration**
   - [ ] Generate strong, unique secrets using `./scripts/generate_secrets.sh`
   - [ ] Run vulnerability scan with `./scripts/scan_vulnerabilities.sh`
   - [ ] Configure real SMTP provider with TLS
   - [ ] Enable database SSL with `sslmode=require`
   - [ ] Set up proper firewall rules
   - [ ] Configure HTTPS with valid SSL certificates
   - [ ] Set proper file permissions on secrets (600)

2. **Environment Setup**
   - [ ] Create production secrets in `secrets/` directory
   - [ ] Update SMTP configuration with production values
   - [ ] Configure proper domain names and URLs
   - [ ] Set up database backups
   - [ ] Configure log rotation and monitoring
   - [ ] Update OAuth redirect URIs for production domain

3. **Testing**
   - [ ] Test password reset email functionality
   - [ ] Verify SMTP TLS connection
   - [ ] Test user registration and authentication
   - [ ] Validate Firefox extension connectivity
   - [ ] Test OAuth integrations (Microsoft/Google)
   - [ ] Perform security audit

### Docker Production Setup

```bash
# Generate production secrets
./scripts/generate_secrets.sh

# Scan for vulnerabilities
./scripts/scan_vulnerabilities.sh

# Production deployment with Docker secrets
docker-compose -f docker-compose.production.yml up -d --build

# Monitor logs
docker-compose -f docker-compose.production.yml logs -f

# Database backup
docker-compose -f docker-compose.production.yml exec db pg_dump -U passq passq > backup.sql
```

### Security Best Practices

- **Secrets Management**: Use Docker secrets in production, never environment variables
- **Image Security**: Regularly update base images and scan for vulnerabilities
- **Network Security**: Use isolated Docker networks and bind services to localhost
- **File Permissions**: Ensure secrets files have restrictive permissions (600)
- **Monitoring**: Set up log monitoring and alerting for security events
- **Backups**: Regular encrypted database backups with tested restore procedures

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust best practices for backend development
- Use React hooks and functional components for frontend
- Maintain the retro cartoon design system
- Write comprehensive tests for new features
- Update documentation for any API changes
- Ensure security best practices in all code

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

For issues, questions, or contributions:

1. Check existing GitHub issues
2. Create a new issue with detailed description
3. Include steps to reproduce for bugs
4. Provide system information and logs when relevant

---

**PassQ** - Secure, stylish, and user-friendly password management for the modern web.