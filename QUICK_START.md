# Quick Start Guide

## Running the App

```bash
cd /personal/projects/vault
cargo run --release
```

## Test Vaults

1. **test2** - Password: `sudounlock`
2. **Test Vault** - Password: `testpass123`

## How to Use

### Login Screen
- `j` / `k` or `↓` / `↑` - Navigate vault list
- `Enter` - Select vault
- `q` - Quit

### Password Entry
- Type password (shows as bullets: ••••)
- `Enter` - Submit and unlock
- `Esc` - Go back to vault list

### Expected Behavior
1. Select vault → Password field appears
2. Type password → See bullets
3. Press Enter → Loading spinner: "⠋ Unlocking vault..."
4. Vault unlocks → Main screen appears

### If Wrong Password
- Error message appears
- Start typing → Error clears immediately ✓

### Features
- ✅ Keyboard navigation (j/k/arrows)
- ✅ Mouse support (click to select)
- ✅ Auto-trim whitespace in passwords
- ✅ Loading indicators
- ✅ User-friendly error messages
- ✅ Error auto-clear on input

## Verification

All 138 tests passing:
```bash
cargo test --release
```

Binary location: `target/release/vault` (3.3MB)
