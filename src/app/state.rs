//! Application state - TEA state management
//!
//! Contains all state types for the application following the Elm Architecture pattern.

use std::path::PathBuf;
use std::time::Instant;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::crypto::EncryptionMethod;
use crate::domain::{CustomField, Item, ItemContent, ItemKind, RecoveryMetadata, Vault};
use crate::input::mouse::LayoutRegions;
use crate::storage::{AppConfig, VaultRegistry};
use crate::ui::screens::{LoginScreen, SettingsScreen};
use crate::ui::widgets::{EditFormState, KindSelectorState, SearchState};

/// Root application state
#[derive(Debug)]
pub struct AppState {
    /// Current application mode
    pub mode: AppMode,
    /// Current screen being displayed
    pub screen: Screen,
    /// Vault state when unlocked
    pub vault_state: Option<VaultState>,
    /// UI-specific state
    pub ui_state: UIState,
    /// Clipboard tracking
    pub clipboard_state: ClipboardState,
    /// Login screen state
    pub login_screen: LoginScreen,
    /// Settings screen state
    pub settings_state: SettingsScreen,
    /// Application configuration
    pub config: AppConfig,
    /// Known vaults registry
    pub registry: VaultRegistry,
    /// Whether a lock was requested and should happen right after successful save
    pub pending_lock: bool,
    /// Whether the app should quit
    pub should_quit: bool,
}

impl AppState {
    /// Create a new application state
    pub fn new(config: AppConfig, registry: VaultRegistry) -> Self {
        let mut ui_state = UIState::default();
        // Start with masked input for login screen password
        ui_state.input_buffer.masked = true;

        Self {
            mode: AppMode::Locked,
            screen: Screen::Login,
            vault_state: None,
            ui_state,
            clipboard_state: ClipboardState::default(),
            login_screen: LoginScreen::new(),
            settings_state: SettingsScreen::new(),
            config,
            registry,
            pending_lock: false,
            should_quit: false,
        }
    }

    /// Check if a vault is currently unlocked
    pub fn is_unlocked(&self) -> bool {
        self.vault_state.is_some() && self.mode == AppMode::Unlocked
    }

    /// Get the current vault if unlocked
    pub fn vault(&self) -> Option<&Vault> {
        self.vault_state.as_ref().map(|vs| &vs.vault)
    }

    /// Get the current vault mutably if unlocked
    pub fn vault_mut(&mut self) -> Option<&mut Vault> {
        self.vault_state.as_mut().map(|vs| &mut vs.vault)
    }

    /// Get the selected item
    pub fn selected_item(&self) -> Option<&Item> {
        let vault_state = self.vault_state.as_ref()?;
        let item_id = vault_state.selected_item_id?;
        vault_state.vault.get_item(item_id)
    }

    /// Check if there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.vault_state
            .as_ref()
            .map(|vs| vs.is_dirty)
            .unwrap_or(false)
    }
}

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    /// No vault loaded, at login screen
    #[default]
    Locked,
    /// Vault is unlocked and accessible
    Unlocked,
    /// Creating a new vault
    Creating,
    /// Exporting vault data
    Exporting,
}

/// Active screen in the application
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Screen {
    /// Login/vault selection screen
    #[default]
    Login,
    /// Main vault view (list + detail)
    Main,
    /// Settings configuration
    Settings,
    /// Export dialog
    Export,
    /// Security questions setup
    SecurityQuestions,
    /// Password recovery flow
    PasswordRecovery,
}

/// State when a vault is unlocked
#[derive(Debug)]
pub struct VaultState {
    /// The loaded vault
    pub vault: Vault,
    /// Path to the vault file
    pub vault_path: PathBuf,
    /// Encryption key (kept in memory while unlocked)
    pub encryption_key: [u8; 32],
    /// Original salt (must be preserved for re-encryption)
    pub salt: [u8; 32],
    /// Whether this vault requires a keyfile
    pub has_keyfile: bool,
    /// Encryption method used by this vault
    pub encryption_method: EncryptionMethod,
    /// Recovery metadata stored in vault header
    pub recovery_metadata: Option<RecoveryMetadata>,
    /// Whether there are unsaved changes
    pub is_dirty: bool,
    /// Currently selected item
    pub selected_item_id: Option<Uuid>,
    /// Undo stack
    pub undo_stack: Vec<UndoEntry>,
    /// Redo stack
    pub redo_stack: Vec<UndoEntry>,
    /// Last activity time (for auto-lock)
    pub last_activity: Instant,
}

impl VaultState {
    /// Create a new vault state
    pub fn new(
        vault: Vault,
        vault_path: PathBuf,
        encryption_key: [u8; 32],
        salt: [u8; 32],
        has_keyfile: bool,
        encryption_method: EncryptionMethod,
        recovery_metadata: Option<RecoveryMetadata>,
    ) -> Self {
        Self {
            vault,
            vault_path,
            encryption_key,
            salt,
            has_keyfile,
            encryption_method,
            recovery_metadata,
            is_dirty: false,
            selected_item_id: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_activity: Instant::now(),
        }
    }

    /// Mark the vault as modified
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
        self.touch();
    }

    /// Update last activity time
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Push an undo entry
    pub fn push_undo(&mut self, entry: UndoEntry) {
        self.undo_stack.push(entry);
        self.redo_stack.clear(); // Clear redo on new action
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

/// Entry in the undo/redo stack
#[derive(Debug, Clone)]
pub struct UndoEntry {
    /// Human-readable description
    pub description: String,
    /// ID of the affected item
    pub item_id: Uuid,
    /// Snapshot of the item before the change
    pub previous_state: ItemSnapshot,
}

/// Snapshot of an item's state
#[derive(Debug, Clone)]
pub struct ItemSnapshot {
    /// Complete item data
    pub item: Item,
}

impl ItemSnapshot {
    /// Create a snapshot from an item
    pub fn from_item(item: &Item) -> Self {
        Self { item: item.clone() }
    }
}

/// UI-specific state
#[derive(Debug, Default)]
pub struct UIState {
    /// Currently focused pane
    pub focused_pane: Pane,
    /// Scroll offset in the item list
    pub list_scroll_offset: usize,
    /// Scroll offset in the detail view
    pub detail_scroll_offset: usize,
    /// Active floating window
    pub floating_window: Option<FloatingWindow>,
    /// Active notifications
    pub notifications: Vec<Notification>,
    /// Whether sensitive content is revealed
    pub content_revealed: bool,
    /// Input buffer for forms
    pub input_buffer: InputBuffer,
    /// Filter state
    pub filter: FilterState,
    /// Mouse click region tracking
    pub layout_regions: LayoutRegions,
    /// Loading state with optional message
    pub loading_message: Option<String>,
    /// Spinner frame counter (0-7 for 8 frames)
    pub spinner_frame: u8,
}

impl UIState {
    /// Add a notification
    pub fn notify(&mut self, message: impl Into<String>, level: NotificationLevel) {
        let notification = Notification::new(message, level);
        self.notifications.push(notification);
    }

    /// Remove expired notifications
    pub fn cleanup_notifications(&mut self) {
        let now = Utc::now();
        self.notifications.retain(|n| n.expires_at > now);
    }

    /// Check if a floating window is open
    pub fn has_floating_window(&self) -> bool {
        self.floating_window.is_some()
    }

    /// Close the floating window
    pub fn close_floating_window(&mut self) {
        self.floating_window = None;
    }

    /// Clear layout regions (call at start of each render)
    pub fn clear_layout_regions(&mut self) {
        self.layout_regions.clear();
    }

    /// Register a clickable region
    pub fn register_region(
        &mut self,
        name: crate::input::mouse::UiRegion,
        region: crate::input::mouse::ClickRegion,
    ) {
        self.layout_regions.set(name, region);
    }

    /// Start loading with a message
    pub fn start_loading(&mut self, message: impl Into<String>) {
        self.loading_message = Some(message.into());
        self.spinner_frame = 0;
    }

    /// Stop loading
    pub fn stop_loading(&mut self) {
        self.loading_message = None;
    }

    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        self.loading_message.is_some()
    }

    /// Advance spinner animation
    pub fn tick_spinner(&mut self) {
        if self.is_loading() {
            self.spinner_frame = (self.spinner_frame + 1) % 10;
        }
    }

    /// Get current spinner character
    pub fn spinner_char(&self) -> char {
        const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER[self.spinner_frame as usize]
    }
}

/// Active pane in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane {
    /// Item list pane
    #[default]
    List,
    /// Item detail pane
    Detail,
    /// Search input
    Search,
}

/// Floating window variants
#[derive(Debug, Clone)]
pub enum FloatingWindow {
    /// Search dialog with state
    Search { state: SearchState },
    /// Delete confirmation
    ConfirmDelete { item_id: Uuid },
    /// Kind selector (first step of new item)
    KindSelector { state: KindSelectorState },
    /// New item form
    NewItem { form: EditFormState },
    /// Edit item form
    EditItem { item_id: Uuid, form: EditFormState },
    /// Help overlay
    Help,
    /// Tag filter
    TagFilter,
}

impl FloatingWindow {
    /// Create a new search dialog
    pub fn new_search() -> Self {
        Self::Search {
            state: SearchState::new(),
        }
    }

    /// Create a new item kind selector
    pub fn new_kind_selector() -> Self {
        Self::KindSelector {
            state: KindSelectorState::default(),
        }
    }

    /// Create a new item form
    pub fn new_item_form(kind: ItemKind) -> Self {
        Self::NewItem {
            form: EditFormState::new(kind, true),
        }
    }

    /// Create an edit item form
    pub fn edit_item_form(item: &Item) -> Self {
        let mut form = EditFormState::new(item.kind, false);

        // Pre-fill the form with existing item data
        form.set_title(&item.title);

        // Fill content fields based on kind
        match &item.content {
            ItemContent::Generic { value } => {
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::Content)
                {
                    form.values[idx] = value.clone();
                }
            }
            ItemContent::CryptoSeed {
                seed_phrase,
                derivation_path,
                network,
            } => {
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::SeedPhrase)
                {
                    form.values[idx] = seed_phrase.clone();
                }
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::DerivationPath)
                    && let Some(dp) = derivation_path
                {
                    form.values[idx] = dp.clone();
                }
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::Network)
                    && let Some(net) = network
                {
                    form.values[idx] = net.clone();
                }
            }
            ItemContent::Password {
                username,
                password,
                url,
                ..
            } => {
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::Username)
                    && let Some(u) = username
                {
                    form.values[idx] = u.clone();
                }
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::Password)
                {
                    form.values[idx] = password.clone();
                }
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::Url)
                    && let Some(u) = url
                {
                    form.values[idx] = u.clone();
                }
            }
            ItemContent::SecureNote { content } => {
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::Content)
                {
                    form.values[idx] = content.clone();
                }
            }
            ItemContent::ApiKey { key, service, .. } => {
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::ApiKey)
                {
                    form.values[idx] = key.clone();
                }
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::Service)
                    && let Some(s) = service
                {
                    form.values[idx] = s.clone();
                }
            }
            ItemContent::Custom { fields } => {
                if let Some(idx) = form
                    .fields
                    .iter()
                    .position(|f| *f == crate::ui::widgets::FormField::CustomFields)
                {
                    form.values[idx] = format_custom_fields_for_form(fields);
                }
            }
        }

        // Fill notes
        if let Some(notes) = &item.notes
            && let Some(idx) = form
                .fields
                .iter()
                .position(|f| *f == crate::ui::widgets::FormField::Notes)
        {
            form.values[idx] = notes.clone();
        }

        Self::EditItem {
            item_id: item.id,
            form,
        }
    }
}

fn format_custom_fields_for_form(fields: &[CustomField]) -> String {
    fields
        .iter()
        .map(|field| {
            format!(
                "{}:{}={}",
                field.field_type.as_str(),
                field.key,
                field.value
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// User notification
#[derive(Debug, Clone)]
pub struct Notification {
    /// Unique ID
    pub id: Uuid,
    /// Message text
    pub message: String,
    /// Severity level
    pub level: NotificationLevel,
    /// When the notification expires
    pub expires_at: DateTime<Utc>,
}

impl Notification {
    /// Create a new notification with default expiry (5 seconds)
    pub fn new(message: impl Into<String>, level: NotificationLevel) -> Self {
        Self {
            id: Uuid::new_v4(),
            message: message.into(),
            level,
            expires_at: Utc::now() + chrono::Duration::seconds(5),
        }
    }

    /// Create with custom duration
    pub fn with_duration(
        message: impl Into<String>,
        level: NotificationLevel,
        seconds: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            message: message.into(),
            level,
            expires_at: Utc::now() + chrono::Duration::seconds(seconds),
        }
    }
}

/// Notification severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Clipboard tracking state
#[derive(Debug, Default)]
pub struct ClipboardState {
    /// Whether clipboard contains sensitive content
    pub has_secure_content: bool,
    /// When to clear the clipboard
    pub clear_at: Option<Instant>,
}

impl ClipboardState {
    /// Set clipboard with secure content and schedule clear
    pub fn set_secure(&mut self, clear_after_secs: u64) {
        self.has_secure_content = true;
        self.clear_at = Some(Instant::now() + std::time::Duration::from_secs(clear_after_secs));
    }

    /// Clear clipboard state
    pub fn clear(&mut self) {
        self.has_secure_content = false;
        self.clear_at = None;
    }

    /// Check if clipboard should be cleared now
    pub fn should_clear(&self) -> bool {
        self.clear_at.map(|t| Instant::now() >= t).unwrap_or(false)
    }
}

/// Input buffer for form fields
#[derive(Debug, Default, Clone)]
pub struct InputBuffer {
    /// Current input text
    pub text: String,
    /// Cursor position
    pub cursor: usize,
    /// Whether input is masked (for passwords)
    pub masked: bool,
}

impl InputBuffer {
    /// Create a new input buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a masked input buffer (for passwords)
    pub fn masked() -> Self {
        Self {
            masked: true,
            ..Default::default()
        }
    }

    /// Insert a character at cursor
    pub fn insert(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let prev_char_boundary = self.text[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.text.remove(prev_char_boundary);
            self.cursor = prev_char_boundary;
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
        }
    }

    /// Move cursor to start
    pub fn home(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn end(&mut self) {
        self.cursor = self.text.len();
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    /// Get the display text (masked if needed)
    pub fn display(&self) -> String {
        if self.masked {
            "•".repeat(self.text.chars().count())
        } else {
            self.text.clone()
        }
    }
}

/// Filter state for item list
#[derive(Debug, Default, Clone)]
pub struct FilterState {
    /// Filter by item kind
    pub kind: Option<ItemKind>,
    /// Filter by tag IDs
    pub tags: Vec<Uuid>,
    /// Show only favorites
    pub favorites_only: bool,
}

impl FilterState {
    /// Check if any filter is active
    pub fn is_active(&self) -> bool {
        self.kind.is_some() || !self.tags.is_empty() || self.favorites_only
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.kind = None;
        self.tags.clear();
        self.favorites_only = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let config = AppConfig::default();
        let registry = VaultRegistry::default();
        let state = AppState::new(config, registry);

        assert_eq!(state.mode, AppMode::Locked);
        assert_eq!(state.screen, Screen::Login);
        assert!(!state.is_unlocked());
        assert!(!state.should_quit);
    }

    #[test]
    fn test_input_buffer() {
        let mut buf = InputBuffer::new();

        buf.insert('H');
        buf.insert('e');
        buf.insert('l');
        buf.insert('l');
        buf.insert('o');

        assert_eq!(buf.text, "Hello");
        assert_eq!(buf.cursor, 5);

        buf.backspace();
        assert_eq!(buf.text, "Hell");

        buf.home();
        assert_eq!(buf.cursor, 0);

        buf.move_right();
        buf.insert('X');
        assert_eq!(buf.text, "HXell");
    }

    #[test]
    fn test_input_buffer_masked() {
        let mut buf = InputBuffer::masked();
        buf.insert('s');
        buf.insert('e');
        buf.insert('c');
        buf.insert('r');
        buf.insert('e');
        buf.insert('t');

        assert_eq!(buf.text, "secret");
        assert_eq!(buf.display(), "••••••");
    }

    #[test]
    fn test_clipboard_state() {
        let mut clip = ClipboardState::default();
        assert!(!clip.has_secure_content);
        assert!(!clip.should_clear());

        clip.set_secure(0); // Immediate clear
        assert!(clip.has_secure_content);

        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(clip.should_clear());

        clip.clear();
        assert!(!clip.has_secure_content);
    }

    #[test]
    fn test_filter_state() {
        let mut filter = FilterState::default();
        assert!(!filter.is_active());

        filter.favorites_only = true;
        assert!(filter.is_active());

        filter.clear();
        assert!(!filter.is_active());
    }

    #[test]
    fn test_notification() {
        let notif = Notification::new("Test message", NotificationLevel::Success);
        assert!(notif.expires_at > Utc::now());
        assert_eq!(notif.level, NotificationLevel::Success);
    }

    #[test]
    fn test_push_undo() {
        let mut vault_state = VaultState::new(
            crate::domain::Vault::new("Test Vault"),
            std::path::PathBuf::from("/tmp/test.vault"),
            [0; 32],
            [0; 32],
            false,
            crate::crypto::EncryptionMethod::Aes256Gcm,
            None,
        );

        let item = crate::domain::Item::new(
            "Test Item",
            crate::domain::ItemKind::Generic,
            crate::domain::ItemContent::Generic {
                value: "test".to_string(),
            },
        );
        let snapshot = ItemSnapshot::from_item(&item);
        let undo_entry1 = UndoEntry {
            description: "First action".to_string(),
            item_id: item.id,
            previous_state: snapshot.clone(),
        };
        let undo_entry2 = UndoEntry {
            description: "Second action".to_string(),
            item_id: item.id,
            previous_state: snapshot.clone(),
        };

        vault_state.undo_stack.push(undo_entry1);
        vault_state.redo_stack.push(undo_entry2);

        assert_eq!(vault_state.undo_stack.len(), 1);
        assert_eq!(vault_state.redo_stack.len(), 1);

        let new_undo_entry = UndoEntry {
            description: "New action".to_string(),
            item_id: item.id,
            previous_state: snapshot,
        };

        vault_state.push_undo(new_undo_entry);

        assert_eq!(vault_state.undo_stack.len(), 2);
        assert_eq!(vault_state.redo_stack.len(), 0);
    }
}
