# Penjelasan Alur Kerja (Workflow) Vault & Fitur "Auto-Save"

## 📖 Ringkasan Untuk Pengguna

### Apa yang Berubah?

**Masalah Sebelumnya:**
- Anda membuat item (misalnya Seed Phrase).
- Tekan Enter untuk "simpan".
- Keluar dari aplikasi.
- Item **HILANG** saat membuka aplikasi lagi. ❌

**Penyebab:**
- Sebelumnya, Enter hanya menyimpan ke **memori** (RAM).
- Tidak menyimpan ke **disk** (file .vault) secara otomatis.
- Saat aplikasi ditutup, data di RAM hilang → item hilang.

**Perbaikan:**
Sekarang saat menekan **Enter**, item langsung:
1. ✅ Disimpan ke memori (objek vault di RAM).
2. ✅ Disimpan ke disk (file vault terkait).

Inilah yang disebut sebagai "**auto-save**" → otomatis ke disk, tidak perlu simpan manual dengan Ctrl+S.

---

## 🎯 Alur Kerja (Workflow)

### Setelah Perbaikan:

```
┌─────────────────────────────────────────────────────────┐
│ 1. Tekan 'n' → Buat Item Baru                           │
│ 2. Isi formulir (Judul, Konten, dll)                    │
│ 3. Tekan ENTER → Item LANGSUNG disimpan ke disk ✅      │
│    - Notifikasi: "Item created and saved"               │
│    - File .vault diperbarui seketika                    │
│ 4. Tekan 'q' → Keluar (Aman, sudah disimpan)            │
│ 5. Buka kembali aplikasi → Item TETAP ADA ✅            │
└─────────────────────────────────────────────────────────┘
```

**Enter = Simpan Langsung Ke Disk** ✅

### Aksi Alternatif:

| Tombol | Aksi | Hasil |
|--------|------|-------|
| **Enter** | Kirim formulir | Simpan ke memori + disk ✅ |
| **Esc** | Batal | Tutup formulir, buang perubahan ❌ |
| **Ctrl+S** | Simpan manual | Simpan status vault saat ini ke disk |
| **q** | Keluar | Keluar aplikasi (aman jika sudah Enter) |
| **Ctrl+Q** | Keluar paksa | Keluar aplikasi seketika |

---

## 🤔 Mengapa Disebut "Auto-Save"?

### Desain Awal:

```
Enter     → Simpan ke memori SAJA
Ctrl+S    → Simpan ke disk (manual)
Quit (q)  → Peringatan jika belum Ctrl+S
```

Pengguna harus **ingat** untuk menekan Ctrl+S sebelum keluar.

### Desain Sekarang:

```
Enter  → Simpan ke memori + disk (OTOMATIS)
Ctrl+S → Masih bisa digunakan untuk simpan manual
Quit   → Aman, data sudah tersimpan otomatis
```

Pengguna **tidak perlu lagi mengingat** Ctrl+S → otomatis dilakukan saat menekan Enter.

**Istilah "auto-save" = otomatis ke disk, tanpa perlu langkah tambahan.**

---

## 🔍 Penjelasan Teknis Sederhana

### Apa itu Memori vs Disk?

**Memori (RAM)**:
- Tempat aplikasi menyimpan data sementara.
- Sangat cepat, tapi **hilang saat aplikasi ditutup**.
- Ibarat kertas coretan sementara.

**Disk (File .vault)**:
- File di penyimpanan fisik (SSD/HDD).
- Lebih lambat, tapi **permanen** (tidak hilang).
- Ibarat arsip permanen dalam laci.

### Alur Data:

```
Input Pengguna
      ↓
Memori (Objek Vault)
      ↓ (saat Enter ditekan)
Disk (File .vault)
```

Sebelum perbaikan: **Data terhenti di memori**.
Setelah perbaikan: **Data langsung diteruskan ke disk**.

---

## ⚠️ Terkait Korupsi Data

**Korupsi vault bisa terjadi jika:**

1. **Aplikasi crash saat menulis ke disk**
   - Solusi: Implementasi atomic write (file sementara + ganti nama).
   - Status: Masih dalam rencana pengembangan.

2. **Multiple save bersamaan**
   - Solusi: Penguncian file (file lock).
   - Status: Aplikasi bersifat single-threaded, risiko rendah.

3. **Kunci enkripsi salah**
   - Solusi: Verifikasi kunci sebelum menulis.
   - Status: Sudah diimplementasikan dalam proses buka kunci.

**Hasil pengujian menunjukkan file vault TIDAK korup** ✅
- 138 pengujian (tests) berhasil.
- Pembukaan vault (unlock) berjalan lancar.

---

## 💡 Rekomendasi

### Untuk Produksi:

**Opsi 1: Pertahankan Auto-Save (Direkomendasikan)**
- ✅ Ramah bagi pengguna.
- ✅ Menghindari kehilangan data.
- ✅ Pengalaman aplikasi modern.
- ⚠️ Memerlukan penulisan atomik untuk mencegah potensi korupsi data.

**Opsi 2: Hanya Simpan Manual**
- ✅ Desain orisinal.
- ✅ Kendali penuh di tangan pengguna.
- ❌ Mudah terlupakan oleh pengguna.
- ❌ Risiko kehilangan data tinggi.

**Kami merekomendasikan Opsi 1 + Penulisan Atomik.**
