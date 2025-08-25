-- Rollback migration for session management tables

-- Drop functions first (due to dependencies)
DROP FUNCTION IF EXISTS cleanup_expired_sessions_and_tokens();
DROP FUNCTION IF EXISTS check_session_limits(UUID, VARCHAR);
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop indexes
DROP INDEX IF EXISTS idx_active_sessions_user_active;
DROP INDEX IF EXISTS idx_active_sessions_cleanup;
DROP INDEX IF EXISTS idx_revoked_tokens_cleanup;
DROP INDEX IF EXISTS idx_token_analytics_cleanup;

-- Drop tables in reverse order (due to foreign key dependencies)
DROP TABLE IF EXISTS session_monitoring_rules;
DROP TABLE IF EXISTS trusted_devices;
DROP TABLE IF EXISTS session_limits;
DROP TABLE IF EXISTS session_security_events;
DROP TABLE IF EXISTS token_analytics;
DROP TABLE IF EXISTS revoked_tokens;
DROP TABLE IF EXISTS active_sessions;