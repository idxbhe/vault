//! Argon2id key derivation function

use std::thread;
use std::time::Instant;

use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::crypto::SecureString;
use crate::utils::error::{Error, Result};
use unicode_normalization::UnicodeNormalization;

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

    /// Calibrate parameters for the current hardware
    pub fn calibrate(target_ms: u128) -> Self {
        calibrate_argon2_params(target_ms)
    }
}

/// Generate a random 32-byte salt
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    use rand::RngCore;
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Calibrate Argon2 parameters based on current hardware performance
pub fn calibrate_argon2_params(target_ms: u128) -> Argon2Params {
    let mut params = Argon2Params {
        memory_kib: 16384, // Start with 16 MiB
        iterations: 1,
        parallelism: thread::available_parallelism()
            .map(|p| p.get() as u32)
            .unwrap_or(2),
    };

    // Safety limits
    const MAX_MEMORY_KIB: u32 = 524288; // 512 MiB
    const MAX_ITERATIONS: u32 = 10;
    const TOTAL_TIMEOUT_MS: u128 = 5000; // 5 seconds max for calibration total

    let start_calibration = Instant::now();
    let dummy_data = b"benchmark_password";
    let salt = [0u8; 32];
    let mut output = [0u8; 32];

    loop {
        let trial_start = Instant::now();

        let argon2_params = Params::new(
            params.memory_kib,
            params.iterations,
            params.parallelism,
            Some(32),
        )
        .unwrap_or_else(|_| Params::default());

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

        let _ = argon2.hash_password_into(dummy_data, &salt, &mut output);

        let elapsed = trial_start.elapsed().as_millis();

        // If we reached the target or hit global timeout/limits, we are done
        if elapsed >= target_ms
            || start_calibration.elapsed().as_millis() >= TOTAL_TIMEOUT_MS
            || (params.memory_kib >= MAX_MEMORY_KIB && params.iterations >= MAX_ITERATIONS)
        {
            break;
        }

        // Scaling logic:
        // 1. Try to increase memory first
        if params.memory_kib < MAX_MEMORY_KIB && elapsed < target_ms / 2 {
            params.memory_kib = (params.memory_kib * 2).min(MAX_MEMORY_KIB);
        }
        // 2. Then increase iterations
        else if params.iterations < MAX_ITERATIONS {
            params.iterations += 1;
        }
        // 3. Fine-tuning memory if we are close
        else if params.memory_kib < MAX_MEMORY_KIB {
            params.memory_kib = (params.memory_kib as f64 * 1.25) as u32;
            if params.memory_kib > MAX_MEMORY_KIB {
                params.memory_kib = MAX_MEMORY_KIB;
            }
        } else {
            break;
        }
    }

    params
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
    // 1. If keyfile is provided, hash it with SHA-256
    let keyfile_hash = if let Some(kf_data) = keyfile {
        let mut hasher = Sha256::new();
        hasher.update(kf_data);
        let result = hasher.finalize().to_vec();
        Some(result)
    } else {
        None
    };

    // 2. Combine password and keyfile hash: SHA256(password || keyfile_hash)
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    if let Some(ref kf_hash) = keyfile_hash {
        hasher.update(kf_hash);
    }
    let mut combined_material = hasher.finalize().to_vec();

    // Create Argon2id hasher
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        params.to_argon2_params()?,
    );

    // Derive the key using raw hash (not PHC string format)
    let mut output_key = [0u8; 32];
    argon2
        .hash_password_into(&combined_material, salt, &mut output_key)
        .map_err(|e| Error::KeyDerivation(e.to_string()))?;

    // Securely clear sensitive materials
    combined_material.zeroize();
    if let Some(mut kf_hash) = keyfile_hash {
        kf_hash.zeroize();
    }

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
        memory_kib: 65536, // 64 MiB - hardened for security questions
        iterations: 3,
        parallelism: 2,
    };

    // Normalize using Unicode Case Folding + NFKC
    let normalized = normalize_security_answer(answer.as_str());
    let normalized_ss = SecureString::from(normalized);

    let key = derive_key(&normalized_ss, None, &salt, &params)?;
    Ok((key.to_vec(), salt))
}

/// Verify a security question answer against stored hash
pub fn verify_security_answer(
    answer: &SecureString,
    stored_hash: &[u8],
    salt: &[u8; 32],
) -> Result<bool> {
    let params = Argon2Params {
        memory_kib: 65536,
        iterations: 3,
        parallelism: 2,
    };

    // Normalize using Unicode Case Folding + NFKC
    let normalized = normalize_security_answer(answer.as_str());
    let normalized_ss = SecureString::from(normalized);

    let derived = derive_key(&normalized_ss, None, salt, &params)?;

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

/// Normalize text using Unicode Case Folding and NFKC normalization.
/// This ensures consistent matching across different languages and visual representations.
fn normalize_security_answer(answer: &str) -> String {
    // Trim whitespace first (Unicode-aware trim is built-in to Rust's trim())
    let trimmed = answer.trim();
    
    // Standard Unicode approach for stable caseless matching:
    // NFKC -> Case Fold -> NFKC
    let nfkc1 = trimmed.nfkc().collect::<String>();
    let folded = caseless::default_case_fold_str(&nfkc1);
    folded.nfkc().collect::<String>()
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
        let answer = SecureString::from_str("My Pet Name");
        let (hash, salt) = hash_security_answer(&answer).unwrap();

        // Correct answer should verify (with same case/trim)
        assert!(verify_security_answer(&answer, &hash, &salt).unwrap());

        // Correct answer with different case and spaces should still verify
        let variations = vec![
            "my pet name",
            "MY PET NAME",
            "  My Pet Name  ",
            "my pet name  ",
        ];

        for var in variations {
            let var_ss = SecureString::from_str(var);
            assert!(
                verify_security_answer(&var_ss, &hash, &salt).unwrap(),
                "Should match for variation: '{}'",
                var
            );
        }

        // Wrong answer should not verify
        let wrong = SecureString::from_str("wrong answer");
        assert!(!verify_security_answer(&wrong, &hash, &salt).unwrap());
    }

    #[test]
    fn test_security_answer_international_chars() {
        // Turkish 'i' cases: İ (U+0130) vs i, I vs ı (U+0131)
        // Note: Standard Unicode Case Folding maps İ to i + combining dot (U+0307)
        // whereas to_lowercase() might behave differently depending on locale.
        let answer_tr = SecureString::from_str("İstanbul");
        let (hash, salt) = hash_security_answer(&answer_tr).unwrap();

        let variations = vec![
            "i\u{0307}stanbul", // correct lowercase (i + combining dot)
            "İSTANBUL",          // uppercase with Turkish dot
            "  İSTANBUL  ",      // with spaces
            // "istanbul" is NOT expected to match "İstanbul" under standard 
            // locale-independent Unicode case folding because İ maps to i+dot.
        ];

        for var in variations {
            let var_ss = SecureString::from_str(var);
            assert!(
                verify_security_answer(&var_ss, &hash, &salt).unwrap(),
                "Turkish variation should match: '{}'",
                var
            );
        }

        // Cyrillic cases
        let answer_cy = SecureString::from_str("Москва"); // Moscow
        let (hash_cy, salt_cy) = hash_security_answer(&answer_cy).unwrap();
        
        let var_cy = SecureString::from_str("москва");
        assert!(verify_security_answer(&var_cy, &hash_cy, &salt_cy).unwrap());

        // Arabic / Ligatures
        // 'ﬁ' (U+FB01) should match 'fi' due to NFKC
        let answer_lig = SecureString::from_str("ﬁle"); 
        let (hash_lig, salt_lig) = hash_security_answer(&answer_lig).unwrap();
        
        let var_lig = SecureString::from_str("File");
        assert!(verify_security_answer(&var_lig, &hash_lig, &salt_lig).unwrap());
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

    #[test]
    fn test_calibrate_argon2_params() {
        // Test with a low target to ensure it completes quickly
        let target_ms = 50;
        let params = calibrate_argon2_params(target_ms);

        assert!(params.memory_kib >= 16384);
        assert!(params.iterations >= 1);
        assert!(params.parallelism >= 1);

        // Ensure it doesn't exceed safety limits
        assert!(params.memory_kib <= 524288);
        assert!(params.iterations <= 10);
    }

    #[test]
    fn test_argon2_params_calibrate_method() {
        let params = Argon2Params::calibrate(30);
        assert!(params.memory_kib >= 16384);
    }
}
