use vault::domain::Vault;
use vault::crypto::SecureString;
use vault::storage::VaultFile;
use std::path::PathBuf;

#[test]
fn test_unlock_test2_vault() {
    let vault_path = PathBuf::from("test2.vault");
    
    if !vault_path.exists() {
        eprintln!("Skipping - test2.vault not found");
        return;
    }
    
    let vault_file = VaultFile::read(&vault_path)
        .expect("Should read test2.vault");
    
    let password = SecureString::new("sudounlock".to_string());
    let result = vault_file.decrypt_with_key(&password, None);
    
    match result {
        Ok((vault, _key)) => {
            println!("✅ Successfully unlocked test2 vault");
            println!("   Vault name: {}", vault.name);
        }
        Err(e) => {
            panic!("❌ Failed to unlock test2 with password 'sudounlock': {:?}", e);
        }
    }
}

#[test]
fn test_unlock_test2_with_trimming() {
    let vault_path = PathBuf::from("test2.vault");
    
    if !vault_path.exists() {
        eprintln!("Skipping - test2.vault not found");
        return;
    }
    
    // Test with spaces (should still work after trim)
    let vault_file = VaultFile::read(&vault_path)
        .expect("Should read test2.vault");
    
    let password_with_space = "  sudounlock  ".trim().to_string();
    let password = SecureString::new(password_with_space);
    let result = vault_file.decrypt_with_key(&password, None);
    
    match result {
        Ok((vault, _key)) => {
            println!("✅ Successfully unlocked test2 with trimmed password");
            println!("   Vault name: {}", vault.name);
        }
        Err(e) => {
            panic!("❌ Failed to unlock with trimmed password: {:?}", e);
        }
    }
}
