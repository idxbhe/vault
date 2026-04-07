# 🚀 Panduan Memulai

## Persyaratan Sistem

### Minimum
- **Rust**: 1.75+ (Edition 2024)
- **Terminal**: Dengan dukungan ANSI escape codes
- **OS**: Linux, macOS, atau Windows 10+

### Direkomendasikan
- **Terminal**: Dengan dukungan True Color (24-bit)
- **Font**: Nerd Font (untuk ikon)
- **Mouse**: Terminal dengan dukungan mouse (opsional)

## Instalasi

### Dari Source Code

```bash
# 1. Clone repository
git clone https://github.com/yourusername/vault.git
cd vault

# 2. Build release binary
cargo build --release

# 3. (Opsional) Install ke PATH
cp target/release/vault ~/.local/bin/
# atau
sudo cp target/release/vault /usr/local/bin/
```

### Verifikasi Instalasi

```bash
# Jalankan aplikasi
vault

# Atau dengan debug logging
RUST_LOG=vault=debug vault
```

## Penggunaan Pertama Kali

### 1. Layar Selamat Datang

Saat pertama kali menjalankan Vault, Anda akan melihat layar "Getting Started":

```
╭─ Getting Started ─────────────────────────────╮
│                                               │
│  Welcome to Vault!                            │
│                                               │
│  No vaults found. Press 'n' to create         │
│  a new vault.                                 │
│                                               │
╰───────────────────────────────────────────────╯

           Enter  Select    n  New Vault    q  Quit
```

### 2. Membuat Vault Baru

Tekan `n` untuk membuat vault baru. Proses pembuatan vault memiliki 3 langkah:

**Langkah 1: Nama Vault**
```
╭─ Create New Vault ────────────────────────────╮
│                                               │
│  Step 1/3: Enter vault name                   │
│                                               │
│  ╭─ Vault Name ─────────────────────────────╮ │
│  │ My Secrets█                              │ │
│  ╰──────────────────────────────────────────╯ │
│                                               │
╰───────────────────────────────────────────────╯
```

**Langkah 2: Password**
```
╭─ Create New Vault ────────────────────────────╮
│                                               │
│  Step 2/3: Enter password                     │
│                                               │
│  ╭─ Password ───────────────────────────────╮ │
│  │ ••••••••█                                │ │
│  ╰──────────────────────────────────────────╯ │
│                                               │
╰───────────────────────────────────────────────╯
```

**Langkah 3: Konfirmasi Password**
```
╭─ Create New Vault ────────────────────────────╮
│                                               │
│  Step 3/3: Confirm password                   │
│                                               │
│  ╭─ Confirm Password ───────────────────────╮ │
│  │ ••••••••█                                │ │
│  ╰──────────────────────────────────────────╯ │
│                                               │
╰───────────────────────────────────────────────╯
```

Setelah konfirmasi, vault akan dibuat dan Anda otomatis masuk ke layar utama.

### 3. Membuka Vault yang Sudah Ada

Jika sudah memiliki vault:

1. Pilih vault dengan `j`/`k` atau panah atas/bawah
2. Tekan `Enter` untuk memilih
3. Masukkan password
4. Tekan `Enter` untuk unlock

### 4. Layar Utama

Setelah vault terbuka, Anda akan melihat layout utama:

```
╭─ Items ──────────────────╮ ╭─ Details ────────────────────────────╮
│ ▸  My Bitcoin Seed       │ │  My Bitcoin Seed                    │
│    Gmail Password        │ │                                      │
│    AWS API Key           │ │  Type: Crypto Seed                   │
│    Server Notes          │ │  Created: 2026-04-06                 │
│                          │ │                                      │
│                          │ │  ╭─ Seed Phrase ──────────────────╮  │
│                          │ │  │ ••••••••••••••••••••••••••••   │  │
│                          │ │  ╰────────────────────────────────╯  │
│                          │ │                                      │
╰──────────────────────────╯ ╰──────────────────────────────────────╯
                              Ctrl+s  Save    ?  Help    q  Quit
```

## Navigasi Dasar

| Tombol | Aksi |
|--------|------|
| `j` / `↓` | Pindah ke bawah |
| `k` / `↑` | Pindah ke atas |
| `Tab` | Pindah antar panel |
| `Enter` | Pilih / Konfirmasi |
| `Esc` | Kembali / Batal |
| `q` | Keluar dari aplikasi |

## Langkah Selanjutnya

- [Referensi Keybinding](./keybindings.md) - Pelajari semua shortcut
- [Manajemen Item](./item-management.md) - Menambah dan mengedit item
- [Konfigurasi](./configuration.md) - Mengatur tema dan preferensi
