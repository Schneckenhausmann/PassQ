-- Drop indexes
DROP INDEX IF EXISTS idx_users_is_sso_user;
DROP INDEX IF EXISTS idx_users_auth_method;
DROP INDEX IF EXISTS idx_oauth_accounts_email;
DROP INDEX IF EXISTS idx_oauth_accounts_provider;
DROP INDEX IF EXISTS idx_oauth_accounts_user_id;

-- Remove OAuth-related columns from users table
ALTER TABLE users DROP COLUMN IF EXISTS sso_avatar_url;
ALTER TABLE users DROP COLUMN IF EXISTS sso_display_name;
ALTER TABLE users DROP COLUMN IF EXISTS is_sso_user;
ALTER TABLE users DROP COLUMN IF EXISTS auth_method;

-- Drop OAuth accounts table
DROP TABLE IF EXISTS oauth_accounts;