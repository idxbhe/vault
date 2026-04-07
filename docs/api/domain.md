# 📦 Domain Models

Referensi API untuk struktur data domain utama.

## Vault

Container utama untuk menyimpan items.

```rust
pub struct Vault {
    /// Unique identifier
    pub id: Uuid,
    
    /// Human-readable name
    pub name: String,
    
    /// Collection of items
    pub items: Vec<Item>,
    
    /// User-defined tags
    pub tags: Vec<Tag>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
}
```

### Methods

```rust
impl Vault {
    /// Create a new vault
    pub fn new(name: &str) -> Self;
    
    /// Add an item
    pub fn add_item(&mut self, item: Item);
    
    /// Remove an item by ID
    pub fn remove_item(&mut self, id: Uuid) -> Option<Item>;
    
    /// Get item by ID (immutable)
    pub fn get_item(&self, id: Uuid) -> Option<&Item>;
    
    /// Get item by ID (mutable)
    pub fn get_item_mut(&mut self, id: Uuid) -> Option<&mut Item>;
    
    /// Add a tag
    pub fn add_tag(&mut self, tag: Tag);
    
    /// Remove a tag
    pub fn remove_tag(&mut self, id: Uuid);
    
    /// Get items by tag
    pub fn items_with_tag(&self, tag_id: Uuid) -> Vec<&Item>;
    
    /// Get favorite items
    pub fn favorites(&self) -> Vec<&Item>;
}
```

### Example

```rust
let mut vault = Vault::new("My Vault");

let item = Item::new(
    "Bitcoin Seed",
    ItemKind::CryptoSeed,
    ItemContent::CryptoSeed {
        seed_phrase: "word1 word2 ...".to_string(),
        derivation_path: "m/44'/0'/0'".to_string(),
        passphrase: None,
    },
);

vault.add_item(item);
```

## Item

Entry individual dalam vault.

```rust
pub struct Item {
    /// Unique identifier
    pub id: Uuid,
    
    /// Display title
    pub title: String,
    
    /// Item type
    pub kind: ItemKind,
    
    /// Content based on kind
    pub content: ItemContent,
    
    /// Associated tag IDs
    pub tags: Vec<Uuid>,
    
    /// Favorite flag
    pub favorite: bool,
    
    /// Optional notes
    pub notes: Option<String>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
}
```

### Methods

```rust
impl Item {
    /// Create a new item
    pub fn new(title: &str, kind: ItemKind, content: ItemContent) -> Self;
    
    /// Get copyable content (for clipboard)
    pub fn get_copyable_content(&self) -> Option<&str>;
    
    /// Update timestamp
    pub fn touch(&mut self);
    
    /// Toggle favorite
    pub fn toggle_favorite(&mut self);
    
    /// Add tag
    pub fn add_tag(&mut self, tag_id: Uuid);
    
    /// Remove tag
    pub fn remove_tag(&mut self, tag_id: Uuid);
}
```

## ItemKind

Enum untuk tipe item.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    /// Generic key-value
    Generic,
    
    /// Cryptocurrency seed phrase
    CryptoSeed,
    
    /// Login credentials
    Password,
    
    /// Encrypted note
    SecureNote,
    
    /// API credentials
    ApiKey,
}
```

### Methods

```rust
impl ItemKind {
    /// Get all variants
    pub fn all() -> Vec<ItemKind>;
    
    /// Get display name
    pub fn display_name(&self) -> &'static str;
    
    /// Get icon
    pub fn icon(&self) -> &'static str;
    
    /// Get default content
    pub fn default_content(&self) -> ItemContent;
}
```

## ItemContent

Content berdasarkan kind.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemContent {
    /// Generic content
    Generic {
        value: String,
    },
    
    /// Cryptocurrency seed
    CryptoSeed {
        seed_phrase: String,
        derivation_path: String,
        passphrase: Option<String>,
    },
    
    /// Password entry
    Password {
        username: String,
        password: String,
        url: Option<String>,
        totp_secret: Option<String>,
    },
    
    /// Secure note
    SecureNote {
        content: String,
    },
    
    /// API key
    ApiKey {
        key: String,
        secret: Option<String>,
        endpoint: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    },
}
```

### Methods

```rust
impl ItemContent {
    /// Get primary value (for copy)
    pub fn primary_value(&self) -> &str;
    
    /// Check if empty
    pub fn is_empty(&self) -> bool;
}
```

## Tag

User-defined categorization.

```rust
pub struct Tag {
    /// Unique identifier
    pub id: Uuid,
    
    /// Tag name
    pub name: String,
    
    /// Optional color (hex)
    pub color: Option<String>,
}
```

### Methods

```rust
impl Tag {
    /// Create a new tag
    pub fn new(name: &str) -> Self;
    
    /// Create with color
    pub fn with_color(name: &str, color: &str) -> Self;
}
```

## VaultState

Runtime state saat vault unlocked.

```rust
pub struct VaultState {
    /// The vault data
    pub vault: Vault,
    
    /// Path to vault file
    pub vault_path: PathBuf,
    
    /// Encryption key (kept for re-encryption)
    pub encryption_key: [u8; 32],
    
    /// Currently selected item
    pub selected_item_id: Option<Uuid>,
    
    /// Has unsaved changes
    pub is_dirty: bool,
    
    /// Undo history
    pub undo_stack: Vec<UndoEntry>,
    
    /// Redo history
    pub redo_stack: Vec<UndoEntry>,
    
    /// Last user activity (for auto-lock)
    pub last_activity: Instant,
}
```

### Methods

```rust
impl VaultState {
    /// Create new vault state
    pub fn new(vault: Vault, path: PathBuf, key: [u8; 32]) -> Self;
    
    /// Mark as dirty (has changes)
    pub fn mark_dirty(&mut self);
    
    /// Push undo entry
    pub fn push_undo(&mut self, entry: UndoEntry);
    
    /// Undo last action
    pub fn undo(&mut self) -> Option<UndoEntry>;
    
    /// Redo last undone action
    pub fn redo(&mut self) -> Option<UndoEntry>;
    
    /// Get selected item
    pub fn selected_item(&self) -> Option<&Item>;
}
```

## UndoEntry

Entry untuk undo/redo history.

```rust
pub struct UndoEntry {
    /// Human-readable description
    pub description: String,
    
    /// Affected item ID
    pub item_id: Uuid,
    
    /// Previous state snapshot
    pub previous_state: ItemSnapshot,
}
```

## ItemSnapshot

Snapshot item untuk restore.

```rust
pub struct ItemSnapshot {
    pub title: String,
    pub content: ItemContent,
    pub tags: Vec<Uuid>,
    pub favorite: bool,
    pub notes: Option<String>,
}
```

### Methods

```rust
impl ItemSnapshot {
    /// Create snapshot from item
    pub fn from_item(item: &Item) -> Self;
    
    /// Restore snapshot to item
    pub fn restore_to(&self, item: &mut Item);
}
```

## Serialization

Semua domain types implement:

```rust
#[derive(Serialize, Deserialize)]
```

### JSON Example

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "Bitcoin Wallet",
  "kind": "CryptoSeed",
  "content": {
    "CryptoSeed": {
      "seed_phrase": "abandon abandon ... about",
      "derivation_path": "m/44'/0'/0'",
      "passphrase": null
    }
  },
  "tags": ["550e8400-e29b-41d4-a716-446655440001"],
  "favorite": true,
  "notes": "Hardware wallet backup",
  "created_at": "2026-04-06T00:00:00Z",
  "updated_at": "2026-04-06T12:00:00Z"
}
```

## Validation

```rust
impl Item {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.title.is_empty() {
            return Err(ValidationError::EmptyTitle);
        }
        
        if self.title.len() > 255 {
            return Err(ValidationError::TitleTooLong);
        }
        
        // Content-specific validation
        match &self.content {
            ItemContent::CryptoSeed { seed_phrase, .. } => {
                let word_count = seed_phrase.split_whitespace().count();
                if word_count != 12 && word_count != 24 {
                    return Err(ValidationError::InvalidSeedPhrase);
                }
            }
            // ...
        }
        
        Ok(())
    }
}
```
