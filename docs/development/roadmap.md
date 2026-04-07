# 🗺️ Roadmap

Rencana pengembangan TUI Vault Manager.

## Current Version: v0.1.0

### ✅ Implemented Features

- [x] Core vault management (create, unlock, save)
- [x] Item CRUD (Generic, Password, CryptoSeed, SecureNote, ApiKey)
- [x] AES-256-GCM encryption with Argon2id KDF
- [x] Neovim-style TUI with Catppuccin/TokyoNight themes
- [x] Vim keybindings + mouse support
- [x] Content masking with reveal toggle
- [x] Secure clipboard with timeout
- [x] Fuzzy search
- [x] Tag system
- [x] Favorites filter
- [x] JSON/encrypted export
- [x] Notifications system
- [x] Statusline with mode indicator

---

## v0.2.0 - Enhanced Security

**Target:** Q1 2025

### Features

#### Keyfile Support
- [ ] Generate random keyfile
- [ ] Keyfile + password authentication
- [ ] Keyfile management UI

#### Security Questions (Password Recovery)
- [ ] Setup security questions during vault creation
- [ ] Progressive password reveal based on correct answers
- [ ] Reveal percentages: 30% → 50% → 80%

#### Auto-Lock
- [ ] Configurable inactivity timeout
- [ ] Lock on terminal suspend (SIGTSTP)
- [ ] Lock on screen saver activation (platform-specific)

#### Session Management
- [ ] Maximum session duration
- [ ] Forced re-authentication after X hours

### Security Improvements

- [ ] Memory-mapped file handling for large vaults
- [ ] Enhanced zeroization verification
- [ ] Constant-time password comparison
- [ ] Input debouncing for brute-force protection

---

## v0.3.0 - TOTP & 2FA

**Target:** Q2 2025

### Features

#### TOTP Integration
- [ ] Store TOTP secrets in Password items
- [ ] Real-time TOTP code generation
- [ ] Countdown timer display
- [ ] Auto-copy TOTP on reveal

#### QR Code Support
- [ ] Import TOTP via QR code scan (if supported by terminal)
- [ ] Export TOTP as QR (Unicode block characters)

#### Yubikey Support (Optional)
- [ ] Challenge-response authentication
- [ ] Yubikey as keyfile replacement

---

## v0.4.0 - Advanced Organization

**Target:** Q3 2025

### Features

#### Folders/Categories
- [ ] Nested folder structure
- [ ] Folder icons and colors
- [ ] Move items between folders
- [ ] Folder-level access control (optional)

#### Smart Collections
- [ ] Auto-generated collections:
  - Recently added
  - Frequently used
  - Expiring soon
  - Weak passwords
  - Duplicates

#### Custom Fields
- [ ] User-defined fields per item type
- [ ] Field types: text, password, URL, date, number
- [ ] Field templates

### UI Improvements

- [ ] Tree view for folders
- [ ] Split view for folder + items
- [ ] Drag-and-drop reordering (mouse)

---

## v0.5.0 - Sync & Backup

**Target:** Q4 2025

### Features

#### Local Sync
- [ ] Watch directory for vault changes
- [ ] Conflict detection and resolution
- [ ] Merge strategies

#### Cloud Backup (Self-hosted)
- [ ] WebDAV support
- [ ] SFTP support
- [ ] S3-compatible storage

#### Import/Export Formats
- [ ] Import from:
  - [ ] 1Password (1pif)
  - [ ] Bitwarden (JSON)
  - [ ] KeePass (KDBX)
  - [ ] LastPass (CSV)
- [ ] Export to:
  - [ ] Bitwarden (JSON)
  - [ ] KeePass (KDBX)
  - [ ] CSV (with warning)

---

## v0.6.0 - Browser Integration

**Target:** Q1 2026

### Features

#### Native Messaging Host
- [ ] Chrome/Firefox extension support
- [ ] Auto-fill passwords in browser
- [ ] Save new passwords from browser

#### CLI Integration
- [ ] `vault get <query>` - quick lookup
- [ ] `vault copy <query>` - copy to clipboard
- [ ] `vault generate` - password generator
- [ ] `vault unlock` - unlock for session

#### Daemon Mode
- [ ] Background daemon for quick access
- [ ] Unix socket / named pipe IPC
- [ ] Auto-lock on daemon idle

---

## v0.7.0 - Team Features

**Target:** Q2 2026

### Features

#### Shared Vaults
- [ ] Multiple vault files
- [ ] Vault linking
- [ ] Read-only vault mode

#### Audit Log
- [ ] Item access history
- [ ] Modification tracking
- [ ] Export audit log

#### Emergency Access
- [ ] Designated emergency contacts
- [ ] Time-delayed access
- [ ] Access request notifications

---

## Future Considerations

### Platform Support
- [ ] Windows installer (MSI)
- [ ] macOS installer (DMG)
- [ ] Linux packages (deb, rpm, AppImage)
- [ ] Homebrew formula
- [ ] Nix package

### Accessibility
- [ ] Screen reader support
- [ ] High contrast themes
- [ ] Keyboard-only navigation improvements
- [ ] Configurable font sizes

### Performance
- [ ] Lazy loading for large vaults (1000+ items)
- [ ] Incremental search
- [ ] Background indexing
- [ ] Vault compression

### Experimental
- [ ] Hardware security key storage
- [ ] Biometric unlock (platform-specific)
- [ ] Voice commands
- [ ] Mobile companion app (separate project)

---

## Contributing

See [Contributing Guide](contributing.md) for how to help with these features.

### Priority Labels

- 🔴 **Critical** - Security-related
- 🟠 **High** - Core functionality
- 🟡 **Medium** - Quality of life
- 🟢 **Low** - Nice to have

### How to Propose Features

1. Open a GitHub Issue with `[Feature Request]` prefix
2. Describe the use case
3. Provide mockups if UI-related
4. Discuss in issue before implementing

---

## Versioning

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes to file format or API
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

### Backward Compatibility

- Vault file format v1 will be supported until v2.0.0
- Minimum 6-month deprecation notice for breaking changes
- Migration tools provided for file format upgrades

---

*Last updated: January 2025*
