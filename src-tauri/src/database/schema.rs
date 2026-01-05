// Database schema definitions

pub const CREATE_TABLES_SQL: &str = r#"
-- Devices table
CREATE TABLE IF NOT EXISTS devices (
    device_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    shared_secret TEXT NOT NULL,
    platform TEXT CHECK(platform IN ('iOS', 'Android')) NOT NULL,
    last_seen DATETIME,
    is_blocked BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Clips table
CREATE TABLE IF NOT EXISTS clips (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    source_device TEXT,
    source_app TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_pinned BOOLEAN DEFAULT 0,
    FOREIGN KEY(source_device) REFERENCES devices(device_id) ON DELETE SET NULL
);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_clips_created_at ON clips(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clips_hash ON clips(content_hash);
CREATE INDEX IF NOT EXISTS idx_clips_pinned ON clips(is_pinned);
CREATE INDEX IF NOT EXISTS idx_devices_last_seen ON devices(last_seen DESC);
"#;
