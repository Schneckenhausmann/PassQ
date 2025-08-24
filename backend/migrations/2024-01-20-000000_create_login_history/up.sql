-- Create login_history table for tracking user login locations and IP addresses
CREATE TABLE login_history (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ip_address VARCHAR(45) NOT NULL,
    country VARCHAR(100),
    region VARCHAR(100),
    city VARCHAR(100),
    latitude VARCHAR(20),
    longitude VARCHAR(20),
    timezone VARCHAR(50),
    isp VARCHAR(255),
    user_agent TEXT,
    login_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_suspicious BOOLEAN NOT NULL DEFAULT FALSE
);

-- Create indexes for efficient querying
CREATE INDEX idx_login_history_user_id ON login_history(user_id);
CREATE INDEX idx_login_history_login_time ON login_history(login_time);
CREATE INDEX idx_login_history_ip_address ON login_history(ip_address);
CREATE INDEX idx_login_history_suspicious ON login_history(is_suspicious);