# ⚙️ Konfigurasi

Panduan lengkap untuk mengkonfigurasi Vault sesuai preferensi Anda.

## Lokasi File Konfigurasi

| OS | Lokasi |
|----|--------|
| Linux | `~/.config/vault/config.json` |
| macOS | `~/Library/Application Support/com.vault.vault/config.json` |
| Windows | `%APPDATA%\vault\vault\config.json` |

## Opsi Konfigurasi

### Konfigurasi Lengkap

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

### Detail Opsi

#### `theme`

Tema warna untuk UI.

| Nilai | Deskripsi |
|-------|-----------|
| `catppuccin_latte` | Light theme dengan warna pastel |
| `catppuccin_frappe` | Medium-light theme |
| `catppuccin_macchiato` | Medium-dark theme |
| `catppuccin_mocha` | Dark theme (default) |
| `tokyo_night` | Tokyo Night dark |
| `tokyo_night_storm` | Tokyo Night Storm variant |
| `tokyo_night_day` | Tokyo Night light theme |

**Default**: `catppuccin_mocha`

#### `clipboard_timeout_secs`

Waktu dalam detik sebelum clipboard otomatis dibersihkan setelah copy.

| Rentang | Deskripsi |
|---------|-----------|
| `0` | Tidak pernah clear (tidak disarankan) |
| `10-30` | Disarankan untuk penggunaan normal |
| `60+` | Untuk workflow yang butuh waktu lebih |

**Default**: `30`

#### `auto_lock_enabled`

Mengaktifkan auto-lock setelah periode inaktivitas.

| Nilai | Deskripsi |
|-------|-----------|
| `true` | Auto-lock aktif |
| `false` | Auto-lock nonaktif |

**Default**: `true`

#### `auto_lock_timeout_secs`

Waktu inaktivitas (dalam detik) sebelum vault otomatis terkunci.

| Nilai | Deskripsi |
|-------|-----------|
| `60` | 1 menit |
| `300` | 5 menit (default) |
| `600` | 10 menit |
| `1800` | 30 menit |

**Default**: `300`

#### `show_icons`

Menampilkan ikon Nerd Font di UI.

| Nilai | Deskripsi |
|-------|-----------|
| `true` | Ikon ditampilkan (membutuhkan Nerd Font) |
| `false` | Ikon disembunyikan (untuk terminal tanpa Nerd Font) |

**Default**: `true`

#### `mouse_enabled`

Mengaktifkan dukungan mouse.

| Nilai | Deskripsi |
|-------|-----------|
| `true` | Mouse aktif (klik, scroll) |
| `false` | Mouse nonaktif (keyboard only) |

**Default**: `true`

## Tema

### Catppuccin Themes

#### Mocha (Dark)
```
Background: #1e1e2e (Base)
Foreground: #cdd6f4 (Text)
Accent:     #89b4fa (Blue)
```

#### Macchiato (Medium-Dark)
```
Background: #24273a (Base)
Foreground: #cad3f5 (Text)
Accent:     #8aadf4 (Blue)
```

#### Frappé (Medium-Light)
```
Background: #303446 (Base)
Foreground: #c6d0f5 (Text)
Accent:     #8caaee (Blue)
```

#### Latte (Light)
```
Background: #eff1f5 (Base)
Foreground: #4c4f69 (Text)
Accent:     #1e66f5 (Blue)
```

### Tokyo Night Themes

#### Night (Dark)
```
Background: #1a1b26 (Background)
Foreground: #c0caf5 (Foreground)
Accent:     #7aa2f7 (Blue)
```

#### Storm (Medium-Dark)
```
Background: #24283b (Background)
Foreground: #c0caf5 (Foreground)
Accent:     #7aa2f7 (Blue)
```

#### Day (Light)
```
Background: #e1e2e7 (Background)
Foreground: #3760bf (Foreground)
Accent:     #2e7de9 (Blue)
```

## Mengubah Konfigurasi

### Via Settings Screen

1. Tekan `,` untuk membuka Settings
2. Navigasi dengan `j`/`k`
3. Pilih opsi dengan `Enter`
4. Perubahan otomatis disimpan

### Via File

1. Buka file konfigurasi dengan editor teks
2. Modifikasi nilai JSON
3. Simpan file
4. Restart aplikasi

## Nerd Fonts

### Instalasi

Vault menggunakan Nerd Fonts untuk ikon. Install salah satu:

- [JetBrains Mono Nerd Font](https://github.com/ryanoasis/nerd-fonts/releases)
- [Fira Code Nerd Font](https://github.com/ryanoasis/nerd-fonts/releases)
- [Hack Nerd Font](https://github.com/ryanoasis/nerd-fonts/releases)

### Konfigurasi Terminal

#### Alacritty
```yaml
font:
  normal:
    family: "JetBrainsMono Nerd Font"
```

#### iTerm2
Preferences → Profiles → Text → Font → Select Nerd Font

#### Windows Terminal
```json
{
  "profiles": {
    "defaults": {
      "fontFace": "JetBrainsMono Nerd Font"
    }
  }
}
```

### Jika Tidak Menggunakan Nerd Font

Set `show_icons: false` di konfigurasi untuk menghilangkan ikon.

## Terminal Requirements

### True Color Support

Vault bekerja optimal dengan terminal yang mendukung true color (24-bit).

**Terminal yang didukung:**
- Alacritty
- iTerm2
- Windows Terminal
- Kitty
- WezTerm
- Konsole
- GNOME Terminal (3.14+)

### Verifikasi True Color

```bash
# Test true color support
curl -s https://gist.githubusercontent.com/lifepillar/09a44b8cf0f9397465614e622979107f/raw/24-bit-color.sh | bash
```

### Mouse Support

Untuk mouse support, terminal harus mendukung xterm mouse reporting.

## Environment Variables

| Variable | Deskripsi |
|----------|-----------|
| `RUST_LOG` | Log level (error, warn, info, debug, trace) |
| `VAULT_CONFIG_DIR` | Override config directory |
| `NO_COLOR` | Disable colors |

### Contoh

```bash
# Enable debug logging
RUST_LOG=vault=debug vault

# Use custom config directory
VAULT_CONFIG_DIR=/custom/path vault
```

## Reset Konfigurasi

Untuk reset ke default, hapus file konfigurasi:

```bash
# Linux
rm ~/.config/vault/config.json

# macOS
rm ~/Library/Application\ Support/com.vault.vault/config.json
```

Konfigurasi default akan dibuat ulang saat aplikasi dijalankan.
