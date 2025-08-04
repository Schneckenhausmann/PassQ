# Backend Documentation

The backend is implemented in Rust using the Actix-web framework.

## Architecture Overview

The backend follows a modular architecture with proper separation of concerns. It uses Rust's type safety features and implements secure coding practices throughout the application.

## Modules

### main.rs
Entry point of the application. Sets up the web server and routes.
- Implements RESTful API endpoints
- Uses Actix-web framework for HTTP handling
- CORS middleware configuration for cross-origin requests
- JWT authentication middleware for protected endpoints
- Environment variable configuration management
- Structured error handling with logging

### db.rs
Handles database interactions with PostgreSQL using Diesel ORM.
- Connection pooling for efficient database access
- Environment variable configuration for database URL
- Proper error handling and logging
- Type-safe database operations

### auth.rs
Comprehensive authentication module with robust security features.
- Username sanitization and validation (alphanumeric + _ - characters only)
- Password strength validation (8+ chars, uppercase, lowercase, digit requirements)
- bcrypt hashing for secure password storage
- JWT token generation and validation with configurable expiration
- Environment variable management for secrets

### crypto.rs
Cryptographic operations module with enhanced security.
- **AES-256-GCM encryption** with proper nonce handling
- Secure key derivation from environment variables
- Fixed nonce vulnerability: nonces are now properly stored with encrypted data
- Password encryption/decryption functions for secure storage
- Environment-based encryption key management

### mfa.rs
Multi-factor authentication implementation.
- TOTP (Time-based One-Time Password) generation
- Secure MFA setup and management
- Rate limiting for authentication attempts

## Security Features

### CORS Configuration
- Cross-Origin Resource Sharing (CORS) middleware configured
- Allows requests from any origin for development purposes
- Supports all HTTP methods and headers
- Proper CORS headers included in all responses

### JWT Authentication
- All password and folder endpoints protected by JWT authentication
- Authorization header validation (Bearer token format)
- Token validation using the auth module's validate_token function
- Returns 401 Unauthorized for invalid or missing tokens

### Input Validation
- Username sanitization: Only allows alphanumeric characters, underscores, and hyphens
- Password strength validation: Enforces 8+ character minimum with uppercase, lowercase, and digit requirements
- Comprehensive input sanitization to prevent injection attacks

### Authentication Security
- bcrypt hashing for secure password storage with DEFAULT_COST
- JWT token generation with configurable expiration (default: 7 days)
- Environment variable management for sensitive data
- Proper error handling without information leakage

### Database Security
- Connection pooling for efficient and secure database access
- Environment variable configuration for database credentials
- Type-safe operations through Diesel ORM
- **User-scoped data access**: All queries filter by authenticated user ID
- Foreign key constraints ensure data integrity
- Proper indexing on user_id fields for performance

### User Authorization
- **Complete user isolation**: Users can only access their own data
- All password operations (CRUD) enforce user ownership
- All folder operations (CRUD) enforce user ownership
- JWT token validation extracts user context for authorization
- Comprehensive error handling prevents information leakage

### Logging and Monitoring
- Comprehensive logging throughout all modules
- Error logging with appropriate severity levels
- Security event tracking for authentication attempts

## Configuration Management

### Environment Variables
- `PORT`: Server port configuration (default: 8080)
- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key for JWT token generation (minimum 32 characters)
- `ENCRYPTION_KEY`: 32-character hex key for AES-256-GCM encryption

### Security Configuration
- Environment variables are properly validated and handled
- Fallback mechanisms for missing configuration
- Secure error handling that doesn't leak sensitive information

## API Endpoints

### Public Endpoints (No Authentication Required)

#### User Registration
```
POST /register
Content-Type: application/json

{
  "username": "string",
  "password": "string"
}
```

#### User Login
```
POST /login
Content-Type: application/json

{
  "username": "string",
  "password": "string"
}

Response:
{
  "success": true,
  "message": "Login successful",
  "data": "jwt_token_string"
}
```

### Protected Endpoints (JWT Authentication Required)

All protected endpoints require an Authorization header with a valid JWT token:
```
Authorization: Bearer <jwt_token>
```

#### Password Management
```
GET /passwords
Authorization: Bearer <jwt_token>

POST /passwords
Authorization: Bearer <jwt_token>
Content-Type: application/json

PUT /passwords/{id}
Authorization: Bearer <jwt_token>
Content-Type: application/json

DELETE /passwords/{id}
Authorization: Bearer <jwt_token>
```

#### Folder Management
```
GET /folders
Authorization: Bearer <jwt_token>

POST /folders
Authorization: Bearer <jwt_token>
Content-Type: application/json

DELETE /folders/{id}
Authorization: Bearer <jwt_token>
```

## Data Models

### User Models
```rust
pub struct UserRegistration {
    pub username: String,
    pub password: String,
}

pub struct UserLogin {
    pub username: String,
    pub password: String,
}

pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub created_at: chrono::NaiveDateTime,
}
```

### Password Models
```rust
pub struct Password {
    pub id: Uuid,
    pub user_id: Uuid,  // Foreign key to users table
    pub folder_id: Option<Uuid>,
    pub website: String,
    pub username: String,
    pub encrypted_password: Vec<u8>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

pub struct NewPassword {
    pub id: Uuid,
    pub user_id: Uuid,  // Ensures user ownership
    pub folder_id: Option<Uuid>,
    pub website: String,
    pub username: String,
    pub encrypted_password: Vec<u8>,
}
```

### Folder Models
```rust
pub struct Folder {
    pub id: Uuid,
    pub user_id: Uuid,  // Foreign key to users table
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

pub struct NewFolder {
    pub id: Uuid,
    pub user_id: Uuid,  // Ensures user ownership
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
}
```

### API Response
```rust
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}
```

## Development Practices

### Code Organization
- Modular architecture with clear separation of concerns
- Type-safe implementations using Rust's type system
- Comprehensive error handling and logging

### Security Best Practices
- Input validation and sanitization
- Secure password hashing with bcrypt
- Environment variable management for secrets
- Proper error handling without information leakage

### Testing Strategy
- Unit tests for core functionality
- Integration tests for API endpoints
- Security-focused testing practices

## Dependencies

### Core Dependencies
- `actix-web`: Web framework for HTTP server
- `actix-cors`: CORS middleware for cross-origin requests
- `diesel`: ORM for database operations
- `bcrypt`: Password hashing library
- `jsonwebtoken`: JWT token handling
- `uuid`: Unique identifier generation
- `serde`: Serialization/deserialization
- `dotenv`: Environment variable management

### Security Dependencies
- `ring`: Cryptographic operations (when implemented)
- `regex`: Input validation patterns

## Recent Security Enhancements ✅

### Database Schema Improvements
- ✅ **Implemented proper user schema** with bcrypt hashing and salt generation
- ✅ **Added user_id foreign keys** to passwords and folders tables
- ✅ **Implemented folder structure management** with user ownership
- ✅ **Database migrations** for schema updates
- ✅ **Proper indexing** on user_id fields for performance

### Security Fixes Applied
- ✅ **Fixed AES-256-GCM nonce vulnerability** in encryption module
- ✅ **Implemented complete user authorization** across all endpoints
- ✅ **Enhanced JWT token validation** with user context extraction
- ✅ **Added comprehensive error handling** with security logging
- ✅ **User data isolation** - users can only access their own data

## Future Enhancements

### Authentication Enhancements
- Multi-factor authentication integration (TOTP module exists)
- Session management improvements
- OAuth2 provider support
- Password reset functionality

### Security Improvements
- Rate limiting for authentication endpoints
- IP-based access controls
- Advanced threat detection mechanisms
- Audit logging for security events

### Sharing Functionality
- Implement password/folder sharing between users
- Permission-based access controls
- Share expiration and revocation

## Performance Considerations

### Database Optimization
- Connection pooling for efficient database access
- Query optimization through Diesel ORM
- Proper indexing strategies

### Caching Strategy
- Implement caching layers for frequently accessed data
- Optimize authentication token validation

## Monitoring and Logging

### Application Logging
- Comprehensive logging throughout all modules
- Structured logging for better analysis
- Security event tracking

### Error Handling
- Proper error handling without information leakage
- Graceful degradation for system failures
- Detailed error logging for debugging purposes