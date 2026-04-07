//! Item data model - the core unit stored in a vault

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::history::HistoryEntry;

/// A single entry in the vault (password, seed phrase, note, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier
    pub id: Uuid,
    /// Display title
    pub title: String,
    /// Type of item (determines UI and validation)
    pub kind: ItemKind,
    /// The actual content (type-specific)
    pub content: ItemContent,
    /// Optional notes/comments
    pub notes: Option<String>,
    /// Associated tag IDs
    pub tags: Vec<Uuid>,
    /// Marked as favorite for quick access
    pub favorite: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
    /// Edit history for undo/redo
    pub history: Vec<HistoryEntry>,
}

impl Item {
    /// Create a new item with generated UUID and timestamps
    pub fn new(title: impl Into<String>, kind: ItemKind, content: ItemContent) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            kind,
            content,
            notes: None,
            tags: Vec::new(),
            favorite: false,
            created_at: now,
            updated_at: now,
            history: Vec::new(),
        }
    }

    /// Create a generic item with simple content
    pub fn generic(title: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(
            title,
            ItemKind::Generic,
            ItemContent::Generic {
                value: value.into(),
            },
        )
    }

    /// Create a crypto seed item
    pub fn crypto_seed(
        title: impl Into<String>,
        seed_phrase: impl Into<String>,
    ) -> Self {
        Self::new(
            title,
            ItemKind::CryptoSeed,
            ItemContent::CryptoSeed {
                seed_phrase: seed_phrase.into(),
                derivation_path: None,
                network: None,
            },
        )
    }

    /// Create a password item
    pub fn password(
        title: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self::new(
            title,
            ItemKind::Password,
            ItemContent::Password {
                username: None,
                password: password.into(),
                url: None,
                totp_secret: None,
            },
        )
    }

    /// Create a secure note
    pub fn secure_note(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self::new(
            title,
            ItemKind::SecureNote,
            ItemContent::SecureNote {
                content: content.into(),
            },
        )
    }

    /// Create an API key item
    pub fn api_key(title: impl Into<String>, key: impl Into<String>) -> Self {
        Self::new(
            title,
            ItemKind::ApiKey,
            ItemContent::ApiKey {
                key: key.into(),
                service: None,
                expires_at: None,
            },
        )
    }

    /// Set notes
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag_id: Uuid) -> Self {
        self.tags.push(tag_id);
        self
    }

    /// Set as favorite
    pub fn with_favorite(mut self, favorite: bool) -> Self {
        self.favorite = favorite;
        self
    }

    /// Update the item and set updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Get the primary sensitive content for copying
    pub fn get_copyable_content(&self) -> Option<&str> {
        match &self.content {
            ItemContent::Generic { value } => Some(value),
            ItemContent::CryptoSeed { seed_phrase, .. } => Some(seed_phrase),
            ItemContent::Password { password, .. } => Some(password),
            ItemContent::SecureNote { content } => Some(content),
            ItemContent::ApiKey { key, .. } => Some(key),
        }
    }
}

/// Types of items that can be stored
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// Generic key-value storage
    #[default]
    Generic,
    /// Cryptocurrency seed phrase
    CryptoSeed,
    /// Login credentials
    Password,
    /// Encrypted text note
    SecureNote,
    /// API key or token
    ApiKey,
}

impl ItemKind {
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            ItemKind::Generic => "Generic",
            ItemKind::CryptoSeed => "Crypto Seed",
            ItemKind::Password => "Password",
            ItemKind::SecureNote => "Secure Note",
            ItemKind::ApiKey => "API Key",
        }
    }

    /// Get icon for UI (Nerd Font)
    pub fn icon(&self) -> &'static str {
        match self {
            ItemKind::Generic => "󰈔",
            ItemKind::CryptoSeed => "󰞃",
            ItemKind::Password => "󰌋",
            ItemKind::SecureNote => "󱞂",
            ItemKind::ApiKey => "󰯄",
        }
    }

    /// Get all available item kinds
    pub fn all() -> &'static [ItemKind] {
        &[
            ItemKind::Generic,
            ItemKind::CryptoSeed,
            ItemKind::Password,
            ItemKind::SecureNote,
            ItemKind::ApiKey,
        ]
    }

    /// Get default content for this item kind
    pub fn default_content(&self) -> ItemContent {
        match self {
            ItemKind::Generic => ItemContent::Generic {
                value: String::new(),
            },
            ItemKind::CryptoSeed => ItemContent::CryptoSeed {
                seed_phrase: String::new(),
                derivation_path: None,
                network: None,
            },
            ItemKind::Password => ItemContent::Password {
                username: None,
                password: String::new(),
                url: None,
                totp_secret: None,
            },
            ItemKind::SecureNote => ItemContent::SecureNote {
                content: String::new(),
            },
            ItemKind::ApiKey => ItemContent::ApiKey {
                key: String::new(),
                service: None,
                expires_at: None,
            },
        }
    }
}

/// Type-specific content for items
/// Note: Uses default externally-tagged format for bincode compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemContent {
    /// Generic: simple value
    Generic {
        value: String,
    },

    /// Crypto seed phrase with optional metadata
    CryptoSeed {
        seed_phrase: String,
        derivation_path: Option<String>,
        network: Option<String>,
    },

    /// Password entry with login details
    Password {
        username: Option<String>,
        password: String,
        url: Option<String>,
        totp_secret: Option<String>,
    },

    /// Secure note (potentially markdown)
    SecureNote {
        content: String,
    },

    /// API key or token
    ApiKey {
        key: String,
        service: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    },
}

impl Default for ItemContent {
    fn default() -> Self {
        ItemContent::Generic {
            value: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_generic() {
        let item = Item::generic("My Secret", "super_secret_value");
        
        assert_eq!(item.title, "My Secret");
        assert_eq!(item.kind, ItemKind::Generic);
        assert_eq!(item.get_copyable_content(), Some("super_secret_value"));
    }

    #[test]
    fn test_item_crypto_seed() {
        let item = Item::crypto_seed(
            "Bitcoin Wallet",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        );
        
        assert_eq!(item.kind, ItemKind::CryptoSeed);
        assert!(item.get_copyable_content().unwrap().contains("abandon"));
    }

    #[test]
    fn test_item_password() {
        let item = Item::password("GitHub", "my_password123")
            .with_notes("Main account")
            .with_favorite(true);
        
        assert_eq!(item.kind, ItemKind::Password);
        assert!(item.favorite);
        assert_eq!(item.notes, Some("Main account".to_string()));
    }

    #[test]
    fn test_item_kind_display() {
        assert_eq!(ItemKind::CryptoSeed.display_name(), "Crypto Seed");
        assert_eq!(ItemKind::Password.icon(), "󰌋");
    }
}
