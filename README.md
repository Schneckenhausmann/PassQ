# PassQ - Secure Password Manager

PassQ is a secure password manager with the following features:
- Multi-user support
- Strong encrypted passwords with master password
- Support for one-time passwords (MFA)
- Favicon fetching for websites
- **Modern frontend with liquid glass design and light theme**
- **Responsive design optimized for all devices**
- **Contemporary UI with glassmorphism effects and smooth animations**
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
├── docs/             # Project documentation
│   ├── backend.md     # Backend documentation
│   └── frontend.md    # Frontend documentation
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

### UI/UX Improvements (Latest)
- ✅ **Modern Design System**: Implemented liquid glass effects with glassmorphism
- ✅ **Light Theme**: Migrated to contemporary light theme with improved accessibility
- ✅ **Text Readability**: Enhanced contrast with dark text on light backgrounds for optimal readability
- ✅ **Background Optimization**: Darkened gradient backgrounds by 30% to improve text visibility
- ✅ **Color Contrast**: Fixed low-opacity text elements and improved overall accessibility
- ✅ **Responsive Design**: Added comprehensive mobile and tablet support
- ✅ **Animation System**: Smooth transitions and hover effects throughout the app
- ✅ **Typography**: Upgraded to Inter font family for better readability
- ✅ **Interactive Elements**: Enhanced buttons, forms, and navigation with modern styling

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