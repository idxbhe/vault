//! Side effects - IO and async operations in TEA
//!
//! Effects represent operations that have side effects (file I/O, clipboard, timers).
//! These are returned from the update function and executed by the runtime.

use std::path::PathBuf;
use std::time::Duration;

use crate::crypto::SecureString;
use crate::domain::Vault;

/// Side effects that need to be executed
#[derive(Debug)]
pub enum Effect {
    /// No effect
    None,
    /// Multiple effects to execute
    Batch(Vec<Effect>),

    // === File I/O ===
    /// Read a vault file
    ReadVaultFile {
        path: PathBuf,
        password: SecureString,
        keyfile: Option<Vec<u8>>,
    },
    /// Write vault to file
    WriteVaultFile {
        path: PathBuf,
        vault: Vault,
        key: [u8; 32],
        salt: [u8; 32],
    },
    /// Read application config
    ReadConfig,
    /// Write application config
    WriteConfig,
    /// Update vault registry
    UpdateRegistry,
    /// Read keyfile
    ReadKeyfile { path: PathBuf },

    // === Clipboard ===
    /// Set clipboard content
    SetClipboard {
        content: String,
        is_sensitive: bool,
    },
    /// Clear clipboard
    ClearClipboard,
    /// Schedule clipboard clear after delay
    ScheduleClipboardClear { delay: Duration },

    // === Timer ===
    /// Schedule auto-lock after delay
    ScheduleAutoLock { delay: Duration },
    /// Cancel scheduled auto-lock
    CancelAutoLock,

    // === Export ===
    /// Export vault to JSON file
    ExportVault {
        path: PathBuf,
        vault: Vault,
        encrypted: bool,
        key: Option<[u8; 32]>,
    },

    // === System ===
    /// Exit the application
    Exit,
}

impl Effect {
    /// Create a no-op effect
    pub fn none() -> Self {
        Self::None
    }

    /// Create a batch of effects
    pub fn batch(effects: Vec<Effect>) -> Self {
        // Flatten and remove no-ops
        let effects: Vec<Effect> = effects
            .into_iter()
            .flat_map(|e| match e {
                Effect::None => vec![],
                Effect::Batch(inner) => inner,
                other => vec![other],
            })
            .collect();

        match effects.len() {
            0 => Effect::None,
            1 => effects.into_iter().next().unwrap(),
            _ => Effect::Batch(effects),
        }
    }

    /// Check if this is a no-op
    pub fn is_none(&self) -> bool {
        matches!(self, Effect::None)
    }
}

impl Default for Effect {
    fn default() -> Self {
        Self::None
    }
}

/// Result of executing an effect
#[derive(Debug)]
pub enum EffectResult {
    /// Effect completed successfully
    Success,
    /// Vault was loaded successfully
    VaultLoaded {
        vault: crate::domain::Vault,
        path: PathBuf,
        key: [u8; 32],
        salt: [u8; 32],
    },
    /// Vault was saved successfully
    VaultSaved,
    /// Export completed successfully
    ExportCompleted { path: PathBuf },
    /// Config was loaded
    ConfigLoaded(crate::storage::AppConfig),
    /// Registry was loaded
    RegistryLoaded(crate::storage::VaultRegistry),
    /// Keyfile was read
    KeyfileLoaded { path: PathBuf, data: Vec<u8> },
    /// Effect failed
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_none() {
        let effect = Effect::none();
        assert!(effect.is_none());
    }

    #[test]
    fn test_effect_batch_empty() {
        let effect = Effect::batch(vec![]);
        assert!(effect.is_none());
    }

    #[test]
    fn test_effect_batch_single() {
        let effect = Effect::batch(vec![Effect::Exit]);
        assert!(matches!(effect, Effect::Exit));
    }

    #[test]
    fn test_effect_batch_flattens() {
        let effect = Effect::batch(vec![
            Effect::None,
            Effect::batch(vec![Effect::ClearClipboard, Effect::Exit]),
            Effect::None,
        ]);

        match effect {
            Effect::Batch(effects) => {
                assert_eq!(effects.len(), 2);
            }
            _ => panic!("Expected Batch"),
        }
    }
}
