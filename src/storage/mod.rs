//! Persistence and file I/O

pub mod config;
pub mod registry;
pub mod vault_file;

pub use config::{AppConfig, IconColorChoice, ThemeChoice};
pub use registry::{VaultRegistry, VaultRegistryEntry};
pub use vault_file::{VaultFile, VaultFileHeader};
