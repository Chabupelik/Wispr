CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    telegram_id VARCHAR(64),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS whitelists (
    id SERIAL PRIMARY KEY,
    type VARCHAR(10) NOT NULL,
    value VARCHAR(64) NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(type, value)
);

CREATE TABLE IF NOT EXISTS device_events (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(255) NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(50) NOT NULL,
    browser_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS wisp_access_logs (
    id BIGSERIAL PRIMARY KEY,
    telegram_id VARCHAR(64),
    target_host VARCHAR(255) NOT NULL,
    target_port INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_wisp_logs_created_at ON wisp_access_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_wisp_logs_telegram_id ON wisp_access_logs(telegram_id);

CREATE TABLE IF NOT EXISTS hwid_names (
    hwid VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
