#[test]
fn test_salt_preservation() {
    use std::path::PathBuf;
    use vault::crypto::SecureString;
    use vault::domain::Vault;
    use vault::storage::VaultFile;

    let test_path = PathBuf::from("/tmp/test_salt_fix.vault");

    // Clean up if exists
    let _ = std::fs::remove_file(&test_path);

    // Step 1: Create vault with password
    let vault1 = Vault::new("Test Salt Fix");
    let password = SecureString::new("testpass123".to_string());

    // Create and save
    let vault_file1 = VaultFile::new(&vault1, &password, None).expect("Failed to create vault");
    let salt1 = vault_file1.encrypted_payload.salt;
    vault_file1
        .write(&test_path)
        .expect("Failed to write vault");

    println!("✓ Created vault with salt: {:?}", &salt1[..8]);

    // Step 2: Read back and get key + salt
    let vault_file2 = VaultFile::read(&test_path).expect("Failed to read vault");
    let salt2 = vault_file2.encrypted_payload.salt;
    let (vault2, key) = vault_file2
        .decrypt_with_key(&password, None)
        .expect("Failed to decrypt");

    println!("✓ Read vault with salt: {:?}", &salt2[..8]);
    assert_eq!(salt1, salt2, "Salt should be unchanged after write");

    // Step 3: Simulate save (re-encrypt with same key and salt)
    let vault_file3 = VaultFile::new_with_key(vault2.clone(), &key, &salt2, false)
        .expect("Failed to create with key");
    let salt3 = vault_file3.encrypted_payload.salt;
    vault_file3
        .write(&test_path)
        .expect("Failed to write updated vault");

    println!("✓ Saved with salt: {:?}", &salt3[..8]);
    assert_eq!(salt2, salt3, "Salt should be preserved on save");

    // Step 4: Verify can still unlock with original password
    let vault_file4 = VaultFile::read(&test_path).expect("Failed to read after save");
    let salt4 = vault_file4.encrypted_payload.salt;
    let result = vault_file4.decrypt_with_key(&password, None);

    println!("✓ Final salt: {:?}", &salt4[..8]);
    assert!(result.is_ok(), "Should decrypt after save");
    assert_eq!(salt3, salt4, "Salt should still match");

    // Clean up
    let _ = std::fs::remove_file(&test_path);

    println!("✅ TEST PASSED: Salt preserved across save/load cycle!");
}
