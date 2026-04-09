use std::path::PathBuf;
use vault::crypto::SecureString;
use vault::storage::VaultFile;

#[test]
fn test_unlock_test_vault() {
    // Test that test_vault.vault can be unlocked with password "testpass123"
    let vault_path = PathBuf::from("test_vault.vault");

    // Verify file exists
    assert!(vault_path.exists(), "test_vault.vault should exist");

    // Read the vault file
    let vault_file = VaultFile::read(&vault_path).expect("Should be able to read test_vault.vault");

    // Try to decrypt with the correct password
    let password = SecureString::new("testpass123".to_string());
    let result = vault_file.decrypt_with_key(&password, None);

    match result {
        Ok((vault, _key)) => {
            println!("✅ Successfully unlocked vault");
            println!("   Vault name: {}", vault.name);
            println!("   Item count: {}", vault.items.len());
            assert_eq!(vault.name, "Test Vault");
        }
        Err(e) => {
            panic!("❌ Failed to unlock vault with correct password: {:?}", e);
        }
    }
}

#[test]
fn test_unlock_wrong_password() {
    // Verify that wrong password fails
    let vault_path = PathBuf::from("test_vault.vault");

    if !vault_path.exists() {
        eprintln!("Skipping test - test_vault.vault not found");
        return;
    }

    let vault_file = VaultFile::read(&vault_path).expect("Should be able to read test_vault.vault");

    let wrong_password = SecureString::new("wrongpassword".to_string());
    let result = vault_file.decrypt_with_key(&wrong_password, None);

    assert!(result.is_err(), "Wrong password should fail to decrypt");
}

#[test]
fn test_password_with_whitespace() {
    // Test if whitespace affects password
    let vault_path = PathBuf::from("test_vault.vault");

    if !vault_path.exists() {
        eprintln!("Skipping test - test_vault.vault not found");
        return;
    }

    let vault_file = VaultFile::read(&vault_path).expect("Should be able to read test_vault.vault");

    // Try with trailing whitespace
    let password_with_space = SecureString::new("testpass123 ".to_string());
    let result = vault_file.decrypt_with_key(&password_with_space, None);

    assert!(result.is_err(), "Password with trailing space should fail");

    // Try with leading whitespace
    let vault_file2 = VaultFile::read(&vault_path).expect("read");
    let password_with_leading = SecureString::new(" testpass123".to_string());
    let result2 = vault_file2.decrypt_with_key(&password_with_leading, None);

    assert!(result2.is_err(), "Password with leading space should fail");
}
