//! Effect runtime - executes side effects
//!
//! Handles clipboard operations, file I/O, and timers.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use crate::crypto::SecureString;
use crate::domain::Vault;
use crate::storage::{AppConfig, VaultFile};

use super::effect::{Effect, EffectResult};
use super::message::Message;

/// Runtime for executing effects
pub struct Runtime {
    /// Channel for sending messages back to the app
    message_tx: Sender<Message>,
    /// Scheduled clipboard clear time
    clipboard_clear_at: Option<Instant>,
    /// Scheduled auto-lock time  
    auto_lock_at: Option<Instant>,
}

impl Runtime {
    /// Create a new runtime with message sender
    pub fn new(message_tx: Sender<Message>) -> Self {
        Self {
            message_tx,
            clipboard_clear_at: None,
            auto_lock_at: None,
        }
    }

    /// Execute an effect and return the result
    pub fn execute(&mut self, effect: Effect) -> EffectResult {
        match effect {
            Effect::None => EffectResult::Success,

            Effect::Batch(effects) => {
                for eff in effects {
                    let result = self.execute(eff);
                    if matches!(result, EffectResult::Error(_)) {
                        return result;
                    }
                }
                EffectResult::Success
            }

            Effect::SetClipboard {
                content,
                is_sensitive: _,
            } => match set_clipboard(&content) {
                Ok(()) => EffectResult::Success,
                Err(e) => EffectResult::Error(e),
            },

            Effect::ClearClipboard => {
                self.clipboard_clear_at = None;
                match clear_clipboard() {
                    Ok(()) => EffectResult::Success,
                    Err(e) => EffectResult::Error(e),
                }
            }

            Effect::ScheduleClipboardClear { delay } => {
                self.clipboard_clear_at = Some(Instant::now() + delay);
                EffectResult::Success
            }

            Effect::ScheduleAutoLock { delay } => {
                self.auto_lock_at = Some(Instant::now() + delay);
                EffectResult::Success
            }

            Effect::CancelAutoLock => {
                self.auto_lock_at = None;
                EffectResult::Success
            }

            Effect::ReadVaultFile {
                path,
                password,
                keyfile,
            } => match read_vault_file(&path, &password, keyfile.as_deref()) {
                Ok((vault, key, salt, has_keyfile, encryption_method, recovery_metadata)) => {
                    EffectResult::VaultLoaded {
                        vault,
                        path,
                        key,
                        salt,
                        has_keyfile,
                        encryption_method,
                        recovery_metadata,
                    }
                }
                Err(e) => EffectResult::Error(e),
            },

            Effect::WriteVaultFile {
                path,
                vault,
                key,
                salt,
                has_keyfile,
                encryption_method,
                recovery_metadata,
            } => match write_vault_file(
                &path,
                &vault,
                &key,
                &salt,
                has_keyfile,
                encryption_method,
                recovery_metadata,
            ) {
                Ok(()) => EffectResult::VaultSaved,
                Err(e) => EffectResult::Error(e),
            },

            Effect::ReadConfig => {
                let config = AppConfig::load_or_default();
                EffectResult::ConfigLoaded(config)
            }

            Effect::WriteConfig => {
                // Config writing is handled by the caller
                EffectResult::Success
            }

            Effect::UpdateRegistry => EffectResult::Success,

            Effect::ReadKeyfile { path } => match std::fs::read(&path) {
                Ok(data) => EffectResult::KeyfileLoaded { path, data },
                Err(e) => EffectResult::Error(format!("Failed to read keyfile: {}", e)),
            },

            Effect::ExportVault {
                path,
                vault,
                encrypted,
                key,
                salt,
                has_keyfile,
            } => match export_vault(
                &path,
                &vault,
                encrypted,
                key.as_ref(),
                salt.as_ref(),
                has_keyfile,
            ) {
                Ok(()) => EffectResult::ExportCompleted { path },
                Err(e) => EffectResult::Error(e),
            },

            Effect::Exit => EffectResult::Success,
        }
    }

    /// Check for scheduled timers and send messages
    pub fn tick(&mut self) {
        let now = Instant::now();

        // Check clipboard clear
        if let Some(clear_at) = self.clipboard_clear_at
            && now >= clear_at {
                self.clipboard_clear_at = None;
                let _ = self.message_tx.send(Message::ClearClipboard);
            }

        // Check auto-lock
        if let Some(lock_at) = self.auto_lock_at
            && now >= lock_at {
                self.auto_lock_at = None;
                let _ = self.message_tx.send(Message::LockVault);
            }
    }

    /// Get time until next scheduled event (for sleep duration)
    pub fn next_tick_delay(&self) -> Duration {
        let now = Instant::now();
        let mut min_delay = Duration::from_millis(100); // Default tick rate

        if let Some(clear_at) = self.clipboard_clear_at
            && clear_at > now {
                min_delay = min_delay.min(clear_at - now);
            }

        if let Some(lock_at) = self.auto_lock_at
            && lock_at > now {
                min_delay = min_delay.min(lock_at - now);
            }

        min_delay
    }

    /// Schedule clipboard clear
    pub fn schedule_clipboard_clear(&mut self, delay: Duration) {
        self.clipboard_clear_at = Some(Instant::now() + delay);
    }

    /// Schedule auto-lock
    pub fn schedule_auto_lock(&mut self, delay: Duration) {
        self.auto_lock_at = Some(Instant::now() + delay);
    }

    /// Cancel auto-lock timer
    pub fn cancel_auto_lock(&mut self) {
        self.auto_lock_at = None;
    }

    /// Check if clipboard should be cleared
    pub fn should_clear_clipboard(&self) -> bool {
        self.clipboard_clear_at
            .map(|t| Instant::now() >= t)
            .unwrap_or(false)
    }
}

/// Set clipboard content using system clipboard
fn set_clipboard(content: &str) -> Result<(), String> {
    #[cfg(feature = "clipboard")]
    {
        use arboard::Clipboard;
        let mut clipboard =
            Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
        clipboard
            .set_text(content)
            .map_err(|e| format!("Failed to set clipboard: {}", e))
    }

    #[cfg(not(feature = "clipboard"))]
    {
        // Fallback for systems without clipboard support
        let _ = content;
        Ok(())
    }
}

/// Clear clipboard content
fn clear_clipboard() -> Result<(), String> {
    #[cfg(feature = "clipboard")]
    {
        use arboard::Clipboard;
        let mut clipboard =
            Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
        clipboard
            .set_text("")
            .map_err(|e| format!("Failed to clear clipboard: {}", e))
    }

    #[cfg(not(feature = "clipboard"))]
    {
        Ok(())
    }
}

/// Read and decrypt a vault file
fn read_vault_file(
    path: &PathBuf,
    password: &SecureString,
    keyfile: Option<&[u8]>,
) -> Result<
    (
        Vault,
        [u8; 32],
        [u8; 32],
        bool,
        crate::crypto::EncryptionMethod,
        Option<crate::domain::RecoveryMetadata>,
    ),
    String,
> {
    let vault_file = VaultFile::read(path).map_err(|e| match e {
        crate::utils::error::Error::VaultNotFound(_) => "Vault file not found".to_string(),
        crate::utils::error::Error::InvalidVaultFormat(_) => {
            "Invalid vault file format".to_string()
        }
        crate::utils::error::Error::FileRead(_, _) => {
            "Cannot read vault file - check permissions".to_string()
        }
        _ => format!("Failed to read vault: {}", e),
    })?;

    if vault_file.header.has_keyfile && keyfile.is_none() {
        return Err("This vault requires a keyfile".to_string());
    }

    let has_keyfile = vault_file.header.has_keyfile;
    let encryption_method = vault_file.header.encryption_method;
    let recovery_metadata = vault_file.header.recovery_metadata.clone();

    // Extract salt before consuming vault_file
    let salt = vault_file.encrypted_payload.salt;

    let (vault, key) = vault_file
        .decrypt_with_key(password, keyfile)
        .map_err(|e| match e {
            crate::utils::error::Error::Decryption => {
                "Wrong password or corrupted vault".to_string()
            }
            crate::utils::error::Error::KeyDerivation(_) => "Key derivation failed".to_string(),
            crate::utils::error::Error::InvalidKeyFile(_) => "Invalid keyfile".to_string(),
            _ => format!("Failed to decrypt vault: {}", e),
        })?;

    Ok((
        vault,
        key,
        salt,
        has_keyfile,
        encryption_method,
        recovery_metadata,
    ))
}

/// Write vault to file (needs vault state, called externally)
pub fn write_vault_file(
    path: &PathBuf,
    vault: &Vault,
    key: &[u8; 32],
    salt: &[u8; 32],
    has_keyfile: bool,
    encryption_method: crate::crypto::EncryptionMethod,
    recovery_metadata: Option<crate::domain::RecoveryMetadata>,
) -> Result<(), String> {
    let vault_file = VaultFile::new_with_key_options(
        vault.clone(),
        key,
        salt,
        has_keyfile,
        encryption_method,
        recovery_metadata,
    )
    .map_err(|e| format!("Failed to create vault file: {}", e))?;
    vault_file
        .write(path)
        .map_err(|e| format!("Failed to write vault: {}", e))
}

/// Export vault to JSON file
pub fn export_vault(
    path: &PathBuf,
    vault: &Vault,
    encrypted: bool,
    key: Option<&[u8; 32]>,
    salt: Option<&[u8; 32]>,
    has_keyfile: bool,
) -> Result<(), String> {
    if encrypted {
        let key = key.ok_or("Encryption key required for encrypted export")?;
        let salt = salt.ok_or("Salt required for encrypted export")?;
        let vault_file = VaultFile::new_with_key(vault.clone(), key, salt, has_keyfile)
            .map_err(|e| format!("Failed to create encrypted export: {}", e))?;
        vault_file
            .write(path)
            .map_err(|e| format!("Failed to write encrypted export: {}", e))
    } else {
        write_plaintext_export_atomic_stream(path, |file| {
            serde_json::to_writer_pretty(file, vault)
                .map_err(|e| format!("Failed to serialize vault: {}", e))
        })
    }
}

fn create_secure_file(path: &Path) -> Result<std::fs::File, String> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;

        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| format!("Failed to create export file: {}", e))
    }

    #[cfg(not(unix))]
    {
        std::fs::File::create(path).map_err(|e| format!("Failed to create export file: {}", e))
    }
}

fn set_secure_permissions(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set export file permissions: {}", e))?;
    }

    Ok(())
}

fn sync_parent_dir(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::File::open(parent)
            .and_then(|dir| dir.sync_all())
            .map_err(|e| format!("Failed to sync export directory: {}", e))?;
    }
    Ok(())
}

fn write_plaintext_export_atomic_stream<
    W: FnOnce(&mut std::io::BufWriter<&std::fs::File>) -> Result<(), String>,
>(
    path: &Path,
    write_fn: W,
) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create export directory: {}", e))?;
    }

    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let tmp_name = format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("vault_export"),
        uuid::Uuid::new_v4()
    );
    let tmp_path = parent.join(tmp_name);

    let file = create_secure_file(&tmp_path)?;
    let write_result = (|| {
        {
            let mut writer = std::io::BufWriter::new(&file);
            let res = write_fn(&mut writer);
            std::io::Write::flush(&mut writer).map_err(|e| format!("Failed to flush: {}", e))?;
            res
        }
        .map_err(|e| format!("Failed to write export: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to sync export: {}", e))
    })();

    if let Err(e) = write_result {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e);
    }

    drop(file);

    std::fs::rename(&tmp_path, path).map_err(|e| format!("Failed to finalize export: {}", e))?;
    set_secure_permissions(path)?;
    sync_parent_dir(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use tempfile::tempdir;

    #[test]
    fn test_runtime_creation() {
        let (tx, _rx) = mpsc::channel();
        let runtime = Runtime::new(tx);
        assert!(runtime.clipboard_clear_at.is_none());
        assert!(runtime.auto_lock_at.is_none());
    }

    #[test]
    fn test_schedule_clipboard_clear() {
        let (tx, _rx) = mpsc::channel();
        let mut runtime = Runtime::new(tx);

        runtime.schedule_clipboard_clear(Duration::from_secs(10));
        assert!(runtime.clipboard_clear_at.is_some());
    }

    #[test]
    fn test_schedule_auto_lock() {
        let (tx, _rx) = mpsc::channel();
        let mut runtime = Runtime::new(tx);

        runtime.schedule_auto_lock(Duration::from_secs(300));
        assert!(runtime.auto_lock_at.is_some());

        runtime.cancel_auto_lock();
        assert!(runtime.auto_lock_at.is_none());
    }

    #[test]
    fn test_tick_delay() {
        let (tx, _rx) = mpsc::channel();
        let mut runtime = Runtime::new(tx);

        // Default delay
        let delay = runtime.next_tick_delay();
        assert!(delay <= Duration::from_millis(100));

        // With scheduled event
        runtime.schedule_clipboard_clear(Duration::from_millis(50));
        let delay = runtime.next_tick_delay();
        assert!(delay <= Duration::from_millis(50));
    }

    #[test]
    fn test_execute_none() {
        let (tx, _rx) = mpsc::channel();
        let mut runtime = Runtime::new(tx);

        let result = runtime.execute(Effect::None);
        assert!(matches!(result, EffectResult::Success));
    }

    #[test]
    fn test_plaintext_export_writes_file_atomically() {
        let dir = tempdir().unwrap();
        let export_path = dir.path().join("export.json");
        let vault = Vault::new("Export Test");

        export_vault(&export_path, &vault, false, None, None, false).expect("export succeeds");

        let contents = std::fs::read_to_string(&export_path).expect("read export");
        assert!(contents.contains("\"name\": \"Export Test\""));
    }

    #[cfg(unix)]
    #[test]
    fn test_plaintext_export_permissions_are_restricted() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let export_path = dir.path().join("secure-export.json");
        let vault = Vault::new("Secure Export");

        export_vault(&export_path, &vault, false, None, None, false).expect("export succeeds");

        let mode = std::fs::metadata(export_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
