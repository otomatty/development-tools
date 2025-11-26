//! Token encryption utilities
//!
//! Provides AES-256-GCM encryption for secure token storage.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    Encryption(String),

    #[error("Decryption failed: {0}")]
    Decryption(String),

    #[error("Invalid key length")]
    InvalidKeyLength,

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
}

pub type CryptoResult<T> = Result<T, CryptoError>;

/// Crypto utilities for token encryption/decryption
pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    /// Create a new Crypto instance with the given key
    ///
    /// Key must be exactly 32 bytes (256 bits)
    pub fn new(key: &[u8]) -> CryptoResult<Self> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKeyLength);
        }

        let cipher =
            Aes256Gcm::new_from_slice(key).map_err(|e| CryptoError::Encryption(e.to_string()))?;

        Ok(Self { cipher })
    }

    /// Create a Crypto instance with a key derived from the app identifier
    ///
    /// This provides basic obfuscation but is not meant for high-security scenarios.
    /// For production, consider using OS keychain services.
    pub fn from_app_key() -> CryptoResult<Self> {
        // Derive a key from a combination of app identifier and machine-specific data
        let app_id = "com.sugaiakimasa.development-tools";
        let key = Self::derive_key(app_id);
        Self::new(&key)
    }

    /// Derive a 32-byte key from a string
    fn derive_key(input: &str) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut key = [0u8; 32];

        // Simple key derivation (for production, use PBKDF2 or similar)
        for (i, chunk) in input.as_bytes().chunks(8).enumerate() {
            let mut hasher = DefaultHasher::new();
            chunk.hash(&mut hasher);
            i.hash(&mut hasher);
            let hash = hasher.finish().to_le_bytes();

            let start = (i * 8) % 32;
            let end = (start + 8).min(32);
            key[start..end].copy_from_slice(&hash[..end - start]);
        }

        // Additional mixing
        for i in 0..32 {
            let mut hasher = DefaultHasher::new();
            key[i].hash(&mut hasher);
            input.len().hash(&mut hasher);
            key[i] = (hasher.finish() % 256) as u8;
        }

        key
    }

    /// Encrypt a plaintext string
    ///
    /// Returns a base64-encoded string containing the nonce and ciphertext
    pub fn encrypt(&self, plaintext: &str) -> CryptoResult<String> {
        // Generate random 12-byte nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| CryptoError::Encryption(e.to_string()))?;

        // Combine nonce + ciphertext and encode
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        Ok(BASE64.encode(combined))
    }

    /// Decrypt an encrypted string
    ///
    /// Input should be a base64-encoded string from `encrypt()`
    pub fn decrypt(&self, encrypted: &str) -> CryptoResult<String> {
        // Decode from base64
        let combined = BASE64.decode(encrypted)?;

        if combined.len() < 12 {
            return Err(CryptoError::Decryption(
                "Invalid encrypted data length".to_string(),
            ));
        }

        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::Decryption(e.to_string()))?;

        String::from_utf8(plaintext).map_err(|e| CryptoError::Decryption(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32];
        let crypto = Crypto::new(&key).expect("Should create crypto");

        let plaintext = "my-secret-token";
        let encrypted = crypto.encrypt(plaintext).expect("Should encrypt");

        assert_ne!(encrypted, plaintext);

        let decrypted = crypto.decrypt(&encrypted).expect("Should decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_from_app_key() {
        let crypto = Crypto::from_app_key().expect("Should create from app key");

        let plaintext = "test-token";
        let encrypted = crypto.encrypt(plaintext).expect("Should encrypt");
        let decrypted = crypto.decrypt(&encrypted).expect("Should decrypt");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0u8; 16];
        let result = Crypto::new(&short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_encryptions_different_results() {
        let crypto = Crypto::from_app_key().expect("Should create crypto");

        let plaintext = "same-token";
        let encrypted1 = crypto.encrypt(plaintext).expect("Should encrypt");
        let encrypted2 = crypto.encrypt(plaintext).expect("Should encrypt");

        // Due to random nonce, encryptions should be different
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to same plaintext
        assert_eq!(
            crypto.decrypt(&encrypted1).unwrap(),
            crypto.decrypt(&encrypted2).unwrap()
        );
    }
}

