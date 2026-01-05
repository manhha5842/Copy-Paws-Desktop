// Clipboard monitoring module for cross-platform clipboard access and change detection

use anyhow::Result;
use arboard::Clipboard;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time;

use crate::crypto::calculate_hash;
use crate::database::models::SyncMode;

const MAX_CONTENT_SIZE: usize = 2 * 1024 * 1024; // 2MB limit

pub struct ClipboardMonitor {
    clipboard: Arc<Mutex<Clipboard>>,
    last_content_hash: Arc<Mutex<Option<String>>>,
    last_remote_clip_id: Arc<Mutex<Option<String>>>,
    last_remote_hash: Arc<Mutex<Option<String>>>,
    last_remote_timestamp: Arc<Mutex<Option<Instant>>>,
    suppress_window_ms: u64,
    sync_mode: Arc<Mutex<SyncMode>>,
}

#[derive(Debug, Clone)]
pub struct ClipboardChange {
    pub content: String,
    pub content_hash: String,
    pub timestamp: Instant,
}

impl ClipboardMonitor {
    /// Create a new clipboard monitor
    pub fn new(suppress_window_ms: u64, sync_mode: SyncMode) -> Result<Self> {
        let clipboard = Clipboard::new()?;
        
        Ok(Self {
            clipboard: Arc::new(Mutex::new(clipboard)),
            last_content_hash: Arc::new(Mutex::new(None)),
            last_remote_clip_id: Arc::new(Mutex::new(None)),
            last_remote_hash: Arc::new(Mutex::new(None)),
            last_remote_timestamp: Arc::new(Mutex::new(None)),
            suppress_window_ms,
            sync_mode: Arc::new(Mutex::new(sync_mode)),
        })
    }

    /// Start monitoring clipboard changes
    pub async fn start_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(ClipboardChange) -> Result<()> + Send + Sync + 'static,
    {
        let clipboard = self.clipboard.clone();
        let last_content_hash = self.last_content_hash.clone();
        let last_remote_hash = self.last_remote_hash.clone();
        let last_remote_timestamp = self.last_remote_timestamp.clone();
        let suppress_window_ms = self.suppress_window_ms;
        let sync_mode = self.sync_mode.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(500)); // Check every 500ms

            loop {
                interval.tick().await;

                // Check if sync is paused
                let current_sync_mode = sync_mode.lock().await.clone();
                if current_sync_mode == SyncMode::Paused {
                    continue;
                }

                // Get current clipboard content
                let content = {
                    let mut clip = clipboard.lock().await;
                    match clip.get_text() {
                        Ok(text) => text,
                        Err(_) => continue, // No text content
                    }
                };

                // Validate content size
                if content.len() > MAX_CONTENT_SIZE {
                    eprintln!("Clipboard content exceeds 2MB limit, skipping");
                    continue;
                }

                // Calculate hash
                let current_hash = calculate_hash(&content);

                // Check if content has changed
                let last_hash = last_content_hash.lock().await.clone();
                if Some(current_hash.clone()) == last_hash {
                    continue; // No change
                }

                // Check if this is within the suppress window (anti-loop)
                if Self::should_suppress(&current_hash, &last_remote_hash, &last_remote_timestamp, suppress_window_ms).await {
                    println!("Suppressing clipboard change (anti-loop)");
                    
                    // Update last hash but don't trigger callback
                    *last_content_hash.lock().await = Some(current_hash.clone());
                    continue;
                }

                // Update last hash
                *last_content_hash.lock().await = Some(current_hash.clone());

                // Create change event
                let change = ClipboardChange {
                    content: content.clone(),
                    content_hash: current_hash,
                    timestamp: Instant::now(),
                };

                // Trigger callback
                if let Err(e) = callback(change) {
                    eprintln!("Error in clipboard change callback: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Set the clipboard content (from remote)
    pub async fn set_clipboard(&self, content: &str, clip_id: &str) -> Result<()> {
        // Validate content size
        if content.len() > MAX_CONTENT_SIZE {
            return Err(anyhow::anyhow!("Content exceeds 2MB limit"));
        }

        // Calculate hash
        let hash = calculate_hash(content);

        // Update suppress state
        *self.last_remote_clip_id.lock().await = Some(clip_id.to_string());
        *self.last_remote_hash.lock().await = Some(hash.clone());
        *self.last_remote_timestamp.lock().await = Some(Instant::now());

        // Set clipboard
        let mut clipboard = self.clipboard.lock().await;
        clipboard.set_text(content)?;

        // Update last content hash
        *self.last_content_hash.lock().await = Some(hash);

        Ok(())
    }

    /// Get current clipboard content
    pub async fn get_clipboard(&self) -> Result<String> {
        let mut clipboard = self.clipboard.lock().await;
        let content = clipboard.get_text()?;
        
        if content.len() > MAX_CONTENT_SIZE {
            return Err(anyhow::anyhow!("Content exceeds 2MB limit"));
        }
        
        Ok(content)
    }

    /// Set sync mode
    pub async fn set_sync_mode(&self, mode: SyncMode) {
        *self.sync_mode.lock().await = mode;
    }

    /// Get sync mode
    pub async fn get_sync_mode(&self) -> SyncMode {
        self.sync_mode.lock().await.clone()
    }

    /// Check if clipboard change should be suppressed (anti-loop mechanism)
    async fn should_suppress(
        current_hash: &str,
        last_remote_hash: &Arc<Mutex<Option<String>>>,
        last_remote_timestamp: &Arc<Mutex<Option<Instant>>>,
        suppress_window_ms: u64,
    ) -> bool {
        let remote_hash = last_remote_hash.lock().await.clone();
        let remote_timestamp = last_remote_timestamp.lock().await.clone();

        if let (Some(hash), Some(timestamp)) = (remote_hash, remote_timestamp) {
            let elapsed = timestamp.elapsed();
            let window = Duration::from_millis(suppress_window_ms);

            // If within suppress window and hash matches, suppress
            if elapsed < window && hash == current_hash {
                return true;
            }
        }

        false
    }

    /// Clear suppress state (for testing)
    pub async fn clear_suppress_state(&self) {
        *self.last_remote_clip_id.lock().await = None;
        *self.last_remote_hash.lock().await = None;
        *self.last_remote_timestamp.lock().await = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clipboard_monitor_creation() {
        let monitor = ClipboardMonitor::new(2000, SyncMode::Auto);
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_sync_mode_change() {
        let monitor = ClipboardMonitor::new(2000, SyncMode::Auto).unwrap();
        
        assert_eq!(monitor.get_sync_mode().await, SyncMode::Auto);
        
        monitor.set_sync_mode(SyncMode::Paused).await;
        assert_eq!(monitor.get_sync_mode().await, SyncMode::Paused);
    }

    #[tokio::test]
    async fn test_content_size_validation() {
        let monitor = ClipboardMonitor::new(2000, SyncMode::Auto).unwrap();
        
        // Create content larger than 2MB
        let large_content = "x".repeat(3 * 1024 * 1024);
        
        let result = monitor.set_clipboard(&large_content, "test-clip-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_suppress_mechanism() {
        let last_remote_hash = Arc::new(Mutex::new(Some("hash123".to_string())));
        let last_remote_timestamp = Arc::new(Mutex::new(Some(Instant::now())));

        // Should suppress - same hash within window
        let should_suppress = ClipboardMonitor::should_suppress(
            "hash123",
            &last_remote_hash,
            &last_remote_timestamp,
            2000,
        ).await;
        assert!(should_suppress);

        // Should not suppress - different hash
        let should_suppress = ClipboardMonitor::should_suppress(
            "hash456",
            &last_remote_hash,
            &last_remote_timestamp,
            2000,
        ).await;
        assert!(!should_suppress);
    }
}
