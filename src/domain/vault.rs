//! Vault data model - the root container for all items

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::item::{Item, ItemKind};
use super::security_question::SecurityQuestion;
use super::tag::Tag;

/// The root vault structure containing all items and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    /// Unique identifier
    pub id: Uuid,
    /// Vault name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
    /// All items in the vault
    pub items: Vec<Item>,
    /// All tags defined in the vault
    pub tags: Vec<Tag>,
    /// Vault-specific settings
    pub settings: VaultSettings,
    /// Security questions for password recovery (up to 3)
    pub security_questions: Vec<SecurityQuestion>,
}

impl Vault {
    /// Create a new empty vault
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            created_at: now,
            updated_at: now,
            items: Vec::new(),
            tags: Vec::new(),
            settings: VaultSettings::default(),
            security_questions: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add an item to the vault
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
        self.touch();
    }

    /// Remove an item by ID
    pub fn remove_item(&mut self, id: Uuid) -> Option<Item> {
        if let Some(pos) = self.items.iter().position(|i| i.id == id) {
            self.touch();
            Some(self.items.remove(pos))
        } else {
            None
        }
    }

    /// Get an item by ID
    pub fn get_item(&self, id: Uuid) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Get a mutable item by ID
    pub fn get_item_mut(&mut self, id: Uuid) -> Option<&mut Item> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Add a tag to the vault
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
        self.touch();
    }

    /// Remove a tag by ID (also removes from all items)
    pub fn remove_tag(&mut self, id: Uuid) -> Option<Tag> {
        // Remove tag from all items
        for item in &mut self.items {
            item.tags.retain(|&t| t != id);
        }

        // Remove the tag itself
        if let Some(pos) = self.tags.iter().position(|t| t.id == id) {
            self.touch();
            Some(self.tags.remove(pos))
        } else {
            None
        }
    }

    /// Get a tag by ID
    pub fn get_tag(&self, id: Uuid) -> Option<&Tag> {
        self.tags.iter().find(|t| t.id == id)
    }

    /// Get items with a specific tag
    pub fn items_with_tag(&self, tag_id: Uuid) -> Vec<&Item> {
        self.items
            .iter()
            .filter(|i| i.tags.contains(&tag_id))
            .collect()
    }

    /// Get favorite items
    pub fn favorite_items(&self) -> Vec<&Item> {
        self.items.iter().filter(|i| i.favorite).collect()
    }

    /// Get items of a specific kind
    pub fn items_of_kind(&self, kind: ItemKind) -> Vec<&Item> {
        self.items.iter().filter(|i| i.kind == kind).collect()
    }

    /// Update the updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Get item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if vault has security questions set up
    pub fn has_security_questions(&self) -> bool {
        !self.security_questions.is_empty()
    }

    /// Add a security question (max 3)
    pub fn add_security_question(&mut self, question: SecurityQuestion) -> bool {
        if self.security_questions.len() < 3 {
            self.security_questions.push(question);
            true
        } else {
            false
        }
    }
}

/// Vault-level settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSettings {
    /// Auto-lock timeout in seconds (None = disabled)
    pub auto_lock_timeout_secs: Option<u64>,
    /// Clipboard clear timeout in seconds
    pub clipboard_clear_secs: u64,
    /// Default item kind when creating new items
    pub default_item_kind: ItemKind,
}

impl Default for VaultSettings {
    fn default() -> Self {
        Self {
            auto_lock_timeout_secs: Some(300), // 5 minutes
            clipboard_clear_secs: 60,
            default_item_kind: ItemKind::Generic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_creation() {
        let vault = Vault::new("My Vault").with_description("Test vault");

        assert_eq!(vault.name, "My Vault");
        assert_eq!(vault.description, Some("Test vault".to_string()));
        assert!(vault.items.is_empty());
        assert!(vault.tags.is_empty());
    }

    #[test]
    fn test_vault_items() {
        let mut vault = Vault::new("Test");

        let item = Item::password("GitHub", "secret123");
        let item_id = item.id;
        vault.add_item(item);

        assert_eq!(vault.item_count(), 1);
        assert!(vault.get_item(item_id).is_some());

        let removed = vault.remove_item(item_id);
        assert!(removed.is_some());
        assert_eq!(vault.item_count(), 0);
    }

    #[test]
    fn test_vault_tags() {
        let mut vault = Vault::new("Test");

        let tag = Tag::new("crypto").with_color("#f7931a");
        let tag_id = tag.id;
        vault.add_tag(tag);

        let mut item = Item::crypto_seed("BTC", "seed words here");
        item.tags.push(tag_id);
        vault.add_item(item);

        let tagged = vault.items_with_tag(tag_id);
        assert_eq!(tagged.len(), 1);

        // Removing tag should also remove from items
        vault.remove_tag(tag_id);
        assert!(vault.get_tag(tag_id).is_none());
        assert!(vault.items[0].tags.is_empty());
    }

    #[test]
    fn test_vault_favorites() {
        let mut vault = Vault::new("Test");

        vault.add_item(Item::password("Item 1", "p1").with_favorite(true));
        vault.add_item(Item::password("Item 2", "p2"));
        vault.add_item(Item::password("Item 3", "p3").with_favorite(true));

        let favorites = vault.favorite_items();
        assert_eq!(favorites.len(), 2);
    }

    #[test]
    fn test_vault_settings() {
        let settings = VaultSettings::default();

        assert_eq!(settings.auto_lock_timeout_secs, Some(300));
        assert_eq!(settings.clipboard_clear_secs, 60);
        assert_eq!(settings.default_item_kind, ItemKind::Generic);
    }
}
