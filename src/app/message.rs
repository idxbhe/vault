//! Application messages - all possible actions in TEA
//!
//! Messages represent all events and actions that can occur in the application.
//! The update function processes these messages to produce new state.

use std::path::PathBuf;

use uuid::Uuid;

use crate::crypto::SecureString;
use crate::domain::{ItemContent, ItemKind, Tag};
use crate::storage::ThemeChoice;

use super::state::{FloatingWindow, NotificationLevel, Pane, Screen};

/// All possible actions in the application
#[derive(Debug, Clone)]
pub enum Message {
    // === Navigation ===
    /// Navigate to a screen
    Navigate(Screen),
    /// Focus a specific pane
    FocusPane(Pane),

    // === Vault Operations ===
    /// Unlock the selected vault from login screen
    UnlockVault {
        password: SecureString,
        keyfile: Option<PathBuf>,
    },
    /// Lock the vault (clear sensitive data)
    LockVault,
    /// Save the vault to disk
    SaveVault,
    /// Close vault and return to login
    CloseVault,

    // === Login Flow ===
    /// Start creating a new vault
    StartCreateVault,
    /// Enter password mode for selected vault
    EnterPasswordMode,
    /// Start forgot-password recovery for selected vault
    StartPasswordRecovery,
    /// Go back to previous step in vault creation
    LoginPrevStep,
    /// Cancel current input operation
    CancelInput,
    /// Delete the currently selected vault from registry
    DeleteSelectedVault,
    /// Confirm deletion of a vault
    ConfirmDeleteVault(usize),
    /// Select next vault in login screen
    LoginSelectNext,
    /// Select previous vault in login screen
    LoginSelectPrev,
    /// Select specific vault by index in login screen
    LoginSelectVault(usize),

    // === Item Operations ===
    /// Select an item by ID
    SelectItem(Uuid),
    /// Select next item in list
    SelectNextItem,
    /// Select previous item in list
    SelectPrevItem,
    /// Create a new item
    CreateItem { kind: ItemKind },
    /// Update an existing item
    UpdateItem { id: Uuid, updates: ItemUpdates },
    /// Delete an item
    DeleteItem(Uuid),
    /// Confirm deletion of an item
    ConfirmDeleteItem(Uuid),
    /// Toggle favorite status
    ToggleFavorite(Uuid),
    /// Duplicate an item
    DuplicateItem(Uuid),

    // === History ===
    /// Undo last action
    Undo,
    /// Redo last undone action
    Redo,

    // === Search ===
    /// Open search dialog
    OpenSearch,
    /// Close search dialog
    CloseSearch,
    /// Update search query
    UpdateSearchQuery(String),
    /// Execute search
    ExecuteSearch,
    /// Select a search result by index
    SelectSearchResult(usize),
    /// Navigate to next search result
    SearchNextResult,
    /// Navigate to previous search result
    SearchPrevResult,
    /// Confirm search selection
    SearchConfirm,

    // === Clipboard ===
    /// Copy content to clipboard
    CopyToClipboard { content: String, is_sensitive: bool },
    /// Copy the current item's primary content
    CopyCurrentItem,
    /// Copy a specific field from the detail view
    CopyField(usize),
    /// Edit a specific field from the detail view
    EditField(usize),
    /// Clear clipboard
    ClearClipboard,

    // === UI ===
    /// Toggle content reveal (show/hide sensitive data)
    ToggleContentReveal,
    /// Open a floating window
    OpenFloatingWindow(FloatingWindow),
    /// Close the current floating window
    CloseFloatingWindow,
    /// Show a notification
    ShowNotification {
        message: String,
        level: NotificationLevel,
    },
    /// Dismiss a notification
    DismissNotification(Uuid),
    /// Scroll in a direction
    Scroll(ScrollDirection),

    // === Input ===
    /// Character input
    InputChar(char),
    /// Backspace
    InputBackspace,
    /// Delete
    InputDelete,
    /// Move cursor left
    InputLeft,
    /// Move cursor right
    InputRight,
    /// Move cursor up
    InputUp,
    /// Move cursor down
    InputDown,
    /// Move cursor to start
    InputHome,
    /// Move cursor to end
    InputEnd,
    /// Submit current input
    InputSubmit,
    /// Cancel current input
    InputCancel,

    // === Form ===
    /// Move to next form field
    FormNextField,
    /// Move to previous form field
    FormPrevField,
    /// Focus a specific form field by index (for mouse clicks)
    FormFocusField(usize),
    /// Focus the detail notes
    FocusDetailNotes,
    /// Edit the detail notes
    EditNotes,
    /// Submit the form
    FormSubmit,
    /// Select item kind in kind selector
    KindSelectorNext,
    /// Move to previous kind
    KindSelectorPrev,
    /// Select kind by index (for mouse clicks)
    KindSelectorSelect(usize),
    /// Confirm kind selection
    KindSelectorConfirm,

    // === Tags ===
    /// Create a new tag
    CreateTag(Tag),
    /// Delete a tag
    DeleteTag(Uuid),
    /// Toggle tag on current item
    ToggleItemTag { item_id: Uuid, tag_id: Uuid },

    // === Filter ===
    /// Set kind filter
    SetKindFilter(Option<ItemKind>),
    /// Toggle tag filter
    ToggleTagFilter(Uuid),
    /// Toggle favorites filter
    ToggleFavoritesFilter,
    /// Clear all filters
    ClearFilters,
    /// Select next category
    NextCategory,
    /// Select previous category
    PrevCategory,

    // === Settings ===
    /// Update configuration
    UpdateConfig(ConfigUpdate),

    // === Security Questions ===
    /// Setup security questions for recovery
    SetupSecurityQuestions(Vec<SecurityQuestionInput>),
    /// Attempt password recovery
    AttemptRecovery {
        question_index: usize,
        answer: SecureString,
    },

    // === Export ===
    /// Export vault to file
    ExportVault { format: ExportFormat, path: PathBuf },

    // === System ===
    /// Timer tick (for auto-lock, clipboard clear, etc.)
    Tick,
    /// Request to quit the application
    Quit,
    /// Force quit without saving
    ForceQuit,
    /// No operation
    Noop,

    // === Async Runtime ===
    /// An asynchronous effect completed
    AsyncEffectCompleted(Box<super::effect::EffectResult>),
}

/// Updates to apply to an item
#[derive(Debug, Clone, Default)]
pub struct ItemUpdates {
    /// New title
    pub title: Option<String>,
    /// New content
    pub content: Option<ItemContent>,
    /// New notes (Some(None) to clear, Some(Some(x)) to set)
    pub notes: Option<Option<String>>,
    /// New tag list
    pub tags: Option<Vec<Uuid>>,
    /// New favorite status
    pub favorite: Option<bool>,
}

impl ItemUpdates {
    /// Create empty updates
    pub fn new() -> Self {
        Self::default()
    }

    /// Set title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set content
    pub fn content(mut self, content: ItemContent) -> Self {
        self.content = Some(content);
        self
    }

    /// Set notes
    pub fn notes(mut self, notes: Option<String>) -> Self {
        self.notes = Some(notes);
        self
    }

    /// Set tags
    pub fn tags(mut self, tags: Vec<Uuid>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set favorite
    pub fn favorite(mut self, favorite: bool) -> Self {
        self.favorite = Some(favorite);
        self
    }

    /// Check if any updates are present
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.content.is_none()
            && self.notes.is_none()
            && self.tags.is_none()
            && self.favorite.is_none()
    }
}

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,
}

/// Configuration updates
#[derive(Debug, Clone)]
pub enum ConfigUpdate {
    /// Set theme
    SetTheme(ThemeChoice),
    /// Enable/disable auto-lock
    SetAutoLock(bool),
    /// Set auto-lock timeout in seconds
    SetAutoLockTimeout(u64),
    /// Set clipboard clear timeout in seconds
    SetClipboardTimeout(u64),
    /// Enable/disable icons
    SetShowIcons(bool),
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Plain JSON (unencrypted)
    Json,
    /// Encrypted JSON
    EncryptedJson,
}

/// Security question input for setup
#[derive(Debug, Clone)]
pub struct SecurityQuestionInput {
    /// The question text
    pub question: String,
    /// The answer (will be hashed)
    pub answer: SecureString,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_updates_builder() {
        let updates = ItemUpdates::new().title("New Title").favorite(true);

        assert_eq!(updates.title, Some("New Title".to_string()));
        assert_eq!(updates.favorite, Some(true));
        assert!(updates.content.is_none());
        assert!(!updates.is_empty());
    }

    #[test]
    fn test_item_updates_empty() {
        let updates = ItemUpdates::new();
        assert!(updates.is_empty());
    }

    #[test]
    fn test_message_variants() {
        // Just ensure the enum compiles and can be created
        let _msg = Message::Navigate(Screen::Main);
        let _msg = Message::Tick;
        let _msg = Message::Scroll(ScrollDirection::Down);
    }
}
