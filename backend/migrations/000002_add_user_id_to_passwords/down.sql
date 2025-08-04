-- Remove user_id column from passwords table
DROP INDEX IF EXISTS idx_passwords_user_id;
ALTER TABLE passwords DROP COLUMN user_id;