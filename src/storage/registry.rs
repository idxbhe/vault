//! Vault registry - tracks known vaults

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::config::AppConfig;
use crate::utils::error::{Error, Result};

/// Entry in the vault registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultRegistryEntry {
    /// Path to the vault file
    pub path: PathBuf,
    /// Vault name (cached from vault)
    pub name: String,
    /// Last time this vault was opened
    pub last_opened: DateTime<Utc>,
    /// Whether this is the default vault
    pub is_default: bool,
}

impl VaultRegistryEntry {
    /// Create a new registry entry
    pub fn new(path: impl Into<PathBuf>, name: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            last_opened: Utc::now(),
            is_default: false,
        }
    }

    /// Update the last opened time
    pub fn touch(&mut self) {
        self.last_opened = Utc::now();
    }

    /// Check if the vault file still exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

/// Registry of known vaults
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VaultRegistry {
    /// All known vault entries
    pub entries: Vec<VaultRegistryEntry>,
}

impl VaultRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Load the registry from disk
    pub fn load() -> Result<Self> {
        let path = Self::registry_path()?;
        if path.exists() {
            let contents =
                fs::read_to_string(&path).map_err(|e| Error::FileRead(path.clone(), e))?;
            let registry: VaultRegistry = serde_json::from_str(&contents)?;
            Ok(registry)
        } else {
            Ok(Self::new())
        }
    }

    /// Save the registry to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::registry_path()?;
        let contents = serde_json::to_string_pretty(self)?;
        write_atomic_secure(&path, contents.as_bytes())
    }

    /// Get the registry file path
    pub fn registry_path() -> Result<PathBuf> {
        let data_dir = AppConfig::data_dir()?;
        Ok(data_dir.join("registry.json"))
    }

    /// Add or update an entry
    pub fn add_or_update(&mut self, path: impl AsRef<Path>, name: impl Into<String>) {
        let path = path.as_ref().to_path_buf();
        let name = name.into();

        if let Some(entry) = self.entries.iter_mut().find(|e| e.path == path) {
            entry.name = name;
            entry.touch();
        } else {
            self.entries.push(VaultRegistryEntry::new(path, name));
        }
    }

    /// Remove an entry by path
    pub fn remove(&mut self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        let len_before = self.entries.len();
        self.entries.retain(|e| e.path != path);
        self.entries.len() < len_before
    }

    /// Get the default vault
    pub fn default_vault(&self) -> Option<&VaultRegistryEntry> {
        self.entries.iter().find(|e| e.is_default)
    }

    /// Set a vault as default
    pub fn set_default(&mut self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        // Clear existing default
        for entry in &mut self.entries {
            entry.is_default = false;
        }

        // Set new default
        if let Some(entry) = self.entries.iter_mut().find(|e| e.path == path) {
            entry.is_default = true;
            true
        } else {
            false
        }
    }

    /// Get entries sorted by last opened (most recent first)
    pub fn sorted_by_recent(&self) -> Vec<&VaultRegistryEntry> {
        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
        entries
    }

    /// Remove entries for vaults that no longer exist
    pub fn cleanup(&mut self) -> usize {
        let len_before = self.entries.len();
        self.entries.retain(|e| e.exists());
        len_before - self.entries.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

fn set_secure_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
    }

    Ok(())
}

fn create_secure_file(path: &Path) -> Result<fs::File> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;

        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))
    }

    #[cfg(not(unix))]
    {
        fs::File::create(path).map_err(|e| Error::FileWrite(path.to_path_buf(), e))
    }
}

fn sync_parent_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        if let Some(parent) = path.parent() {
            fs::File::open(parent)
                .and_then(|dir| dir.sync_all())
                .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
        }
    }
    Ok(())
}

fn write_atomic_secure(path: &Path, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp_name = format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("registry"),
        uuid::Uuid::new_v4()
    );
    let tmp_path = parent.join(tmp_name);

    let mut file = create_secure_file(&tmp_path)?;
    let write_result = (|| {
        file.write_all(contents)
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
        file.sync_all()
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))
    })();

    if let Err(e) = write_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }

    drop(file);
    fs::rename(&tmp_path, path).map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
    set_secure_permissions(path)?;
    sync_parent_dir(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_entry() {
        let entry = VaultRegistryEntry::new("/path/to/vault.vault", "My Vault");

        assert_eq!(entry.name, "My Vault");
        assert!(!entry.is_default);
    }

    #[test]
    fn test_registry_add_update() {
        let mut registry = VaultRegistry::new();

        registry.add_or_update("/vault1.vault", "Vault 1");
        assert_eq!(registry.len(), 1);

        registry.add_or_update("/vault2.vault", "Vault 2");
        assert_eq!(registry.len(), 2);

        // Update existing
        registry.add_or_update("/vault1.vault", "Updated Vault 1");
        assert_eq!(registry.len(), 2);

        let entry = registry
            .entries
            .iter()
            .find(|e| e.path == PathBuf::from("/vault1.vault"));
        assert_eq!(entry.unwrap().name, "Updated Vault 1");
    }

    #[test]
    fn test_registry_default() {
        let mut registry = VaultRegistry::new();

        registry.add_or_update("/vault1.vault", "Vault 1");
        registry.add_or_update("/vault2.vault", "Vault 2");

        assert!(registry.default_vault().is_none());

        registry.set_default("/vault1.vault");
        assert_eq!(
            registry.default_vault().unwrap().path,
            PathBuf::from("/vault1.vault")
        );

        // Change default
        registry.set_default("/vault2.vault");
        assert_eq!(
            registry.default_vault().unwrap().path,
            PathBuf::from("/vault2.vault")
        );

        // Old default should be unset
        let v1 = registry
            .entries
            .iter()
            .find(|e| e.path == PathBuf::from("/vault1.vault"));
        assert!(!v1.unwrap().is_default);
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = VaultRegistry::new();

        registry.add_or_update("/vault1.vault", "Vault 1");
        registry.add_or_update("/vault2.vault", "Vault 2");

        assert!(registry.remove("/vault1.vault"));
        assert_eq!(registry.len(), 1);

        assert!(!registry.remove("/nonexistent.vault"));
    }

    #[test]
    fn test_registry_sorted() {
        let mut registry = VaultRegistry::new();

        registry.add_or_update("/old.vault", "Old");
        std::thread::sleep(std::time::Duration::from_millis(10));
        registry.add_or_update("/new.vault", "New");

        let sorted = registry.sorted_by_recent();
        assert_eq!(sorted[0].name, "New");
        assert_eq!(sorted[1].name, "Old");
    }

    #[cfg(unix)]
    #[test]
    fn test_set_secure_permissions() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("registry.json");
        std::fs::write(&path, "{}").unwrap();

        set_secure_permissions(&path).unwrap();
        let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn test_write_atomic_secure_writes_content() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("registry.json");
        write_atomic_secure(&path, br#"{"entries":[]}"#).unwrap();

        let contents = std::fs::read_to_string(path).unwrap();
        assert_eq!(contents, r#"{"entries":[]}"#);
    }

    #[cfg(unix)]
    #[test]
    fn test_write_atomic_secure_sets_permissions() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("registry.json");
        write_atomic_secure(&path, b"{}").unwrap();

        let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
