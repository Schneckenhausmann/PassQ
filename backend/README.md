# Backend

The backend is implemented in Rust using the Actix-web framework with enhanced security features.

## Setup

1. Install Rust: https://www.rust-lang.org/tools/install
2. Install PostgreSQL and create a database
3. Install Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`
4. Copy `.env.example` to `.env` and update with your configuration
5. Run database migrations: `diesel migration run`
6. Run the server: `cargo run`

## Environment Variables

⚠️ **Security Notice**: Use strong, randomly generated values for production!

- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key for JWT tokens (minimum 32 characters)
- `ENCRYPTION_KEY`: 32-character hex key for AES-256-GCM encryption
- `PORT`: Server port (default: 8080)

## Database Migrations

The application uses Diesel for database migrations:

```bash
# Run all pending migrations
diesel migration run

# Revert the last migration
diesel migration revert

# Check migration status
diesel migration list
```

## Security Features

- **AES-256-GCM Encryption**: Secure password storage with proper nonce handling
- **JWT Authentication**: Stateless authentication with user context
- **User Authorization**: Complete data isolation between users
- **bcrypt Password Hashing**: Secure user password storage
- **Input Validation**: Comprehensive sanitization and validation

## Modules

- `main.rs`: Entry point, server setup, and API endpoints with user authorization
- `db.rs`: Database connection pooling and management
- `auth.rs`: JWT authentication, password hashing, and user validation
- `crypto.rs`: AES-256-GCM encryption with secure nonce handling
- `mfa.rs`: Multi-factor authentication (TOTP)
- `models.rs`: Database models with user relationships
- `schema.rs`: Database schema definitions

## Recent Security Updates

- ✅ Fixed encryption nonce vulnerability
- ✅ Implemented proper user authorization across all endpoints
- ✅ Added user_id foreign keys to ensure data ownership
- ✅ Enhanced error handling with security logging
- ✅ Database migrations for schema improvements