//! AES-256-GCM authenticated encryption

use chacha20poly1305::ChaCha20Poly1305;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::crypto::Argon2Params;
use crate::utils::error::{Error, Result};

/// Supported vault encryption methods.
///
/// Only AES-256-GCM is implemented today, but this enum is versioned and
/// serialized so additional methods can be added without redesigning headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionMethod {
    #[default]
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl EncryptionMethod {
    /// All available encryption methods.
    pub fn all() -> &'static [EncryptionMethod] {
        &[EncryptionMethod::Aes256Gcm, EncryptionMethod::ChaCha20Poly1305]
    }

    /// Display name for UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            EncryptionMethod::Aes256Gcm => "AES-256-GCM",
            EncryptionMethod::ChaCha20Poly1305 => "ChaCha20-Poly1305",
        }
    }

    /// Security level label for UI.
    pub fn security_level(&self) -> &'static str {
        match self {
            EncryptionMethod::Aes256Gcm => "High",
            EncryptionMethod::ChaCha20Poly1305 => "High",
        }
    }

    /// Decryption speed label for UI.
    pub fn decryption_speed(&self) -> &'static str {
        match self {
            EncryptionMethod::Aes256Gcm => "Slow Decryption",
            EncryptionMethod::ChaCha20Poly1305 => "Fast Decryption",
        }
    }

    /// Full descriptive label for UI.
    pub fn profile_label(&self) -> String {
        format!(
            "{} ({}, {})",
            self.display_name(),
            self.security_level(),
            self.decryption_speed()
        )
    }
}

/// Encrypted payload containing ciphertext and all data needed for decryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    /// The encrypted data (ciphertext + auth tag)
    pub ciphertext: Vec<u8>,
    /// 12-byte nonce (unique per encryption)
    pub nonce: [u8; 12],
    /// 32-byte salt used for key derivation
    pub salt: [u8; 32],
    /// Argon2 parameters used for key derivation
    pub argon2_params: Argon2Params,
}

impl EncryptedPayload {
    /// Create a new payload from components
    pub fn new(
        ciphertext: Vec<u8>,
        nonce: [u8; 12],
        salt: [u8; 32],
        argon2_params: Argon2Params,
    ) -> Self {
        Self {
            ciphertext,
            nonce,
            salt,
            argon2_params,
        }
    }
}

/// Generate a random 12-byte nonce for AES-GCM
fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Encrypt plaintext using AES-256-GCM
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `key` - 32-byte encryption key (derived from password)
/// * `salt` - 32-byte salt used to derive the key
/// * `argon2_params` - Parameters used to derive the key
///
/// # Returns
/// `EncryptedPayload` containing the ciphertext and metadata
pub fn encrypt(
    plaintext: &[u8],
    key: &[u8; 32],
    salt: [u8; 32],
    argon2_params: Argon2Params,
) -> Result<EncryptedPayload> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| Error::Encryption(e.to_string()))?;

    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| Error::Encryption(e.to_string()))?;

    Ok(EncryptedPayload::new(
        ciphertext,
        nonce_bytes,
        salt,
        argon2_params,
    ))
}

/// Decrypt ciphertext using AES-256-GCM
///
/// # Arguments
/// * `payload` - The encrypted payload
/// * `key` - 32-byte encryption key (must match the key used for encryption)
///
/// # Returns
/// The decrypted plaintext bytes
pub fn decrypt(payload: &EncryptedPayload, key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| Error::Decryption)?;

    let nonce = Nonce::from_slice(&payload.nonce);

    cipher
        .decrypt(nonce, payload.ciphertext.as_slice())
        .map_err(|_| Error::Decryption)
}

/// Encrypt plaintext using the selected encryption method.

/// Encrypt plaintext using ChaCha20-Poly1305
pub fn encrypt_chacha(
    plaintext: &[u8],
    key: &[u8; 32],
    salt: [u8; 32],
    argon2_params: Argon2Params,
) -> Result<EncryptedPayload> {
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|e| Error::Encryption(e.to_string()))?;

    let nonce_bytes = generate_nonce();
    let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| Error::Encryption(e.to_string()))?;

    Ok(EncryptedPayload::new(
        ciphertext,
        nonce_bytes,
        salt,
        argon2_params,
    ))
}

/// Decrypt ciphertext using ChaCha20-Poly1305
pub fn decrypt_chacha(payload: &EncryptedPayload, key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|_| Error::Decryption)?;

    let nonce = chacha20poly1305::Nonce::from_slice(&payload.nonce);

    cipher
        .decrypt(nonce, payload.ciphertext.as_slice())
        .map_err(|_| Error::Decryption)
}

pub fn encrypt_with_method(
    method: EncryptionMethod,
    plaintext: &[u8],
    key: &[u8; 32],
    salt: [u8; 32],
    argon2_params: Argon2Params,
) -> Result<EncryptedPayload> {
    match method {
        EncryptionMethod::Aes256Gcm => encrypt(plaintext, key, salt, argon2_params),
        EncryptionMethod::ChaCha20Poly1305 => encrypt_chacha(plaintext, key, salt, argon2_params),
    }
}

/// Decrypt payload using the selected encryption method.
pub fn decrypt_with_method(
    method: EncryptionMethod,
    payload: &EncryptedPayload,
    key: &[u8; 32],
) -> Result<Vec<u8>> {
    match method {
        EncryptionMethod::Aes256Gcm => decrypt(payload, key),
        EncryptionMethod::ChaCha20Poly1305 => decrypt_chacha(payload, key),
    }
}

/// Encrypt and serialize a value
pub fn encrypt_value<T: Serialize>(
    value: &T,
    key: &[u8; 32],
    salt: [u8; 32],
    argon2_params: Argon2Params,
) -> Result<EncryptedPayload> {
    let plaintext = bincode::serialize(value)
        .map_err(|e| Error::Encryption(format!("Serialization failed: {}", e)))?;
    encrypt(&plaintext, key, salt, argon2_params)
}

/// Decrypt and deserialize a value
pub fn decrypt_value<T: for<'de> Deserialize<'de>>(
    payload: &EncryptedPayload,
    key: &[u8; 32],
) -> Result<T> {
    let plaintext = decrypt(payload, key)?;
    bincode::deserialize(&plaintext).map_err(|_| Error::Decryption)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let salt = [1u8; 32];
        let params = Argon2Params::default();
        let plaintext = b"Hello, Vault!";

        let encrypted = encrypt(plaintext, &key, salt, params).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let key = [42u8; 32];
        let salt = [1u8; 32];
        let params = Argon2Params::default();
        let plaintext = b"Same plaintext";

        let encrypted1 = encrypt(plaintext, &key, salt, params).unwrap();
        let encrypted2 = encrypt(plaintext, &key, salt, params).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key = [42u8; 32];
        let wrong_key = [43u8; 32];
        let salt = [1u8; 32];
        let params = Argon2Params::default();
        let plaintext = b"Secret data";

        let encrypted = encrypt(plaintext, &key, salt, params).unwrap();
        let result = decrypt(&encrypted, &wrong_key);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let key = [42u8; 32];
        let salt = [1u8; 32];
        let params = Argon2Params::default();
        let plaintext = b"Secret data";

        let mut encrypted = encrypt(plaintext, &key, salt, params).unwrap();

        // Tamper with the ciphertext
        if let Some(byte) = encrypted.ciphertext.first_mut() {
            *byte ^= 0xFF;
        }

        let result = decrypt(&encrypted, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_value() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            name: String,
            value: i32,
        }

        let key = [42u8; 32];
        let salt = [1u8; 32];
        let params = Argon2Params::default();

        let original = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let encrypted = encrypt_value(&original, &key, salt, params).unwrap();
        let decrypted: TestData = decrypt_value(&encrypted, &key).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_empty_plaintext() {
        let key = [42u8; 32];
        let salt = [1u8; 32];
        let params = Argon2Params::default();
        let plaintext = b"";

        let encrypted = encrypt(plaintext, &key, salt, params).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_large_plaintext() {
        let key = [42u8; 32];
        let salt = [1u8; 32];
        let params = Argon2Params::default();

        // 1 MB of data
        let plaintext: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();

        let encrypted = encrypt(&plaintext, &key, salt, params).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encryption_method_profile_label() {
        let label = EncryptionMethod::Aes256Gcm.profile_label();
        assert!(label.contains("AES-256-GCM"));
        assert!(label.contains("High"));
    }
}
