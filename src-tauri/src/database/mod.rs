// Database module for SQLite operations
pub mod models;
pub mod schema;

use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;
use self::models::{Device, Clip};

pub struct Database {
    conn: Connection,
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

    // Clip operations
    pub fn add_clip(&self, clip: &Clip) -> Result<()> {
        self.conn.execute(
            "INSERT INTO clips (id, content, content_hash, source_device, source_app, created_at, is_pinned)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &clip.id,
                &clip.content,
                &clip.content_hash,
                &clip.source_device,
                &clip.source_app,
                &clip.created_at,
                &clip.is_pinned,
            ],
        )?;
        Ok(())
    }

    pub fn get_clips(&self, limit: usize) -> Result<Vec<Clip>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, content_hash, source_device, source_app, created_at, is_pinned
             FROM clips ORDER BY created_at DESC LIMIT ?1"
        )?;

        let clips = stmt.query_map(params![limit], |row| {
            Ok(Clip {
                id: row.get(0)?,
                content: row.get(1)?,
                content_hash: row.get(2)?,
                source_device: row.get(3)?,
                source_app: row.get(4)?,
                created_at: row.get(5)?,
                is_pinned: row.get(6)?,
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
        self.conn.execute(
            "DELETE FROM clips WHERE is_pinned = 0 AND id NOT IN (
                SELECT id FROM clips WHERE is_pinned = 0 ORDER BY created_at DESC LIMIT ?1
            )",
            params![max_items],
        )?;

        // Delete by age (older than ttl_days, excluding pinned)
        self.conn.execute(
            "DELETE FROM clips WHERE is_pinned = 0 
             AND created_at < datetime('now', '-' || ?1 || ' days')",
            params![ttl_days],
        )?;

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
}
