# ⚡ Effects API

Referensi kontrak `Effect` dan `EffectResult` yang digunakan pola TEA di runtime saat ini.

## Overview

`update()` mengembalikan `Effect` untuk operasi side-effect (I/O, clipboard, timer, export).
`Runtime::execute()` mengeksekusi effect tersebut lalu mengembalikan `EffectResult`.

## Effect Enum (aktual)

```rust
pub enum Effect {
    None,
    Batch(Vec<Effect>),

    // File I/O
    ReadVaultFile {
        path: PathBuf,
        password: SecureString,
        keyfile: Option<Vec<u8>>,
    },
    WriteVaultFile {
        path: PathBuf,
        vault: Vault,
        key: [u8; 32],
        salt: [u8; 32],
        has_keyfile: bool,
    },
    ReadConfig,
    WriteConfig,
    UpdateRegistry,
    ReadKeyfile { path: PathBuf },

    // Clipboard
    SetClipboard { content: String, is_sensitive: bool },
    ClearClipboard,
    ScheduleClipboardClear { delay: Duration },

    // Timer
    ScheduleAutoLock { delay: Duration },
    CancelAutoLock,

    // Export
    ExportVault {
        path: PathBuf,
        vault: Vault,
        encrypted: bool,
        key: Option<[u8; 32]>,
        salt: Option<[u8; 32]>,
        has_keyfile: bool,
    },

    // System
    Exit,
}
```

## EffectResult Enum (aktual)

```rust
pub enum EffectResult {
    Success,
    VaultLoaded {
        vault: Vault,
        path: PathBuf,
        key: [u8; 32],
        salt: [u8; 32],
        has_keyfile: bool,
    },
    VaultSaved,
    ExportCompleted { path: PathBuf },
    ConfigLoaded(AppConfig),
    RegistryLoaded(VaultRegistry),
    KeyfileLoaded { path: PathBuf, data: Vec<u8> },
    Error(String),
}
```

## Kontrak Penting

1. `WriteVaultFile` **wajib** membawa `salt` dan `has_keyfile` agar metadata vault tetap konsisten saat save ulang.
2. `VaultLoaded` mengembalikan `salt` + `has_keyfile` untuk menjaga state unlock yang lengkap.
3. `ExportVault`:
   - `encrypted = true` membutuhkan `key` + `salt`
   - `encrypted = false` menulis JSON plaintext (dengan hardening file write di runtime)
4. `Effect::batch(...)` akan flatten nested batch dan membuang `None`.

## Alur Runtime Ringkas

```rust
let effect = update(&mut state, message);
let result = runtime.execute(effect);
handle_effect_result(&mut app, result);
```

`handle_effect_result` bertanggung jawab memutakhirkan state/UI (mis. transisi login->main saat `VaultLoaded`, notifikasi saat `VaultSaved`, dll).

## Catatan Integritas Save/Lock

Flow lock saat dirty menggunakan `pending_lock`:

1. `Message::LockVault` pada vault dirty -> emit `WriteVaultFile` dan set `pending_lock=true`
2. `EffectResult::VaultSaved` -> jika `pending_lock`, trigger lock transition
3. State hanya dibersihkan setelah write sukses

Ini mencegah data loss saat save gagal di tengah lock.
