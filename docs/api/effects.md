# ⚡ Effects

Referensi API untuk sistem side effects.

## Overview

Effects merepresentasikan side effects yang perlu dieksekusi di luar update function. Effect handler (runtime) mengeksekusi effects dan mengembalikan EffectResult.

## Effect Enum

```rust
#[derive(Debug, Clone)]
pub enum Effect {
    None,
    Batch(Vec<Effect>),
    
    // File I/O
    ReadVaultFile {
        path: PathBuf,
        password: SecureString,
        keyfile: Option<Vec<u8>>,
    },
    WriteVaultFile {
        path: PathBuf,
        vault: Vault,
        key: [u8; 32],
    },
    CreateVaultFile {
        path: PathBuf,
        vault: Vault,
        password: String,
    },
    ExportVault {
        path: PathBuf,
        vault: Vault,
        encrypted: bool,
        key: Option<[u8; 32]>,
    },
    
    // Registry
    LoadRegistry,
    SaveRegistry { registry: VaultRegistry },
    
    // Config
    LoadConfig,
    SaveConfig { config: AppConfig },
    
    // Clipboard
    SetClipboard {
        content: String,
        sensitive: bool,
    },
    ClearClipboard,
    ScheduleClipboardClear {
        delay: Duration,
    },
    
    // Timer
    ScheduleAutoLock {
        delay: Duration,
    },
    CancelAutoLock,
    
    // System
    Exit,
}
```

## EffectResult Enum

```rust
#[derive(Debug)]
pub enum EffectResult {
    None,
    
    // Vault
    VaultLoaded {
        vault: Vault,
        path: PathBuf,
        key: [u8; 32],
    },
    VaultCreated {
        vault: Vault,
        path: PathBuf,
        key: [u8; 32],
    },
    VaultSaved,
    ExportCompleted {
        path: PathBuf,
    },
    
    // Registry/Config
    RegistryLoaded(VaultRegistry),
    ConfigLoaded(AppConfig),
    
    // Clipboard
    ClipboardSet,
    ClipboardCleared,
    
    // Error
    Error(String),
}
```

## Effect Categories

### File I/O Effects

#### `ReadVaultFile`

Membaca dan mendekripsi vault file.

```rust
Effect::ReadVaultFile {
    path: PathBuf::from("~/.local/share/vault/my.vault"),
    password: SecureString::new("password".to_string()),
    keyfile: None,
}
```

**Returns:** `EffectResult::VaultLoaded { vault, path, key }`

#### `WriteVaultFile`

Menyimpan vault ke file dengan enkripsi.

```rust
Effect::WriteVaultFile {
    path: vault_path.clone(),
    vault: vault.clone(),
    key: encryption_key,
}
```

**Returns:** `EffectResult::VaultSaved`

#### `CreateVaultFile`

Membuat vault file baru.

```rust
Effect::CreateVaultFile {
    path: PathBuf::from("~/.local/share/vault/new.vault"),
    vault: Vault::new("My Vault".to_string()),
    password: "secure_password".to_string(),
}
```

**Returns:** `EffectResult::VaultCreated { vault, path, key }`

#### `ExportVault`

Export vault ke file eksternal.

```rust
Effect::ExportVault {
    path: PathBuf::from("export.json"),
    vault: vault.clone(),
    encrypted: false,
    key: None,
}
```

**Returns:** `EffectResult::ExportCompleted { path }`

### Registry Effects

#### `LoadRegistry`

Memuat vault registry dari disk.

```rust
Effect::LoadRegistry
```

**Returns:** `EffectResult::RegistryLoaded(registry)`

#### `SaveRegistry`

Menyimpan vault registry.

```rust
Effect::SaveRegistry {
    registry: registry.clone(),
}
```

**Returns:** `EffectResult::None`

### Config Effects

#### `LoadConfig`

Memuat konfigurasi aplikasi.

```rust
Effect::LoadConfig
```

**Returns:** `EffectResult::ConfigLoaded(config)`

#### `SaveConfig`

Menyimpan konfigurasi.

```rust
Effect::SaveConfig {
    config: config.clone(),
}
```

**Returns:** `EffectResult::None`

### Clipboard Effects

#### `SetClipboard`

Menyalin konten ke clipboard.

```rust
Effect::SetClipboard {
    content: "secret value".to_string(),
    sensitive: true,  // Will auto-clear
}
```

**Returns:** `EffectResult::ClipboardSet`

#### `ClearClipboard`

Membersihkan clipboard.

```rust
Effect::ClearClipboard
```

**Returns:** `EffectResult::ClipboardCleared`

#### `ScheduleClipboardClear`

Menjadwalkan pembersihan clipboard.

```rust
Effect::ScheduleClipboardClear {
    delay: Duration::from_secs(30),
}
```

**Returns:** `EffectResult::None` (immediate), triggers `ClearClipboard` later

### Timer Effects

#### `ScheduleAutoLock`

Menjadwalkan auto-lock vault.

```rust
Effect::ScheduleAutoLock {
    delay: Duration::from_secs(300),  // 5 minutes
}
```

#### `CancelAutoLock`

Membatalkan timer auto-lock.

```rust
Effect::CancelAutoLock
```

### System Effects

#### `Exit`

Keluar dari aplikasi.

```rust
Effect::Exit
```

### Composite Effects

#### `None`

Tidak ada effect.

```rust
Effect::None
```

#### `Batch`

Menjalankan multiple effects.

```rust
Effect::Batch(vec![
    Effect::WriteVaultFile { ... },
    Effect::SaveRegistry { ... },
])
```

## Runtime Implementation

### Runtime Struct

```rust
pub struct Runtime {
    clipboard: Option<Clipboard>,
    clipboard_timer: Option<tokio::time::Instant>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            clipboard: Clipboard::new().ok(),
            clipboard_timer: None,
        }
    }
    
    pub async fn execute(&mut self, effect: Effect) -> EffectResult {
        match effect {
            Effect::None => EffectResult::None,
            Effect::Batch(effects) => {
                for e in effects {
                    let _ = self.execute(e).await;
                }
                EffectResult::None
            }
            Effect::ReadVaultFile { path, password, keyfile } => {
                self.read_vault_file(&path, &password, keyfile.as_deref()).await
            }
            // ... other handlers
        }
    }
}
```

### Error Handling

```rust
async fn read_vault_file(
    &self,
    path: &Path,
    password: &SecureString,
    keyfile: Option<&[u8]>,
) -> EffectResult {
    match VaultFile::read(path) {
        Ok(vault_file) => {
            match vault_file.decrypt(password, keyfile) {
                Ok((vault, key)) => EffectResult::VaultLoaded {
                    vault,
                    path: path.to_path_buf(),
                    key,
                },
                Err(e) => EffectResult::Error(
                    format!("Failed to decrypt: {}", e)
                ),
            }
        }
        Err(e) => EffectResult::Error(
            format!("Failed to read vault: {}", e)
        ),
    }
}
```

## Effect Flow in Main Loop

```rust
// main.rs
loop {
    // 1. Read events
    if let Some(event) = read_event()? {
        // 2. Route to message
        let message = route_event(&app.state, event);
        
        // 3. Update state, get effect
        let effect = update(&mut app.state, message);
        
        // 4. Execute effect
        if let Some(effect) = effect {
            let result = runtime.execute(effect).await;
            
            // 5. Handle result
            handle_effect_result(&mut app, result);
        }
    }
    
    // 6. Render
    terminal.draw(|f| ui::draw(f, &app.state))?;
}
```

## Handling EffectResult

```rust
fn handle_effect_result(app: &mut App, result: EffectResult) {
    match result {
        EffectResult::VaultLoaded { vault, path, key } => {
            app.handle_vault_loaded(vault, path, key);
        }
        EffectResult::VaultCreated { vault, path, key } => {
            app.handle_vault_created(vault, path, key);
        }
        EffectResult::VaultSaved => {
            app.show_notification("Vault saved", NotificationLevel::Success);
            app.state.vault_state.as_mut()
                .map(|vs| vs.is_dirty = false);
        }
        EffectResult::ExportCompleted { path } => {
            app.show_notification(
                &format!("Exported to {}", path.display()),
                NotificationLevel::Success
            );
        }
        EffectResult::Error(msg) => {
            app.show_notification(&msg, NotificationLevel::Error);
        }
        _ => {}
    }
}
```

## Testing Effects

```rust
#[tokio::test]
async fn test_vault_creation() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.vault");
    
    let effect = Effect::CreateVaultFile {
        path: path.clone(),
        vault: Vault::new("Test".to_string()),
        password: "password".to_string(),
    };
    
    let mut runtime = Runtime::new();
    let result = runtime.execute(effect).await;
    
    match result {
        EffectResult::VaultCreated { vault, path: p, key } => {
            assert_eq!(vault.name, "Test");
            assert_eq!(p, path);
            assert_eq!(key.len(), 32);
        }
        _ => panic!("Expected VaultCreated"),
    }
}
```

## Best Practices

1. **Effects are descriptions, not executions**
   - update() returns Effect, doesn't execute
   - Runtime executes effects

2. **Keep effects minimal**
   - One effect per logical operation
   - Use Batch for related operations

3. **Handle all results**
   - Always handle EffectResult::Error
   - Show user feedback for operations

4. **Async where needed**
   - File I/O should be async
   - Clipboard operations can be sync
