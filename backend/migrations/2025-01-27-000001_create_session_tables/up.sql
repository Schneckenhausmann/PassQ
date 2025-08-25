-- Create session management tables for enterprise-grade token and session tracking

-- Active sessions table
CREATE TABLE active_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(255) UNIQUE NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token_jti VARCHAR(255) NOT NULL,
    refresh_token_jti VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    device_fingerprint VARCHAR(255),
    device_name VARCHAR(255),
    device_type VARCHAR(50), -- 'web', 'mobile', 'desktop', 'api'
    location_country VARCHAR(100),
    location_region VARCHAR(100),
    location_city VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by_ip VARCHAR(45),
    last_seen_ip VARCHAR(45),
    session_flags JSONB DEFAULT '{}', -- Additional session metadata
    INDEX idx_active_sessions_user_id (user_id),
    INDEX idx_active_sessions_session_id (session_id),
    INDEX idx_active_sessions_last_activity (last_activity),
    INDEX idx_active_sessions_expires_at (expires_at)
);

-- Revoked tokens table for token blacklist
CREATE TABLE revoked_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jti VARCHAR(255) UNIQUE NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id VARCHAR(255),
    token_type VARCHAR(20) NOT NULL CHECK (token_type IN ('access', 'refresh')),
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revocation_reason VARCHAR(100) NOT NULL,
    revoked_by_user_id UUID REFERENCES users(id),
    revoked_by_admin BOOLEAN DEFAULT FALSE,
    original_expiry TIMESTAMPTZ,
    ip_address VARCHAR(45),
    user_agent TEXT,
    INDEX idx_revoked_tokens_jti (jti),
    INDEX idx_revoked_tokens_user_id (user_id),
    INDEX idx_revoked_tokens_expires_at (expires_at),
    INDEX idx_revoked_tokens_revoked_at (revoked_at)
);

-- Token analytics table for monitoring and auditing
CREATE TABLE token_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id VARCHAR(255),
    event_type VARCHAR(50) NOT NULL, -- 'issued', 'refreshed', 'revoked', 'expired', 'validated'
    token_type VARCHAR(20) NOT NULL CHECK (token_type IN ('access', 'refresh', 'pair')),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45),
    user_agent TEXT,
    success BOOLEAN NOT NULL DEFAULT TRUE,
    error_code VARCHAR(50),
    error_message TEXT,
    device_fingerprint VARCHAR(255),
    geolocation JSONB, -- {"country": "US", "region": "CA", "city": "San Francisco"}
    risk_score INTEGER DEFAULT 0, -- 0-100 risk assessment
    additional_data JSONB DEFAULT '{}',
    INDEX idx_token_analytics_user_id (user_id),
    INDEX idx_token_analytics_timestamp (timestamp),
    INDEX idx_token_analytics_event_type (event_type),
    INDEX idx_token_analytics_session_id (session_id)
);

-- Session security events table
CREATE TABLE session_security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL, -- 'concurrent_login', 'suspicious_location', 'device_change', 'token_theft_detected'
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45),
    user_agent TEXT,
    description TEXT,
    action_taken VARCHAR(100), -- 'session_terminated', 'user_notified', 'account_locked', 'none'
    resolved BOOLEAN DEFAULT FALSE,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id),
    metadata JSONB DEFAULT '{}',
    INDEX idx_session_security_events_session_id (session_id),
    INDEX idx_session_security_events_user_id (user_id),
    INDEX idx_session_security_events_timestamp (timestamp),
    INDEX idx_session_security_events_severity (severity)
);

-- Session limits configuration table
CREATE TABLE session_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    max_concurrent_sessions INTEGER NOT NULL DEFAULT 5,
    max_sessions_per_device INTEGER NOT NULL DEFAULT 3,
    session_timeout_minutes INTEGER NOT NULL DEFAULT 15, -- Access token timeout
    refresh_timeout_days INTEGER NOT NULL DEFAULT 7, -- Refresh token timeout
    enforce_single_session BOOLEAN DEFAULT FALSE,
    allow_concurrent_mobile BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_session_limits_user_id (user_id)
);

-- Device trust table for device-based session management
CREATE TABLE trusted_devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_fingerprint VARCHAR(255) NOT NULL,
    device_name VARCHAR(255),
    device_type VARCHAR(50),
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    trust_level VARCHAR(20) NOT NULL DEFAULT 'untrusted' CHECK (trust_level IN ('trusted', 'untrusted', 'suspicious', 'blocked')),
    trust_score INTEGER DEFAULT 0, -- 0-100 trust score
    ip_addresses JSONB DEFAULT '[]', -- Array of IP addresses seen from this device
    user_agent_patterns JSONB DEFAULT '[]', -- Array of user agent patterns
    location_history JSONB DEFAULT '[]', -- Array of location data
    session_count INTEGER DEFAULT 0,
    last_session_id VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, device_fingerprint),
    INDEX idx_trusted_devices_user_id (user_id),
    INDEX idx_trusted_devices_fingerprint (device_fingerprint),
    INDEX idx_trusted_devices_trust_level (trust_level),
    INDEX idx_trusted_devices_last_seen (last_seen)
);

-- Session monitoring rules table
CREATE TABLE session_monitoring_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_name VARCHAR(255) UNIQUE NOT NULL,
    rule_type VARCHAR(50) NOT NULL, -- 'concurrent_sessions', 'location_anomaly', 'device_anomaly', 'time_anomaly'
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    conditions JSONB NOT NULL, -- Rule conditions in JSON format
    actions JSONB NOT NULL, -- Actions to take when rule triggers
    threshold_value INTEGER,
    time_window_minutes INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_triggered TIMESTAMPTZ,
    trigger_count INTEGER DEFAULT 0,
    INDEX idx_session_monitoring_rules_enabled (enabled),
    INDEX idx_session_monitoring_rules_type (rule_type)
);

-- Insert default session monitoring rules
INSERT INTO session_monitoring_rules (rule_name, rule_type, severity, conditions, actions, threshold_value, time_window_minutes) VALUES
('Excessive Concurrent Sessions', 'concurrent_sessions', 'high', 
 '{"max_sessions": 10, "check_device_type": true}', 
 '{"terminate_oldest": true, "notify_user": true, "log_event": true}', 
 10, 5),

('Suspicious Location Login', 'location_anomaly', 'medium', 
 '{"distance_threshold_km": 1000, "time_threshold_hours": 2}', 
 '{"require_mfa": true, "notify_user": true, "log_event": true}', 
 1000, 120),

('New Device Login', 'device_anomaly', 'medium', 
 '{"new_device": true, "require_verification": true}', 
 '{"require_mfa": true, "notify_user": true, "log_event": true}', 
 1, 0),

('After Hours Access', 'time_anomaly', 'low', 
 '{"business_hours_start": 8, "business_hours_end": 18, "weekdays_only": true}', 
 '{"log_event": true, "increase_monitoring": true}', 
 0, 0),

('Token Refresh Anomaly', 'token_anomaly', 'high', 
 '{"refresh_frequency_threshold": 50, "time_window_minutes": 60}', 
 '{"revoke_tokens": true, "notify_security": true, "log_event": true}', 
 50, 60);

-- Create function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at columns
CREATE TRIGGER update_session_limits_updated_at BEFORE UPDATE ON session_limits
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_trusted_devices_updated_at BEFORE UPDATE ON trusted_devices
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_session_monitoring_rules_updated_at BEFORE UPDATE ON session_monitoring_rules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create function to clean up expired sessions and tokens
CREATE OR REPLACE FUNCTION cleanup_expired_sessions_and_tokens()
RETURNS INTEGER AS $$
DECLARE
    deleted_sessions INTEGER;
    deleted_tokens INTEGER;
    deleted_analytics INTEGER;
BEGIN
    -- Delete expired sessions
    DELETE FROM active_sessions WHERE expires_at < NOW() OR last_activity < NOW() - INTERVAL '30 days';
    GET DIAGNOSTICS deleted_sessions = ROW_COUNT;
    
    -- Delete expired revoked tokens (keep for 30 days after expiry)
    DELETE FROM revoked_tokens WHERE expires_at < NOW() - INTERVAL '30 days';
    GET DIAGNOSTICS deleted_tokens = ROW_COUNT;
    
    -- Delete old analytics data (keep for 90 days)
    DELETE FROM token_analytics WHERE timestamp < NOW() - INTERVAL '90 days';
    GET DIAGNOSTICS deleted_analytics = ROW_COUNT;
    
    -- Log cleanup results
    INSERT INTO token_analytics (user_id, event_type, token_type, success, additional_data)
    VALUES (
        '00000000-0000-0000-0000-000000000000'::UUID,
        'cleanup',
        'system',
        TRUE,
        json_build_object(
            'deleted_sessions', deleted_sessions,
            'deleted_tokens', deleted_tokens,
            'deleted_analytics', deleted_analytics
        )
    );
    
    RETURN deleted_sessions + deleted_tokens + deleted_analytics;
END;
$$ LANGUAGE plpgsql;

-- Create function to detect concurrent session violations
CREATE OR REPLACE FUNCTION check_session_limits(p_user_id UUID, p_device_fingerprint VARCHAR)
RETURNS TABLE(
    violation_type VARCHAR,
    current_count INTEGER,
    limit_value INTEGER,
    action_required VARCHAR
) AS $$
DECLARE
    user_limits RECORD;
    current_sessions INTEGER;
    device_sessions INTEGER;
BEGIN
    -- Get user session limits
    SELECT * INTO user_limits FROM session_limits WHERE user_id = p_user_id;
    
    -- If no limits set, use defaults
    IF user_limits IS NULL THEN
        INSERT INTO session_limits (user_id) VALUES (p_user_id);
        SELECT * INTO user_limits FROM session_limits WHERE user_id = p_user_id;
    END IF;
    
    -- Count current active sessions
    SELECT COUNT(*) INTO current_sessions 
    FROM active_sessions 
    WHERE user_id = p_user_id AND is_active = TRUE AND expires_at > NOW();
    
    -- Count sessions for this device
    SELECT COUNT(*) INTO device_sessions 
    FROM active_sessions 
    WHERE user_id = p_user_id AND device_fingerprint = p_device_fingerprint 
          AND is_active = TRUE AND expires_at > NOW();
    
    -- Check concurrent session limit
    IF current_sessions >= user_limits.max_concurrent_sessions THEN
        RETURN QUERY SELECT 
            'concurrent_sessions'::VARCHAR,
            current_sessions,
            user_limits.max_concurrent_sessions,
            'terminate_oldest'::VARCHAR;
    END IF;
    
    -- Check device session limit
    IF device_sessions >= user_limits.max_sessions_per_device THEN
        RETURN QUERY SELECT 
            'device_sessions'::VARCHAR,
            device_sessions,
            user_limits.max_sessions_per_device,
            'terminate_device_oldest'::VARCHAR;
    END IF;
    
    -- Check single session enforcement
    IF user_limits.enforce_single_session AND current_sessions > 0 THEN
        RETURN QUERY SELECT 
            'single_session'::VARCHAR,
            current_sessions,
            1,
            'terminate_all_others'::VARCHAR;
    END IF;
    
    RETURN;
END;
$$ LANGUAGE plpgsql;

-- Create indexes for performance optimization
CREATE INDEX CONCURRENTLY idx_active_sessions_user_active ON active_sessions(user_id, is_active) WHERE is_active = TRUE;
CREATE INDEX CONCURRENTLY idx_active_sessions_cleanup ON active_sessions(expires_at, last_activity);
CREATE INDEX CONCURRENTLY idx_revoked_tokens_cleanup ON revoked_tokens(expires_at);
CREATE INDEX CONCURRENTLY idx_token_analytics_cleanup ON token_analytics(timestamp);

-- Add comments for documentation
COMMENT ON TABLE active_sessions IS 'Stores active user sessions with detailed tracking information';
COMMENT ON TABLE revoked_tokens IS 'Blacklist of revoked JWT tokens to prevent reuse';
COMMENT ON TABLE token_analytics IS 'Analytics and audit trail for token operations';
COMMENT ON TABLE session_security_events IS 'Security events related to user sessions';
COMMENT ON TABLE session_limits IS 'Per-user session limits and configuration';
COMMENT ON TABLE trusted_devices IS 'Device trust management for enhanced security';
COMMENT ON TABLE session_monitoring_rules IS 'Configurable rules for session monitoring and alerting';

COMMENT ON FUNCTION cleanup_expired_sessions_and_tokens() IS 'Automated cleanup of expired sessions, tokens, and analytics data';
COMMENT ON FUNCTION check_session_limits(UUID, VARCHAR) IS 'Validates session limits and returns violations';