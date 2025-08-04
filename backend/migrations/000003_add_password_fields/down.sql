-- Remove new fields from passwords table
DROP INDEX IF EXISTS idx_passwords_attachments;
ALTER TABLE passwords DROP COLUMN IF EXISTS attachments;
ALTER TABLE passwords DROP COLUMN IF EXISTS otp_secret;
ALTER TABLE passwords DROP COLUMN IF EXISTS notes;