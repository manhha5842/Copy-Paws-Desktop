// Sync Manager - Orchestrates clipboard sync between desktop and mobile devices

use anyhow::Result;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use chrono::Utc;

use crate::clipboard::{ClipboardMonitor, ClipboardChange, ClipboardContentType};
use crate::crypto::calculate_hash;
use crate::database::{Database, models::*};
use crate::websocket::{WebSocketServer, IncomingMessage};

pub struct SyncManager {
    db: Arc<Mutex<Database>>,
    websocket_server: Arc<RwLock<WebSocketServer>>,
    clipboard_monitor: Arc<ClipboardMonitor>,
    settings: Arc<RwLock<AppSettings>>,
    
    // Event channels
    clip_tx: mpsc::UnboundedSender<ClipEvent>,
    incoming_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<IncomingMessage>>>,
}

#[derive(Debug, Clone)]
pub enum ClipEvent {
    LocalCopy { content: String, content_type: ClipboardContentType, source_app: Option<String> },
    RemotePush { content: String, content_type: ClipboardContentType, device_id: String, clip_id: String },
}

impl SyncManager {
    pub fn new(
        db: Arc<Mutex<Database>>,
        websocket_server: Arc<RwLock<WebSocketServer>>,
        clipboard_monitor: Arc<ClipboardMonitor>,
        settings: Arc<RwLock<AppSettings>>,
        incoming_rx: mpsc::UnboundedReceiver<IncomingMessage>,
    ) -> Self {
        let (clip_tx, _) = mpsc::unbounded_channel();
        
        Self {
            db,
            websocket_server,
            clipboard_monitor,
            settings,
            clip_tx,
            incoming_rx: Arc::new(tokio::sync::Mutex::new(incoming_rx)),
        }
    }

    /// Start the sync manager
    pub async fn start(&self) -> Result<()> {
        let (clip_tx, mut clip_rx) = mpsc::unbounded_channel::<ClipEvent>();
        
        // Clone references for the event handler
        let db = self.db.clone();
        let ws_server = self.websocket_server.clone();
        let settings = self.settings.clone();

        // Spawn event handler task
        tokio::spawn(async move {
            while let Some(event) = clip_rx.recv().await {
                match event {
                    ClipEvent::LocalCopy { content, content_type, source_app } => {
                        if let Err(e) = Self::handle_local_copy(
                            &db, &ws_server, &settings, content, content_type, source_app
                        ).await {
                            eprintln!("Error handling local copy: {}", e);
                        }
                    }
                    ClipEvent::RemotePush { content, content_type, device_id, clip_id } => {
                        if let Err(e) = Self::handle_remote_push(
                            &db, content, content_type, device_id, clip_id
                        ).await {
                            eprintln!("Error handling remote push: {}", e);
                        }
                    }
                }
            }
        });

        // Spawn incoming WebSocket message handler
        let incoming_rx = self.incoming_rx.clone();
        let clip_tx_incoming = clip_tx.clone();
        let clipboard_monitor = self.clipboard_monitor.clone();
        let db_incoming = self.db.clone();
        let ws_server_incoming = self.websocket_server.clone();
        
        tokio::spawn(async move {
            let mut rx = incoming_rx.lock().await;
            while let Some(msg) = rx.recv().await {
                println!("Received incoming message from device: {}", msg.device_id);
                match msg.message {
                    WsMessage::ClipPush { payload_encrypted, iv: _, content_type, device_info: _ } => {
                        // TODO: Decrypt payload using device's shared secret
                        let content = payload_encrypted; // Placeholder - should decrypt
                        let clip_content_type = match content_type.as_deref() {
                            Some("image") => ClipboardContentType::Image,
                            _ => ClipboardContentType::Text,
                        };
                        
                        let clip_id = Uuid::new_v4().to_string();
                        
                        // Set local clipboard based on content type
                        match clip_content_type {
                            ClipboardContentType::Image => {
                                if let Err(e) = clipboard_monitor.set_clipboard_image(&content, &clip_id).await {
                                    eprintln!("Failed to set clipboard image: {}", e);
                                    continue;
                                }
                            }
                            ClipboardContentType::Text => {
                                if let Err(e) = clipboard_monitor.set_clipboard(&content, &clip_id).await {
                                    eprintln!("Failed to set clipboard: {}", e);
                                    continue;
                                }
                            }
                        }
                        
                        // Store in database
                        let _ = clip_tx_incoming.send(ClipEvent::RemotePush {
                            content,
                            content_type: clip_content_type,
                            device_id: msg.device_id,
                            clip_id,
                        });
                    }
                    WsMessage::GetLatest => {
                        // Send latest clip to requesting device
                        // Get clip data first, then drop lock before await
                        let clip_opt = {
                            let db_lock = db_incoming.lock().unwrap();
                            db_lock.get_latest_clip().ok().flatten()
                        };
                        
                        if let Some(clip) = clip_opt {
                            let ws = ws_server_incoming.read().await;
                            let message = WsMessage::ClipBroadcast {
                                clip_id: clip.id,
                                payload_encrypted: clip.content, // TODO: Encrypt
                                iv: String::new(),
                                content_type: clip.content_type.clone(),
                                source_app: clip.source_app,
                                timestamp: Utc::now().timestamp(),
                            };
                            if let Err(e) = ws.send_to_device(&msg.device_id, message).await {
                                eprintln!("Failed to send latest clip: {}", e);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        // Start clipboard monitoring with callback
        let clip_tx_clone = clip_tx.clone();
        self.clipboard_monitor.start_monitoring(move |change: ClipboardChange| {
            let _ = clip_tx_clone.send(ClipEvent::LocalCopy {
                content: change.content,
                content_type: change.content_type,
                source_app: None, // TODO: Detect source app
            });
            Ok(())
        }).await?;

        Ok(())
    }

    /// Handle local clipboard copy event
    async fn handle_local_copy(
        db: &Arc<Mutex<Database>>,
        ws_server: &Arc<RwLock<WebSocketServer>>,
        settings: &Arc<RwLock<AppSettings>>,
        content: String,
        content_type: ClipboardContentType,
        source_app: Option<String>,
    ) -> Result<()> {
        // Check sync mode
        let current_settings = settings.read().await;
        match current_settings.sync_mode {
            SyncMode::Paused | SyncMode::ReceiveOnly => {
                return Ok(()); // Don't sync
            }
            SyncMode::HotkeyOnly => {
                // TODO: Check if triggered by hotkey
                return Ok(());
            }
            SyncMode::Auto => {
                // Continue with sync
            }
        }
        drop(current_settings);

        // Validate content size based on type
        let max_size = match content_type {
            ClipboardContentType::Text => 2 * 1024 * 1024,
            ClipboardContentType::Image => 10 * 1024 * 1024,
        };
        if content.len() > max_size {
            eprintln!("Content exceeds size limit, skipping sync");
            return Ok(());
        }

        // Calculate hash
        let content_hash = calculate_hash(&content);

        // Check for duplicate - skip if same hash exists in recent clips
        {
            let db_lock = db.lock().unwrap();
            if let Ok(clips) = db_lock.get_clips(5) {
                if clips.iter().any(|c| c.content_hash == content_hash) {
                    println!("Duplicate clip detected (hash match), skipping");
                    return Ok(());
                }
            }
        }

        // Determine content_type string
        let content_type_str = match content_type {
            ClipboardContentType::Text => "text",
            ClipboardContentType::Image => "image",
        };

        // Create clip record
        let clip_id = Uuid::new_v4().to_string();
        let clip = Clip {
            id: clip_id.clone(),
            content: content.clone(),
            content_hash: content_hash.clone(),
            content_type: Some(content_type_str.to_string()),
            source_device: None, // None for local clips
            source_app,
            created_at: Utc::now().to_rfc3339(),
            is_pinned: false,
        };

        // Save to database
        {
            let db_lock = db.lock().unwrap();
            db_lock.add_clip(&clip)?;
        }

        // Broadcast to connected devices
        let ws = ws_server.read().await;
        let message = WsMessage::ClipBroadcast {
            clip_id: clip_id.clone(),
            payload_encrypted: content.clone(), // TODO: Encrypt per-device
            iv: String::new(), // TODO: Generate IV
            content_type: Some(content_type_str.to_string()),
            source_app: clip.source_app.clone(),
            timestamp: Utc::now().timestamp(),
        };
        
        ws.broadcast(message, None).await?;

        println!("Synced local clip: {} bytes", content.len());
        Ok(())
    }

    /// Handle remote clipboard push from mobile device
    async fn handle_remote_push(
        db: &Arc<Mutex<Database>>,
        content: String,
        content_type: ClipboardContentType,
        device_id: String,
        clip_id: String,
    ) -> Result<()> {
        // Calculate hash
        let content_hash = calculate_hash(&content);
        
        // Determine content_type string
        let content_type_str = match content_type {
            ClipboardContentType::Text => "text",
            ClipboardContentType::Image => "image",
        };

        // Create clip record
        let clip = Clip {
            id: clip_id.clone(),
            content: content.clone(),
            content_hash,
            content_type: Some(content_type_str.to_string()),
            source_device: Some(device_id.clone()),
            source_app: None,
            created_at: Utc::now().to_rfc3339(),
            is_pinned: false,
        };

        // Save to database
        {
            let db_lock = db.lock().unwrap();
            db_lock.add_clip(&clip)?;
        }

        // Note: Clipboard is set by the caller after decryption
        println!("Received remote clip from {}: {} bytes", device_id, content.len());
        Ok(())
    }

    /// Process incoming WebSocket message
    pub async fn process_message(&self, device_id: &str, message: WsMessage) -> Result<()> {
        match message {
            WsMessage::ClipPush { payload_encrypted, iv: _, content_type, device_info: _ } => {
                // TODO: Decrypt payload using device's shared secret
                let content = payload_encrypted; // Placeholder - should decrypt
                let clip_content_type = match content_type.as_deref() {
                    Some("image") => ClipboardContentType::Image,
                    _ => ClipboardContentType::Text,
                };
                
                let clip_id = Uuid::new_v4().to_string();
                
                // Set local clipboard based on content type
                match clip_content_type {
                    ClipboardContentType::Image => {
                        self.clipboard_monitor.set_clipboard_image(&content, &clip_id).await?;
                    }
                    ClipboardContentType::Text => {
                        self.clipboard_monitor.set_clipboard(&content, &clip_id).await?;
                    }
                }
                
                // Store in database
                let event = ClipEvent::RemotePush {
                    content,
                    content_type: clip_content_type,
                    device_id: device_id.to_string(),
                    clip_id,
                };
                let _ = self.clip_tx.send(event);
            }
            WsMessage::GetLatest => {
                // Send latest clip to requesting device
                let db_lock = self.db.lock().unwrap();
                if let Ok(Some(clip)) = db_lock.get_latest_clip() {
                    let ws = self.websocket_server.read().await;
                    let message = WsMessage::ClipBroadcast {
                        clip_id: clip.id,
                        payload_encrypted: clip.content, // TODO: Encrypt
                        iv: String::new(),
                        content_type: clip.content_type.clone(),
                        source_app: clip.source_app,
                        timestamp: Utc::now().timestamp(),
                    };
                    ws.send_to_device(device_id, message).await?;
                }
            }
            _ => {
                // Handle other message types
            }
        }
        Ok(())
    }

    /// Manually trigger sync (for hotkey mode)
    pub async fn manual_sync(&self) -> Result<()> {
        let content = self.clipboard_monitor.get_clipboard().await?;
        let event = ClipEvent::LocalCopy {
            content,
            content_type: ClipboardContentType::Text, // Manual sync assumes text
            source_app: None,
        };
        let _ = self.clip_tx.send(event);
        Ok(())
    }

    /// Get current sync status
    pub async fn get_sync_status(&self) -> SyncStatus {
        let settings = self.settings.read().await;
        let ws = self.websocket_server.read().await;
        
        SyncStatus {
            mode: settings.sync_mode.clone(),
            connected_devices: ws.get_client_count().await,
            server_status: ws.get_status().await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub mode: SyncMode,
    pub connected_devices: usize,
    pub server_status: ServerStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_event_creation() {
        let event = ClipEvent::LocalCopy {
            content: "Test content".to_string(),
            content_type: ClipboardContentType::Text,
            source_app: Some("Test App".to_string()),
        };
        
        match event {
            ClipEvent::LocalCopy { content, content_type, source_app } => {
                assert_eq!(content, "Test content");
                assert_eq!(content_type, ClipboardContentType::Text);
                assert_eq!(source_app, Some("Test App".to_string()));
            }
            _ => panic!("Wrong event type"),
        }
    }
}
