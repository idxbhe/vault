# 🗄️ Manajemen Vault

Panduan lengkap untuk membuat, membuka, menyimpan, dan mengelola vault.

## Konsep Vault

**Vault** adalah kontainer terenkripsi yang menyimpan koleksi **Item**. Setiap vault:

- Dilindungi dengan password master
- Menggunakan enkripsi AES-256-GCM
- Disimpan sebagai file `.vault`
- Dapat berisi unlimited items

## Membuat Vault Baru

### Via Login Screen

1. Di layar login, tekan `n` untuk memulai pembuatan vault
2. **Step 1**: Masukkan nama vault (misalnya "Personal", "Work")
3. **Step 2**: Masukkan password (minimal 4 karakter, disarankan 12+)
4. **Step 3**: Konfirmasi password
5. Vault akan dibuat dan dibuka otomatis

### Lokasi File

Vault disimpan di direktori data standar OS:

| OS | Lokasi |
|----|--------|
| Linux | `~/.local/share/vault/` |
| macOS | `~/Library/Application Support/com.vault.vault/` |
| Windows | `%APPDATA%\vault\vault\` |

Contoh path: `~/.local/share/vault/personal.vault`

## Membuka Vault

### Via Login Screen

1. Pilih vault dari daftar menggunakan `j`/`k`
2. Tekan `Enter` untuk memilih
3. Masukkan password
4. Tekan `Enter` untuk unlock

### Error Handling

| Error | Penyebab | Solusi |
|-------|----------|--------|
| "Wrong password" | Password salah | Coba lagi dengan password yang benar |
| "File not found" | File vault dihapus | Hapus entri dari registry |
| "Corrupted file" | File rusak | Restore dari backup |

## Menyimpan Vault

### Manual Save

Tekan `Ctrl+S` untuk menyimpan perubahan ke file.

```
┌─────────────────────────────────────┐
│ ✓ Vault saved successfully!        │
└─────────────────────────────────────┘
```

### Auto-Save

Saat ini Vault tidak memiliki auto-save. Selalu simpan sebelum keluar!

### Dirty State Indicator

Jika ada perubahan yang belum disimpan, aplikasi akan menampilkan peringatan saat keluar:

```
⚠ Unsaved changes! Press Ctrl+Q again to force quit
```

## Mengunci Vault

Tekan `Ctrl+L` untuk mengunci vault dan kembali ke login screen.

**Apa yang terjadi saat lock:**
1. Vault otomatis disimpan jika dirty
2. Data vault dihapus dari memori
3. Kembali ke login screen
4. Perlu password untuk membuka kembali

### Auto-Lock

Vault dapat dikonfigurasi untuk auto-lock setelah periode inaktivitas:

```json
{
  "auto_lock_enabled": true,
  "auto_lock_timeout_secs": 300
}
```

## Export Vault

### Quick Export (JSON)

Tekan `Ctrl+E` untuk export vault ke file JSON di direktori saat ini.

```
✓ Exported to vault_export.json
```

**⚠️ PERINGATAN**: Export JSON **TIDAK TERENKRIPSI**! Data sensitif akan terlihat dalam plaintext.

### Format Export

```json
{
  "id": "uuid-v4",
  "name": "My Vault",
  "items": [
    {
      "id": "uuid-v4",
      "title": "My Bitcoin Seed",
      "kind": "CryptoSeed",
      "content": {
        "CryptoSeed": {
          "seed_phrase": "word1 word2 ... word24",
          "derivation_path": "m/44'/0'/0'"
        }
      },
      "tags": [],
      "favorite": true,
      "created_at": "2026-04-06T00:00:00Z",
      "updated_at": "2026-04-06T00:00:00Z"
    }
  ],
  "tags": [],
  "created_at": "2026-04-06T00:00:00Z",
  "updated_at": "2026-04-06T00:00:00Z"
}
```

### Export Terenkripsi

Untuk export terenkripsi, gunakan format `.vault` (sama dengan format penyimpanan normal).

## Vault Registry

Vault menyimpan daftar vault yang diketahui di registry:

**Lokasi**: `~/.local/share/vault/registry.json`

```json
{
  "entries": [
    {
      "name": "Personal",
      "path": "/home/user/.local/share/vault/personal.vault",
      "last_opened": "2026-04-06T00:00:00Z"
    },
    {
      "name": "Work",
      "path": "/home/user/.local/share/vault/work.vault",
      "last_opened": "2026-04-05T00:00:00Z"
    }
  ]
}
```

## Backup & Recovery

### Backup Manual

```bash
# Copy file vault
cp ~/.local/share/vault/personal.vault ~/backups/

# Atau dengan timestamp
cp ~/.local/share/vault/personal.vault \
   ~/backups/personal_$(date +%Y%m%d).vault
```

### Restore dari Backup

```bash
# Copy kembali ke direktori vault
cp ~/backups/personal_20260406.vault \
   ~/.local/share/vault/personal.vault
```

### Best Practices

1. **Backup reguler**: Backup vault secara berkala
2. **Multiple locations**: Simpan backup di lokasi berbeda
3. **Test restore**: Pastikan backup dapat di-restore
4. **Encrypt backups**: Jika menyimpan di cloud, enkripsi tambahan

## Troubleshooting

### Vault tidak muncul di list

1. Periksa apakah file `.vault` ada di direktori
2. Periksa registry.json
3. Tambahkan manual ke registry jika perlu

### Lupa password

**Sayangnya, tidak ada cara untuk recovery jika lupa password.**

Vault menggunakan enkripsi yang kuat tanpa backdoor. Satu-satunya cara adalah:
1. Gunakan security questions (jika di-setup)
2. Restore dari backup yang passwordnya diingat

### File corrupt

Jika file vault corrupt:
1. Restore dari backup
2. Jika tidak ada backup, data tidak dapat di-recover
