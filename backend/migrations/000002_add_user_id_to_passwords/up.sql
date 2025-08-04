-- Add user_id column to passwords table for proper authorization
ALTER TABLE passwords ADD COLUMN user_id UUID NOT NULL REFERENCES users(id);

-- Create index for better query performance
CREATE INDEX idx_passwords_user_id ON passwords(user_id);