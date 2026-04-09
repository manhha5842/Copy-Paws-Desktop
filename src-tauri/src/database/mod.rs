// Database module for SQLite operations
pub mod models;
pub mod schema;

use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use rusqlite::{Connection, params};
use rusqlite::types::ValueRef;
use std::path::Path;
use self::models::{Device, Clip};

const CLIP_SORT_EPOCH_SQL: &str = r#"
COALESCE(
    CASE
        WHEN trim(created_at) GLOB '[0-9]*' AND length(trim(created_at)) > 10
            THEN CAST(substr(trim(created_at), 1, 10) AS INTEGER)
        WHEN trim(created_at) GLOB '[0-9]*'
            THEN CAST(trim(created_at) AS INTEGER)
        ELSE CAST(strftime('%s', created_at) AS INTEGER)
    END,
    0
)
"#;

pub struct Database {
    conn: Connection,
}

fn normalize_timestamp_value(value: ValueRef<'_>) -> String {
    let raw = match value {
        ValueRef::Null => return String::new(),
        ValueRef::Integer(value) => value.to_string(),
        ValueRef::Real(value) => value.trunc().to_string(),
        ValueRef::Text(value) => String::from_utf8_lossy(value).trim().to_string(),
        ValueRef::Blob(value) => String::from_utf8_lossy(value).trim().to_string(),
    };

    normalize_timestamp_string(&raw).unwrap_or(raw)
}

fn normalize_timestamp_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(timestamp) = trimmed.parse::<i64>() {
        let absolute = timestamp.unsigned_abs();
        let parsed = if absolute >= 1_000_000_000_000_000_000 {
            Utc.timestamp_millis_opt(timestamp / 1_000_000).single()
        } else if absolute >= 1_000_000_000_000_000 {
            Utc.timestamp_millis_opt(timestamp / 1_000).single()
        } else if absolute >= 1_000_000_000_000 {
            Utc.timestamp_millis_opt(timestamp).single()
        } else {
            Utc.timestamp_opt(timestamp, 0).single()
        };

        if let Some(date_time) = parsed {
            return Some(date_time.to_rfc3339());
        }
    }

    if let Ok(date_time) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(date_time.with_timezone(&Utc).to_rfc3339());
    }

    for format in [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
    ] {
        if let Ok(date_time) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Some(Utc.from_utc_datetime(&date_time).to_rfc3339());
        }
    }

    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let date_time = date.and_hms_opt(0, 0, 0)?;
        return Some(Utc.from_utc_datetime(&date_time).to_rfc3339());
    }

    None
}

impl Database {
    /// Create a new database connection
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(schema::CREATE_TABLES_SQL)?;
        Ok(())
    }

    // Device operations
    pub fn add_device(&self, device: &Device) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO devices (device_id, name, shared_secret, platform, last_seen, is_blocked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &device.device_id,
                &device.name,
                &device.shared_secret,
                &device.platform,
                &device.last_seen,
                &device.is_blocked,
            ],
        )?;
        Ok(())
    }

    pub fn get_devices(&self) -> Result<Vec<Device>> {
        let mut stmt = self.conn.prepare(
            "SELECT device_id, name, shared_secret, platform, last_seen, is_blocked, created_at
             FROM devices ORDER BY last_seen DESC"
        )?;

        let devices = stmt.query_map([], |row| {
            Ok(Device {
                device_id: row.get(0)?,
                name: row.get(1)?,
                shared_secret: row.get(2)?,
                platform: row.get(3)?,
                last_seen: row.get(4)?,
                is_blocked: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(devices)
    }

    pub fn get_device(&self, device_id: &str) -> Result<Option<Device>> {
        let mut stmt = self.conn.prepare(
            "SELECT device_id, name, shared_secret, platform, last_seen, is_blocked, created_at
             FROM devices WHERE device_id = ?1"
        )?;

        let mut devices = stmt.query_map(params![device_id], |row| {
            Ok(Device {
                device_id: row.get(0)?,
                name: row.get(1)?,
                shared_secret: row.get(2)?,
                platform: row.get(3)?,
                last_seen: row.get(4)?,
                is_blocked: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;

        if let Some(device) = devices.next() {
            Ok(Some(device?))
        } else {
            Ok(None)
        }
    }

    pub fn update_device_name(&self, device_id: &str, new_name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE devices SET name = ?1 WHERE device_id = ?2",
            params![new_name, device_id],
        )?;
        Ok(())
    }

    pub fn revoke_device(&self, device_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM devices WHERE device_id = ?1",
            params![device_id],
        )?;
        Ok(())
    }

    pub fn block_device(&self, device_id: &str, blocked: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE devices SET is_blocked = ?1 WHERE device_id = ?2",
            params![blocked, device_id],
        )?;
        Ok(())
    }

    /// Update device last_seen timestamp
    pub fn update_device_last_seen(&self, device_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE devices SET last_seen = CURRENT_TIMESTAMP WHERE device_id = ?1",
            params![device_id],
        )?;
        Ok(())
    }

    // Clip operations
    pub fn add_clip(&self, clip: &Clip) -> Result<()> {
        self.conn.execute(
            "INSERT INTO clips (id, content, content_hash, content_type, source_device, source_app, created_at, is_pinned)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &clip.id,
                &clip.content,
                &clip.content_hash,
                &clip.content_type,
                &clip.source_device,
                &clip.source_app,
                &clip.created_at,
                &clip.is_pinned,
            ],
        )?;
        Ok(())
    }

    pub fn get_clips(&self, limit: usize) -> Result<Vec<Clip>> {
        let query = format!(
            "SELECT id, content, content_hash, content_type, source_device, source_app, created_at, is_pinned
             FROM clips
             ORDER BY {sort_sql} DESC, rowid DESC
             LIMIT ?1",
            sort_sql = CLIP_SORT_EPOCH_SQL,
        );
        let mut stmt = self.conn.prepare(&query)?;

        let clips = stmt.query_map(params![limit], |row| {
            Ok(Clip {
                id: row.get(0)?,
                content: row.get(1)?,
                content_hash: row.get(2)?,
                content_type: row.get(3)?,
                source_device: row.get(4)?,
                source_app: row.get(5)?,
                created_at: normalize_timestamp_value(row.get_ref(6)?),
                is_pinned: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(clips)
    }

    pub fn get_latest_clip(&self) -> Result<Option<Clip>> {
        let mut clips = self.get_clips(1)?;
        Ok(clips.pop())
    }

    pub fn pin_clip(&self, id: &str, pinned: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE clips SET is_pinned = ?1 WHERE id = ?2",
            params![pinned, id],
        )?;
        Ok(())
    }

    pub fn delete_clip(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM clips WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn clear_all_clips(&self) -> Result<()> {
        self.conn.execute("DELETE FROM clips WHERE is_pinned = 0", [])?;
        Ok(())
    }

    /// Clean up old clips based on retention policy
    pub fn cleanup_old_clips(&self, max_items: usize, ttl_days: i64) -> Result<()> {
        // Delete by count (keep only max_items most recent, excluding pinned)
        let cleanup_by_count = format!(
            "DELETE FROM clips WHERE is_pinned = 0 AND id NOT IN (
                SELECT id FROM clips WHERE is_pinned = 0 ORDER BY {sort_sql} DESC, rowid DESC LIMIT ?1
            )",
            sort_sql = CLIP_SORT_EPOCH_SQL,
        );
        self.conn.execute(&cleanup_by_count, params![max_items])?;

        // Delete by age (older than ttl_days, excluding pinned)
        let cleanup_by_age = format!(
            "DELETE FROM clips WHERE is_pinned = 0
             AND {sort_sql} < (CAST(strftime('%s', 'now') AS INTEGER) - (?1 * 86400))",
            sort_sql = CLIP_SORT_EPOCH_SQL,
        );
        self.conn.execute(&cleanup_by_age, params![ttl_days])?;

        Ok(())
    }

    // Settings operations
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) 
             VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            params![key, value],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::new(":memory:").unwrap();
        assert!(db.get_clips(10).is_ok());
    }

    #[test]
    fn test_add_and_get_clip() {
        let db = Database::new(":memory:").unwrap();
        let clip = Clip {
            id: "test-123".to_string(),
            content: "Hello World".to_string(),
            content_hash: "hash123".to_string(),
            content_type: Some("text".to_string()),
            source_device: Some("LOCAL".to_string()),
            source_app: Some("Test".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            is_pinned: false,
        };

        db.add_clip(&clip).unwrap();
        let clips = db.get_clips(10).unwrap();
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].content, "Hello World");
    }

    #[test]
    fn test_get_clips_normalizes_mixed_timestamp_formats() {
        let db = Database::new(":memory:").unwrap();

        db.conn.execute(
            "INSERT INTO clips (id, content, content_hash, content_type, created_at, is_pinned)
             VALUES (?1, ?2, ?3, 'text', ?4, 0)",
            params!["sqlite-style", "SQLite", "hash-sqlite", "2026-04-08 09:30:45"],
        ).unwrap();

        db.conn.execute(
            "INSERT INTO clips (id, content, content_hash, content_type, created_at, is_pinned)
             VALUES (?1, ?2, ?3, 'text', ?4, 0)",
            params!["epoch-ms", "Epoch", "hash-epoch", "1775640645000"],
        ).unwrap();

        let clips = db.get_clips(10).unwrap();

        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].id, "epoch-ms");
        assert_eq!(clips[1].id, "sqlite-style");
        assert_eq!(clips[0].created_at, "2026-04-08T09:30:45+00:00");
        assert_eq!(clips[1].created_at, "2026-04-08T09:30:45+00:00");
    }
}
