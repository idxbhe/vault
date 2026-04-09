# Vault Architecture

## Overview

Vault adalah aplikasi TUI (Terminal User Interface) untuk manajemen password dan secrets dengan enkripsi end-to-end. Menggunakan Rust dengan pola TEA (The Elm Architecture).

## Technology Stack

- **Language**: Rust
- **UI Framework**: Ratatui (TUI)
- **Architecture Pattern**: TEA (The Elm Architecture)
- **Encryption**: AES-256-GCM
- **Key Derivation**: Argon2id
- **Serialization**: JSON (Serde)

## Architecture Pattern - TEA

### The Elm Architecture (TEA)

```
┌─────────────────────────────────────┐
│         Application Loop             │
│                                       │
│  ┌──────────────────────────────┐   │
│  │  Model (AppState)            │   │
│  │  - Immutable state            │   │
│  │  - Contains UI state + vault  │   │
│  └──────────────────────────────┘   │
│                 ↑                     │
│                 │                     │
│  ┌──────────────────────────────┐   │
│  │  Update (Message Handler)     │   │
│  │  - Process messages           │   │
│  │  - Return new state + effects │   │
│  └──────────────────────────────┘   │
│                 ↑                     │
│                 │                     │
│  ┌──────────────────────────────┐   │
│  │  View (UI Rendering)         │   │
│  │  - Render state to terminal  │   │
│  └──────────────────────────────┘   │
│                 ↑                     │
│                 │                     │
│  ┌──────────────────────────────┐   │
│  │  Runtime (Effect Execution)  │   │
│  │  - Async I/O, crypto ops    │   │
│  │  - Return effects            │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
```

### Data Flow

1. **User Input** → Event (terminal input, mouse, timer)
2. **Message** → Enum describing what happened
3. **Update** → Handle message, modify state, return effects
4. **View** → Render current state to terminal
5. **Effect** → Async operation (I/O, crypto)
6. **Effect Result** → Message with result, loops back to Update

## Project Structure

```
vault/
├── src/
│   ├── main.rs                    # Entry point, main loop
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs               # AppState model
│   │   ├── message.rs             # Message enum
│   │   ├── update.rs              # Update handler
│   │   ├── effect.rs              # Effect definitions
│   │   └── runtime.rs             # Effect execution
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── vault_file.rs          # Encryption/decryption
│   │   ├── keyfile.rs             # Keyfile handling
│   │   └── vault.rs               # Vault data model
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── encryption.rs          # AES-256-GCM
│   │   ├── key_derivation.rs      # Argon2id
│   │   └── secure_string.rs       # Sensitive data handling
│   ├── input/
│   │   ├── mod.rs
│   │   ├── router.rs              # Input routing
│   │   └── handler.rs             # Input handler
│   └── ui/
│       ├── mod.rs
│       ├── app.rs                 # App wrapper
│       ├── components/
│       │   ├── input_buffer.rs    # Text input
│       │   ├── modal.rs           # Modal dialogs
│       │   └── table.rs           # Table widget
│       └── screens/
│           ├── login.rs           # Login screen
│           ├── main.rs            # Main vault view
│           └── form.rs            # Item form
├── tests/                         # Integration tests
├── Cargo.toml
└── docs/                          # Documentation
```

## Key Concepts

### 1. AppState (Model)

Represents complete application state - immutable during frame.

```rust
pub struct AppState {
    pub mode: AppMode,                    // Login, Unlocked, etc.
    pub vault_state: Option<VaultState>,  // Currently open vault
    pub login_state: LoginState,          // Login UI state
    pub form_state: Option<FormState>,    // Item edit form
    pub error: Option<String>,            // Error message
    pub loading_message: Option<String>,  // Loading indicator
    // ... more fields
}

pub struct VaultState {
    pub vault: Vault,                     // Item collection
    pub encryption_key: [u8; 32],         // Derived key (never disk)
    pub salt: [u8; 32],                   // For key derivation
    pub vault_path: PathBuf,              // File location
}
```

### 2. Message (Events)

```rust
pub enum Message {
    // Login
    LoginSelectNext,
    LoginSelectPrev,
    LoginSelectVault(usize),
    EnterPasswordMode,
    UnlockVault { password: SecureString, keyfile: Option<PathBuf> },

    // Core actions
    SaveVault,
    LockVault,
    ExportVault { format: ExportFormat, path: PathBuf },

    // Input/system
    InputChar(char),
    InputBackspace,
    InputSubmit,
    Tick,
    Quit,
}
```

### 3. Effects (Async Operations)

```rust
pub enum Effect {
    ReadVaultFile { path, password, keyfile },
    WriteVaultFile { path, vault, key, salt, has_keyfile },
    ExportVault { path, vault, encrypted, key, salt, has_keyfile },
    ReadConfig,
    WriteConfig,
    UpdateRegistry,
}
```

## Security Architecture

### Key Derivation

1. User enters password (SecureString)
2. **Argon2id** derives key from password + salt
   - Time cost: 2 iterations
   - Memory cost: 19 MiB
   - Parallelism: 1 thread
   - Delay: 1-3 seconds per unlock
3. Derived key never written to disk

### Encryption

1. Vault (JSON) serialized to bytes
2. **AES-256-GCM** encrypts vault bytes
   - Key: 256 bits (derived from password)
   - IV: 96 bits (random, stored in vault file)
   - AEAD mode: authenticated encryption
3. EncryptedPayload stores: `{ encrypted_data, iv, salt }`

### Secure String Handling

```rust
pub struct SecureString {
    bytes: Vec<u8>,  // Sensitive data
    // Zeroized on drop
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Clear sensitive data from memory
        volatile_memzero(&mut self.bytes);
    }
}
```

## Critical Bug Fix - Salt Regeneration

### Problem
Every save regenerated salt, breaking key derivation:
- Unlock: password + salt_A → KEY_A (works)
- Save: encrypt with KEY_A but NEW salt_B (bug!)
- Next unlock: password + salt_B → KEY_B ≠ KEY_A (fails!)

### Solution
1. Extract salt on unlock, store in VaultState
2. Pass stored salt to `new_with_key(vault, key, salt)`
3. Reuse same salt for all saves
4. Result: password + salt always derive same key

### Implementation Files
- `src/app/state.rs` - VaultState.salt field
- `src/storage/vault_file.rs` - new_with_key accepts salt
- `src/app/runtime.rs` - Extract and return salt
- `src/app/update.rs` - Pass salt in all WriteVaultFile calls
- `tests/test_salt_fix.rs` - Automated verification

## Workflow - Save/Load Cycle

### Save Flow (Auto-Save on Enter)

```
User creates item → FormSubmit
    ↓
1. Create item in memory
2. Mark vault dirty
3. Create WriteVaultFile effect (with stored salt)
    ↓
[Effect Runtime]
1. VaultFile::new_with_key(vault, key, salt)
2. Encrypt vault bytes with key + salt
3. Write encrypted payload to disk
4. Return VaultWritten effect
    ↓
Update handler
1. Clear dirty flag
2. Show "Item saved" notification
```

### Load Flow

```
User selects vault → ReadVaultFile effect
    ↓
[Effect Runtime]
1. Read VaultFile from disk
2. Extract salt from encrypted payload
3. Derive key from password + salt (1-3 sec)
4. Decrypt with derived key
5. Return VaultLoaded with (vault, key, salt)
    ↓
Update handler
1. Create VaultState with stored key and salt
2. Set mode to Unlocked
3. Display vault contents
```

## File Format

### VaultFile (JSON on disk)

```json
{
  "version": "1.0",
  "encrypted_payload": {
    "salt": [u8; 32],           // For key derivation
    "iv": [u8; 12],             // For AES-256-GCM
    "ciphertext": "base64...",  // Encrypted vault
    "tag": [u8; 16]             // Authentication tag
  }
}
```

### Vault (JSON, encrypted)

```json
{
  "items": [
    {
      "id": "uuid",
      "category": "Password|SeedPhrase|PrivateKey|Note",
      "title": "GitHub",
      "content": "secret123",
      "tags": ["work"],
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

## Testing Strategy

### Unit Tests
- Crypto operations (encryption/decryption)
- Key derivation
- Data model validation

### Integration Tests
- Full unlock flow with correct password
- Failure cases (wrong password, corrupted vault)
- Salt preservation across save/load
- Auto-save functionality

### Test Vaults
- `test_vault.vault` - password: "testpass123"
- `test2.vault` - password: "sudounlock"

## Performance Considerations

1. **Key Derivation**: Intentional 1-3 second delay (Argon2id)
   - Makes brute force attacks expensive
   - User sees loading spinner

2. **Encryption**: Negligible delay (AES-256-GCM hardware accelerated)
   - Happens during auto-save

3. **UI Rendering**: ~50ms per frame (ratatui)
   - Efficient terminal drawing
   - Only updates changed regions

## Future Improvements

1. Atomic writes (crash-safe saves)
2. Transaction log (undo/redo)
3. Master password + session timeout
4. Multi-vault support with quick switch
5. Encrypted export/import
6. Cloud sync with conflict resolution
7. Hardware security key integration

## References

- TEA Pattern: https://guide.elm-lang.org/architecture/
- AES-256-GCM: NIST SP 800-38D
- Argon2id: https://github.com/P-H-C/phc-winner-argon2
- Ratatui: https://docs.rs/ratatui/
