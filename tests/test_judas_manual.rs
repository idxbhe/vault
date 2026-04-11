#[test]
#[ignore]
fn test_judas_vault_decrypt() {
    use std::path::PathBuf;
    use vault::crypto::SecureString;
    use vault::storage::VaultFile;

    let path = PathBuf::from("/home/idxbhe/.local/share/vault/judas.vault");
    let password = SecureString::new("imrich".to_string());

    // Read vault
    let vault_file = VaultFile::read(&path).expect("Failed to read judas.vault");
    println!("✓ Read success: {}", vault_file.header.vault_name);

    // Try decrypt
    let result = vault_file.decrypt_with_key(&password, None);
    match result {
        Ok((vault, _key)) => {
            println!("✓ Decrypt SUCCESS! Items: {}", vault.items.len());
            assert_eq!(vault.name, "judas");
        }
        Err(e) => {
            panic!("✗ Decrypt FAILED: {:?}", e);
        }
    }
}
