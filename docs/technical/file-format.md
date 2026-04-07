# 📄 Format File Vault

Spesifikasi lengkap format file `.vault`.

## Overview

File vault menggunakan format binary custom yang terdiri dari:

1. **Magic bytes** - Identifikasi tipe file
2. **Version** - Versi format
3. **Header** - Metadata tidak terenkripsi
4. **Encrypted payload** - Data vault terenkripsi

## Binary Layout

```
┌──────────────────────────────────────────────────────────────┐
│                        VAULT FILE                             │
├──────────────────────────────────────────────────────────────┤
│ Offset   │ Size     │ Field           │ Description          │
├──────────┼──────────┼─────────────────┼──────────────────────┤
│ 0x00     │ 4 bytes  │ Magic           │ "VALT" (0x56414C54)  │
│ 0x04     │ 2 bytes  │ Version         │ Format version (LE)  │
│ 0x06     │ 4 bytes  │ Header Length   │ Size of header (LE)  │
│ 0x0A     │ variable │ Header          │ Bincode-serialized   │
│ var      │ 12 bytes │ Nonce           │ AES-GCM nonce        │
│ var      │ 32 bytes │ Salt            │ Argon2 salt          │
│ var      │ variable │ Argon2 Params   │ KDF parameters       │
│ var      │ rest     │ Ciphertext      │ Encrypted + Auth Tag │
└──────────────────────────────────────────────────────────────┘
```

## Field Details

### Magic Bytes (4 bytes)

```
Bytes: 0x56 0x41 0x4C 0x54
ASCII: "VALT"
```

Digunakan untuk identifikasi cepat tipe file.

### Version (2 bytes)

```
Current: 0x0001 (version 1)
Format: Little-endian u16
```

Memungkinkan backward compatibility untuk versi masa depan.

### Header Length (4 bytes)

```
Format: Little-endian u32
Range: 0 - 4,294,967,295 bytes
```

Ukuran header dalam bytes, memungkinkan parser untuk skip ke encrypted section.

### Header (Variable)

Serialized menggunakan bincode format:

```rust
#[derive(Serialize, Deserialize)]
pub struct VaultHeader {
    /// Unique identifier for this vault
    pub vault_id: Uuid,          // 16 bytes
    
    /// Human-readable name
    pub vault_name: String,      // variable
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>, // 12 bytes
    
    /// Optional security questions for recovery
    pub security_questions: Option<Vec<SecurityQuestion>>,
}

#[derive(Serialize, Deserialize)]
pub struct SecurityQuestion {
    pub question: String,
    pub answer_hash: [u8; 32],  // SHA-256 of answer
}
```

**Note:** Header TIDAK terenkripsi. Jangan simpan data sensitif di sini.

### Nonce (12 bytes)

```
Purpose: AES-256-GCM initialization vector
Generation: Cryptographically secure random
Uniqueness: Must be unique per encryption
```

### Salt (32 bytes)

```
Purpose: Argon2id salt for key derivation
Generation: Cryptographically secure random
Storage: Stored with file, regenerated only on password change
```

### Argon2 Parameters (Variable)

```rust
#[derive(Serialize, Deserialize)]
pub struct Argon2Params {
    /// Memory size in KB
    pub memory_kb: u32,      // Default: 65536 (64 MB)
    
    /// Number of iterations
    pub iterations: u32,     // Default: 3
    
    /// Parallelism factor
    pub parallelism: u32,    // Default: 4
    
    /// Output length in bytes
    pub output_len: u32,     // Fixed: 32
}
```

### Ciphertext (Variable)

```
Content: AES-256-GCM encrypted VaultPayload
Format: [encrypted_data][16-byte auth tag]
```

## Encrypted Payload

Setelah dekripsi, payload berisi:

```rust
#[derive(Serialize, Deserialize)]
pub struct VaultPayload {
    /// All items in the vault
    pub items: Vec<Item>,
    
    /// User-defined tags
    pub tags: Vec<Tag>,
    
    /// Last modification time
    pub updated_at: DateTime<Utc>,
}
```

### Item Structure

```rust
#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub title: String,
    pub kind: ItemKind,
    pub content: ItemContent,
    pub tags: Vec<Uuid>,
    pub favorite: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub enum ItemKind {
    Generic,
    CryptoSeed,
    Password,
    SecureNote,
    ApiKey,
}

#[derive(Serialize, Deserialize)]
pub enum ItemContent {
    Generic {
        value: String,
    },
    CryptoSeed {
        seed_phrase: String,
        derivation_path: String,
        passphrase: Option<String>,
    },
    Password {
        username: String,
        password: String,
        url: Option<String>,
        totp_secret: Option<String>,
    },
    SecureNote {
        content: String,
    },
    ApiKey {
        key: String,
        secret: Option<String>,
        endpoint: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    },
}
```

### Tag Structure

```rust
#[derive(Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
}
```

## Encryption Process

### Write (Encrypt)

```
1. Serialize VaultPayload to bincode bytes
2. Generate random 32-byte salt (if new) 
3. Generate random 12-byte nonce
4. Derive 256-bit key: Argon2id(password, salt)
5. Encrypt: AES-256-GCM(plaintext, key, nonce)
6. Build header and write file
```

```rust
pub fn write_vault(vault: &Vault, password: &str, path: &Path) -> Result<()> {
    // 1. Prepare payload
    let payload = VaultPayload {
        items: vault.items.clone(),
        tags: vault.tags.clone(),
        updated_at: Utc::now(),
    };
    let plaintext = bincode::serialize(&payload)?;
    
    // 2. Generate cryptographic material
    let salt = generate_salt();
    let nonce = generate_nonce();
    
    // 3. Derive key
    let key = derive_key(password.as_bytes(), &salt)?;
    
    // 4. Encrypt
    let ciphertext = encrypt(&plaintext, &key, &nonce)?;
    
    // 5. Build header
    let header = VaultHeader {
        vault_id: vault.id,
        vault_name: vault.name.clone(),
        created_at: vault.created_at,
        security_questions: None,
    };
    
    // 6. Write file
    let mut file = File::create(path)?;
    file.write_all(MAGIC)?;
    file.write_all(&VERSION.to_le_bytes())?;
    // ... write rest
    
    Ok(())
}
```

### Read (Decrypt)

```
1. Read and validate magic bytes
2. Read version and header
3. Read nonce, salt, argon2 params
4. Read ciphertext
5. Derive key: Argon2id(password, salt)
6. Decrypt: AES-256-GCM(ciphertext, key, nonce)
7. Deserialize VaultPayload from bincode
```

```rust
pub fn read_vault(path: &Path, password: &str) -> Result<Vault> {
    let mut file = File::open(path)?;
    
    // 1. Validate magic
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(Error::InvalidFormat);
    }
    
    // 2. Read version
    let mut version_bytes = [0u8; 2];
    file.read_exact(&mut version_bytes)?;
    let version = u16::from_le_bytes(version_bytes);
    
    // ... read header, nonce, salt, params, ciphertext
    
    // 5. Derive key
    let key = derive_key(password.as_bytes(), &salt)?;
    
    // 6. Decrypt
    let plaintext = decrypt(&ciphertext, &key, &nonce)?;
    
    // 7. Deserialize
    let payload: VaultPayload = bincode::deserialize(&plaintext)?;
    
    Ok(Vault {
        id: header.vault_id,
        name: header.vault_name,
        items: payload.items,
        tags: payload.tags,
        created_at: header.created_at,
        updated_at: payload.updated_at,
    })
}
```

## Error Handling

| Error | Cause | User Message |
|-------|-------|--------------|
| InvalidMagic | Wrong file type | "Not a valid vault file" |
| UnsupportedVersion | Future version | "Vault version not supported" |
| DecryptionFailed | Wrong password | "Invalid password" |
| CorruptedData | File damaged | "Vault file is corrupted" |
| IOError | Disk error | "Failed to read/write file" |

## Version History

| Version | Changes |
|---------|---------|
| 1 | Initial release |

## Compatibility

- **Forward**: Old versions cannot read new formats
- **Backward**: New versions can read old formats (planned)

## Security Considerations

1. **Header Exposure**: Vault name visible without password
2. **No Plausible Deniability**: Magic bytes identify file type
3. **Metadata Leakage**: File size reveals approximate content size

## File Extension

- **Primary**: `.vault`
- **Export**: `.json` (unencrypted)
