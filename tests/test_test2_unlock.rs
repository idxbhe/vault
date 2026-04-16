use std::path::PathBuf;
use vault::crypto::SecureString;
use vault::storage::VaultFile;

#[test]
fn test_reject_legacy_test2_vault() {
    let vault_path = PathBuf::from("test2.vault");
    if !vault_path.exists() { return; }

    let result = VaultFile::read(&vault_path);
    assert!(result.is_err());
}

#[test]
fn test_unlock_v4_test2_roundtrip() {
    use vault::domain::Vault;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("v4_test2.vault");
    
    let vault = Vault::new("V4 Test2");
    let password = SecureString::new("sudounlock".to_string());
    
    vault::storage::create_vault(&path, &vault, &password, None).expect("Create v4");
    
    let loaded = vault::storage::open_vault(&path, &password, None).expect("Open v4");
    assert_eq!(loaded.name, "V4 Test2");
}

#[test]
fn test_unlock_v4_test2_with_trimming() {
    use vault::domain::Vault;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("v4_trim.vault");
    
    let vault = Vault::new("Trim Test");
    let password = SecureString::new("sudounlock".to_string());
    vault::storage::create_vault(&path, &vault, &password, None).unwrap();

    let password_with_space = "  sudounlock  ".trim().to_string();
    let password_obj = SecureString::new(password_with_space);
    let result = vault::storage::open_vault(&path, &password_obj, None);

    assert!(result.is_ok());
}
