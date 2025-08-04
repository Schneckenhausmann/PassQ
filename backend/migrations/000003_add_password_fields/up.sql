-- Add new fields to passwords table
ALTER TABLE passwords ADD COLUMN notes TEXT;
ALTER TABLE passwords ADD COLUMN otp_secret VARCHAR;
ALTER TABLE passwords ADD COLUMN attachments JSONB DEFAULT '[]'::jsonb;

-- Create index for attachments JSONB field
CREATE INDEX idx_passwords_attachments ON passwords USING gin (attachments);