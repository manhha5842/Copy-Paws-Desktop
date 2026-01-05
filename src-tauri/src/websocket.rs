// WebSocket server for handling mobile device connections

use anyhow::{anyhow, Result};
use futures_util::{StreamExt, SinkExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use base64::Engine;

use crate::crypto::Crypto;
use crate::database::models::{ServerStatus, ServerState, WsMessage, Device, DeviceInfo};
use crate::database::Database;
use crate::pairing::PairingManager;

type ClientSender = mpsc::UnboundedSender<Message>;
type ClientMap = Arc<RwLock<HashMap<String, ClientInfo>>>;

#[derive(Clone)]
pub struct ClientInfo {
    pub device_id: String,
    pub device_name: String,
    pub sender: ClientSender,
    pub crypto: Arc<Crypto>,
}

pub struct WebSocketServer {
    port: u16,
    state: Arc<Mutex<ServerState>>,
    clients: ClientMap,
    broadcast_tx: mpsc::UnboundedSender<BroadcastMessage>,
    db: Arc<Mutex<Database>>,
    pairing_manager: Arc<PairingManager>,
    incoming_tx: mpsc::UnboundedSender<IncomingMessage>,
}

#[derive(Debug, Clone)]
pub struct BroadcastMessage {
    pub exclude_device_id: Option<String>,
    pub message: WsMessage,
}

#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub device_id: String,
    pub message: WsMessage,
}

impl WebSocketServer {
    pub fn new(
        port: u16, 
        db: Arc<Mutex<Database>>,
        pairing_manager: Arc<PairingManager>
    ) -> (Self, mpsc::UnboundedReceiver<IncomingMessage>) {
        let (broadcast_tx, _) = mpsc::unbounded_channel();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        
        let server = Self {
            port,
            state: Arc::new(Mutex::new(ServerState::Stopped)),
            clients: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            db,
            pairing_manager,
            incoming_tx,
        };
        
        (server, incoming_rx)
    }

    /// Start the WebSocket server
    pub async fn start(&mut self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        {
            let mut state = self.state.lock().unwrap();
            *state = ServerState::Running;
        }

        println!("WebSocket server listening on {}", addr);

        let clients = self.clients.clone();
        let state = self.state.clone();
        let (broadcast_tx, mut broadcast_rx) = mpsc::unbounded_channel::<BroadcastMessage>();
        self.broadcast_tx = broadcast_tx.clone();

        // Spawn broadcast handler
        let broadcast_clients = clients.clone();
        tokio::spawn(async move {
            while let Some(broadcast_msg) = broadcast_rx.recv().await {
                let clients_read = broadcast_clients.read().await;
                
                for (device_id, client_info) in clients_read.iter() {
                    // Skip the sender if exclude_device_id is set
                    if let Some(ref exclude_id) = broadcast_msg.exclude_device_id {
                        if device_id == exclude_id {
                            continue;
                        }
                    }

                    // Serialize and encrypt message
                    if let Ok(json) = serde_json::to_string(&broadcast_msg.message) {
                        if let Ok((encrypted, iv)) = client_info.crypto.encrypt_to_base64(&json) {
                            let encrypted_msg = serde_json::json!({
                                "type": "ENCRYPTED",
                                "payload": encrypted,
                                "iv": iv
                            });
                            
                            if let Ok(msg_str) = serde_json::to_string(&encrypted_msg) {
                                let _ = client_info.sender.send(Message::Text(msg_str));
                            }
                        }
                    }
                }
            }
        });

        // Accept connections loop
        let accept_clients = clients.clone();
        let accept_state = state.clone();
        let accept_db = self.db.clone();
        let accept_pm = self.pairing_manager.clone();
        let accept_incoming_tx = self.incoming_tx.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let current_state = accept_state.lock().unwrap().clone();
                        
                        if current_state != ServerState::Running {
                            continue;
                        }

                        let clients_clone = accept_clients.clone();
                        let db_clone = accept_db.clone();
                        let pm_clone = accept_pm.clone();
                        let incoming_tx_clone = accept_incoming_tx.clone();
                        
                        tokio::spawn(async move {
                            handle_connection(stream, addr, clients_clone, db_clone, pm_clone, incoming_tx_clone).await;
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the WebSocket server
    pub async fn stop(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        *state = ServerState::Stopped;
        
        // Close all client connections
        let mut clients = self.clients.write().await;
        clients.clear();
        
        Ok(())
    }

    /// Pause the WebSocket server
    pub async fn pause(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        *state = ServerState::Paused;
        Ok(())
    }

    /// Resume the WebSocket server
    pub async fn resume(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        *state = ServerState::Running;
        Ok(())
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, message: WsMessage, exclude_device_id: Option<String>) -> Result<()> {
        let broadcast_msg = BroadcastMessage {
            exclude_device_id,
            message,
        };
        
        self.broadcast_tx.send(broadcast_msg)
            .map_err(|e| anyhow!("Failed to send broadcast: {}", e))?;
        
        Ok(())
    }

    /// Send a message to a specific device
    pub async fn send_to_device(&self, device_id: &str, message: WsMessage) -> Result<()> {
        let clients = self.clients.read().await;
        
        if let Some(client_info) = clients.get(device_id) {
            let json = serde_json::to_string(&message)?;
            let (encrypted, iv) = client_info.crypto.encrypt_to_base64(&json)?;
            
            let encrypted_msg = serde_json::json!({
                "type": "ENCRYPTED",
                "payload": encrypted,
                "iv": iv
            });
            
            let msg_str = serde_json::to_string(&encrypted_msg)?;
            client_info.sender.send(Message::Text(msg_str))
                .map_err(|e| anyhow!("Failed to send message: {}", e))?;
        } else {
            return Err(anyhow!("Device not connected: {}", device_id));
        }
        
        Ok(())
    }

    /// Get current server status
    pub async fn get_status(&self) -> ServerStatus {
        let state = self.state.lock().unwrap().clone();
        let ip_address = get_local_ip().unwrap_or_else(|| "0.0.0.0".to_string());
        
        ServerStatus {
            status: state,
            ip_address,
            port: self.port,
        }
    }

    /// Register a new client
    pub async fn register_client(&self, device_id: String, device_name: String, sender: ClientSender, crypto: Arc<Crypto>) {
        let client_info = ClientInfo {
            device_id: device_id.clone(),
            device_name,
            sender,
            crypto,
        };
        
        let mut clients = self.clients.write().await;
        clients.insert(device_id, client_info);
    }

    /// Unregister a client
    pub async fn unregister_client(&self, device_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(device_id);
    }

    /// Get connected client count
    pub async fn get_client_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }
    
    /// Get list of connected device IDs
    pub async fn get_connected_device_ids(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }
}

/// Handle individual WebSocket connection
async fn handle_connection(
    stream: TcpStream, 
    addr: SocketAddr, 
    clients: ClientMap,
    db: Arc<Mutex<Database>>,
    pairing_manager: Arc<PairingManager>,
    incoming_tx: mpsc::UnboundedSender<IncomingMessage>,
) {
    println!("New connection from: {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Spawn task to send messages to client
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let mut device_id: Option<String> = None;

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received message: {}", text);
                
                // Parse message and handle authentication/registration
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    match ws_msg {
                        WsMessage::Handshake { device_id: dev_id_handshake } => {
                            // 1. Check DB for device
                            let device_opt = {
                                let db_lock = db.lock().unwrap();
                                db_lock.get_device(&dev_id_handshake).unwrap_or(None)
                            };
                            
                            if let Some(device) = device_opt {
                                // 2. Load Crypto from shared secret
                                use base64::Engine;
                                if let Ok(secret_bytes) = base64::engine::general_purpose::STANDARD.decode(&device.shared_secret) {
                                    let mut key_array: [u8; 32] = [0u8; 32];
                                    if secret_bytes.len() == 32 {
                                        key_array.copy_from_slice(&secret_bytes);
                                        let crypto = Arc::new(Crypto::new(&key_array));
                                        
                                        // 3. Register Client
                                        device_id = Some(dev_id_handshake.clone());
                                        
                                        // Update last_seen in DB - TODO: Add update_last_seen method to DB
                                        
                                        let mut clients_write = clients.write().await;
                                        clients_write.insert(dev_id_handshake.clone(), ClientInfo {
                                            device_id: dev_id_handshake.clone(),
                                            device_name: device.name.clone(),
                                            sender: tx.clone(),
                                            crypto,
                                        });
                                        
                                        println!("Device reconnected and authenticated: {}", device.name);
                                        
                                        // Send handshake response (optional, but good for confirmation)
                                        let response = serde_json::json!({
                                            "type": "HANDSHAKE_RESPONSE",
                                            "success": true
                                        });
                                        if let Ok(resp_str) = serde_json::to_string(&response) {
                                            let _ = tx.send(Message::Text(resp_str));
                                        }
                                    } else {
                                         eprintln!("Invalid key length for device {}", dev_id_handshake);
                                    }
                                }
                            } else {
                                println!("Handshake failed: Device not found {}", dev_id_handshake);
                                // Send failure response
                                let response = serde_json::json!({
                                    "type": "HANDSHAKE_RESPONSE",
                                    "success": false,
                                    "error": "Device not found"
                                });
                                if let Ok(resp_str) = serde_json::to_string(&response) {
                                    let _ = tx.send(Message::Text(resp_str));
                                }
                            }
                        }
                        WsMessage::PairingRequest { device_id: dev_id, device_name, platform, pairing_token } => {
                            // Validate pairing token
                            match pairing_manager.validate_pairing(&pairing_token) {
                                Ok(pairing_data) => {
                                    // Pairing valid!
                                    let shared_secret = pairing_data.shared_secret.clone();
                                    
                                    // Decode shared secret for Crypto
                                    if let Ok(secret_bytes) = base64::engine::general_purpose::STANDARD.decode(&shared_secret) {
                                        // Convert Vec<u8> to [u8; 32]
                                        if secret_bytes.len() != 32 {
                                            eprintln!("Invalid shared secret length: expected 32 bytes, got {}", secret_bytes.len());
                                            continue;
                                        }
                                        let mut key_array: [u8; 32] = [0u8; 32];
                                        key_array.copy_from_slice(&secret_bytes);
                                        
                                        let crypto = Arc::new(Crypto::new(&key_array));
                                        
                                        // Save device to DB
                                        let device = Device {
                                            device_id: dev_id.clone(),
                                            name: device_name.clone(),
                                            shared_secret: shared_secret.clone(),
                                            platform: platform.unwrap_or("Android".to_string()),
                                            last_seen: Some(chrono::Utc::now().to_rfc3339()),
                                            is_blocked: false,
                                            created_at: chrono::Utc::now().to_rfc3339(),
                                        };
                                        
                                        let mut success = false;
                                        let mut error_msg = String::new();
                                        
                                        // DB operation in its own scope so lock is dropped before await
                                        {
                                            let db_lock = db.lock().unwrap();
                                            match db_lock.add_device(&device) {
                                                Ok(_) => {
                                                    success = true;
                                                    println!("Device paired and saved: {}", device_name);
                                                },
                                                Err(e) => {
                                                    error_msg = format!("Database error: {}", e);
                                                    eprintln!("{}", error_msg);
                                                }
                                            }
                                        } // db_lock dropped here
                                        
                                        if success {
                                            // Send success response
                                            let response = WsMessage::PairingResponse {
                                                success: true,
                                                message: "Pairing successful".to_string(),
                                                encryption_key: Some(shared_secret.clone())
                                            };
                                            
                                            if let Ok(resp_str) = serde_json::to_string(&response) {
                                                let _ = tx.send(Message::Text(resp_str));
                                            }
                                            
                                            // Register client (now safe to await)
                                            device_id = Some(dev_id.clone());
                                            let mut clients_write = clients.write().await;
                                            
                                            clients_write.insert(dev_id.clone(), ClientInfo {
                                                device_id: dev_id,
                                                device_name: device_name,
                                                sender: tx.clone(),
                                                crypto,
                                            });
                                        } else {
                                            let response = WsMessage::PairingResponse {
                                                success: false,
                                                message: error_msg,
                                                encryption_key: None
                                            };
                                            if let Ok(resp_str) = serde_json::to_string(&response) {
                                                let _ = tx.send(Message::Text(resp_str));
                                            }
                                        }

                                    } else {
                                        eprintln!("Invalid shared secret format");
                                    }
                                },
                                Err(e) => {
                                    eprintln!("Pairing validation failed: {}", e);
                                    let response = WsMessage::PairingResponse {
                                        success: false,
                                        message: format!("Pairing failed: {}", e),
                                        encryption_key: None
                                    };
                                    if let Ok(resp_str) = serde_json::to_string(&response) {
                                        let _ = tx.send(Message::Text(resp_str));
                                    }
                                }
                            }
                        }
                        WsMessage::ClipPush { payload_encrypted, iv, device_info: _ } => {
                            // Decrypt payload using device's crypto
                            if let Some(ref dev_id) = device_id {
                                let clients_read = clients.read().await;
                                if let Some(client_info) = clients_read.get(dev_id) {
                                    // Decrypt the payload
                                    match client_info.crypto.decrypt_from_base64(&payload_encrypted, &iv) {
                                        Ok(decrypted_content) => {
                                            println!("Decrypted clip from {}: {} bytes", dev_id, decrypted_content.len());
                                            
                                            // Create new message with decrypted content
                                            let decrypted_msg = WsMessage::ClipPush {
                                                payload_encrypted: decrypted_content, // Now contains plaintext
                                                iv: String::new(),
                                                device_info: DeviceInfo {
                                                    name: client_info.device_name.clone(),
                                                    battery: None,
                                                },
                                            };
                                            
                                            let _ = incoming_tx.send(IncomingMessage {
                                                device_id: dev_id.clone(),
                                                message: decrypted_msg,
                                            });
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to decrypt clip from {}: {}", dev_id, e);
                                        }
                                    }
                                } else {
                                    eprintln!("Device {} not found in clients", dev_id);
                                }
                            }
                        }
                        WsMessage::GetLatest => {
                            // Forward to SyncManager
                            if let Some(ref dev_id) = device_id {
                                let _ = incoming_tx.send(IncomingMessage {
                                    device_id: dev_id.clone(),
                                    message: ws_msg,
                                });
                            }
                        }
                        _ => {
                            // Other messages (ClipBroadcast, DeviceStatus, PairingResponse are outbound)
                            println!("Received unhandled message type");
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client disconnected: {} (Device: {:?})", addr, device_id);
                break;
            }
            Err(e) => {
                eprintln!("WebSocket error for {} (Device: {:?}): {}", addr, device_id, e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup on disconnect
    if let Some(dev_id) = device_id {
        let mut clients_write = clients.write().await;
        clients_write.remove(&dev_id);
    }
}

/// Get local IP address
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    
    // Connect to a remote address to determine local IP
    // This doesn't actually send data
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

#[cfg(test)]
mod tests {
    // Tests require DB and PairingManager setup.
    // Skipping for now as they need integration testing.
}
