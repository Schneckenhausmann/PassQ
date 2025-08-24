-- Encrypt metadata: Change website and username fields to encrypted binary storage

-- Add new encrypted columns
ALTER TABLE passwords ADD COLUMN encrypted_website BYTEA;
ALTER TABLE passwords ADD COLUMN encrypted_username BYTEA;

-- Note: Data migration should be handled by application code
-- The old columns (website, username) will be dropped in a future migration
-- after data has been migrated to the encrypted columns