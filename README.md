# 🔐 Vault - TUI Vault Manager

A secure terminal-based vault application for storing sensitive data like seed phrases, passwords, and API keys. Built with Rust, featuring a Neovim-style interface with Vim keybindings.

## ✨ Features

- **🔒 Military-Grade Security**
  - AES-256-GCM authenticated encryption
  - Argon2id key derivation (memory-hard, GPU-resistant)
  - Secure memory handling with automatic zeroization
  - Optional keyfile support for two-factor security
  - Optional security-question recovery with progressive password hints

- **🎨 Modern TUI Aesthetics**
  - Catppuccin themes (Latte, Frappé, Macchiato, Mocha)
  - Tokyo Night themes (Night, Storm, Day)
  - Nerd Font icons throughout
  - Rounded borders and focus highlighting

- **⌨️ Vim-Style Navigation**
  - Full keyboard navigation (`j`, `k`, `h`, `l`, `gg`, `G`)
  - Fuzzy search with `/`
  - Mouse support for click and scroll
  - Context-sensitive keybindings

- **📦 Flexible Data Storage**
  - Generic items (key-value)
  - Crypto seed phrases with derivation paths
  - Passwords with optional TOTP
  - Secure notes
  - API keys with expiration
  - Custom entries with typed dynamic fields (`text`, `secret`, `url`, `number`)

- **⚙️ Account & Vault Settings**
  - Change master password from Settings
  - Configure or disable security-question recovery from Settings
  - Encryption method selector shown during vault creation (currently AES-256-GCM)

- **🛡️ Security Features**
  - Content masking by default (`****`)
  - Temporary reveal toggle
  - Secure clipboard with auto-clear
  - Auto-lock timeout
  - Undo/redo history

## 🚀 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/vault.git
cd vault

# Build release binary
cargo build --release

# The binary is at target/release/vault
```

### Requirements

- Rust 1.75+ (2024 edition)
- A terminal with:
  - True color support (recommended)
  - Nerd Font (for icons)
  - Mouse support (optional)

## 📖 Usage

### Starting the Application

```bash
# Run with default settings
vault

# Enable debug logging
RUST_LOG=vault=debug vault
```

### Basic Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Focus list / Previous |
| `l` / `→` / `Enter` | Focus detail / Select |
| `Tab` | Switch pane |
| `gg` | Jump to top |
| `G` | Jump to bottom |

### Actions

| Key | Action |
|-----|--------|
| `/` | Search |
| `n` / `i` | New item |
| `e` | Edit item |
| `d` | Delete item |
| `y` | Copy content |
| `r` | Toggle reveal |
| `f` | Toggle favorite |

### Login / Recovery

| Key | Action |
|-----|--------|
| `f` (in password prompt) | Start forgot-password recovery |
| `Esc` | Cancel login or recovery input |

For custom entries, fill the form field as `type:key=value;type:key=value` (example: `text:username=alice;secret:token=abc123`).

### System

| Key | Action |
|-----|--------|
| `u` | Undo |
| `Ctrl+r` | Redo |
| `Ctrl+l` | Lock vault |
| `Ctrl+s` | Save vault |
| `?` | Help |
| `Esc` | Close / Back |
| `:q` / `Ctrl+q` | Quit |

## 📁 File Format

Vault files (`.vault`) use a custom binary format:

```
┌─────────────────────────────────────┐
│ Magic: "VALT" (4 bytes)             │
│ Version: u16 (2 bytes)              │
│ Header Length: u32 (4 bytes)        │
├─────────────────────────────────────┤
│ Header (bincode-serialized):        │
│   - Vault ID, Name, Created         │
│   - Encryption method metadata      │
│   - Recovery metadata (optional)    │
├─────────────────────────────────────┤
│ Encrypted Payload:                  │
│   - Nonce (12 bytes)                │
│   - Salt (32 bytes)                 │
│   - Argon2 parameters               │
│   - Ciphertext (vault data)         │
└─────────────────────────────────────┘
```

## 🏗️ Architecture

This application follows **The Elm Architecture (TEA)** pattern:

```
Model (State) → View (UI) → Message → Update → Effect
     ↑                                            │
     └────────────────────────────────────────────┘
```

### Module Structure

- `app/` - Application core (state, messages, effects)
- `domain/` - Business logic and data models
- `crypto/` - Cryptographic operations
- `storage/` - File I/O and persistence
- `ui/` - User interface components
- `input/` - Input handling and keybindings
- `utils/` - Shared utilities

## 🔧 Configuration

Configuration is stored at:
- Linux: `~/.config/vault/config.json`
- macOS: `~/Library/Application Support/com.vault.vault/config.json`
- Windows: `%APPDATA%\vault\vault\config.json`

### Options

```json
{
  "theme": "catppuccin_mocha",
  "clipboard_timeout_secs": 30,
  "auto_lock_enabled": true,
  "auto_lock_timeout_secs": 300,
  "show_icons": true,
  "mouse_enabled": true
}
```

## 🛡️ Security Considerations

1. **Password Strength**: Use a strong, unique master password
2. **Keyfile**: Consider using a keyfile stored on a separate device
3. **Backup**: Keep encrypted backups of your vault files
4. **Memory**: Sensitive data is zeroized from memory on drop
5. **Clipboard**: Content is auto-cleared after timeout

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [RustCrypto](https://github.com/RustCrypto) - Cryptographic implementations
- [Catppuccin](https://github.com/catppuccin) - Color palette inspiration
- [Tokyo Night](https://github.com/enkia/tokyo-night-vscode-theme) - Color palette inspiration
