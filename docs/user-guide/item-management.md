# 📝 Manajemen Item

Panduan lengkap untuk membuat, mengedit, menghapus, dan mengelola item di dalam vault.

## Tipe Item

Vault mendukung berbagai tipe item untuk kebutuhan berbeda:

### 1. Generic (🔑)

Item serbaguna untuk menyimpan pasangan key-value sederhana.

| Field | Deskripsi |
|-------|-----------|
| Title | Nama item |
| Content | Konten utama (teks) |
| Notes | Catatan tambahan (opsional) |

**Use case**: API tokens sederhana, license keys, dll.

### 2. Crypto Seed (₿)

Khusus untuk menyimpan seed phrase cryptocurrency.

| Field | Deskripsi |
|-------|-----------|
| Title | Nama wallet |
| Seed Phrase | 12/24 kata BIP39 |
| Derivation Path | Path derivasi (e.g., m/44'/0'/0') |
| Passphrase | BIP39 passphrase (opsional) |
| Notes | Catatan tambahan |

**Use case**: Bitcoin, Ethereum, dan wallet crypto lainnya.

### 3. Password (🔒)

Untuk menyimpan kredensial login.

| Field | Deskripsi |
|-------|-----------|
| Title | Nama layanan |
| Username | Username/email |
| Password | Password |
| URL | URL login (opsional) |
| TOTP Secret | Secret untuk 2FA (opsional) |
| Notes | Catatan tambahan |

**Use case**: Login website, aplikasi, dll.

### 4. Secure Note (📝)

Untuk catatan panjang yang perlu diamankan.

| Field | Deskripsi |
|-------|-----------|
| Title | Judul catatan |
| Content | Isi catatan (multiline) |

**Use case**: Dokumen rahasia, instruksi recovery, dll.

### 5. API Key (🔌)

Untuk menyimpan API key dengan metadata.

| Field | Deskripsi |
|-------|-----------|
| Title | Nama service |
| Key | API key |
| Secret | API secret (opsional) |
| Endpoint | Base URL API |
| Expires | Tanggal kadaluarsa (opsional) |
| Notes | Catatan tambahan |

**Use case**: AWS keys, Stripe keys, dll.

## Membuat Item Baru

### Langkah-langkah

1. Tekan `n` atau `i` untuk membuat item baru
2. Pilih tipe item dari dialog Kind Selector:

```
╭─ Select Item Type ─────────────────────────────╮
│                                                │
│  ▸ 🔑 Generic      - Simple key-value pair     │
│    ₿  Crypto Seed  - Cryptocurrency seed       │
│    🔒 Password     - Login credentials         │
│    📝 Secure Note  - Encrypted notes           │
│    🔌 API Key      - API credentials           │
│                                                │
╰────────────────────────────────────────────────╯
```

3. Isi form sesuai tipe item
4. Tekan `Enter` untuk menyimpan

### Navigasi Form

| Tombol | Aksi |
|--------|------|
| `Tab` | Field berikutnya |
| `Shift+Tab` | Field sebelumnya |
| `Enter` | Submit form |
| `Esc` | Batal |

### Validasi

- **Title**: Wajib diisi, tidak boleh kosong
- **Content fields**: Tergantung tipe item
- Error akan ditampilkan jika validasi gagal

## Mengedit Item

### Langkah-langkah

1. Pilih item yang ingin diedit
2. Tekan `e` untuk membuka form edit
3. Form akan terisi dengan data existing
4. Modifikasi field yang diinginkan
5. Tekan `Enter` untuk menyimpan

### Undo/Redo

Setiap perubahan dicatat dalam history:

| Tombol | Aksi |
|--------|------|
| `u` | Undo perubahan terakhir |
| `Ctrl+r` | Redo perubahan yang di-undo |

## Menghapus Item

### Langkah-langkah

1. Pilih item yang ingin dihapus
2. Tekan `d` untuk membuka dialog konfirmasi:

```
╭─ Confirm Delete ───────────────────────────────╮
│                                                │
│  Are you sure you want to delete               │
│  "My Bitcoin Seed"?                            │
│                                                │
│  This action cannot be undone.                 │
│                                                │
│           [y] Yes    [n] No                    │
│                                                │
╰────────────────────────────────────────────────╯
```

3. Tekan `y` untuk konfirmasi atau `n`/`Esc` untuk batal

### Recovery

Item yang dihapus dapat di-restore dengan `u` (undo) segera setelah penghapusan, selama vault belum disimpan dan ditutup.

## Melihat Item

### Panel Detail

Setelah memilih item, detail ditampilkan di panel kanan:

```
╭─ Details ──────────────────────────────────────╮
│                                                │
│  📝 My Bitcoin Seed                            │
│  ─────────────────────────────────────────     │
│                                                │
│  Type: Crypto Seed                             │
│  Created: 2026-04-06 10:30                     │
│  Updated: 2026-04-06 14:22                     │
│                                                │
│  ╭─ Seed Phrase ────────────────────────────╮  │
│  │ ••••••••••••••••••••••••••••••••••••••   │  │
│  ╰──────────────────────────────────────────╯  │
│                                                │
│  Derivation: m/44'/0'/0'                       │
│                                                │
│  ╭─ Notes ──────────────────────────────────╮  │
│  │ Hardware wallet backup                   │  │
│  ╰──────────────────────────────────────────╯  │
│                                                │
╰────────────────────────────────────────────────╯
```

### Toggle Reveal

Konten sensitif di-mask secara default (`••••••`). Untuk melihat:

| Tombol | Aksi |
|--------|------|
| `r` | Toggle reveal/hide konten |

Reveal bersifat sementara dan akan otomatis hide saat:
- Pindah ke item lain
- Lock vault
- Timeout (jika dikonfigurasi)

## Copy ke Clipboard

### Quick Copy

Tekan `y` untuk menyalin konten utama item ke clipboard.

**Konten yang di-copy berdasarkan tipe:**

| Tipe | Yang Di-copy |
|------|--------------|
| Generic | Content value |
| Crypto Seed | Seed phrase |
| Password | Password |
| Secure Note | Full content |
| API Key | API key |

### Clipboard Security

- Clipboard otomatis dibersihkan setelah timeout (default: 30 detik)
- Notifikasi ditampilkan saat copy berhasil
- Countdown timer ditampilkan di status bar

```
✓ Copied to clipboard (clears in 30s)
```

## Favorites

### Toggle Favorite

Tekan `f` untuk menandai/menghapus item sebagai favorit.

Item favorit ditampilkan dengan ikon ⭐ dan dapat difilter.

### Filter Favorites

Toggle filter untuk hanya menampilkan item favorit (coming soon).

## Tags

### Konsep

Tags memungkinkan kategorisasi item untuk organisasi yang lebih baik.

Contoh tags:
- `personal`
- `work`
- `finance`
- `social`

### Menambah Tag

(Feature dalam pengembangan)

### Filter by Tag

(Feature dalam pengembangan)

## Pencarian

### Fuzzy Search

Tekan `/` untuk membuka dialog pencarian:

```
╭─ Search ───────────────────────────────────────╮
│                                                │
│  🔍 bit█                                       │
│                                                │
│  Results:                                      │
│  ▸ My Bitcoin Seed                             │
│    Bitwarden Password                          │
│                                                │
╰────────────────────────────────────────────────╯
```

### Navigasi Hasil

| Tombol | Aksi |
|--------|------|
| `↓` / `Ctrl+n` | Hasil berikutnya |
| `↑` / `Ctrl+p` | Hasil sebelumnya |
| `Enter` | Pilih hasil |
| `Esc` | Tutup pencarian |

### Algoritma

Pencarian menggunakan fuzzy matching yang:
- Case-insensitive
- Mencocokkan substring
- Mendukung karakter yang tidak berurutan

## Tips & Best Practices

1. **Gunakan title deskriptif**: Memudahkan pencarian
2. **Isi notes**: Tambahkan konteks untuk future reference
3. **Gunakan tipe yang tepat**: Setiap tipe memiliki fields yang optimal
4. **Backup seed phrases**: Crypto seed sangat penting, backup offline juga
5. **Reveal hanya saat perlu**: Minimalkan exposure data sensitif
