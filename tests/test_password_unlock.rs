use std::path::PathBuf;
use vault::crypto::SecureString;
use vault::storage::VaultFile;

#[test]
fn test_reject_legacy_v3_vault() {
    // Explicitly verify that version 3 vault is rejected
    let vault_path = PathBuf::from("test_vault.vault");
    if !vault_path.exists() { return; }

    let result = VaultFile::read(&vault_path);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unsupported vault version"));
    assert!(err.contains("version 4"));
}

#[test]
fn test_unlock_v4_vault_roundtrip() {
    use vault::domain::Vault;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("v4_test.vault");
    
    let vault = Vault::new("V4 Vault");
    let password = SecureString::new("v4password".to_string());
    
    // Create new v4 vault
    vault::storage::create_vault(&path, &vault, &password, None).expect("Create v4");
    
    // Read and unlock
    let loaded = vault::storage::open_vault(&path, &password, None).expect("Open v4");
    assert_eq!(loaded.name, "V4 Vault");
}

#[test]
fn test_unlock_wrong_password_v4() {
    use vault::domain::Vault;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("v4_wrong_pass.vault");
    
    let vault = Vault::new("Wrong Pass Test");
    let password = SecureString::new("correct".to_string());
    vault::storage::create_vault(&path, &vault, &password, None).unwrap();

    let wrong_password = SecureString::new("wrong".to_string());
    let result = vault::storage::open_vault(&path, &wrong_password, None);

    assert!(result.is_err());
}

#[test]
fn test_password_with_whitespace_v4() {
    use vault::domain::Vault;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("v4_whitespace.vault");
    
    let vault = Vault::new("Whitespace Test");
    let password = SecureString::new("testpass123".to_string());
    vault::storage::create_vault(&path, &vault, &password, None).unwrap();

    // Try with trailing whitespace
    let password_with_space = SecureString::new("testpass123 ".to_string());
    let result = vault::storage::open_vault(&path, &password_with_space, None);
    assert!(result.is_err());

    // Try with leading whitespace
    let password_with_leading = SecureString::new(" testpass123".to_string());
    let result2 = vault::storage::open_vault(&path, &password_with_leading, None);
    assert!(result2.is_err());
}
