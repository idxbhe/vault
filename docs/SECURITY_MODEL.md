# Security Model

## Threat Model

### Assets We Protect
- User passwords and secrets stored in vault
- Private cryptographic keys
- Seed phrases for cryptocurrency wallets
- Sensitive notes

### Threats We Defend Against
1. **Weak passwords** → Solved by Argon2id key derivation
2. **Brute force attacks** → Solved by key derivation cost (1-3 sec/attempt)
3. **Plaintext data in memory** → Solved by SecureString (volatile_memzero)
4. **File corruption** → Solved by AEAD authentication tags
5. **Salt regeneration** → Solved by storing salt in VaultState
6. **Weak RNG** → Using getrandom() for IV and salt

### Threats Out of Scope
- Malware on system (can read decrypted vault after unlock)
- Physical memory attacks (cold boot attacks)
- Keyboard sniffing (malware can capture password input)
- Timing attacks (implementation not constant-time)
- Metadata leaks (file size reveals approx. number of items)

## Encryption Details

### Key Derivation - Argon2id

```
Input: password (SecureString), salt (32 bytes)
Output: 256-bit key

Parameters:
  - Time cost: 2 iterations (m_cost = 19)
  - Memory cost: 19 MiB
  - Parallelism: 1 thread
  - Algorithm: Argon2id (hybrid: data-independent + data-dependent)

Result: ~1-3 seconds per derivation
  Purpose: Make brute force expensive
```

### Symmetric Encryption - AES-256-GCM

```
Input: vault_bytes (plaintext), key (256-bit), salt (32 bytes)
Output: encrypted_payload = { salt, iv, ciphertext, tag }

Algorithm: AES-256-GCM (Galois/Counter Mode)
  - Block size: 128 bits
  - Key size: 256 bits
  - IV size: 96 bits (random, never repeated with same key)
  - Tag size: 128 bits (authentication)

Why GCM?
  ✓ Authenticated encryption (detects tampering)
  ✓ Parallel encryption (fast)
  ✓ Hardware-accelerated (AES-NI)
  ✓ Industry standard (NIST SP 800-38D)
```

## Secure String Handling

### Problem
Sensitive data (passwords, decrypted vaults) can leak:
- Garbage collection doesn't zero memory
- Data can be read from swap, crash dumps, memory debuggers

### Solution - SecureString

```rust
pub struct SecureString {
    bytes: Vec<u8>,
    // Zeroized on drop via volatile_memzero
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Volatile write ensures compiler doesn't optimize away
        volatile_memzero(&mut self.bytes);
    }
}
```

### Usage
- Password input stored in SecureString
- Passed to key derivation
- Automatically zeroed after use

## Vault State Memory

### Decrypted Vault in Memory

After unlock, vault exists in memory:
- `VaultState.vault` - full decrypted vault
- `VaultState.encryption_key` - derived key
- `VaultState.salt` - for re-encryption

### When Does It Get Cleared?

```
Lock Vault:
  ↓
Update handler SetLocked effect
  ↓
Write vault to disk (if dirty)
  ↓
Clear VaultState (drop)
  ↓
Vault bytes explicitly zeroized? NO (Rust does not auto-zero)
  ↓
⚠️ ISSUE: Decrypted vault remains in memory until GC/reuse
```

### Mitigation (Current)
1. Lock before exiting (`q` key)
2. Vault only decrypted while in use
3. Locking writes to disk, clears state

### Future: Authenticated Encryption on Unlock
- Decrypt in memory
- Use SecureVault wrapper that zeros on drop
- Re-encrypt before every write

## File Security

### On Disk

**VaultFile (encrypted)**
```
Path: ~/.vault/mypriv.vlt (or user-specified)
Permissions: 600 (user read/write only)
Format: JSON with encrypted payload
```

**Permissions**
- Should be 0600 (read/write for owner only)
- Currently: User responsibility (TODO: enforce)

### Keyfile (Optional)

```
Path: ~/.vault/mypriv.key (same directory as vault)
Permissions: 600 (user read/write only)
Content: 64 hex characters (256-bit random key)

Purpose:
  - Extra security layer beyond password
  - Key mixed with password during derivation
  - If keyfile stolen but vault not, vault still protected by password
```

## Audit Trail

### What We Log
- ✅ Errors (wrong password, I/O failures)
- ✅ Key operations (vault created, items saved)

### What We Don't Log
- ❌ Sensitive data (passwords, vault contents)
- ❌ User input details
- ❌ Decrypted vault in log files

### Current Implementation
```rust
// ERROR level only - no debug spam
tracing::error!("Failed to decrypt vault: {}", err);  // ✓ Safe

// NEVER log passwords!
tracing::error!("Password: {}", password);  // ❌ Dangerous
```

## Testing Security

### Test Vaults
- `test_vault.vault` - password: "testpass123"
- `test2.vault` - password: "sudounlock"
- Intentionally weak passwords for easy testing

### Test Coverage
```
✓ Correct password unlocks vault
✓ Wrong password fails to decrypt
✓ Salt preserved across saves
✓ Corrupted vault detected (invalid tag)
✓ Item data persists after lock/unlock
```

### What's NOT Tested
- Memory zeroization (cannot reliably test in Rust)
- Timing attacks (outside scope)
- Side-channel attacks (outside scope)

## Compliance & Standards

### Cryptographic Standards
- **AES-256-GCM**: NIST SP 800-38D
- **Argon2id**: Password hashing (PHC winner 2015)
- **PBKDF2-compatible**: Potential future use

### Industry Best Practices
- ✓ Authenticated encryption (AEAD)
- ✓ Proper key derivation (Argon2id with salt)
- ✓ Random IVs (never repeated)
- ✓ Secure random generation (getrandom)
- ✓ No custom cryptography

### Known Limitations
- ⚠️ No perfect forward secrecy (single master key)
- ⚠️ No session timeout (app always has decrypted vault)
- ⚠️ No multi-device sync (local only)
- ⚠️ No audit log of access

## Recommendations for Users

### Strong Practices
1. ✓ Use strong passwords (20+ chars with mix of types)
2. ✓ Lock vault when away (`q` key)
3. ✓ Keep vault file backed up
4. ✓ Consider using keyfile for extra security
5. ✓ Update regularly to get security fixes

### Weak Practices to Avoid
1. ✗ Reusing same password across vaults
2. ✗ Leaving app unlocked while away
3. ✗ Storing vault on shared computers
4. ✗ Backing up vault without encryption
5. ✗ Using simple passwords

## Security Incidents - Response Plan

### If Vault File Compromised
1. **Attacker has**: Encrypted vault file
2. **Attacker cannot**: Decrypt without password
3. **Your action**: Change password (not supported yet - TODO)

### If Password Leaked
1. **Attacker has**: Password (plaintext)
2. **Attacker needs**: Vault file (encrypted)
3. **Your action**: Move vault file, create new one

### If Keyfile Compromised
1. **Attacker has**: Keyfile (256-bit key)
2. **Attacker needs**: Password + vault file
3. **Your action**: Regenerate keyfile (current: manual process)

## Future Security Improvements

1. **Master Password + Session Timeout**
   - Lock after 5 min inactivity
   - Require re-entry of master password

2. **Encrypted Backups**
   - Sync to cloud with client-side encryption
   - Conflict resolution for multi-device

3. **Audit Log**
   - Record all access attempts
   - Timestamp, source, action
   - Encrypted, immutable

4. **Hardware Security Keys**
   - FIDO2 integration
   - Or TPM for local key storage

5. **Atomic Writes**
   - fsync() before confirming write
   - Recover from incomplete writes

6. **Constant-Time Operations**
   - Prevent timing attacks on decryption
   - Would require crypto library changes

## References

- NIST SP 800-132: PBKDF2
- NIST SP 800-38D: GCM Mode
- RFC 7914: Argon2 Algorithm
- Ratatui Security: https://docs.rs/ratatui/
- Zeroize: https://docs.rs/zeroize/
