//! Application error types

use std::path::PathBuf;
use thiserror::Error;

/// Application-wide result type
pub type Result<T> = std::result::Result<T, Error>;

/// Application error types
#[derive(Debug, Error)]
pub enum Error {
    // === Crypto Errors ===
    #[error("Encryption failed: {0}")]
    Encryption(String),

    #[error("Decryption failed: invalid password or corrupted data")]
    Decryption,

    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("Invalid key file: {0}")]
    InvalidKeyFile(String),

    // === Storage Errors ===
    #[error("Vault file not found: {}", .0.display())]
    VaultNotFound(PathBuf),

    #[error("Invalid vault file format: {0}")]
    InvalidVaultFormat(String),

    #[error("Vault file corrupted: {0}")]
    VaultCorrupted(String),

    #[error("Failed to read file: {}", .0.display())]
    FileRead(PathBuf, #[source] std::io::Error),

    #[error("Failed to write file: {}", .0.display())]
    FileWrite(PathBuf, #[source] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    // === Domain Errors ===
    #[error("Item not found: {0}")]
    ItemNotFound(uuid::Uuid),

    #[error("Tag not found: {0}")]
    TagNotFound(uuid::Uuid),

    #[error("Vault is locked")]
    VaultLocked,

    #[error("Invalid item data: {0}")]
    InvalidItem(String),

    // === Security Errors ===
    #[error("Authentication failed: incorrect password")]
    AuthenticationFailed,

    #[error("Security question verification failed")]
    SecurityQuestionFailed,

    #[error("Maximum recovery attempts exceeded")]
    MaxRecoveryAttempts,

    // === UI Errors ===
    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    // === General ===
    #[error("Operation cancelled")]
    Cancelled,

    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Error::InvalidVaultFormat(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::InvalidVaultFormat(err.to_string())
    }
}
