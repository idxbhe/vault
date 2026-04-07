//! Key file generation and handling

use rand::RngCore;
use std::fs;
use std::path::Path;

use crate::utils::error::{Error, Result};

/// Key file size in bytes (256 bits)
pub const KEYFILE_SIZE: usize = 32;

/// Represents a key file used for additional vault security
#[derive(Debug, Clone)]
pub struct KeyFile {
    /// The key file data
    data: Vec<u8>,
}

impl KeyFile {
    /// Generate a new random key file
    pub fn generate() -> Self {
        let mut data = vec![0u8; KEYFILE_SIZE];
        rand::rngs::OsRng.fill_bytes(&mut data);
        Self { data }
    }

    /// Load a key file from disk
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let data = fs::read(path).map_err(|e| Error::FileRead(path.to_path_buf(), e))?;
        
        if data.is_empty() {
            return Err(Error::InvalidKeyFile("Key file is empty".to_string()));
        }
        
        // Allow any size key file, but warn if too small
        if data.len() < 8 {
            return Err(Error::InvalidKeyFile(
                "Key file must be at least 8 bytes".to_string(),
            ));
        }
        
        Ok(Self { data })
    }

    /// Save the key file to disk
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
        }
        
        fs::write(path, &self.data).map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
        
        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)
                .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?
                .permissions();
            perms.set_mode(0o600); // Owner read/write only
            fs::set_permissions(path, perms)
                .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
        }
        
        Ok(())
    }

    /// Get the key file data as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the size of the key file
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the key file is empty (should never be true for valid key files)
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for KeyFile {
    fn drop(&mut self) {
        // Zero out the key file data
        use zeroize::Zeroize;
        self.data.zeroize();
    }
}

/// Check if a path looks like a valid key file
pub fn is_valid_keyfile_path<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    
    // Must be a file
    if !path.is_file() {
        return false;
    }
    
    // Check size (should be reasonable)
    if let Ok(metadata) = fs::metadata(path) {
        let size = metadata.len();
        // Between 8 bytes and 10 MB
        size >= 8 && size <= 10 * 1024 * 1024
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_keyfile_generate() {
        let kf = KeyFile::generate();
        assert_eq!(kf.len(), KEYFILE_SIZE);
        assert!(!kf.is_empty());
    }

    #[test]
    fn test_keyfile_generate_unique() {
        let kf1 = KeyFile::generate();
        let kf2 = KeyFile::generate();
        assert_ne!(kf1.as_bytes(), kf2.as_bytes());
    }

    #[test]
    fn test_keyfile_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.key");
        
        let original = KeyFile::generate();
        original.save(&path).unwrap();
        
        let loaded = KeyFile::load(&path).unwrap();
        assert_eq!(original.as_bytes(), loaded.as_bytes());
    }

    #[test]
    fn test_keyfile_load_nonexistent() {
        let result = KeyFile::load("/nonexistent/path/keyfile.key");
        assert!(result.is_err());
    }

    #[test]
    fn test_keyfile_load_empty() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.key");
        fs::write(&path, "").unwrap();
        
        let result = KeyFile::load(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_keyfile_load_too_small() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("small.key");
        fs::write(&path, "tiny").unwrap(); // 4 bytes
        
        let result = KeyFile::load(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_keyfile_path() {
        let dir = tempdir().unwrap();
        let valid_path = dir.path().join("valid.key");
        fs::write(&valid_path, vec![0u8; 32]).unwrap();
        
        assert!(is_valid_keyfile_path(&valid_path));
        assert!(!is_valid_keyfile_path("/nonexistent/path"));
        assert!(!is_valid_keyfile_path(dir.path())); // Directory, not file
    }
}
