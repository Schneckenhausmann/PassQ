# PassQ - Secure Password Manager

PassQ is a secure password manager with the following features:
- Multi-user support
- Strong encrypted passwords with master password
- Support for one-time passwords (MFA)
- Favicon fetching for websites
- **CSV export/import with password confirmation for data portability**
- **Firefox browser extension for seamless autofill**
- **Retro cartoon-style design with thick black borders**
- **Responsive design optimized for all devices**
- **Clean black and white aesthetic with cartoon-style animations**
- Folder creation with subfolders
- Ability to share folders or entries with other users

## Project Structure
```
passq/
├── backend/          # Rust backend
│   ├── src/
│   │   ├── main.rs    # Entry point
│   │   ├── db.rs      # Database interactions
│   │   ├── auth.rs    # Authentication logic
│   │   ├── crypto.rs  # Encryption/decryption
│   │   └── mfa.rs     # MFA handling
│   ├── Cargo.toml    # Rust dependencies
│   └── README.md     # Backend documentation
├── frontend/         # React frontend
│   ├── public/
│   │   └── favicon.ico
│   ├── src/
│   │   ├── components/  # React components
│   │   ├── App.js       # Main app component
│   │   └── index.js     # Entry point
│   ├── package.json    # Node dependencies
│   └── README.md       # Frontend documentation
├── firefox-extension/ # Browser extension for password autofill
├── docs/             # Project documentation
│   ├── backend.md     # Backend documentation
│   ├── frontend.md    # Frontend documentation
│   └── firefox-extension.md # Firefox extension documentation
├── docker-compose.yml # For local development with PostgreSQL
└── README.md          # Main project documentation
```

## Getting Started

### Backend
```bash
cd backend
cargo run
```

### Frontend
```bash
cd frontend
npm start
```

## Security Measures
- **AES-256-GCM Encryption**: Secure password encryption with proper nonce handling
- **JWT Authentication**: Stateless authentication with secure token validation
- **User Authorization**: All endpoints enforce proper user access controls
- **Database Security**: User-scoped data access with foreign key constraints
- **Secure Password Hashing**: bcrypt with salt for user passwords
- **Environment Security**: Configurable secrets via environment variables

## Database Schema
The application uses PostgreSQL with the following main tables:
- `users`: User accounts with hashed passwords
- `folders`: User-owned folders for organizing passwords
- `passwords`: Encrypted password entries linked to users
- `shares`: Folder/password sharing between users

## Recent Updates

### Data Management Features (Latest)
- ✅ **CSV Export with Password Confirmation**: Secure password export requiring user password verification
- ✅ **CSV Import Support**: Import passwords from various password managers (Bitwarden, LastPass, 1Password)
- ✅ **Data Portability**: Full backup and restore capabilities for user data migration

### UI/UX Improvements
- ✅ **Retro Cartoon Design System**: Implemented thick black borders and cartoon-style aesthetics
- ✅ **Clean Theme**: Black and white design with selective accent colors for clarity
- ✅ **Text Readability**: Bold typography with high contrast for optimal readability
- ✅ **Border Consistency**: Applied 3px black borders across all UI components
- ✅ **Button Styling**: Cartoon-style buttons with shadow effects and hover animations
- ✅ **Responsive Design**: Added comprehensive mobile and tablet support
- ✅ **Animation System**: Smooth cartoon-style transitions and hover effects
- ✅ **Typography**: Bold, uppercase styling with wide letter spacing for retro feel
- ✅ **Interactive Elements**: Unified cartoon-style design language throughout the app

### Security Updates
- ✅ Fixed encryption nonce vulnerability in AES-256-GCM implementation
- ✅ Implemented proper user authorization across all API endpoints
- ✅ Added user_id foreign key to passwords table
- ✅ Enhanced error handling with security logging
- ⚠️ **Action Required**: Update environment variables with strong secrets before production deployment

## Environment Configuration
Before running the application, ensure these environment variables are set with strong values:
```bash
# Backend (.env)
DATABASE_URL=postgresql://username:password@localhost/passq
JWT_SECRET=your_very_secure_jwt_secret_here_minimum_32_chars
ENCRYPTION_KEY=your_32_character_encryption_key_here

# Docker Compose
POSTGRES_PASSWORD=your_secure_database_password
```