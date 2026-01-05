// Clipboard Hub - Desktop Application
// Main library entry point

mod database;
mod crypto;
mod websocket;
mod clipboard;
mod pairing;
mod sync_manager;
mod mdns;
mod shortcuts;
use shortcuts::ShortcutsManager;

use database::{Database, models::*};
use pairing::PairingManager;
use websocket::WebSocketServer;
use clipboard::ClipboardMonitor;
use sync_manager::SyncManager;
use mdns::MdnsService;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
use anyhow::Result;
use uuid::Uuid;
use tokio::sync::RwLock;
use base64::Engine;

// Application state
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub pairing_manager: Arc<PairingManager>,
    pub server_id: String,
    pub ws_server: Arc<RwLock<WebSocketServer>>,
    pub clipboard_monitor: Arc<ClipboardMonitor>,
    pub sync_manager: Arc<SyncManager>,
    pub mdns_service: Arc<Mutex<MdnsService>>,
}

// Tauri commands

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_server_status(_state: State<'_, AppState>) -> Result<ServerStatus, String> {
    // Get local IP
    let ip_address = get_local_ip().unwrap_or_else(|| "0.0.0.0".to_string());
    
    Ok(ServerStatus {
        status: ServerState::Running,
        ip_address,
        port: 8765,
    })
}

#[tauri::command]
async fn get_clips(limit: usize, state: State<'_, AppState>) -> Result<Vec<Clip>, String> {
    let db = state.db.lock().unwrap();
    db.get_clips(limit)
        .map_err(|e| format!("Failed to get clips: {}", e))
}

#[tauri::command]
async fn get_devices(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    // Get devices from DB
    let devices = {
        let db = state.db.lock().unwrap();
        db.get_devices()
            .map_err(|e| format!("Failed to get devices: {}", e))?
    };
    
    // Get connected device IDs from WebSocket server
    let ws_server = state.ws_server.read().await;
    let connected_device_ids = ws_server.get_connected_device_ids().await;
    
    // Convert devices to JSON with is_connected field
    let devices_with_status: Vec<serde_json::Value> = devices.into_iter().map(|device| {
        let is_connected = connected_device_ids.contains(&device.device_id);
        
        serde_json::json!({
            "device_id": device.device_id,
            "name": device.name,
            "platform": device.platform,
            "last_seen": device.last_seen,
            "is_blocked": device.is_blocked,
            "is_connected": is_connected,
        })
    }).collect();
    
    Ok(devices_with_status)
}

#[tauri::command]
async fn pin_clip(clip_id: String, pinned: bool, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.pin_clip(&clip_id, pinned)
        .map_err(|e| format!("Failed to pin clip: {}", e))
}

#[tauri::command]
async fn delete_clip(clip_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.delete_clip(&clip_id)
        .map_err(|e| format!("Failed to delete clip: {}", e))
}

#[tauri::command]
async fn clear_all_clips(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.clear_all_clips()
        .map_err(|e| format!("Failed to clear clips: {}", e))
}

#[tauri::command]
async fn rename_device(device_id: String, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.update_device_name(&device_id, &new_name)
        .map_err(|e| format!("Failed to rename device: {}", e))
}

#[tauri::command]
async fn revoke_device(device_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.revoke_device(&device_id)
        .map_err(|e| format!("Failed to revoke device: {}", e))
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.read().await;
    Ok(settings.clone())
}

// Pairing commands

#[tauri::command]
async fn generate_pairing_qr(state: State<'_, AppState>) -> Result<String, String> {
    let ip_address = get_local_ip().unwrap_or_else(|| "0.0.0.0".to_string());
    let port = 8765; // TODO: Get from settings
    
    let pairing_data = state.pairing_manager
        .generate_pairing(ip_address, port)
        .map_err(|e| format!("Failed to generate pairing: {}", e))?;
    
    // Return QR code as SVG
    pairing_data.generate_qr_svg()
        .map_err(|e| format!("Failed to generate QR code: {}", e))
}

#[tauri::command]
async fn get_pairing_data(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    // Try to get existing pairing, or generate a new one
    let pairing_data = match state.pairing_manager.get_latest_pairing() {
        Some(data) => data,
        None => {
            // Generate new pairing
            let ip_address = get_local_ip().unwrap_or_else(|| "0.0.0.0".to_string());
            let port = 8765;
            state.pairing_manager
                .generate_pairing(ip_address, port)
                .map_err(|e| format!("Failed to generate pairing: {}", e))?
        }
    };
    
    let pairing_json = serde_json::to_string(&pairing_data)
        .map_err(|e| format!("Failed to serialize pairing data: {}", e))?;

    let qr_svg = pairing_data.generate_qr_svg()
        .map_err(|e| format!("Failed to generate QR code: {}", e))?;
        
    Ok(serde_json::json!({
        "pairing_url": pairing_json,
        "qr_svg": qr_svg,
        "token": pairing_data.pairing_token,
        "ip": pairing_data.server_ip,
        "port": pairing_data.server_port,
        "expiry": pairing_data.expires_at
    }))
}

#[tauri::command]
async fn validate_pairing(token: String, state: State<'_, AppState>) -> Result<bool, String> {
    match state.pairing_manager.validate_pairing(&token) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false)
    }
}

// Helper function to get local IP
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

#[tauri::command]
async fn update_settings(
    sync_mode: String,
    max_history_items: usize,
    retention_days: i64,
    state: State<'_, AppState>
) -> Result<(), String> {
    // Update in-memory settings
    let mut settings = state.settings.write().await;
    settings.sync_mode = match sync_mode.as_str() {
        "Auto" => SyncMode::Auto,
        "HotkeyOnly" => SyncMode::HotkeyOnly,
        "ReceiveOnly" => SyncMode::ReceiveOnly,
        "Paused" => SyncMode::Paused,
        _ => SyncMode::Auto,
    };
    settings.retention_max_items = max_history_items;
    settings.retention_ttl_days = retention_days;
    
    // Also save to database
    let db = state.db.lock().unwrap();
    db.set_setting("sync_mode", &sync_mode)
        .map_err(|e| format!("Failed to save setting: {}", e))?;
    db.set_setting("retention_max_items", &max_history_items.to_string())
        .map_err(|e| format!("Failed to save setting: {}", e))?;
    db.set_setting("retention_ttl_days", &retention_days.to_string())
        .map_err(|e| format!("Failed to save setting: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn toggle_sync(state: State<'_, AppState>) -> Result<bool, String> {
    let mut settings = state.settings.write().await;
    
    // Toggle between Auto and Paused
    let new_mode = match settings.sync_mode {
        SyncMode::Paused => SyncMode::Auto,
        _ => SyncMode::Paused,
    };
    
    settings.sync_mode = new_mode.clone();
    
    // Update clipboard monitor
    state.clipboard_monitor.set_sync_mode(new_mode.clone()).await;
    
    // Save to database
    let db = state.db.lock().unwrap();
    let mode_str = match new_mode {
        SyncMode::Auto => "Auto",
        SyncMode::Paused => "Paused",
        SyncMode::HotkeyOnly => "HotkeyOnly",
        SyncMode::ReceiveOnly => "ReceiveOnly",
    };
    db.set_setting("sync_mode", mode_str).ok();
    
    let is_active = !matches!(new_mode, SyncMode::Paused);
    println!("Sync toggled: {}", if is_active { "Active" } else { "Paused" });
    
    Ok(is_active)
}

#[tauri::command]
async fn get_sync_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let settings = state.settings.read().await;
    let ws_server = state.ws_server.read().await;
    
    let is_active = !matches!(settings.sync_mode, SyncMode::Paused);
    let connected_devices = ws_server.get_client_count().await;
    let ip = get_local_ip().unwrap_or_else(|| "0.0.0.0".to_string());
    
    Ok(serde_json::json!({
        "is_active": is_active,
        "sync_mode": format!("{:?}", settings.sync_mode),
        "connected_devices": connected_devices,
        "ip": ip,
        "port": 8765
    }))
}

#[tauri::command]
async fn set_sync_mode(state: State<'_, AppState>, mode: String) -> Result<serde_json::Value, String> {
    let new_mode = match mode.as_str() {
        "Auto" => SyncMode::Auto,
        "Paused" => SyncMode::Paused,
        "HotkeyOnly" => SyncMode::HotkeyOnly,
        "ReceiveOnly" => SyncMode::ReceiveOnly,
        _ => return Err(format!("Invalid sync mode: {}", mode)),
    };
    
    // Update settings
    {
        let mut settings = state.settings.write().await;
        settings.sync_mode = new_mode.clone();
    }
    
    // Update clipboard monitor
    state.clipboard_monitor.set_sync_mode(new_mode.clone()).await;
    
    // Save to database
    let db = state.db.lock().unwrap();
    db.set_setting("sync_mode", &mode).ok();
    
    println!("Sync mode changed to: {}", mode);
    
    let is_active = !matches!(new_mode, SyncMode::Paused);
    
    Ok(serde_json::json!({
        "success": true,
        "sync_mode": mode,
        "is_active": is_active
    }))
}

#[tauri::command]
async fn block_device(state: State<'_, AppState>, device_id: String, blocked: bool) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.block_device(&device_id, blocked)
        .map_err(|e| format!("Failed to update device: {}", e))?;
    
    println!("Device {} {}", device_id, if blocked { "blocked" } else { "unblocked" });
    Ok(())
}

#[tauri::command]
async fn get_network_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let ip = get_local_ip().unwrap_or_else(|| "0.0.0.0".to_string());
    let ws_server = state.ws_server.read().await;
    let connected_devices = ws_server.get_client_count().await;
    
    Ok(serde_json::json!({
        "ip": ip,
        "port": 8765,
        "server_id": state.server_id,
        "connected_devices": connected_devices,
        "mdns_advertised": true
    }))
}


#[tauri::command]
async fn copy_to_clipboard(content: String) -> Result<(), String> {
    use arboard::Clipboard;
    
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;
    clipboard.set_text(&content)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn manual_sync(state: State<'_, AppState>) -> Result<(), String> {
    // Trigger manual sync (for Hotkey-only mode)
    state.sync_manager.manual_sync().await
        .map_err(|e| format!("Failed to trigger manual sync: {}", e))?;
    
    println!("Manual sync triggered");
    Ok(())
}


// Test commands for development
#[tauri::command]
async fn add_test_clip(content: String, state: State<'_, AppState>) -> Result<(), String> {
    use chrono::Utc;
    use crate::crypto::calculate_hash;
    
    let clip_id = Uuid::new_v4().to_string();
    let content_hash = calculate_hash(&content);
    
    // Simulate a local copy (no source device) to avoid FK constraint issues
    let clip = Clip {
        id: clip_id,
        content,
        content_hash,
        source_device: None, 
        source_app: Some("Test Client".to_string()),
        created_at: Utc::now().to_rfc3339(),
        is_pinned: false,
    };
    
    let db = state.db.lock().unwrap();
    db.add_clip(&clip)
        .map_err(|e| format!("Failed to add clip: {}", e))?;
    
    println!("Test clip added successfully: {}", clip.id);
    
    Ok(())
}

#[tauri::command]
async fn add_test_device(name: String, state: State<'_, AppState>) -> Result<(), String> {
    use chrono::Utc;
    use crate::crypto::generate_shared_secret;
    
    let device_id = Uuid::new_v4().to_string();
    let shared_secret = generate_shared_secret();
    let shared_secret_b64 = base64::engine::general_purpose::STANDARD.encode(&shared_secret);
    
    // Use "Android" to satisfy CHECK constraint (iOS, Android)
    let device = Device {
        device_id,
        name,
        shared_secret: shared_secret_b64,
        platform: "Android".to_string(), 
        last_seen: Some(Utc::now().to_rfc3339()),
        is_blocked: false,
        created_at: Utc::now().to_rfc3339(),
    };
    
    let db = state.db.lock().unwrap();
    db.add_device(&device)
        .map_err(|e| format!("Failed to add device: {}", e))?;
    
    println!("Test device added successfully: {}", device.name);
    
    Ok(())
}



// ... existing code ...

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Generate server ID
    let server_id = Uuid::new_v4().to_string();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(move |app, shortcut, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                let shortcuts_manager = ShortcutsManager::new(app.clone());
                shortcuts_manager.handle_shortcut_event(shortcut.to_string());
            }
        }).build())
        .setup(move |app| {
            // Get app data directory using portable approach
            let app_dir = if cfg!(target_os = "windows") {
                std::env::var("APPDATA")
                    .map(|p| std::path::PathBuf::from(p).join("CopyPaws"))
                    .unwrap_or_else(|_| std::path::PathBuf::from("./data"))
            } else if cfg!(target_os = "macos") {
                std::env::var("HOME")
                    .map(|p| std::path::PathBuf::from(p).join("Library/Application Support/CopyPaws"))
                    .unwrap_or_else(|_| std::path::PathBuf::from("./data"))
            } else {
                std::env::var("HOME")
                    .map(|p| std::path::PathBuf::from(p).join(".local/share/copypaws"))
                    .unwrap_or_else(|_| std::path::PathBuf::from("./data"))
            };
            
            // Create directory if it doesn't exist
            std::fs::create_dir_all(&app_dir).ok();
            
            let db_path = app_dir.join("copypaws.db");

            // Initialize database
            let db = Database::new(db_path).expect("Failed to initialize database");
            let db = Arc::new(Mutex::new(db));

            // Initialize settings
            let settings = Arc::new(RwLock::new(AppSettings::default()));
            
            // Initialize pairing manager
            let pairing_manager = Arc::new(PairingManager::new(server_id.clone()));
            
            // Initialize WebSocket server
            let port = 8765;
            let (ws_server, incoming_rx) = WebSocketServer::new(port, db.clone(), pairing_manager.clone());
            let ws_server = Arc::new(RwLock::new(ws_server));

            // Initialize Clipboard Monitor
            let clipboard_monitor = ClipboardMonitor::new(1000, SyncMode::Auto)
                .expect("Failed to initialize clipboard monitor");
            let clipboard_monitor = Arc::new(clipboard_monitor);
            
            // Initialize Sync Manager with incoming message receiver
            let sync_manager = SyncManager::new(
                db.clone(),
                ws_server.clone(),
                clipboard_monitor.clone(),
                settings.clone(),
                incoming_rx,
            );
            let sync_manager = Arc::new(sync_manager);
            
            // Start Sync Manager in background
            let sync_manager_clone = sync_manager.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = sync_manager_clone.start().await {
                    eprintln!("Sync manager error: {}", e);
                }
            });

            // Initialize mDNS service
            let mut mdns_service = MdnsService::new(server_id.clone(), port);
            if let Err(e) = mdns_service.start_advertising() {
                eprintln!("Failed to start mDNS advertising: {}", e);
            }
            let mdns_service = Arc::new(Mutex::new(mdns_service));

            let app_state = AppState { 
                db, 
                settings, 
                pairing_manager,
                server_id: server_id.clone(),
                ws_server: ws_server.clone(),
                clipboard_monitor,
                sync_manager,
                mdns_service,
            };

            // Manage state
            app.manage(app_state);

            // Register shortcuts
            let shortcuts_manager = ShortcutsManager::new(app.handle().clone());
            tauri::async_runtime::spawn(async move {
                // Give some time for app to initialize state
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if let Err(e) = shortcuts_manager.register_shortcuts().await {
                   eprintln!("Failed to register shortcuts: {}", e);
                }
            });
            
            // Start WebSocket server in background
            let ws_clone = ws_server.clone();
            tauri::async_runtime::spawn(async move {
                let mut server = ws_clone.write().await;
                if let Err(e) = server.start().await {
                    eprintln!("Failed to start WebSocket server: {}", e);
                } else {
                    println!("WebSocket server started on port 8765");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_server_status,
            get_clips,
            get_devices,
            pin_clip,
            delete_clip,
            clear_all_clips,
            rename_device,
            revoke_device,
            block_device,
            get_settings,
            generate_pairing_qr,
            get_pairing_data,
            validate_pairing,
            update_settings,
            toggle_sync,
            get_sync_status,
            set_sync_mode,
            get_network_info,
            copy_to_clipboard,
            manual_sync,
            add_test_clip,
            add_test_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
