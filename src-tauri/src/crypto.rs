// Cryptography module for AES-256-GCM encryption

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    /// Create a new Crypto instance with a 32-byte key
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(key.into());
        Self { cipher }
    }

    /// Create Crypto from a base64-encoded key
    pub fn from_base64_key(key_b64: &str) -> Result<Self> {
        let key_bytes = base64::decode(key_b64)?;
        if key_bytes.len() != 32 {
            return Err(anyhow!("Key must be 32 bytes"));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(Self::new(&key))
    }

    /// Encrypt plaintext and return encrypted data with IV
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        // Generate random 96-bit nonce (12 bytes for GCM)
        let mut iv = [0u8; 12];
        OsRng.fill_bytes(&mut iv);

        let nonce = Nonce::from_slice(&iv);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedData {
            data: ciphertext,
            iv,
        })
    }

    /// Decrypt ciphertext using the provided IV
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(&encrypted.iv);

        let plaintext = self
            .cipher
            .decrypt(nonce, encrypted.data.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Encrypt and return base64-encoded result
    pub fn encrypt_to_base64(&self, plaintext: &str) -> Result<(String, String)> {
        let encrypted = self.encrypt(plaintext.as_bytes())?;
        let data_b64 = base64::encode(&encrypted.data);
        let iv_b64 = base64::encode(&encrypted.iv);
        Ok((data_b64, iv_b64))
    }

    /// Decrypt from base64-encoded strings
    pub fn decrypt_from_base64(&self, data_b64: &str, iv_b64: &str) -> Result<String> {
        let data = base64::decode(data_b64)?;
        let iv_bytes = base64::decode(iv_b64)?;

        if iv_bytes.len() != 12 {
            return Err(anyhow!("IV must be 12 bytes"));
        }

        let mut iv = [0u8; 12];
        iv.copy_from_slice(&iv_bytes);

        let encrypted = EncryptedData { data, iv };
        let plaintext = self.decrypt(&encrypted)?;

        Ok(String::from_utf8(plaintext)?)
    }
}

#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub data: Vec<u8>,
    pub iv: [u8; 12],
}

/// Generate a cryptographically secure random 32-byte key
pub fn generate_shared_secret() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Generate a base64-encoded shared secret
pub fn generate_shared_secret_base64() -> String {
    let key = generate_shared_secret();
    base64::encode(key)
}

/// Calculate SHA-256 hash of content
pub fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

// Base64 encoding/decoding utilities
mod base64 {
    use base64::{engine::general_purpose, Engine as _};

    pub fn encode(data: impl AsRef<[u8]>) -> String {
        general_purpose::STANDARD.encode(data)
    }

    pub fn decode(data: impl AsRef<[u8]>) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_shared_secret();
        let crypto = Crypto::new(&key);

        let plaintext = "Hello, World!";
        let encrypted = crypto.encrypt(plaintext.as_bytes()).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext.as_bytes(), decrypted.as_slice());
    }

    #[test]
    fn test_base64_encryption() {
        let key = generate_shared_secret();
        let crypto = Crypto::new(&key);

        let plaintext = "Secret message";
        let (data_b64, iv_b64) = crypto.encrypt_to_base64(plaintext).unwrap();
        let decrypted = crypto.decrypt_from_base64(&data_b64, &iv_b64).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_hash_calculation() {
        let content = "Test content";
        let hash1 = calculate_hash(content);
        let hash2 = calculate_hash(content);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex characters
    }
}
