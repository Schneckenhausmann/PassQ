-- Add missing columns to shares table
ALTER TABLE shares ADD COLUMN shared_with_user_id UUID NOT NULL REFERENCES users(id);
ALTER TABLE shares ADD COLUMN permission_level VARCHAR NOT NULL DEFAULT 'view';
ALTER TABLE shares ADD COLUMN expires_at TIMESTAMP;
ALTER TABLE shares ADD COLUMN created_at TIMESTAMP NOT NULL DEFAULT NOW();

-- Make password_id and folder_id nullable and add constraints
ALTER TABLE shares ALTER COLUMN password_id DROP NOT NULL;
ALTER TABLE shares ALTER COLUMN folder_id DROP NOT NULL;

-- Add constraint to ensure either password_id or folder_id is set
ALTER TABLE shares ADD CONSTRAINT shares_item_check CHECK (
    (password_id IS NOT NULL AND folder_id IS NULL) OR 
    (password_id IS NULL AND folder_id IS NOT NULL)
);