//! Create test2 vault with password "sudounlock"

use vault::domain::Vault;
use vault::storage::VaultFile;
use vault::crypto::SecureString;
use std::path::PathBuf;

#[test]
#[ignore]
fn create_test2_vault() {
    let vault = Vault::new("test2");
    let password = SecureString::new("sudounlock".to_string());
    
    let vault_file = VaultFile::new(&vault, &password, None).unwrap();
    let path = PathBuf::from("test2.vault");
    vault_file.write(&path).unwrap();
    
    println!("Created test2.vault with password 'sudounlock'");
}
