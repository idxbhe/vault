# Cara Unlock Vault test2

## Password
**test2** → Password: `sudounlock` (10 karakter, lowercase, no spaces)

## Langkah-Langkah

### 1. Build terbaru
```bash
cd /personal/projects/vault
cargo build --release
```

### 2. Jalankan app
```bash
cargo run --release

# ATAU jalankan binary langsung:
./target/release/vault
```

### 3. Di TUI
1. Tekan `j` atau `↓` untuk navigate ke **test2**
   - Pastikan highlight di: `▸ 󱉼 test2`
   - Jangan pilih "Test Vault" (itu vault berbeda)

2. Tekan `Enter`
   - Password field muncul

3. Ketik: `sudounlock`
   - Akan tampil sebagai: `••••••••••` (10 bullets)
   - JANGAN ketik spasi di awal atau akhir

4. Tekan `Enter`
   - Loading spinner: "⠋ Unlocking vault..."
   - Vault unlock → masuk main screen

## Troubleshooting

### Error: "Wrong password or corrupted vault"

**Kemungkinan 1: Salah vault**
- Pastikan highlight di "test2", BUKAN "Test Vault"
- Test Vault password-nya berbeda: `testpass123`

**Kemungkinan 2: Typo password**
- Password: `sudounlock` (huruf kecil semua)
- BUKAN: `sudo unlock`, `SudoUnlock`, atau `SUDOUNLOCK`

**Kemungkinan 3: Error lama masih tertampil**
- Tekan `Esc` untuk kembali ke vault list
- Pilih test2 lagi (error akan clear otomatis)
- Ketik password lagi

**Kemungkinan 4: Build belum update**
```bash
# Force rebuild
cargo clean
cargo build --release
./target/release/vault
```

### Verify Password Bekerja

Test langsung tanpa TUI:
```bash
cd /personal/projects/vault
cargo test --test test_test2_unlock --release -- --nocapture
```

Harus show:
```
✅ Successfully unlocked test2 vault
   Vault name: test2
test result: ok. 2 passed
```

Jika test ini PASS tapi TUI masih error, kemungkinan:
- Salah pilih vault di UI
- Error message lama yang tertinggal

### Debug Mode

Run dengan logging:
```bash
cargo run 2>&1 | tee /tmp/vault_debug.log

# Setelah error muncul, check log:
grep -E "PASSWORD|UNLOCK|Decrypt" /tmp/vault_debug.log
```

Log akan show:
- Password yang di-input
- Vault yang dipilih
- Hasil decrypt

## Summary

- ✅ Vault: **test2**
- ✅ Password: **sudounlock**
- ✅ Integration tests: PASSING
- ✅ Direct decrypt: WORKS
- ✅ Error auto-clear: IMPLEMENTED
- ✅ Password trimming: IMPLEMENTED

Jika masih error setelah ikuti semua langkah, share output dari:
```bash
cargo test --test test_test2_unlock --release -- --nocapture
```
