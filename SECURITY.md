# Security Policy

## Overview

Vault is designed with security as the primary concern. This document outlines the security measures implemented and provides guidance for secure usage.

## Threat Model

### Protected Against

- **Data at Rest**: All vault data is encrypted with AES-256-GCM
- **Password Brute Force**: Argon2id KDF with tunable memory/time costs
- **Memory Scraping**: Sensitive data zeroized on drop using `zeroize` crate
- **Clipboard Exposure**: Auto-clear clipboard after configurable timeout
- **Shoulder Surfing**: Content masked by default with `****`

### Out of Scope

- **Physical Access**: If an attacker has physical access to an unlocked vault
- **Keyloggers**: Cannot protect against system-level key capture
- **Memory Forensics**: While we zeroize, cannot guarantee against swap/hibernation
- **Screen Capture**: Cannot prevent screenshots or screen recording

## Cryptographic Details

### Encryption

- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Size**: 256 bits
- **Nonce Size**: 96 bits (12 bytes), randomly generated per encryption
- **Tag Size**: 128 bits

### Key Derivation

- **Algorithm**: Argon2id (hybrid mode)
- **Default Parameters**:
  - Memory: 64 MiB
  - Iterations: 3
  - Parallelism: 4
- **Salt Size**: 256 bits (32 bytes), randomly generated per vault

### Keyfile (Optional)

- **Size**: 64 bytes (512 bits)
- **Purpose**: Combined with password for two-factor security
- **Generation**: Cryptographically secure random bytes

## Secure Usage Guidelines

### Password Selection

1. Use a unique password not used elsewhere
2. Minimum 12 characters recommended
3. Mix of uppercase, lowercase, numbers, and symbols
4. Consider using a passphrase

### Keyfile Usage

1. Store keyfile separately from vault file
2. Consider using a USB drive or secure storage
3. Make secure backups of the keyfile
4. If keyfile is lost, vault cannot be recovered

### Backup Strategy

1. Regularly backup your `.vault` files
2. Store backups in multiple locations
3. Test restoration periodically
4. Keyfiles must be backed up separately

### Operational Security

1. Never share your master password
2. Use auto-lock timeout (default: 5 minutes)
3. Manually lock (`Ctrl+L`) when stepping away
4. Close terminal when done
5. Consider full-disk encryption for vault storage

## Vulnerability Reporting

If you discover a security vulnerability, please:

1. **Do not** open a public issue
2. Email security concerns to the maintainer
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Audit Status

This codebase has not undergone a formal security audit. While best practices are followed, use at your own risk for high-value data.

## Dependencies

Security-critical dependencies:

| Crate | Purpose | Notes |
|-------|---------|-------|
| `aes-gcm` | Encryption | RustCrypto implementation |
| `argon2` | KDF | RustCrypto implementation |
| `zeroize` | Memory safety | Guarantees zeroing on drop |
| `rand` | RNG | OS-backed CSPRNG |

All cryptographic implementations are from the [RustCrypto](https://github.com/RustCrypto) project.

## Version History

| Version | Changes |
|---------|---------|
| 0.1.0 | Initial release |

## Questions

For security-related questions, please review this document first, then contact the maintainer if clarification is needed.
