//! Vault file format and I/O

use bincode::Options;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;

use crate::crypto::{
    Argon2Params, EncryptedPayload, EncryptionMethod, SecureString, decrypt_with_method,
    derive_key, encrypt_with_method, generate_salt,
};
use crate::domain::{RecoveryMetadata, Vault};
use crate::utils::error::{Error, Result};

/// Magic bytes to identify vault files
pub const VAULT_MAGIC: &[u8; 4] = b"VALT";

/// Current file format version
pub const VAULT_VERSION: u16 = 4;
const MAX_HEADER_SIZE: usize = 64 * 1024;
const MAX_PAYLOAD_SIZE: usize = 16 * 1024 * 1024;

/// Vault file header (stored unencrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFileHeader {
    /// Vault UUID
    pub vault_id: Uuid,
    /// Vault name (for display in registry)
    pub vault_name: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Whether a keyfile is required
    pub has_keyfile: bool,
    /// Number of security questions
    pub security_question_count: u8,
    /// Security question texts (for display)
    pub security_questions: Vec<String>,
    /// Encryption method used for the vault payload
    pub encryption_method: EncryptionMethod,
    /// Argon2 parameters used for key derivation
    pub argon2_params: Argon2Params,
    /// Recovery metadata for forgot-password flow (optional)
    pub recovery_metadata: Option<RecoveryMetadata>,
}

impl VaultFileHeader {
    /// Create a header from a vault
    pub fn from_vault(
        vault: &Vault,
        has_keyfile: bool,
        encryption_method: EncryptionMethod,
        argon2_params: Argon2Params,
        recovery_metadata: Option<RecoveryMetadata>,
    ) -> Self {
        let question_texts = if let Some(metadata) = recovery_metadata.as_ref() {
            metadata
                .questions
                .iter()
                .map(|q| q.question.clone())
                .collect::<Vec<_>>()
        } else {
            vault
                .security_questions
                .iter()
                .map(|q| q.question.clone())
                .collect::<Vec<_>>()
        };

        Self {
            vault_id: vault.id,
            vault_name: vault.name.clone(),
            created_at: vault.created_at,
            has_keyfile,
            security_question_count: question_texts.len() as u8,
            security_questions: question_texts,
            encryption_method,
            argon2_params,
            recovery_metadata,
        }
    }
}


/// Complete vault file structure
#[derive(Debug)]
pub struct VaultFile {
    /// Unencrypted header
    pub header: VaultFileHeader,
    /// Encrypted vault data
    pub encrypted_payload: EncryptedPayload,
    /// Serialized header bytes used as AAD (for version 3+)
    pub(crate) header_bytes: Vec<u8>,
    /// File format version
    pub(crate) version: u16,
}

impl VaultFile {
    /// Create a new vault file from a vault
    pub fn new(vault: &Vault, password: &SecureString, keyfile: Option<&[u8]>) -> Result<Self> {
        Self::new_with_params(vault, password, keyfile, Argon2Params::default())
    }

    /// Create a new vault file with custom Argon2 parameters
    pub fn new_with_params(
        vault: &Vault,
        password: &SecureString,
        keyfile: Option<&[u8]>,
        params: Argon2Params,
    ) -> Result<Self> {
        Self::new_with_options(
            vault,
            password,
            keyfile,
            params,
            EncryptionMethod::Aes256Gcm,
            None,
        )
    }

    /// Create a new vault file with explicit encryption/recovery options.
    pub fn new_with_options(
        vault: &Vault,
        password: &SecureString,
        keyfile: Option<&[u8]>,
        params: Argon2Params,
        encryption_method: EncryptionMethod,
        recovery_metadata: Option<RecoveryMetadata>,
    ) -> Result<Self> {
        let salt = generate_salt();
        let key = derive_key(password, keyfile, &salt, &params)?;

        let header = VaultFileHeader::from_vault(
            vault,
            keyfile.is_some(),
            encryption_method,
            params,
            recovery_metadata,
        );

        // Serialize header to use as AAD
        let header_bytes = bincode::serialize(&header)
            .map_err(|e| Error::Encryption(format!("Header serialization failed: {}", e)))?;

        // Serialize the vault
        let vault_bytes = bincode::serialize(vault)
            .map_err(|e| Error::Encryption(format!("Serialization failed: {}", e)))?;

        // Encrypt the vault data with header as AAD
        let encrypted_payload =
            encrypt_with_method(encryption_method, &vault_bytes, &key, salt, &header_bytes)?;

        Ok(Self {
            header,
            encrypted_payload,
            header_bytes,
            version: VAULT_VERSION,
        })
    }

    /// Create a vault file using an existing derived key
    /// Used for re-saving after edits without re-deriving from password
    pub fn new_with_key(
        vault: Vault,
        key: &[u8; 32],
        salt: &[u8; 32],
        has_keyfile: bool,
    ) -> Result<Self> {
        Self::new_with_key_options(
            vault,
            key,
            salt,
            has_keyfile,
            EncryptionMethod::Aes256Gcm,
            None,
        )
    }

    /// Create a vault file with explicit header options using an existing key.
    pub fn new_with_key_options(
        vault: Vault,
        key: &[u8; 32],
        salt: &[u8; 32],
        has_keyfile: bool,
        encryption_method: EncryptionMethod,
        recovery_metadata: Option<RecoveryMetadata>,
    ) -> Result<Self> {
        let salt = *salt; // Use provided salt instead of generating new one
        let params = Argon2Params::default();

        let header =
            VaultFileHeader::from_vault(&vault, has_keyfile, encryption_method, params, recovery_metadata);

        // Serialize header to use as AAD
        let header_bytes = bincode::serialize(&header)
            .map_err(|e| Error::Encryption(format!("Header serialization failed: {}", e)))?;

        // Serialize the vault
        let vault_bytes = bincode::serialize(&vault)
            .map_err(|e| Error::Encryption(format!("Serialization failed: {}", e)))?;

        // Encrypt the vault data with header as AAD
        let encrypted_payload =
            encrypt_with_method(encryption_method, &vault_bytes, key, salt, &header_bytes)?;

        Ok(Self {
            header,
            encrypted_payload,
            header_bytes,
            version: VAULT_VERSION,
        })
    }

    /// Decrypt and return the vault
    pub fn decrypt(&self, password: &SecureString, keyfile: Option<&[u8]>) -> Result<Vault> {
        let key = derive_key(
            password,
            keyfile,
            &self.encrypted_payload.salt,
            &self.header.argon2_params,
        )?;

        let vault_bytes =
            decrypt_with_method(self.header.encryption_method, &self.encrypted_payload, &key, &self.header_bytes)?;

        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_limit(MAX_PAYLOAD_SIZE as u64)
            .deserialize(&vault_bytes)
            .map_err(|_| Error::Decryption)
    }

    /// Decrypt and return the vault with the derived key
    /// Returns (Vault, encryption_key) for use in saving later
    pub fn decrypt_with_key(
        &self,
        password: &SecureString,
        keyfile: Option<&[u8]>,
    ) -> Result<(Vault, [u8; 32])> {
        let key = derive_key(
            password,
            keyfile,
            &self.encrypted_payload.salt,
            &self.header.argon2_params,
        )?;

        let vault_bytes =
            decrypt_with_method(self.header.encryption_method, &self.encrypted_payload, &key, &self.header_bytes)?;

        let vault: Vault = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_limit(MAX_PAYLOAD_SIZE as u64)
            .deserialize(&vault_bytes)
            .map_err(|_| Error::Decryption)?;

        Ok((vault, key))
    }

    /// Read a vault file from disk
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut file = fs::File::open(path).map_err(|e| Error::FileRead(path.to_path_buf(), e))?;

        // Read and verify magic
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)
            .map_err(|e| Error::FileRead(path.to_path_buf(), e))?;
        if &magic != VAULT_MAGIC {
            return Err(Error::InvalidVaultFormat(
                "Invalid vault file magic".to_string(),
            ));
        }

        // Read version
        let mut version_bytes = [0u8; 2];
        file.read_exact(&mut version_bytes)
            .map_err(|e| Error::FileRead(path.to_path_buf(), e))?;
        let version = u16::from_le_bytes(version_bytes);
        if version != VAULT_VERSION {
            return Err(Error::InvalidVaultFormat(format!(
                "Unsupported vault version: {}. This version of Vault only supports version {}.",
                version, VAULT_VERSION
            )));
        }

        // Read header length
        let mut header_len_bytes = [0u8; 4];
        file.read_exact(&mut header_len_bytes)
            .map_err(|e| Error::FileRead(path.to_path_buf(), e))?;
        let header_len = u32::from_le_bytes(header_len_bytes) as usize;
        if header_len > MAX_HEADER_SIZE {
            return Err(Error::InvalidVaultFormat(format!(
                "Header too large: {} bytes",
                header_len
            )));
        }

        // Read header
        let mut header_bytes = vec![0u8; header_len];
        file.read_exact(&mut header_bytes)
            .map_err(|e| Error::FileRead(path.to_path_buf(), e))?;

        let header: VaultFileHeader = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_limit(MAX_HEADER_SIZE as u64)
            .deserialize(&header_bytes)
            .map_err(|e| Error::InvalidVaultFormat(format!("Invalid header: {}", e)))?;

        // Sanity Check / Hard Limit on Argon2 parameters to prevent KDF DoS
        if header.argon2_params.memory_kib > 262144    // 256 MiB
            || header.argon2_params.iterations > 10
            || header.argon2_params.parallelism > 8
        {
            return Err(Error::InvalidVaultFormat(
                "Argon2 parameters exceed safety limits".to_string(),
            ));
        }

        let aad_bytes = header_bytes;

        // Read encrypted payload length
        let mut payload_len_bytes = [0u8; 4];
        file.read_exact(&mut payload_len_bytes)
            .map_err(|e| Error::FileRead(path.to_path_buf(), e))?;
        let payload_len = u32::from_le_bytes(payload_len_bytes) as usize;
        if payload_len > MAX_PAYLOAD_SIZE {
            return Err(Error::InvalidVaultFormat(format!(
                "Payload too large: {} bytes",
                payload_len
            )));
        }

        // Read encrypted payload
        let mut payload_bytes = vec![0u8; payload_len];
        file.read_exact(&mut payload_bytes)
            .map_err(|e| Error::FileRead(path.to_path_buf(), e))?;
        let encrypted_payload: EncryptedPayload = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_limit(MAX_PAYLOAD_SIZE as u64)
            .deserialize(&payload_bytes)
            .map_err(|e| Error::InvalidVaultFormat(format!("Invalid payload: {}", e)))?;

        Ok(Self {
            header,
            encrypted_payload,
            header_bytes: aad_bytes,
            version,
        })
    }

    /// Write the vault file to disk
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
        }

        // Use stored header bytes if version 3+ to ensure AAD consistency.
        // For older versions, we re-serialize (AAD is empty anyway).
        let header_bytes = if self.version >= 3 && !self.header_bytes.is_empty() {
            self.header_bytes.clone()
        } else {
            bincode::serialize(&self.header)
                .map_err(|e| Error::Encryption(format!("Header serialization failed: {}", e)))?
        };

        let payload_bytes = bincode::serialize(&self.encrypted_payload)
            .map_err(|e| Error::Encryption(format!("Payload serialization failed: {}", e)))?;

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let tmp_name = format!(
            ".{}.{}.tmp",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("vault"),
            Uuid::new_v4()
        );
        let tmp_path = parent.join(tmp_name);

        let mut file = create_secure_file(&tmp_path)?;

        let write_result = (|| {
            file.write_all(VAULT_MAGIC)
                .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;

            file.write_all(&self.version.to_le_bytes())
                .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;

            file.write_all(&(header_bytes.len() as u32).to_le_bytes())
                .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
            file.write_all(&header_bytes)
                .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;

            file.write_all(&(payload_bytes.len() as u32).to_le_bytes())
                .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
            file.write_all(&payload_bytes)
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

        Ok(())
    }
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

fn set_secure_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))?;
    }

    Ok(())
}

/// Quick function to read only the header (for registry updates)
pub fn read_header<P: AsRef<Path>>(path: P) -> Result<VaultFileHeader> {
    let vault_file = VaultFile::read(path)?;
    Ok(vault_file.header)
}

/// Create a new vault file
pub fn create_vault<P: AsRef<Path>>(
    path: P,
    vault: &Vault,
    password: &SecureString,
    keyfile: Option<&[u8]>,
) -> Result<()> {
    let vault_file = VaultFile::new(vault, password, keyfile)?;
    vault_file.write(path)
}

/// Open and decrypt a vault file
pub fn open_vault<P: AsRef<Path>>(
    path: P,
    password: &SecureString,
    keyfile: Option<&[u8]>,
) -> Result<Vault> {
    let vault_file = VaultFile::read(path)?;
    vault_file.decrypt(password, keyfile)
}

/// Save a vault to an existing file
pub fn save_vault<P: AsRef<Path>>(
    path: P,
    vault: &Vault,
    password: &SecureString,
    keyfile: Option<&[u8]>,
) -> Result<()> {
    create_vault(path, vault, password, keyfile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Item;
    use tempfile::tempdir;

    // Helper to create vaults with fast test params
    fn create_vault_fast<P: AsRef<Path>>(
        path: P,
        vault: &Vault,
        password: &SecureString,
        keyfile: Option<&[u8]>,
    ) -> Result<()> {
        let params = Argon2Params {
            memory_kib: 1024,
            iterations: 1,
            parallelism: 1,
        };
        let vault_file = VaultFile::new_with_params(vault, password, keyfile, params)?;
        vault_file.write(path)
    }

    #[test]
    fn test_vault_file_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.vault");

        let mut vault = Vault::new("Test Vault");
        vault.add_item(Item::password("GitHub", "secret123"));

        let password = SecureString::from_str("master_password");

        // Create and write with fast params
        create_vault_fast(&path, &vault, &password, None).unwrap();

        // Verify file was written
        assert!(path.exists(), "Vault file should exist");

        // Read and decrypt
        let loaded = open_vault(&path, &password, None).expect("Should decrypt successfully");

        assert_eq!(loaded.name, vault.name);
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].title, "GitHub");
    }

    #[test]
    fn test_vault_file_with_keyfile() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.vault");

        let vault = Vault::new("Secured Vault");
        let password = SecureString::from_str("password");
        let keyfile = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        create_vault_fast(&path, &vault, &password, Some(&keyfile)).unwrap();

        // Should fail without keyfile
        assert!(open_vault(&path, &password, None).is_err());

        // Should succeed with keyfile
        let loaded = open_vault(&path, &password, Some(&keyfile)).unwrap();
        assert_eq!(loaded.name, "Secured Vault");
    }

    #[test]
    fn test_vault_file_wrong_password() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.vault");

        let vault = Vault::new("Test");
        let password = SecureString::from_str("correct_password");
        let wrong = SecureString::from_str("wrong_password");

        create_vault_fast(&path, &vault, &password, None).unwrap();

        assert!(open_vault(&path, &wrong, None).is_err());
    }

    #[test]
    fn test_read_header() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.vault");

        let vault = Vault::new("Header Test").with_description("Testing header read");
        let password = SecureString::from_str("password");

        create_vault_fast(&path, &vault, &password, None).unwrap();

        let header = read_header(&path).unwrap();
        assert_eq!(header.vault_name, "Header Test");
        assert!(!header.has_keyfile);
    }

    #[test]
    fn test_vault_file_magic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.vault");

        // Write invalid magic
        fs::write(&path, b"FAKE").unwrap();

        assert!(VaultFile::read(&path).is_err());
    }

    #[test]
    fn test_new_with_key_preserves_keyfile_flag() {
        let vault = Vault::new("Has Keyfile");
        let key = [7u8; 32];
        let salt = [9u8; 32];

        let with_keyfile = VaultFile::new_with_key(vault.clone(), &key, &salt, true).unwrap();
        assert!(with_keyfile.header.has_keyfile);

        let without_keyfile = VaultFile::new_with_key(vault, &key, &salt, false).unwrap();
        assert!(!without_keyfile.header.has_keyfile);
    }

    #[test]
    fn test_read_rejects_oversized_header() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("oversized-header.vault");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(VAULT_MAGIC);
        bytes.extend_from_slice(&VAULT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&((MAX_HEADER_SIZE as u32) + 1).to_le_bytes());
        fs::write(&path, bytes).unwrap();

        assert!(matches!(
            VaultFile::read(&path),
            Err(Error::InvalidVaultFormat(msg)) if msg.contains("Header too large")
        ));
    }

    #[test]
    fn test_read_rejects_oversized_payload() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("oversized-payload.vault");
        let header = VaultFileHeader::from_vault(
            &Vault::new("Payload Test"),
            false,
            EncryptionMethod::Aes256Gcm,
            Argon2Params::default(),
            None,
        );
        let header_bytes = bincode::serialize(&header).unwrap();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(VAULT_MAGIC);
        bytes.extend_from_slice(&VAULT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&header_bytes);
        bytes.extend_from_slice(&((MAX_PAYLOAD_SIZE as u32) + 1).to_le_bytes());
        fs::write(&path, bytes).unwrap();

        assert!(matches!(
            VaultFile::read(&path),
            Err(Error::InvalidVaultFormat(msg)) if msg.contains("Payload too large")
        ));
    }

    #[cfg(unix)]
    #[test]
    fn test_vault_file_permissions_are_restricted() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let path = dir.path().join("secure.vault");
        let vault = Vault::new("Secure Vault");
        let password = SecureString::from_str("password");

        create_vault_fast(&path, &vault, &password, None).unwrap();

        let mode = fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }


    #[test]
    fn test_tampered_header_fails_decryption() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tampered.vault");

        let vault = Vault::new("Tamper Test");
        let password = SecureString::from_str("password");

        create_vault_fast(&path, &vault, &password, None).unwrap();

        // Tamper with the header in the file
        let mut file_bytes = fs::read(&path).unwrap();

        // Layout: Magic(4) + Version(2) + HeaderLen(4) + Header(var)
        // Let's change a byte in the header region (starting at offset 10)
        file_bytes[10] ^= 0xFF;

        fs::write(&path, file_bytes).unwrap();

        // Reading should either fail deserialization or decryption
        let loaded_res = VaultFile::read(&path);
        if let Ok(loaded) = loaded_res {
            let decrypted = loaded.decrypt(&password, None);
            assert!(decrypted.is_err(), "Decryption should fail after header tampering");
        } else {
            // If deserialization of header fails, that's also a win for tamper-evidence
            // although AEAD AAD mismatch specifically happens during decrypt.
        }
    }

    #[test]
    fn test_read_rejects_malicious_argon2_params() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("malicious.vault");

        // 1. Create a header with malicious params (e.g., 4GB memory)
        let mut params = Argon2Params::default();
        params.memory_kib = 4 * 1024 * 1024; // 4 GiB

        let vault = Vault::new("Malicious Vault");
        let header = VaultFileHeader::from_vault(
            &vault,
            false,
            EncryptionMethod::Aes256Gcm,
            params,
            None,
        );

        let header_bytes = bincode::serialize(&header).unwrap();
        let payload = EncryptedPayload::new(vec![0; 32], [0; 12], [0; 32]);
        let payload_bytes = bincode::serialize(&payload).unwrap();

        // 2. Write manually to file
        let mut bytes = Vec::new();
        bytes.extend_from_slice(VAULT_MAGIC);
        bytes.extend_from_slice(&VAULT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&header_bytes);
        bytes.extend_from_slice(&(payload_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload_bytes);

        fs::write(&path, bytes).unwrap();

        // 3. Attempt to read
        let result = VaultFile::read(&path);

        assert!(matches!(
            result,
            Err(Error::InvalidVaultFormat(msg)) if msg.contains("Argon2 parameters exceed safety limits")
        ));
    }
}
