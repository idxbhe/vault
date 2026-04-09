# 🏗️ Arsitektur Sistem

Dokumentasi arsitektur komprehensif untuk Vault TUI Manager.

## Overview

Vault dibangun menggunakan **The Elm Architecture (TEA)** pattern yang diadaptasi untuk Rust. Arsitektur ini memberikan:

- **Predictable state management**: State hanya berubah melalui messages
- **Separation of concerns**: UI, logic, dan side effects terpisah
- **Testability**: Pure functions mudah di-test
- **Maintainability**: Alur data yang jelas dan terstruktur

## Diagram Arsitektur

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              MAIN LOOP                                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                                                                   │   │
│  │   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │   │
│  │   │  Input  │───▶│ Router  │───▶│ Message │───▶│ Update  │      │   │
│  │   │ Events  │    │         │    │         │    │         │      │   │
│  │   └─────────┘    └─────────┘    └─────────┘    └────┬────┘      │   │
│  │        ▲                                            │            │   │
│  │        │                                            ▼            │   │
│  │   ┌────┴────┐                              ┌─────────────┐       │   │
│  │   │Terminal │                              │   Effect    │       │   │
│  │   │(ratatui)│◀─────────────────────────────│             │       │   │
│  │   └────┬────┘                              └──────┬──────┘       │   │
│  │        │                                          │              │   │
│  │        ▼                                          ▼              │   │
│  │   ┌─────────┐    ┌─────────┐              ┌─────────────┐       │   │
│  │   │  View   │◀───│  State  │◀─────────────│   Runtime   │       │   │
│  │   │         │    │         │              │ (Executor)  │       │   │
│  │   └─────────┘    └─────────┘              └─────────────┘       │   │
│  │                                                                   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Komponen Utama

### 1. State (Model)

State menyimpan semua data aplikasi dalam struktur immutable.

```rust
pub struct AppState {
    // Core state
    pub mode: AppMode,
    pub screen: Screen,
    pub vault_state: Option<VaultState>,
    
    // UI state
    pub ui_state: UIState,
    pub login_screen: LoginScreen,
    
    // Configuration
    pub config: AppConfig,
    pub registry: VaultRegistry,
    
    // System
    pub clipboard_state: ClipboardState,
    pub should_quit: bool,
}
```

**Karakteristik:**
- Single source of truth
- Immutable updates (clone + modify)
- Serializable untuk persistence

### 2. Message

Messages adalah satu-satunya cara untuk mengubah state.

```rust
pub enum Message {
    // Navigation
    Navigate(Screen),
    FocusPane(Pane),
    SelectItem(Uuid),
    
    // Actions
    CreateItem { kind: ItemKind },
    UpdateItem { id: Uuid, updates: ItemUpdates },
    DeleteItem(Uuid),
    
    // Effects
    SaveVault,
    ExportVault { format: ExportFormat, path: PathBuf },
    
    // Input
    InputChar(char),
    InputSubmit,
    InputCancel,
    
    // System
    Tick,
    Quit,
}
```

**Karakteristik:**
- Enum dengan semua possible actions
- Carries data needed for update
- Easy to log/debug

### 3. Update Function

Pure function yang mengambil state + message, menghasilkan state baru + effects.

```rust
pub fn update(state: &mut AppState, msg: Message) -> Effect {
    match msg {
        Message::CreateItem { kind } => {
            if let Some(ref mut vs) = state.vault_state {
                let item = Item::new("New Item", kind, kind.default_content());
                vs.vault.add_item(item);
                vs.mark_dirty();
            }
            Effect::none()
        }
        
        Message::SaveVault => {
            if let Some(ref vs) = state.vault_state {
                Effect::WriteVaultFile {
                    path: vs.vault_path.clone(),
                    vault: vs.vault.clone(),
                    key: vs.encryption_key,
                    salt: vs.salt,
                    has_keyfile: vs.has_keyfile,
                }
            } else {
                Effect::none()
            }
        }
        
        // ... more handlers
    }
}
```

**Karakteristik:**
- Pattern matching exhaustive
- Returns Effect for side effects
- Modifies state in-place (Rust optimization)

### 4. Effect

Side effects yang perlu dijalankan oleh runtime.

```rust
pub enum Effect {
    None,
    Batch(Vec<Effect>),
    
    // File I/O
    ReadVaultFile { path: PathBuf, password: SecureString, keyfile: Option<Vec<u8>> },
    WriteVaultFile {
        path: PathBuf,
        vault: Vault,
        key: [u8; 32],
        salt: [u8; 32],
        has_keyfile: bool,
    },
    ExportVault {
        path: PathBuf,
        vault: Vault,
        encrypted: bool,
        key: Option<[u8; 32]>,
        salt: Option<[u8; 32]>,
        has_keyfile: bool,
    },
    
    // Clipboard
    SetClipboard { content: String, is_sensitive: bool },
    ClearClipboard,
    ScheduleClipboardClear { delay: Duration },
    
    // Timer
    ScheduleAutoLock { delay: Duration },
    CancelAutoLock,
    
    // System
    Exit,
}
```

**Karakteristik:**
- Describes what to do, not how
- Composable with Batch
- Executed by Runtime

### 5. Runtime

Executes effects dan mengembalikan results.

```rust
pub struct Runtime {
    message_tx: Sender<Message>,
    clipboard_clear_at: Option<Instant>,
    auto_lock_at: Option<Instant>,
}

impl Runtime {
    pub fn execute(&mut self, effect: Effect) -> EffectResult {
        match effect {
            Effect::ReadVaultFile { path, password, keyfile } => {
                match read_vault_file(&path, &password, keyfile.as_deref()) {
                    Ok((vault, key, salt, has_keyfile)) => EffectResult::VaultLoaded {
                        vault,
                        path,
                        key,
                        salt,
                        has_keyfile,
                    },
                    Err(e) => EffectResult::Error(e),
                }
            }
            // ... more handlers
        }
    }
}
```

### 6. View

Renders state ke terminal menggunakan ratatui.

```rust
pub fn render(frame: &mut Frame, app: &mut App) {
    let state = app.state();
    let theme = app.theme();
    
    match state.screen {
        Screen::Login => login::render(frame, state, theme),
        Screen::Main => main::render(frame, state, theme),
        Screen::Settings => settings::render(frame, state, theme),
    }
}
```

## Data Flow

### Input → Message

```
User Input (keyboard/mouse)
    │
    ▼
crossterm::Event
    │
    ▼
route_event(state, event, keybindings)
    │
    ▼
Message
```

### Message → State Change

```
Message
    │
    ▼
update(state, message)
    │
    ├──▶ Mutate state
    │
    └──▶ Return Effect
```

### Effect → Side Effect

```
Effect
    │
    ▼
runtime.execute(effect)
    │
    ├──▶ Perform I/O (file, clipboard, etc.)
    │
    └──▶ Return EffectResult
```

### EffectResult → State Update

```
EffectResult
    │
    ▼
handle_effect_result(app, result)
    │
    └──▶ Update state based on result
```

## Layer Architecture

```
┌─────────────────────────────────────────────────┐
│                 Presentation Layer               │
│  ┌─────────────────────────────────────────┐    │
│  │  UI (screens, widgets, themes, icons)    │    │
│  └─────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────┐    │
│  │  Input (router, keybindings)             │    │
│  └─────────────────────────────────────────┘    │
├─────────────────────────────────────────────────┤
│                 Application Layer                │
│  ┌─────────────────────────────────────────┐    │
│  │  App (state, message, update, effect)    │    │
│  └─────────────────────────────────────────┘    │
├─────────────────────────────────────────────────┤
│                  Domain Layer                    │
│  ┌─────────────────────────────────────────┐    │
│  │  Domain (vault, item, tag, etc.)         │    │
│  └─────────────────────────────────────────┘    │
├─────────────────────────────────────────────────┤
│               Infrastructure Layer               │
│  ┌───────────────────┐ ┌───────────────────┐    │
│  │  Crypto            │ │  Storage          │    │
│  │  (encryption,      │ │  (file format,    │    │
│  │   hashing)         │ │   config)         │    │
│  └───────────────────┘ └───────────────────┘    │
└─────────────────────────────────────────────────┘
```

## Security Architecture

```
┌─────────────────────────────────────────────────┐
│                  Memory Protection               │
│  ┌─────────────────────────────────────────┐    │
│  │  SecureString - Zeroized on drop         │    │
│  │  Encryption Key - Kept in VaultState     │    │
│  │  Password - Never stored, derived only   │    │
│  └─────────────────────────────────────────┘    │
├─────────────────────────────────────────────────┤
│                  Encryption Layer                │
│  ┌─────────────────────────────────────────┐    │
│  │  AES-256-GCM - Authenticated encryption  │    │
│  │  Argon2id - Key derivation               │    │
│  │  Random - Cryptographically secure RNG   │    │
│  └─────────────────────────────────────────┘    │
├─────────────────────────────────────────────────┤
│                  Access Control                  │
│  ┌─────────────────────────────────────────┐    │
│  │  Password verification                   │    │
│  │  Auto-lock timeout                       │    │
│  │  Clipboard auto-clear                    │    │
│  └─────────────────────────────────────────┘    │
└─────────────────────────────────────────────────┘
```

## Concurrency Model

Vault adalah aplikasi single-threaded dengan cooperative multitasking:

```
Main Thread:
  ┌─────────────────────────────────────────┐
  │  Event Loop                              │
  │  ┌─────────────────────────────────────┐ │
  │  │  1. Poll for input events           │ │
  │  │  2. Process message                 │ │
  │  │  3. Execute effects (sync)          │ │
  │  │  4. Handle effect results           │ │
  │  │  5. Render UI                       │ │
  │  │  6. Check timers                    │ │
  │  │  7. Repeat                          │ │
  │  └─────────────────────────────────────┘ │
  └─────────────────────────────────────────┘
```

**Tidak ada:**
- Background threads
- Async/await
- Race conditions
- Mutex/locks

## Testing Strategy

```
┌─────────────────────────────────────────────────┐
│                  Unit Tests                      │
│  ┌─────────────────────────────────────────┐    │
│  │  - Domain models (Vault, Item)           │    │
│  │  - Crypto functions                      │    │
│  │  - Update function handlers              │    │
│  │  - Utility functions                     │    │
│  └─────────────────────────────────────────┘    │
├─────────────────────────────────────────────────┤
│               Integration Tests                  │
│  ┌─────────────────────────────────────────┐    │
│  │  - File I/O (read/write vault)           │    │
│  │  - Effect execution                      │    │
│  │  - State transitions                     │    │
│  └─────────────────────────────────────────┘    │
└─────────────────────────────────────────────────┘
```

## Lihat Juga

- [Struktur Proyek](./project-structure.md) - Organisasi file dan modul
- [TEA Pattern](./tea-pattern.md) - Detail implementasi TEA
- [Keamanan](./security.md) - Detail implementasi security
