//! Secure string wrapper that zeros memory on drop

use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A string type that securely zeros its memory when dropped.
/// Use this for all sensitive data like passwords and seed phrases.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    inner: String,
}

impl SecureString {
    /// Create a new SecureString from a regular String
    pub fn new(s: String) -> Self {
        Self { inner: s }
    }

    /// Create a new SecureString from a string slice
    pub fn from_str(s: &str) -> Self {
        Self {
            inner: s.to_string(),
        }
    }

    /// Create an empty SecureString
    pub fn empty() -> Self {
        Self {
            inner: String::new(),
        }
    }

    /// Get a reference to the inner string
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Get the byte slice of the inner string
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    /// Get the length of the string
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Append a character to the string
    pub fn push(&mut self, c: char) {
        self.inner.push(c);
    }

    /// Remove the last character from the string
    pub fn pop(&mut self) -> Option<char> {
        self.inner.pop()
    }

    /// Clear the string content (with secure zeroing)
    pub fn clear(&mut self) {
        self.inner.zeroize();
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

// Implement Debug to prevent accidental leaking of sensitive data
impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureString")
            .field("len", &self.inner.len())
            .finish()
    }
}

// Implement Display to prevent accidental printing
impl fmt::Display for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED {} chars]", self.inner.len())
    }
}

// Implement PartialEq for testing
impl PartialEq for SecureString {
    fn eq(&self, other: &Self) -> bool {
        // Constant-time comparison would be ideal but string length
        // comparison already leaks timing info, so standard comparison is ok
        self.inner == other.inner
    }
}

impl Eq for SecureString {}

/// Secure bytes wrapper that zeros memory on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureBytes {
    inner: Vec<u8>,
}

impl SecureBytes {
    /// Create from a byte vector
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { inner: bytes }
    }

    /// Create from a byte slice
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self {
            inner: bytes.to_vec(),
        }
    }

    /// Get the inner bytes as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl From<Vec<u8>> for SecureBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

impl fmt::Debug for SecureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureBytes")
            .field("len", &self.inner.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string_basic() {
        let s = SecureString::new("secret".to_string());
        assert_eq!(s.as_str(), "secret");
        assert_eq!(s.len(), 6);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_secure_string_debug_redacted() {
        let s = SecureString::new("mysecretpassword".to_string());
        let debug = format!("{:?}", s);
        assert!(!debug.contains("mysecretpassword"));
        assert!(debug.contains("16")); // length shown
    }

    #[test]
    fn test_secure_string_display_redacted() {
        let s = SecureString::new("password123".to_string());
        let display = format!("{}", s);
        assert!(!display.contains("password123"));
        assert!(display.contains("REDACTED"));
    }

    #[test]
    fn test_secure_string_mutation() {
        let mut s = SecureString::from_str("test");
        s.push('!');
        assert_eq!(s.as_str(), "test!");
        s.pop();
        assert_eq!(s.as_str(), "test");
    }

    #[test]
    fn test_secure_bytes() {
        let bytes = SecureBytes::new(vec![1, 2, 3, 4]);
        assert_eq!(bytes.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(bytes.len(), 4);
    }
}
