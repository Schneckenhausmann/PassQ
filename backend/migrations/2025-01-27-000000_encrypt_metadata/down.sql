-- Reverse metadata encryption: Remove encrypted columns

ALTER TABLE passwords DROP COLUMN IF EXISTS encrypted_website;
ALTER TABLE passwords DROP COLUMN IF EXISTS encrypted_username;