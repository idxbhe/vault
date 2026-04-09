//! Tag data model for item categorization

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tag for categorizing items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    /// Unique identifier
    pub id: Uuid,
    /// Tag name (e.g., "crypto", "work", "personal")
    pub name: String,
    /// Optional hex color (e.g., "#ff5733")
    pub color: Option<String>,
    /// Optional Nerd Font icon
    pub icon: Option<String>,
}

impl Tag {
    /// Create a new tag with a generated UUID
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            color: None,
            icon: None,
        }
    }

    /// Create a tag with color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Create a tag with icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("crypto").with_color("#f7931a").with_icon("󰞃");

        assert_eq!(tag.name, "crypto");
        assert_eq!(tag.color, Some("#f7931a".to_string()));
        assert_eq!(tag.icon, Some("󰞃".to_string()));
    }
}
