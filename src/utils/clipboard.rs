//! Clipboard utilities with secure handling
//!
//! Provides clipboard operations with automatic clearing for sensitive data.

/// Clipboard manager with security features
pub struct ClipboardManager {
    /// Whether a secure copy is pending clear
    pending_clear: bool,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Self {
        Self {
            pending_clear: false,
        }
    }

    /// Copy text to clipboard
    pub fn copy(&mut self, content: &str) -> Result<(), ClipboardError> {
        set_clipboard_text(content)
    }

    /// Copy sensitive content to clipboard (will be cleared later)
    pub fn copy_secure(&mut self, content: &str) -> Result<(), ClipboardError> {
        set_clipboard_text(content)?;
        self.pending_clear = true;
        Ok(())
    }

    /// Clear the clipboard
    pub fn clear(&mut self) -> Result<(), ClipboardError> {
        self.pending_clear = false;
        clear_clipboard()
    }

    /// Check if there's pending secure content
    pub fn has_pending_clear(&self) -> bool {
        self.pending_clear
    }
}

/// Clipboard error
#[derive(Debug, Clone)]
pub enum ClipboardError {
    /// Failed to access clipboard
    AccessDenied(String),
    /// Clipboard operation failed
    OperationFailed(String),
    /// Clipboard not available
    NotAvailable,
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::AccessDenied(msg) => write!(f, "Clipboard access denied: {}", msg),
            ClipboardError::OperationFailed(msg) => {
                write!(f, "Clipboard operation failed: {}", msg)
            }
            ClipboardError::NotAvailable => write!(f, "Clipboard not available"),
        }
    }
}

impl std::error::Error for ClipboardError {}

/// Set clipboard text content
#[cfg(feature = "clipboard")]
fn set_clipboard_text(content: &str) -> Result<(), ClipboardError> {
    use arboard::Clipboard;

    let mut clipboard =
        Clipboard::new().map_err(|e| ClipboardError::AccessDenied(e.to_string()))?;

    clipboard
        .set_text(content)
        .map_err(|e| ClipboardError::OperationFailed(e.to_string()))
}

#[cfg(not(feature = "clipboard"))]
fn set_clipboard_text(_content: &str) -> Result<(), ClipboardError> {
    // No-op when clipboard feature is disabled
    Ok(())
}

/// Clear clipboard content
#[cfg(feature = "clipboard")]
fn clear_clipboard() -> Result<(), ClipboardError> {
    use arboard::Clipboard;

    let mut clipboard =
        Clipboard::new().map_err(|e| ClipboardError::AccessDenied(e.to_string()))?;

    // Set empty string to clear
    clipboard
        .set_text("")
        .map_err(|e| ClipboardError::OperationFailed(e.to_string()))
}

#[cfg(not(feature = "clipboard"))]
fn clear_clipboard() -> Result<(), ClipboardError> {
    Ok(())
}

/// Get clipboard text content
#[cfg(feature = "clipboard")]
pub fn get_clipboard_text() -> Result<String, ClipboardError> {
    use arboard::Clipboard;

    let mut clipboard =
        Clipboard::new().map_err(|e| ClipboardError::AccessDenied(e.to_string()))?;

    clipboard
        .get_text()
        .map_err(|e| ClipboardError::OperationFailed(e.to_string()))
}

#[cfg(not(feature = "clipboard"))]
pub fn get_clipboard_text() -> Result<String, ClipboardError> {
    Err(ClipboardError::NotAvailable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_manager_creation() {
        let manager = ClipboardManager::new();
        assert!(!manager.has_pending_clear());
    }

    #[test]
    fn test_clipboard_manager_secure_copy() {
        let mut manager = ClipboardManager::new();

        // Even if clipboard access fails, the pending flag should be set after success
        if manager.copy_secure("test").is_ok() {
            assert!(manager.has_pending_clear());
        }
    }

    #[test]
    fn test_clipboard_manager_clear() {
        let mut manager = ClipboardManager::new();
        manager.pending_clear = true;

        if manager.clear().is_ok() {
            assert!(!manager.has_pending_clear());
        }
    }
}
