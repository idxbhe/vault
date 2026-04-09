# ⌨️ Referensi Keybinding

Vault menggunakan keybinding bergaya Vim untuk navigasi cepat dan efisien. Semua keybinding dapat dikustomisasi.

## Navigasi Global

### Pergerakan Dasar

| Tombol | Aksi | Konteks |
|--------|------|---------|
| `j` / `↓` | Pindah ke bawah | List, Detail |
| `k` / `↑` | Pindah ke atas | List, Detail |
| `h` / `←` | Fokus ke panel list | Main |
| `l` / `→` | Fokus ke panel detail | Main |
| `Tab` | Panel berikutnya | Global |
| `Shift+Tab` | Panel sebelumnya | Global |

### Jump Navigation

| Tombol | Aksi | Konteks |
|--------|------|---------|
| `gg` | Lompat ke awal | List |
| `G` | Lompat ke akhir | List |
| `Ctrl+u` / `PageUp` | Scroll satu halaman ke atas | Detail |
| `Ctrl+d` / `PageDown` | Scroll satu halaman ke bawah | Detail |

### Fokus Panel

| Tombol | Aksi |
|--------|------|
| `1` | Fokus ke panel list |
| `2` | Fokus ke panel detail |

## Operasi Item

### CRUD (Create, Read, Update, Delete)

| Tombol | Aksi | Deskripsi |
|--------|------|-----------|
| `n` / `i` | Item baru | Membuka dialog pilih tipe item |
| `e` | Edit item | Membuka form edit untuk item terpilih |
| `d` | Hapus item | Membuka dialog konfirmasi hapus |
| `Enter` | Pilih item | Memilih item dan fokus ke detail |

### Clipboard & Keamanan

| Tombol | Aksi | Deskripsi |
|--------|------|-----------|
| `y` | Copy content | Menyalin konten ke clipboard (dengan timeout) |
| `r` | Toggle reveal | Menampilkan/menyembunyikan konten sensitif |
| `f` | Toggle favorite | Menandai item sebagai favorit |

### History

| Tombol | Aksi | Deskripsi |
|--------|------|-----------|
| `u` | Undo | Membatalkan aksi terakhir |
| `Ctrl+r` | Redo | Mengulangi aksi yang dibatalkan |

## Operasi Vault

| Tombol | Aksi | Deskripsi |
|--------|------|-----------|
| `Ctrl+s` | Simpan vault | Menyimpan semua perubahan ke file |
| `Ctrl+e` | Export vault | Export vault terenkripsi (`.vault`) |
| `Ctrl+l` | Lock vault | Mengunci vault dan kembali ke login |

## Pencarian

| Tombol | Aksi | Deskripsi |
|--------|------|-----------|
| `/` | Buka pencarian | Membuka dialog pencarian fuzzy |
| `Enter` | Konfirmasi hasil | Memilih hasil pencarian |
| `Ctrl+n` / `↓` | Hasil berikutnya | Pindah ke hasil pencarian berikutnya |
| `Ctrl+p` / `↑` | Hasil sebelumnya | Pindah ke hasil pencarian sebelumnya |
| `Esc` | Tutup pencarian | Menutup dialog tanpa memilih |

## Mode & Dialog

### Dialog Help

| Tombol | Aksi |
|--------|------|
| `?` | Buka/tutup help |
| `Esc` / `Enter` | Tutup help |

### Dialog Settings

| Tombol | Aksi |
|--------|------|
| `,` | Buka settings |
| `Esc` | Tutup settings |
| `j`/`k` | Navigasi opsi |
| `Enter` | Pilih opsi |

### Dialog Konfirmasi Hapus

| Tombol | Aksi |
|--------|------|
| `y` / `Enter` | Konfirmasi hapus |
| `n` / `Esc` | Batal |

### Dialog Kind Selector (New Item)

| Tombol | Aksi |
|--------|------|
| `j` / `↓` | Pilihan berikutnya |
| `k` / `↑` | Pilihan sebelumnya |
| `Enter` | Konfirmasi pilihan |
| `Esc` | Batal |

### Form Input (New/Edit Item)

| Tombol | Aksi |
|--------|------|
| `Tab` | Field berikutnya |
| `Shift+Tab` | Field sebelumnya |
| `Enter` | Submit form |
| `Esc` | Batal |

## Login Screen

### Vault Selection Mode

| Tombol | Aksi |
|--------|------|
| `j` / `↓` | Vault berikutnya |
| `k` / `↑` | Vault sebelumnya |
| `Enter` | Pilih vault (masuk ke password) |
| `n` | Buat vault baru |
| `q` | Keluar aplikasi |

### Password Entry Mode

| Tombol | Aksi |
|--------|------|
| `Enter` | Submit password |
| `Esc` | Kembali ke pilihan vault |
| `Ctrl+q` | Force quit |

### Create Vault Mode

| Tombol | Aksi |
|--------|------|
| `Enter` | Lanjut ke langkah berikutnya |
| `Esc` | Batal dan kembali |

## Sistem

| Tombol | Aksi | Deskripsi |
|--------|------|-----------|
| `q` | Quit | Keluar (dengan konfirmasi jika ada perubahan) |
| `Ctrl+q` | Force quit | Keluar paksa tanpa konfirmasi |
| `Ctrl+c` | Force quit | Sama dengan Ctrl+q |

## Mouse Support

Vault mendukung input mouse untuk interaksi yang lebih intuitif:

| Aksi | Fungsi |
|------|--------|
| **Klik kiri** | Memilih item atau elemen UI |
| **Scroll up** | Scroll ke atas |
| **Scroll down** | Scroll ke bawah |

## Customization

Keybinding dapat dikustomisasi melalui file konfigurasi:

```json
// ~/.config/vault/keybindings.json (coming soon)
{
  "navigation": {
    "up": ["k", "Up"],
    "down": ["j", "Down"]
  },
  "actions": {
    "new_item": ["n", "i"],
    "edit_item": ["e"]
  }
}
```

## Tips

1. **Vim Users**: Keybinding dirancang agar familiar bagi pengguna Vim
2. **Fuzzy Search**: Pencarian menggunakan fuzzy matching, ketik sebagian nama saja
3. **Quick Copy**: Gunakan `y` untuk copy cepat tanpa reveal konten
4. **Clipboard Safety**: Clipboard otomatis dibersihkan setelah timeout (default 30 detik)
