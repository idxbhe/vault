- [x] navigasi untuk memmilih vault tidak berfungsi
- [x] ux menggunakan mouse belum terimplementasi
- [x] saat deskripsi gagal error terjadi dan langsung tampil di ui. error: Error vault::ui::app
- [x] tambahkan animasi loading sederhana saat proses deskripsi agar app tidak terkesan stuck.
- [x] error masih muncul di bagian field passwod.
- [x] ada masalah mengenai deskripsi vault, saya selalu gagal masuk vault padahal saya yakin passwod sudah benar.
- [x] item tidak tersimpan saat dibuat/diedit. sekarang auto-save saat enter ditekan.
- [x] vault korup saat tekan esc. sekarang esc aman - hanya close form tanpa save.
- [x] **CRITICAL BUG FIXED**: Salt regeneration bug yang menyebabkan vault corrupt setelah save. Sekarang salt disimpan dan digunakan kembali saat save.

## 🖱️ Mouse Navigation (NEW)

### Fitur Mouse yang Tersedia

**Login Screen:**
- Klik vault untuk memilih vault
- Scroll wheel untuk navigasi vault list

**Main Screen:**
- Klik item di list untuk memilih
- Scroll wheel di list untuk navigasi item
- Scroll wheel di detail pane untuk scroll konten
- Klik panel untuk fokus (list/detail)

**Floating Windows:**
- Klik form field untuk fokus ke field tersebut
- Klik kind option di selector untuk memilih jenis item
- Klik search result untuk memilih hasil pencarian
- Klik di luar floating window untuk menutup

### Double-Click Support
- Deteksi double-click tersedia (400ms threshold)
- Dapat digunakan untuk future features (edit on double-click, etc.)

## Notes untuk User

### ⚠️ Vault yang Sudah Corrupt (judas, test5)
- **TIDAK BISA dipulihkan** - salt sudah overwrite dengan nilai salah
- Solusi: Buat vault baru atau restore dari backup (jika ada)

### ✅ Setelah Fix Ini
- Vault baru akan bekerja dengan benar
- Auto-save AMAN - salt dijaga konsisten
- Tidak ada lagi corruption pada save
- Item akan persist setelah restart

### Cara Test
```bash
cargo run --release
# 1. Buat vault BARU (jangan gunakan judas/test5)
# 2. Create item, tekan Enter (auto-save)
# 3. Quit dengan 'q'
# 4. Restart dan unlock vault yang sama
# 5. ✅ Item harus masih ada!
```


