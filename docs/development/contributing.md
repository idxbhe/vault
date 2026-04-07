# 🤝 Contributing

Panduan untuk berkontribusi pada proyek TUI Vault Manager.

## Getting Started

### Prerequisites

- Rust 1.75+ (edition 2024)
- Cargo
- Git
- Nerd Font (untuk ikon)

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/yourusername/vault.git
cd vault

# Build project
cargo build

# Run tests
cargo test

# Run application
cargo run
```

## Code Style

### Formatting

Gunakan `rustfmt` dengan konfigurasi default:

```bash
cargo fmt
```

### Linting

Gunakan `clippy` untuk linting:

```bash
cargo clippy -- -W clippy::all
```

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Structs | PascalCase | `VaultState` |
| Enums | PascalCase | `ItemKind` |
| Functions | snake_case | `render_item_detail` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_TIMEOUT` |
| Modules | snake_case | `vault_list` |
| Type parameters | Single uppercase | `T`, `B` |

### Import Order

```rust
// 1. Standard library
use std::collections::HashMap;
use std::path::PathBuf;

// 2. External crates
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};

// 3. Crate modules
use crate::domain::Vault;
use crate::ui::theme::ThemePalette;

// 4. Super/self
use super::Widget;
```

## Architecture Guidelines

### TEA Pattern

Semua perubahan state harus melalui message system:

```rust
// ❌ Don't
state.vault_state.as_mut().unwrap().is_dirty = true;

// ✅ Do
fn update(state: &mut AppState, msg: Message) -> Option<Effect> {
    match msg {
        Message::MarkDirty => {
            if let Some(vs) = state.vault_state.as_mut() {
                vs.is_dirty = true;
            }
            None
        }
        // ...
    }
}
```

### Pure Functions

Update logic harus pure (no side effects):

```rust
// ❌ Don't
fn update(state: &mut AppState, msg: Message) {
    match msg {
        Message::SaveVault => {
            // Direct file I/O in update
            std::fs::write("vault.dat", &data).unwrap();
        }
    }
}

// ✅ Do
fn update(state: &mut AppState, msg: Message) -> Option<Effect> {
    match msg {
        Message::SaveVault => {
            Some(Effect::WriteVaultFile {
                path: state.vault_path.clone(),
                vault: state.vault.clone(),
                key: state.encryption_key,
            })
        }
    }
}
```

### Widget Composition

Widgets harus composable dan reusable:

```rust
// ✅ Good - composable widget
pub struct VaultList<'a> {
    items: &'a [Item],
    selected: Option<usize>,
}

impl<'a> VaultList<'a> {
    pub fn new(items: &'a [Item]) -> Self {
        Self { items, selected: None }
    }
    
    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }
}

// Usage
VaultList::new(&items)
    .selected(Some(0))
    .render(area, buf);
```

## Testing

### Unit Tests

Setiap module harus memiliki unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vault_creation() {
        let vault = Vault::new("Test".to_string());
        assert_eq!(vault.name, "Test");
        assert!(vault.items.is_empty());
    }
}
```

### Integration Tests

Tests untuk flows lengkap di `tests/`:

```rust
// tests/vault_flow_tests.rs
#[tokio::test]
async fn test_create_unlock_save_flow() {
    // Setup
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.vault");
    
    // Create vault
    let vault = Vault::new("Test".to_string());
    let password = "password";
    VaultFile::write(&path, &vault, password).unwrap();
    
    // Unlock vault
    let (loaded_vault, key) = VaultFile::read(&path, password).unwrap();
    assert_eq!(loaded_vault.name, "Test");
    
    // Modify and save
    // ...
}
```

### Test Coverage

Minimal coverage requirements:
- `crypto/`: 90%
- `domain/`: 80%
- `storage/`: 80%
- `app/update.rs`: 70%

## Pull Request Process

### Before Submitting

1. **Format code**: `cargo fmt`
2. **Run clippy**: `cargo clippy`
3. **Run tests**: `cargo test`
4. **Update docs** jika diperlukan
5. **Write/update tests** untuk perubahan

### PR Title Format

```
<type>(<scope>): <description>

Types:
- feat: New feature
- fix: Bug fix
- docs: Documentation
- style: Formatting
- refactor: Code restructure
- test: Adding tests
- chore: Maintenance

Examples:
- feat(vault): add export to encrypted JSON
- fix(clipboard): clear timeout not working
- docs(api): add Effect documentation
```

### PR Description Template

```markdown
## Description
[Describe your changes]

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review performed
- [ ] Documentation updated
- [ ] No new warnings
```

## Branch Strategy

```
main
├── develop
│   ├── feature/export-csv
│   ├── feature/totp-support
│   └── fix/clipboard-timeout
└── release/v0.2.0
```

- `main`: Production-ready code
- `develop`: Integration branch
- `feature/*`: New features
- `fix/*`: Bug fixes
- `release/*`: Release preparation

## Security Guidelines

### Sensitive Data

```rust
// ❌ Don't
let password = String::from("secret");
println!("Password: {}", password);  // Logged!

// ✅ Do
let password = SecureString::new("secret".to_string());
// SecureString implements Zeroize, auto-wipes on drop
// No Debug or Display implementation
```

### Cryptographic Operations

- Gunakan crate yang established (`aes-gcm`, `argon2`)
- Jangan implement crypto sendiri
- Review nonce/salt generation
- Test edge cases

### Memory Safety

```rust
// Always zeroize sensitive data
#[derive(Zeroize, ZeroizeOnDrop)]
struct SecretData {
    key: [u8; 32],
    password: String,
}
```

## Documentation

### Code Comments

```rust
/// Encrypts the vault with AES-256-GCM.
///
/// # Arguments
///
/// * `vault` - The vault to encrypt
/// * `key` - 32-byte encryption key
///
/// # Returns
///
/// Encrypted payload with nonce and ciphertext
///
/// # Errors
///
/// Returns `CryptoError` if encryption fails
pub fn encrypt_vault(vault: &Vault, key: &[u8; 32]) -> Result<EncryptedPayload, CryptoError>
```

### Module Documentation

```rust
//! # Crypto Module
//!
//! This module provides cryptographic operations for vault encryption.
//!
//! ## Features
//!
//! - AES-256-GCM encryption
//! - Argon2id key derivation
//! - Secure string handling
//!
//! ## Example
//!
//! ```rust
//! use vault::crypto::{derive_key, encrypt};
//!
//! let key = derive_key(password, salt)?;
//! let encrypted = encrypt(data, &key)?;
//! ```
```

## Release Process

1. Create release branch: `git checkout -b release/v0.2.0`
2. Update version in `Cargo.toml`
3. Update `CHANGELOG.md`
4. Run full test suite
5. Create PR to main
6. After merge, tag release: `git tag v0.2.0`
7. Push tags: `git push --tags`

## Getting Help

- **Issues**: GitHub Issues untuk bugs/features
- **Discussions**: GitHub Discussions untuk questions
- **Security**: Lihat `SECURITY.md` untuk vulnerability reporting

## License

Kontribusi dilisensikan under MIT License.
