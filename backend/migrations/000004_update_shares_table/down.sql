-- Remove the constraint
ALTER TABLE shares DROP CONSTRAINT IF EXISTS shares_item_check;

-- Remove added columns
ALTER TABLE shares DROP COLUMN IF EXISTS shared_with_user_id;
ALTER TABLE shares DROP COLUMN IF EXISTS permission_level;
ALTER TABLE shares DROP COLUMN IF EXISTS expires_at;
ALTER TABLE shares DROP COLUMN IF EXISTS created_at;

-- Restore NOT NULL constraints (if needed)
-- Note: This assumes the original table had these as NOT NULL
-- ALTER TABLE shares ALTER COLUMN password_id SET NOT NULL;
-- ALTER TABLE shares ALTER COLUMN folder_id SET NOT NULL;