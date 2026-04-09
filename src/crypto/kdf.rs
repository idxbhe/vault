//! Argon2id key derivation function

use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::crypto::SecureString;
use crate::utils::error::{Error, Result};

/// Argon2 parameters for key derivation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Argon2Params {
    /// Memory cost in KiB (default: 64 MiB = 65536 KiB)
    pub memory_kib: u32,
    /// Number of iterations (default: 3)
    pub iterations: u32,
    /// Degree of parallelism (default: 4)
    pub parallelism: u32,
}

impl Default for Argon2Params {
    fn default() -> Self {
        // OWASP recommended minimum parameters for Argon2id
        // Tuned for ~0.5 second derivation on modern hardware
        Self {
            memory_kib: 65536, // 64 MiB
            iterations: 3,
            parallelism: 4,
        }
    }
}

impl Argon2Params {
    /// Create faster parameters for testing
    #[cfg(test)]
    pub fn fast_for_testing() -> Self {
        Self {
            memory_kib: 1024, // 1 MiB
            iterations: 1,
            parallelism: 1,
        }
    }

    /// Convert to argon2 crate Params
    fn to_argon2_params(&self) -> Result<Params> {
        Params::new(
            self.memory_kib,
            self.iterations,
            self.parallelism,
            Some(32), // 256-bit output
        )
        .map_err(|e| Error::KeyDerivation(e.to_string()))
    }
}

/// Generate a random 32-byte salt
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    use rand::RngCore;
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Derive a 256-bit encryption key from password, optional keyfile, and salt
///
/// The key derivation process:
/// 1. If keyfile is provided, XOR it with the password bytes (or append if lengths differ)
/// 2. Use Argon2id to derive a 32-byte key
pub fn derive_key(
    password: &SecureString,
    keyfile: Option<&[u8]>,
    salt: &[u8; 32],
    params: &Argon2Params,
) -> Result<[u8; 32]> {
    // Combine password and keyfile material
    let mut key_material = password.as_bytes().to_vec();

    if let Some(kf_data) = keyfile {
        // XOR keyfile with password, extending if keyfile is longer
        for (i, &kf_byte) in kf_data.iter().enumerate() {
            if i < key_material.len() {
                key_material[i] ^= kf_byte;
            } else {
                key_material.push(kf_byte);
            }
        }
    }

    // Create Argon2id hasher
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        params.to_argon2_params()?,
    );

    // Derive the key using raw hash (not PHC string format)
    let mut output_key = [0u8; 32];
    argon2
        .hash_password_into(&key_material, salt, &mut output_key)
        .map_err(|e| Error::KeyDerivation(e.to_string()))?;

    // Securely clear the key material
    key_material.zeroize();

    Ok(output_key)
}

/// Derive key and return additional salt (for new vault creation)
pub fn derive_key_with_new_salt(
    password: &SecureString,
    keyfile: Option<&[u8]>,
    params: &Argon2Params,
) -> Result<([u8; 32], [u8; 32])> {
    let salt = generate_salt();
    let key = derive_key(password, keyfile, &salt, params)?;
    Ok((key, salt))
}

/// Hash a security question answer for storage
/// Returns (hash, salt)
pub fn hash_security_answer(answer: &SecureString) -> Result<(Vec<u8>, [u8; 32])> {
    let salt = generate_salt();
    let params = Argon2Params {
        memory_kib: 16384, // 16 MiB - faster for security questions
        iterations: 2,
        parallelism: 2,
    };

    let key = derive_key(answer, None, &salt, &params)?;
    Ok((key.to_vec(), salt))
}

/// Verify a security question answer against stored hash
pub fn verify_security_answer(
    answer: &SecureString,
    stored_hash: &[u8],
    salt: &[u8; 32],
) -> Result<bool> {
    let params = Argon2Params {
        memory_kib: 16384,
        iterations: 2,
        parallelism: 2,
    };

    let derived = derive_key(answer, None, salt, &params)?;

    // Constant-time comparison
    Ok(constant_time_compare(&derived, stored_hash))
}

/// Constant-time byte comparison to prevent timing attacks
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        assert_ne!(salt1, salt2);
        assert_eq!(salt1.len(), 32);
    }

    #[test]
    fn test_derive_key_basic() {
        let password = SecureString::from_str("test_password");
        let salt = generate_salt();
        let params = Argon2Params::fast_for_testing();

        let key = derive_key(&password, None, &salt, &params).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let password = SecureString::from_str("test_password");
        let salt = [42u8; 32];
        let params = Argon2Params::fast_for_testing();

        let key1 = derive_key(&password, None, &salt, &params).unwrap();
        let key2 = derive_key(&password, None, &salt, &params).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_with_keyfile() {
        let password = SecureString::from_str("test_password");
        let keyfile = vec![1, 2, 3, 4, 5];
        let salt = [42u8; 32];
        let params = Argon2Params::fast_for_testing();

        let key_without_kf = derive_key(&password, None, &salt, &params).unwrap();
        let key_with_kf = derive_key(&password, Some(&keyfile), &salt, &params).unwrap();

        assert_ne!(key_without_kf, key_with_kf);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let password1 = SecureString::from_str("password1");
        let password2 = SecureString::from_str("password2");
        let salt = [42u8; 32];
        let params = Argon2Params::fast_for_testing();

        let key1 = derive_key(&password1, None, &salt, &params).unwrap();
        let key2 = derive_key(&password2, None, &salt, &params).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_security_answer_hash_verify() {
        let answer = SecureString::from_str("my pet name");
        let (hash, salt) = hash_security_answer(&answer).unwrap();

        // Correct answer should verify
        assert!(verify_security_answer(&answer, &hash, &salt).unwrap());

        // Wrong answer should not verify
        let wrong = SecureString::from_str("wrong answer");
        assert!(!verify_security_answer(&wrong, &hash, &salt).unwrap());
    }

    #[test]
    fn test_constant_time_compare() {
        let a = [1, 2, 3, 4];
        let b = [1, 2, 3, 4];
        let c = [1, 2, 3, 5];
        let d = [1, 2, 3];

        assert!(constant_time_compare(&a, &b));
        assert!(!constant_time_compare(&a, &c));
        assert!(!constant_time_compare(&a, &d));
    }
}
