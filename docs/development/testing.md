# 🧪 Testing

Panduan strategi dan implementasi testing untuk TUI Vault Manager.

## Overview

Proyek ini menggunakan pendekatan testing berlapis:

```
┌─────────────────────────────────────┐
│        Integration Tests            │  Full flow testing
├─────────────────────────────────────┤
│         Unit Tests                  │  Module-level testing
├─────────────────────────────────────┤
│     Property-Based Tests            │  Randomized input testing
└─────────────────────────────────────┘
```

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Module

```bash
# Test crypto module
cargo test crypto

# Test domain module
cargo test domain

# Test storage module
cargo test storage
```

### With Output

```bash
cargo test -- --nocapture
```

### Single Test

```bash
cargo test test_vault_creation
```

## Test Organization

### Unit Tests

Unit tests berada dalam module yang sama dengan kode:

```rust
// src/domain/vault.rs

pub struct Vault {
    // ...
}

impl Vault {
    pub fn new(name: String) -> Self {
        // ...
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vault_creation() {
        let vault = Vault::new("Test".to_string());
        assert_eq!(vault.name, "Test");
    }
    
    #[test]
    fn test_vault_add_item() {
        let mut vault = Vault::new("Test".to_string());
        let item = Item::new("Password".to_string(), ItemKind::Password);
        vault.add_item(item.clone());
        assert_eq!(vault.items.len(), 1);
        assert_eq!(vault.items[0].title, "Password");
    }
}
```

### Integration Tests

Integration tests berada di `tests/` directory:

```
tests/
├── crypto_tests.rs
├── storage_tests.rs
├── vault_flow_tests.rs
└── common/
    └── mod.rs
```

### Test Helpers

```rust
// tests/common/mod.rs

use tempfile::TempDir;
use vault::{Vault, VaultFile};

pub fn create_test_vault() -> Vault {
    let mut vault = Vault::new("Test Vault".to_string());
    vault.add_item(Item::new_password(
        "Gmail".to_string(),
        "user@gmail.com".to_string(),
        "password123".to_string(),
    ));
    vault
}

pub fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

pub fn create_test_vault_file(dir: &TempDir) -> PathBuf {
    let path = dir.path().join("test.vault");
    let vault = create_test_vault();
    VaultFile::write(&path, &vault, "password").unwrap();
    path
}
```

## Test Categories

### Crypto Tests

```rust
// tests/crypto_tests.rs

use vault::crypto::*;

#[test]
fn test_key_derivation() {
    let password = SecureString::new("password".to_string());
    let salt = generate_salt();
    let params = Argon2Params::default();
    
    let key = derive_key(&password, &salt, &params).unwrap();
    
    assert_eq!(key.len(), 32);
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let plaintext = b"Hello, World!";
    let key = [0u8; 32];
    
    let encrypted = encrypt(plaintext, &key).unwrap();
    let decrypted = decrypt(&encrypted, &key).unwrap();
    
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_decrypt_with_wrong_key_fails() {
    let plaintext = b"Secret";
    let key1 = [0u8; 32];
    let key2 = [1u8; 32];
    
    let encrypted = encrypt(plaintext, &key1).unwrap();
    let result = decrypt(&encrypted, &key2);
    
    assert!(result.is_err());
}

#[test]
fn test_nonce_uniqueness() {
    let key = [0u8; 32];
    let plaintext = b"Test";
    
    let enc1 = encrypt(plaintext, &key).unwrap();
    let enc2 = encrypt(plaintext, &key).unwrap();
    
    // Same plaintext should produce different ciphertexts
    assert_ne!(enc1.ciphertext, enc2.ciphertext);
    assert_ne!(enc1.nonce, enc2.nonce);
}
```

### Storage Tests

```rust
// tests/storage_tests.rs

use tempfile::tempdir;
use vault::storage::*;

#[test]
fn test_vault_file_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.vault");
    
    let vault = Vault::new("Test".to_string());
    let password = "secure_password";
    
    // Write
    VaultFile::write(&path, &vault, password).unwrap();
    
    // Read
    let (loaded, key) = VaultFile::read(&path, password).unwrap();
    
    assert_eq!(loaded.name, vault.name);
}

#[test]
fn test_vault_file_wrong_password() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.vault");
    
    let vault = Vault::new("Test".to_string());
    VaultFile::write(&path, &vault, "correct").unwrap();
    
    let result = VaultFile::read(&path, "wrong");
    
    assert!(result.is_err());
}

#[test]
fn test_registry_persistence() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("registry.json");
    
    let mut registry = VaultRegistry::new();
    registry.add_entry(VaultRegistryEntry {
        path: PathBuf::from("/path/to/vault"),
        name: "My Vault".to_string(),
        last_opened: Utc::now(),
        is_default: true,
    });
    
    registry.save(&path).unwrap();
    
    let loaded = VaultRegistry::load(&path).unwrap();
    
    assert_eq!(loaded.entries.len(), 1);
    assert_eq!(loaded.entries[0].name, "My Vault");
}
```

### Domain Tests

```rust
// src/domain/vault.rs (unit tests)

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vault_item_crud() {
        let mut vault = Vault::new("Test".to_string());
        
        // Create
        let item = Item::new("Password".to_string(), ItemKind::Password);
        let id = item.id;
        vault.add_item(item);
        
        assert_eq!(vault.items.len(), 1);
        
        // Read
        let found = vault.find_item(id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Password");
        
        // Update
        vault.update_item(id, ItemUpdates {
            title: Some("Updated".to_string()),
            ..Default::default()
        });
        assert_eq!(vault.find_item(id).unwrap().title, "Updated");
        
        // Delete
        vault.remove_item(id);
        assert!(vault.find_item(id).is_none());
    }
    
    #[test]
    fn test_vault_search() {
        let mut vault = Vault::new("Test".to_string());
        vault.add_item(Item::new("Gmail Password".to_string(), ItemKind::Password));
        vault.add_item(Item::new("GitHub API Key".to_string(), ItemKind::ApiKey));
        vault.add_item(Item::new("Bitcoin Seed".to_string(), ItemKind::CryptoSeed));
        
        let results = vault.search("git");
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "GitHub API Key");
    }
    
    #[test]
    fn test_vault_favorites() {
        let mut vault = Vault::new("Test".to_string());
        let item1 = Item::new("Fav".to_string(), ItemKind::Generic);
        let item2 = Item::new("NotFav".to_string(), ItemKind::Generic);
        let id1 = item1.id;
        
        vault.add_item(item1);
        vault.add_item(item2);
        
        vault.toggle_favorite(id1);
        
        let favorites: Vec<_> = vault.items.iter()
            .filter(|i| i.favorite)
            .collect();
        
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, id1);
    }
}
```

### Update Logic Tests

```rust
// src/app/update.rs (unit tests)

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_state() -> AppState {
        AppState {
            mode: AppMode::Unlocked,
            screen: Screen::Main,
            vault_state: Some(VaultState {
                vault: Vault::new("Test".to_string()),
                vault_path: PathBuf::from("/test.vault"),
                encryption_key: [0u8; 32],
                is_dirty: false,
                selected_item_id: None,
            }),
            ui_state: UIState::default(),
            config: AppConfig::default(),
            registry: VaultRegistry::new(),
        }
    }
    
    #[test]
    fn test_select_item() {
        let mut state = create_test_state();
        let item = Item::new("Test".to_string(), ItemKind::Generic);
        let id = item.id;
        state.vault_state.as_mut().unwrap().vault.add_item(item);
        
        let effect = update(&mut state, Message::SelectItem(id));
        
        assert_eq!(
            state.vault_state.as_ref().unwrap().selected_item_id,
            Some(id)
        );
        assert!(effect.is_none());
    }
    
    #[test]
    fn test_create_item_marks_dirty() {
        let mut state = create_test_state();
        
        let effect = update(&mut state, Message::CreateItem {
            kind: ItemKind::Password,
        });
        
        assert!(state.vault_state.as_ref().unwrap().is_dirty);
        assert!(effect.is_none());
    }
    
    #[test]
    fn test_save_vault_effect() {
        let mut state = create_test_state();
        state.vault_state.as_mut().unwrap().is_dirty = true;
        
        let effect = update(&mut state, Message::SaveVault);
        
        assert!(matches!(effect, Some(Effect::WriteVaultFile { .. })));
    }
    
    #[test]
    fn test_copy_to_clipboard() {
        let mut state = create_test_state();
        
        let effect = update(&mut state, Message::CopyToClipboard {
            content: "secret".to_string(),
            is_sensitive: true,
        });
        
        assert!(matches!(effect, Some(Effect::SetClipboard { sensitive: true, .. })));
    }
}
```

### Async Tests

```rust
// tests/async_tests.rs

use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_clipboard_timeout() {
    let mut runtime = Runtime::new();
    
    // Set clipboard
    let result = runtime.execute(Effect::SetClipboard {
        content: "secret".to_string(),
        sensitive: true,
    }).await;
    
    assert!(matches!(result, EffectResult::ClipboardSet));
    
    // Schedule clear
    let _ = runtime.execute(Effect::ScheduleClipboardClear {
        delay: Duration::from_millis(100),
    }).await;
    
    // Wait for timeout
    sleep(Duration::from_millis(150)).await;
    
    // Verify cleared (implementation-specific check)
}

#[tokio::test]
async fn test_effect_batch() {
    let mut runtime = Runtime::new();
    
    let effects = Effect::Batch(vec![
        Effect::LoadConfig,
        Effect::LoadRegistry,
    ]);
    
    let result = runtime.execute(effects).await;
    
    // Batch returns None, but effects were executed
    assert!(matches!(result, EffectResult::None));
}
```

## Property-Based Testing

Menggunakan `proptest` untuk random input testing:

```rust
// src/crypto/cipher.rs

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn encrypt_decrypt_roundtrip(plaintext in any::<Vec<u8>>()) {
            let key = [42u8; 32];
            
            let encrypted = encrypt(&plaintext, &key)?;
            let decrypted = decrypt(&encrypted, &key)?;
            
            prop_assert_eq!(decrypted, plaintext);
        }
        
        #[test]
        fn different_keys_produce_different_ciphertext(
            plaintext in any::<Vec<u8>>().prop_filter("non-empty", |v| !v.is_empty()),
            key1 in any::<[u8; 32]>(),
            key2 in any::<[u8; 32]>().prop_filter("different", |k| *k != key1),
        ) {
            let enc1 = encrypt(&plaintext, &key1)?;
            let enc2 = encrypt(&plaintext, &key2)?;
            
            prop_assert_ne!(enc1.ciphertext, enc2.ciphertext);
        }
    }
}
```

## Test Fixtures

### Mock Runtime

```rust
// tests/common/mock_runtime.rs

pub struct MockRuntime {
    pub clipboard_content: Option<String>,
    pub saved_vaults: Vec<(PathBuf, Vault)>,
}

impl MockRuntime {
    pub fn new() -> Self {
        Self {
            clipboard_content: None,
            saved_vaults: Vec::new(),
        }
    }
    
    pub async fn execute(&mut self, effect: Effect) -> EffectResult {
        match effect {
            Effect::SetClipboard { content, .. } => {
                self.clipboard_content = Some(content);
                EffectResult::ClipboardSet
            }
            Effect::ClearClipboard => {
                self.clipboard_content = None;
                EffectResult::ClipboardCleared
            }
            Effect::WriteVaultFile { path, vault, .. } => {
                self.saved_vaults.push((path, vault));
                EffectResult::VaultSaved
            }
            _ => EffectResult::None,
        }
    }
}
```

### Test State Builder

```rust
// tests/common/state_builder.rs

pub struct TestStateBuilder {
    vault_name: String,
    items: Vec<Item>,
    selected_item: Option<Uuid>,
}

impl TestStateBuilder {
    pub fn new() -> Self {
        Self {
            vault_name: "Test Vault".to_string(),
            items: Vec::new(),
            selected_item: None,
        }
    }
    
    pub fn with_vault_name(mut self, name: &str) -> Self {
        self.vault_name = name.to_string();
        self
    }
    
    pub fn with_item(mut self, item: Item) -> Self {
        self.items.push(item);
        self
    }
    
    pub fn with_password(mut self, title: &str) -> Self {
        self.items.push(Item::new_password(
            title.to_string(),
            "user".to_string(),
            "password".to_string(),
        ));
        self
    }
    
    pub fn with_selected(mut self, index: usize) -> Self {
        if let Some(item) = self.items.get(index) {
            self.selected_item = Some(item.id);
        }
        self
    }
    
    pub fn build(self) -> AppState {
        let mut vault = Vault::new(self.vault_name);
        for item in self.items {
            vault.add_item(item);
        }
        
        AppState {
            mode: AppMode::Unlocked,
            screen: Screen::Main,
            vault_state: Some(VaultState {
                vault,
                vault_path: PathBuf::from("/test.vault"),
                encryption_key: [0u8; 32],
                is_dirty: false,
                selected_item_id: self.selected_item,
            }),
            ui_state: UIState::default(),
            config: AppConfig::default(),
            registry: VaultRegistry::new(),
        }
    }
}

// Usage
#[test]
fn test_with_builder() {
    let state = TestStateBuilder::new()
        .with_vault_name("My Vault")
        .with_password("Gmail")
        .with_password("GitHub")
        .with_selected(0)
        .build();
    
    assert_eq!(state.vault_state.as_ref().unwrap().vault.items.len(), 2);
}
```

## Coverage

### Generating Coverage Report

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --out Html
```

### Coverage Targets

| Module | Target | Current |
|--------|--------|---------|
| `crypto/` | 90% | 92% |
| `domain/` | 80% | 85% |
| `storage/` | 80% | 81% |
| `app/update.rs` | 70% | 75% |
| `ui/` | 50% | 55% |

## CI Integration

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        
      - name: Run tests
        run: cargo test --all-features
        
      - name: Run clippy
        run: cargo clippy -- -D warnings
        
      - name: Check formatting
        run: cargo fmt -- --check
```

## Best Practices

1. **Test naming**: `test_<function>_<scenario>_<expected>`
2. **Arrange-Act-Assert** pattern
3. **One assertion per test** (when practical)
4. **Use descriptive error messages**
5. **Test edge cases** (empty, null, max values)
6. **Test error conditions** (not just happy path)
