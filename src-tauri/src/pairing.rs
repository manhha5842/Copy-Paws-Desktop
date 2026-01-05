// QR Code pairing module for device connection

use anyhow::{anyhow, Result};
use qrcode::QrCode;
use qrcode::render::svg;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::crypto::generate_shared_secret_base64;

/// Pairing data structure encoded in QR code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingData {
    pub server_ip: String,
    pub server_port: u16,
    pub pairing_token: String,
    pub server_id: String,
    pub shared_secret: String,
    pub expires_at: u64, // Unix timestamp
}

impl PairingData {
    /// Create new pairing data with generated token and secret
    pub fn new(server_ip: String, server_port: u16, server_id: String) -> Self {
        let pairing_token = Uuid::new_v4().to_string();
        let shared_secret = generate_shared_secret_base64();
        
        // Token expires in 5 minutes
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 300;

        Self {
            server_ip,
            server_port,
            pairing_token,
            server_id,
            shared_secret,
            expires_at,
        }
    }

    /// Check if the pairing data has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }

    /// Generate QR code as SVG string
    pub fn generate_qr_svg(&self) -> Result<String> {
        // Create URI format that mobile app expects:
        // copypaws://pair?ip=xxx&port=xxx&token=xxx&secret=xxx&name=xxx&id=xxx
        let qr_uri = format!(
            "copypaws://pair?ip={}&port={}&token={}&secret={}&name={}&id={}",
            urlencoding::encode(&self.server_ip),
            self.server_port,
            urlencoding::encode(&self.pairing_token),
            urlencoding::encode(&self.shared_secret),
            urlencoding::encode("Desktop Hub"),
            urlencoding::encode(&self.server_id)
        );
        
        let code = QrCode::new(qr_uri.as_bytes())?;
        
        let svg_str = code.render::<svg::Color>()
            .min_dimensions(200, 200)
            .max_dimensions(400, 400)
            .build();
        
        Ok(svg_str)
    }
}

/// Pairing manager to handle device pairing workflow
pub struct PairingManager {
    pending_pairings: std::sync::Mutex<std::collections::HashMap<String, PairingData>>,
    server_id: String,
}

impl PairingManager {
    pub fn new(server_id: String) -> Self {
        Self {
            pending_pairings: std::sync::Mutex::new(std::collections::HashMap::new()),
            server_id,
        }
    }

    /// Generate new pairing QR code
    pub fn generate_pairing(&self, server_ip: String, server_port: u16) -> Result<PairingData> {
        let pairing_data = PairingData::new(
            server_ip,
            server_port,
            self.server_id.clone(),
        );

        // Store pending pairing
        let mut pending = self.pending_pairings.lock().unwrap();
        pending.insert(pairing_data.pairing_token.clone(), pairing_data.clone());

        // Clean up expired pairings
        pending.retain(|_, data| !data.is_expired());

        Ok(pairing_data)
    }

    /// Validate a pairing request from mobile
    pub fn validate_pairing(&self, pairing_token: &str) -> Result<PairingData> {
        let mut pending = self.pending_pairings.lock().unwrap();

        if let Some(pairing_data) = pending.remove(pairing_token) {
            if pairing_data.is_expired() {
                return Err(anyhow!("Pairing token has expired"));
            }
            Ok(pairing_data)
        } else {
            Err(anyhow!("Invalid pairing token"))
        }
    }

    /// Cancel a pending pairing
    pub fn cancel_pairing(&self, pairing_token: &str) {
        let mut pending = self.pending_pairings.lock().unwrap();
        pending.remove(pairing_token);
    }

    /// Get count of pending pairings
    pub fn pending_count(&self) -> usize {
        let pending = self.pending_pairings.lock().unwrap();
        pending.len()
    }

    /// Get the most recently created pending pairing
    pub fn get_latest_pairing(&self) -> Option<PairingData> {
        let pending = self.pending_pairings.lock().unwrap();
        // Find the pairing with the furthest expiration time (newest)
        pending.values()
            .max_by_key(|p| p.expires_at)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_data_creation() {
        let data = PairingData::new(
            "192.168.1.100".to_string(),
            8765,
            "server-123".to_string(),
        );

        assert_eq!(data.server_ip, "192.168.1.100");
        assert_eq!(data.server_port, 8765);
        assert!(!data.pairing_token.is_empty());
        assert!(!data.shared_secret.is_empty());
        assert!(!data.is_expired());
    }

    #[test]
    fn test_qr_svg_generation() {
        let data = PairingData::new(
            "192.168.1.100".to_string(),
            8765,
            "server-123".to_string(),
        );

        let svg = data.generate_qr_svg();
        assert!(svg.is_ok());
        assert!(svg.unwrap().contains("<svg"));
    }

    #[test]
    fn test_pairing_manager() {
        let manager = PairingManager::new("server-123".to_string());
        
        let pairing = manager.generate_pairing(
            "192.168.1.100".to_string(),
            8765,
        ).unwrap();

        assert_eq!(manager.pending_count(), 1);

        // Validate should succeed
        let validated = manager.validate_pairing(&pairing.pairing_token);
        assert!(validated.is_ok());
        
        // After validation, pending count should be 0
        assert_eq!(manager.pending_count(), 0);
    }
}
