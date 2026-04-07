# 🔐 Keamanan

Dokumentasi lengkap implementasi keamanan dalam Vault.

## Overview

Vault dirancang dengan prinsip **defense in depth** - multiple layers of security untuk melindungi data sensitif.

```
┌─────────────────────────────────────────────────┐
│           Layer 1: Access Control               │
│  Password + Optional Keyfile + Auto-lock        │
├─────────────────────────────────────────────────┤
│           Layer 2: Encryption                   │
│  AES-256-GCM + Argon2id + Random Nonce          │
├─────────────────────────────────────────────────┤
│           Layer 3: Memory Protection            │
│  Zeroization + No Swap + Minimal Exposure       │
├─────────────────────────────────────────────────┤
│           Layer 4: Runtime Protection           │
│  Clipboard Timeout + Content Masking            │
└─────────────────────────────────────────────────┘
```

## Enkripsi

### Algoritma

| Komponen | Algoritma | Spesifikasi |
|----------|-----------|-------------|
| Encryption | AES-256-GCM | NIST approved, authenticated |
| Key Derivation | Argon2id | Memory-hard, GPU-resistant |
| Nonce Generation | ChaCha20 | Cryptographically secure |

### AES-256-GCM

**Authenticated Encryption with Associated Data (AEAD)**

```
┌─────────────────────────────────────────────────┐
│                  Encryption                      │
│                                                  │
│  Plaintext ──┬──▶ AES-256-GCM ──▶ Ciphertext    │
│              │                                   │
│  Key ────────┤                                   │
│              │                                   │
│  Nonce ──────┴──────────────────▶ Auth Tag      │
│                                                  │
└─────────────────────────────────────────────────┘
```

**Karakteristik:**
- **Confidentiality**: Data tidak dapat dibaca tanpa key
- **Integrity**: Modifikasi terdeteksi
- **Authentication**: Origin data terverifikasi

**Implementation:**

```rust
use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};

pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new(Key::from_slice(key));
    let nonce = generate_nonce();
    
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    
    // Format: nonce || ciphertext
    let mut result = nonce.to_vec();
    result.extend(ciphertext);
    Ok(result)
}
```

### Argon2id

**Memory-Hard Key Derivation Function**

```
┌─────────────────────────────────────────────────┐
│              Key Derivation                      │
│                                                  │
│  Password ──┬──▶ Argon2id ──▶ 256-bit Key       │
│             │                                    │
│  Salt ──────┤                                    │
│             │                                    │
│  Params ────┘                                    │
│  - Memory: 64 MB                                 │
│  - Iterations: 3                                 │
│  - Parallelism: 4                                │
│                                                  │
└─────────────────────────────────────────────────┘
```

**Why Argon2id:**
- Winner of Password Hashing Competition (2015)
- Hybrid of Argon2i (side-channel resistant) dan Argon2d (GPU resistant)
- Memory-hard: Requires significant RAM, prevents GPU/ASIC attacks
- Time-hard: Configurable iterations

**Parameters:**

| Parameter | Value | Purpose |
|-----------|-------|---------|
| Memory | 64 MB | High memory requirement |
| Iterations | 3 | Time cost |
| Parallelism | 4 | Lane count |
| Output | 32 bytes | 256-bit key |
| Salt | 32 bytes | Random per-vault |

**Implementation:**

```rust
use argon2::{Argon2, Params, Version};

pub fn derive_key(password: &[u8], salt: &[u8; 32]) -> Result<[u8; 32], CryptoError> {
    let params = Params::new(
        64 * 1024,  // 64 MB memory
        3,          // 3 iterations
        4,          // 4 parallel lanes
        Some(32),   // 32 byte output
    )?;
    
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    let mut key = [0u8; 32];
    argon2.hash_password_into(password, salt, &mut key)?;
    
    Ok(key)
}
```

## Format File Vault

### Binary Structure

```
┌─────────────────────────────────────────────────┐
│ Offset │ Size   │ Description                   │
├────────┼────────┼───────────────────────────────┤
│ 0      │ 4      │ Magic: "VALT"                 │
│ 4      │ 2      │ Version: u16                  │
│ 6      │ 4      │ Header Length: u32            │
│ 10     │ n      │ Header (bincode)              │
│ 10+n   │ 12     │ Nonce                         │
│ 22+n   │ 32     │ Salt                          │
│ 54+n   │ m      │ Argon2 Params (bincode)       │
│ 54+n+m │ rest   │ Ciphertext + Auth Tag         │
└─────────────────────────────────────────────────┘
```

### Header (Unencrypted)

```rust
pub struct VaultHeader {
    pub vault_id: Uuid,
    pub vault_name: String,
    pub created_at: DateTime<Utc>,
    pub security_questions: Option<Vec<SecurityQuestion>>,
}
```

**Note:** Header tidak berisi data sensitif, hanya metadata.

### Encrypted Payload

```rust
// Payload yang dienkripsi
pub struct VaultPayload {
    pub items: Vec<Item>,
    pub tags: Vec<Tag>,
    pub updated_at: DateTime<Utc>,
}
```

## Memory Protection

### SecureString

String wrapper dengan auto-zeroization:

```rust
use zeroize::Zeroizing;

pub struct SecureString {
    inner: Zeroizing<String>,
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Automatically zeroed by Zeroizing<T>
    }
}
```

**Guarantees:**
- Memory di-zero saat variabel di-drop
- Compiler tidak akan optimize away zeroing
- No copies di memory (move-only semantics)

### Key Handling

```rust
pub struct VaultState {
    pub vault: Vault,
    pub vault_path: PathBuf,
    pub encryption_key: [u8; 32],  // Kept in memory while unlocked
    // ...
}

impl Drop for VaultState {
    fn drop(&mut self) {
        self.encryption_key.zeroize();
    }
}
```

### Password Handling

Password **tidak pernah** disimpan:

1. User memasukkan password
2. Password di-derive menjadi key dengan Argon2id
3. Original password di-zeroize
4. Key digunakan untuk decrypt
5. Key disimpan di VaultState selama unlocked

```rust
let password = SecureString::new(user_input);
let key = derive_key(password.expose().as_bytes(), &salt)?;
// password automatically zeroized when it goes out of scope
```

## Clipboard Security

### Auto-Clear

Clipboard otomatis dibersihkan setelah timeout:

```rust
pub struct ClipboardState {
    pub has_content: bool,
    pub clear_at: Option<Instant>,
    pub is_sensitive: bool,
}

impl ClipboardState {
    pub fn should_clear(&self) -> bool {
        if let Some(clear_at) = self.clear_at {
            Instant::now() >= clear_at
        } else {
            false
        }
    }
}
```

**Default timeout:** 30 detik (configurable)

### Implementation

```rust
pub fn set_clipboard(content: &str, is_sensitive: bool) -> Result<(), Error> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(content)?;
    Ok(())
}

pub fn clear_clipboard() -> Result<(), Error> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text("")?;
    Ok(())
}
```

## Access Control

### Password Verification

```
User Password
    │
    ▼
Argon2id(password, stored_salt)
    │
    ▼
Derived Key
    │
    ▼
AES-256-GCM Decrypt
    │
    ├──▶ Success: Vault unlocked
    │
    └──▶ Failure: Invalid password (auth tag mismatch)
```

### Auto-Lock

Vault otomatis terkunci setelah periode inaktivitas:

```rust
pub fn check_auto_lock(&self) -> bool {
    if !self.config.auto_lock_enabled {
        return false;
    }
    
    if let Some(ref vs) = self.vault_state {
        let elapsed = vs.last_activity.elapsed();
        elapsed.as_secs() >= self.config.auto_lock_timeout_secs
    } else {
        false
    }
}
```

**Default:** 5 menit (300 detik)

### Keyfile Support (Optional)

Two-factor authentication dengan keyfile:

```
┌─────────────────────────────────────────────────┐
│              Two-Factor Derivation               │
│                                                  │
│  Password ──────┐                                │
│                 ├──▶ Combined Hash ──▶ Key      │
│  Keyfile ───────┘                                │
│                                                  │
└─────────────────────────────────────────────────┘
```

**Keyfile:**
- File arbitrary (any file)
- SHA-256 hash dari konten
- Combined dengan password sebelum Argon2id

## Content Protection

### Masking

Konten sensitif di-mask secara default:

```rust
pub fn mask_content(content: &str) -> String {
    "•".repeat(content.len().min(20))
}

// Example:
// "my-secret-password" → "••••••••••••••••••"
```

### Temporary Reveal

Reveal bersifat sementara dan state-based:

```rust
pub struct UIState {
    pub content_revealed: bool,
    // ...
}

// Auto-hide saat:
// - Switch item
// - Lock vault
// - Close detail panel
```

## Security Best Practices

### For Users

1. **Strong Password**
   - Minimum 12 characters
   - Mix of upper, lower, numbers, symbols
   - Tidak menggunakan kata dictionary

2. **Keyfile (Recommended)**
   - Simpan di device terpisah (USB)
   - Backup keyfile securely
   - Jangan share keyfile

3. **Auto-Lock**
   - Enable auto-lock
   - Set timeout yang reasonable (5-15 menit)

4. **Clipboard**
   - Biarkan clipboard timeout enabled
   - Avoid copying sensitive data jika tidak perlu

5. **Backup**
   - Backup vault files secara regular
   - Test restore dari backup
   - Encrypt backup locations

### For Developers

1. **Code Review**
   - Security-sensitive code harus di-review
   - Cari pattern yang bisa leak memory

2. **Dependencies**
   - Audit crypto dependencies
   - Keep dependencies updated
   - Avoid unnecessary dependencies

3. **Testing**
   - Test encryption/decryption round-trip
   - Test error handling (wrong password)
   - Test memory cleanup

## Known Limitations

1. **No Protection Against:**
   - Keyloggers (password entry)
   - Screen capture (saat reveal)
   - Memory forensics (saat unlocked)
   - Physical access to unlocked vault

2. **Password Recovery:**
   - Tidak ada recovery jika lupa password
   - Security questions adalah fallback, bukan recovery

3. **Side Channels:**
   - Timing attacks partially mitigated
   - Power analysis tidak addressed

## Security Audit

Vault belum di-audit oleh pihak ketiga. Gunakan dengan risiko sendiri untuk data sangat sensitif.

### Recommendations

1. Untuk data sangat sensitif, gunakan hardware wallet atau HSM
2. Vault cocok untuk password management day-to-day
3. Pertimbangkan threat model Anda sebelum menyimpan data
