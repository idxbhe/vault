# 📚 Vault Documentation

Selamat datang di dokumentasi lengkap **Vault TUI Manager** - aplikasi terminal untuk menyimpan data sensitif dengan keamanan tingkat militer.

## 📖 Daftar Isi

### Panduan Pengguna
- [Panduan Memulai](./user-guide/getting-started.md) - Instalasi dan penggunaan dasar
- [Referensi Keybinding](./user-guide/keybindings.md) - Daftar lengkap shortcut keyboard
- [Manajemen Vault](./user-guide/vault-management.md) - Membuat, membuka, dan mengelola vault
- [Manajemen Item](./user-guide/item-management.md) - CRUD operasi untuk item
- [Konfigurasi](./user-guide/configuration.md) - Pengaturan dan tema

### Panduan Teknis
- [Arsitektur](./technical/architecture.md) - Desain sistem dan pola arsitektur
- [Struktur Proyek](./technical/project-structure.md) - Organisasi kode dan modul
- [Keamanan](./technical/security.md) - Implementasi kriptografi dan keamanan
- [Format File](./technical/file-format.md) - Spesifikasi format file vault
- [TEA Pattern](./technical/tea-pattern.md) - The Elm Architecture dalam Rust

### Referensi API
- [Domain Models](./api/domain.md) - Struktur data utama
- [Messages](./api/messages.md) - Sistem pesan dan event
- [Effects](./api/effects.md) - Side effects dan runtime
- [UI Components](./api/ui-components.md) - Widget dan screen

### Pengembangan
- [Kontribusi](./development/contributing.md) - Panduan kontribusi
- [Testing](./development/testing.md) - Strategi dan cara testing
- [Roadmap](./development/roadmap.md) - Rencana pengembangan

## 🚀 Quick Start

```bash
# Clone repository
git clone https://github.com/yourusername/vault.git
cd vault

# Build dan jalankan
cargo run

# Atau build release
cargo build --release
./target/release/vault
```

## 📊 Status Proyek

| Komponen | Status |
|----------|--------|
| Core Architecture | ✅ Complete |
| Vault Create/Unlock | ✅ Complete |
| Item CRUD | ✅ Complete |
| Save/Export | ✅ Complete |
| Themes | ✅ Complete |
| Tests | ✅ 133 Passing |

## 🔗 Link Eksternal

- [Repository GitHub](https://github.com/yourusername/vault)
- [Crates.io](https://crates.io/crates/vault) (coming soon)
- [Documentation Online](https://docs.rs/vault) (coming soon)

---

*Dokumentasi ini diperbarui pada April 2026*
