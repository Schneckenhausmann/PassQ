-- Add email column to users table
ALTER TABLE users ADD COLUMN email VARCHAR NOT NULL DEFAULT '';

-- Create unique index for email
CREATE UNIQUE INDEX idx_users_email ON users(email);

-- Remove the default constraint after adding the column
ALTER TABLE users ALTER COLUMN email DROP DEFAULT;