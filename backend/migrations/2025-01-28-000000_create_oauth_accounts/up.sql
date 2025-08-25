-- Create OAuth accounts table for SSO integration
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

-- Add OAuth-related columns to users table
ALTER TABLE users ADD COLUMN auth_method VARCHAR(20) DEFAULT 'local';
-- Values: 'local', 'microsoft', 'google', 'hybrid'
ALTER TABLE users ADD COLUMN is_sso_user BOOLEAN DEFAULT FALSE;
ALTER TABLE users ADD COLUMN sso_display_name VARCHAR(255);
ALTER TABLE users ADD COLUMN sso_avatar_url VARCHAR(500);

-- Create indexes for performance
CREATE INDEX idx_oauth_accounts_user_id ON oauth_accounts(user_id);
CREATE INDEX idx_oauth_accounts_provider ON oauth_accounts(provider);
CREATE INDEX idx_oauth_accounts_email ON oauth_accounts(email);
CREATE INDEX idx_users_auth_method ON users(auth_method);
CREATE INDEX idx_users_is_sso_user ON users(is_sso_user);