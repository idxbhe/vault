//! Helper to create a test vault file

use std::path::PathBuf;
use vault::crypto::SecureString;
use vault::domain::Vault;
use vault::storage::VaultFile;

#[test]
#[ignore] // Run manually with: cargo test create_test_vault -- --ignored
fn create_test_vault() {
    let vault = Vault::new("Test Vault");
    let password = SecureString::new("testpass123".to_string());

    let vault_file = VaultFile::new(&vault, &password, None).unwrap();
    let path = PathBuf::from("test_vault.vault");
    vault_file.write(&path).unwrap();

    println!("Created test_vault.vault with password 'testpass123'");
}
