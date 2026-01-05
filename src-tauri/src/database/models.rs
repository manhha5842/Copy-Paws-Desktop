// Data models for the application

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_id: String,
    pub name: String,
    pub shared_secret: String,
    pub platform: String,
    pub last_seen: Option<String>,
    pub is_blocked: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub content: String,
    pub content_hash: String,
    pub source_device: Option<String>,
    pub source_app: Option<String>,
    pub created_at: String,
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub server_port: u16,
    pub sync_mode: SyncMode,
    pub suppress_window_ms: u64,
    pub retention_max_items: usize,
    pub retention_ttl_days: i64,
    pub autostart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncMode {
    Auto,
    HotkeyOnly,
    ReceiveOnly,
    Paused,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            server_port: 8765,
            sync_mode: SyncMode::Auto,
            suppress_window_ms: 2000,
            retention_max_items: 1000,
            retention_ttl_days: 30,
            autostart: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub status: ServerState,
    pub ip_address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerState {
    Running,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingData {
    pub server_ip: String,
    pub server_port: u16,
    pub pairing_token: String,
    pub server_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub data: Vec<u8>,
    pub iv: [u8; 12],
}

// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "CLIP_BROADCAST")]
    ClipBroadcast {
        clip_id: String,
        payload_encrypted: String, // base64
        iv: String,               // base64
        source_app: Option<String>,
        timestamp: i64,
    },
    #[serde(rename = "CLIP_PUSH")]
    ClipPush {
        payload_encrypted: String,
        iv: String,
        device_info: DeviceInfo,
    },
    #[serde(rename = "GET_LATEST")]
    GetLatest,
    #[serde(rename = "DEVICE_STATUS")]
    DeviceStatus {
        device_id: String,
        status: String,
    },
    #[serde(rename = "PAIRING_REQUEST")]
    PairingRequest {
        device_id: String,
        device_name: String,
        platform: Option<String>,
        pairing_token: String,
    },
    #[serde(rename = "PAIRING_RESPONSE")]
    PairingResponse {
        success: bool,
        message: String,
        encryption_key: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub battery: Option<String>,
}
