# 📁 Struktur Proyek

Dokumentasi lengkap organisasi kode dan modul dalam proyek Vault.

## Tree Overview

```
vault/
├── Cargo.toml              # Package manifest
├── Cargo.lock              # Dependency lock file
├── README.md               # Project README
├── SECURITY.md             # Security policy
├── .gitignore              # Git ignore rules
│
├── docs/                   # Documentation
│   ├── README.md
│   ├── user-guide/
│   ├── technical/
│   ├── api/
│   └── development/
│
├── src/
│   ├── main.rs             # Application entry point
│   ├── lib.rs              # Library crate root
│   │
│   ├── app/                # Application core (TEA)
│   │   ├── mod.rs
│   │   ├── state.rs        # AppState & related types
│   │   ├── message.rs      # Message enum
│   │   ├── update.rs       # Update function
│   │   ├── effect.rs       # Effect & EffectResult
│   │   └── runtime.rs      # Effect executor
│   │
│   ├── domain/             # Business domain models
│   │   ├── mod.rs
│   │   ├── vault.rs        # Vault struct
│   │   ├── item.rs         # Item & ItemContent
│   │   ├── tag.rs          # Tag system
│   │   └── security_question.rs
│   │
│   ├── crypto/             # Cryptography
│   │   ├── mod.rs
│   │   ├── encryption.rs   # AES-256-GCM
│   │   ├── hashing.rs      # Argon2id
│   │   ├── secure_string.rs
│   │   └── random.rs       # Secure RNG
│   │
│   ├── storage/            # Persistence
│   │   ├── mod.rs
│   │   ├── vault_file.rs   # .vault file format
│   │   ├── config.rs       # AppConfig
│   │   └── registry.rs     # VaultRegistry
│   │
│   ├── ui/                 # User interface
│   │   ├── mod.rs
│   │   ├── app.rs          # App wrapper
│   │   ├── theme.rs        # Theme definitions
│   │   ├── icons.rs        # Nerd Font icons
│   │   ├── screens/        # Full-screen views
│   │   │   ├── mod.rs
│   │   │   ├── login.rs
│   │   │   ├── main.rs
│   │   │   └── settings.rs
│   │   └── widgets/        # Reusable UI components
│   │       ├── mod.rs
│   │       ├── item_list.rs
│   │       ├── item_detail.rs
│   │       ├── search_dialog.rs
│   │       ├── edit_form.rs
│   │       ├── kind_selector.rs
│   │       ├── statusline.rs
│   │       ├── notification.rs
│   │       └── help.rs
│   │
│   ├── input/              # Input handling
│   │   ├── mod.rs
│   │   ├── router.rs       # Event → Message routing
│   │   └── keybindings.rs  # Keybinding configuration
│   │
│   └── utils/              # Shared utilities
│       ├── mod.rs
│       ├── fuzzy.rs        # Fuzzy search
│       └── mask.rs         # Content masking
│
└── tests/                  # Integration tests
    └── create_test_vault.rs
```

## Deskripsi Modul

### `src/main.rs`

Entry point aplikasi. Bertanggung jawab untuk:
- Inisialisasi terminal (raw mode, alternate screen)
- Setup logging dengan tracing
- Main event loop
- Cleanup saat exit

```rust
fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut terminal = Terminal::new(backend)?;
    
    // Main loop
    loop {
        // Render
        terminal.draw(|f| render(f, &mut app))?;
        
        // Handle events
        if event::poll(tick_rate)? {
            let event = event::read()?;
            let msg = route_event(&app.state(), event, &keybindings);
            let effect = update(app.state_mut(), msg);
            let result = runtime.execute(effect);
            handle_effect_result(&mut app, result);
        }
        
        // Check quit
        if app.state().should_quit {
            break;
        }
    }
    
    // Cleanup
    disable_raw_mode()?;
    Ok(())
}
```

### `src/lib.rs`

Library crate root. Re-exports semua public modules:

```rust
pub mod app;
pub mod crypto;
pub mod domain;
pub mod input;
pub mod storage;
pub mod ui;
pub mod utils;
```

### `src/app/`

Core application logic menggunakan TEA pattern.

#### `state.rs`

Semua state types:

```rust
pub struct AppState { ... }
pub struct VaultState { ... }
pub struct UIState { ... }
pub struct LoginScreen { ... }
pub struct ClipboardState { ... }
pub enum AppMode { ... }
pub enum Screen { ... }
pub enum Pane { ... }
pub enum FloatingWindow { ... }
```

#### `message.rs`

Message enum dan related types:

```rust
pub enum Message {
    // ~50+ message variants
}
pub enum ExportFormat { ... }
pub struct ItemUpdates { ... }
```

#### `update.rs`

Update function dengan semua handlers:

```rust
pub fn update(state: &mut AppState, msg: Message) -> Effect {
    match msg {
        // Pattern matching untuk setiap message
    }
}
```

#### `effect.rs`

Effect dan result types:

```rust
pub enum Effect { ... }
pub enum EffectResult { ... }
```

#### `runtime.rs`

Effect executor:

```rust
pub struct Runtime { ... }
impl Runtime {
    pub fn execute(&mut self, effect: Effect) -> EffectResult { ... }
    pub fn tick(&mut self) { ... }
}
```

### `src/domain/`

Business domain models.

#### `vault.rs`

Vault container:

```rust
pub struct Vault {
    pub id: Uuid,
    pub name: String,
    pub items: Vec<Item>,
    pub tags: Vec<Tag>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Vault {
    pub fn new(name: &str) -> Self { ... }
    pub fn add_item(&mut self, item: Item) { ... }
    pub fn remove_item(&mut self, id: Uuid) -> Option<Item> { ... }
    // ...
}
```

#### `item.rs`

Item dan content types:

```rust
pub struct Item {
    pub id: Uuid,
    pub title: String,
    pub kind: ItemKind,
    pub content: ItemContent,
    pub tags: Vec<Uuid>,
    pub favorite: bool,
    // ...
}

pub enum ItemKind {
    Generic,
    CryptoSeed,
    Password,
    SecureNote,
    ApiKey,
}

pub enum ItemContent {
    Generic { value: String },
    CryptoSeed { seed_phrase: String, derivation_path: String, passphrase: Option<String> },
    Password { username: String, password: String, url: Option<String>, totp_secret: Option<String> },
    SecureNote { content: String },
    ApiKey { key: String, secret: Option<String>, endpoint: Option<String>, expires_at: Option<DateTime<Utc>> },
}
```

### `src/crypto/`

Cryptographic operations.

#### `encryption.rs`

AES-256-GCM encryption:

```rust
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> { ... }
pub fn decrypt(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> { ... }
```

#### `hashing.rs`

Argon2id key derivation:

```rust
pub fn derive_key(password: &[u8], salt: &[u8; 32]) -> Result<[u8; 32], CryptoError> { ... }
pub fn generate_salt() -> [u8; 32] { ... }
```

#### `secure_string.rs`

Secure string dengan auto-zeroization:

```rust
pub struct SecureString {
    inner: Zeroizing<String>,
}

impl SecureString {
    pub fn new(s: String) -> Self { ... }
    pub fn expose(&self) -> &str { ... }
}
```

### `src/storage/`

File persistence.

#### `vault_file.rs`

Vault file format:

```rust
pub struct VaultFile {
    header: VaultHeader,
    encrypted_data: Vec<u8>,
}

impl VaultFile {
    pub fn new(vault: &Vault, password: &SecureString, keyfile: Option<&[u8]>) -> Result<Self, StorageError> { ... }
    pub fn read(path: &Path) -> Result<Self, StorageError> { ... }
    pub fn write(&self, path: &Path) -> Result<(), StorageError> { ... }
    pub fn decrypt(&self, password: &SecureString, keyfile: Option<&[u8]>) -> Result<Vault, StorageError> { ... }
}
```

#### `config.rs`

Application configuration:

```rust
pub struct AppConfig {
    pub theme: String,
    pub clipboard_timeout_secs: u64,
    pub auto_lock_enabled: bool,
    pub auto_lock_timeout_secs: u64,
    pub show_icons: bool,
    pub mouse_enabled: bool,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> { ... }
    pub fn save(&self) -> Result<(), ConfigError> { ... }
}
```

#### `registry.rs`

Vault registry:

```rust
pub struct VaultRegistry {
    pub entries: Vec<RegistryEntry>,
}

pub struct RegistryEntry {
    pub name: String,
    pub path: PathBuf,
    pub last_opened: DateTime<Utc>,
}
```

### `src/ui/`

User interface components.

#### `app.rs`

App wrapper yang mengelola state dan screen state:

```rust
pub struct App {
    state: AppState,
    login_screen_state: LoginScreenState,
    main_screen_state: MainScreenState,
    settings_screen_state: SettingsScreenState,
}
```

#### `theme.rs`

Theme definitions:

```rust
pub struct ThemePalette {
    pub bg: Color,
    pub fg: Color,
    pub fg_muted: Color,
    pub accent: Color,
    pub border: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
}

pub fn get_theme(name: &str) -> ThemePalette { ... }
```

#### `icons.rs`

Nerd Font icon constants:

```rust
pub mod ui {
    pub const VAULT: &str = "󱉼";
    pub const VAULT_LOCKED: &str = "";
    pub const SEARCH: &str = "";
    // ...
}

pub mod item {
    pub const GENERIC: &str = "󰌆";
    pub const CRYPTO: &str = "󰠓";
    pub const PASSWORD: &str = "";
    // ...
}
```

#### `screens/`

Full-screen views.

#### `widgets/`

Reusable UI components.

### `src/input/`

Input handling.

#### `router.rs`

Event routing:

```rust
pub fn route_event(
    state: &AppState,
    event: Event,
    keybindings: &KeybindingConfig,
) -> Message { ... }
```

#### `keybindings.rs`

Keybinding configuration:

```rust
pub enum KeyAction { ... }
pub struct KeyCombo { ... }
pub struct KeybindingConfig { ... }
```

### `src/utils/`

Shared utilities.

#### `fuzzy.rs`

Fuzzy search implementation:

```rust
pub struct FuzzyMatcher { ... }
pub fn fuzzy_match(query: &str, text: &str) -> Option<Score> { ... }
```

#### `mask.rs`

Content masking:

```rust
pub fn mask_content(content: &str) -> String { ... }
pub fn partial_reveal(content: &str, chars: usize) -> String { ... }
```

## Dependencies

### Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.26 | Terminal UI framework |
| `crossterm` | 0.27 | Terminal manipulation |
| `serde` | 1.0 | Serialization |
| `serde_json` | 1.0 | JSON handling |
| `uuid` | 1.0 | Unique identifiers |
| `chrono` | 0.4 | Date/time handling |

### Crypto Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `aes-gcm` | 0.10 | AES-256-GCM encryption |
| `argon2` | 0.5 | Key derivation |
| `zeroize` | 1.7 | Secure memory wiping |
| `rand` | 0.8 | Cryptographic RNG |

### Storage Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `directories` | 5.0 | Platform directories |
| `bincode` | 1.3 | Binary serialization |

### Utility Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `anyhow` | 1.0 | Error handling |
| `thiserror` | 1.0 | Error derive |
| `tracing` | 0.1 | Logging |
| `arboard` | 3.0 | Clipboard access |
