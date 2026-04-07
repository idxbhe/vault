//! Keybinding definitions
//!
//! Vim-style keybindings with customizable mappings.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vim-style key actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyAction {
    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    JumpToTop,
    JumpToBottom,
    PageUp,
    PageDown,

    // Actions
    Select,
    Back,
    Search,
    NewItem,
    EditItem,
    DeleteItem,
    CopyContent,
    ToggleReveal,
    ToggleFavorite,
    Save,
    Export,

    // Modes
    Help,
    Settings,
    Lock,
    Quit,
    ForceQuit,

    // History
    Undo,
    Redo,

    // Focus
    FocusList,
    FocusDetail,
    NextPane,
    PrevPane,
}

/// A key combination (key + modifiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyCombo {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn plain(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    pub fn ctrl(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::CONTROL)
    }

    pub fn shift(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::SHIFT)
    }
}

impl From<KeyEvent> for KeyCombo {
    fn from(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

/// Keybinding configuration
#[derive(Debug, Clone)]
pub struct KeybindingConfig {
    bindings: HashMap<KeyCombo, KeyAction>,
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // Vim-style navigation
        bindings.insert(KeyCombo::plain(KeyCode::Char('j')), KeyAction::MoveDown);
        bindings.insert(KeyCombo::plain(KeyCode::Char('k')), KeyAction::MoveUp);
        bindings.insert(KeyCombo::plain(KeyCode::Char('h')), KeyAction::MoveLeft);
        bindings.insert(KeyCombo::plain(KeyCode::Char('l')), KeyAction::MoveRight);
        bindings.insert(KeyCombo::plain(KeyCode::Down), KeyAction::MoveDown);
        bindings.insert(KeyCombo::plain(KeyCode::Up), KeyAction::MoveUp);
        bindings.insert(KeyCombo::plain(KeyCode::Left), KeyAction::MoveLeft);
        bindings.insert(KeyCombo::plain(KeyCode::Right), KeyAction::MoveRight);
        
        // Jump navigation
        bindings.insert(KeyCombo::plain(KeyCode::Char('g')), KeyAction::JumpToTop);
        bindings.insert(KeyCombo::shift(KeyCode::Char('G')), KeyAction::JumpToBottom);
        bindings.insert(KeyCombo::ctrl(KeyCode::Char('u')), KeyAction::PageUp);
        bindings.insert(KeyCombo::ctrl(KeyCode::Char('d')), KeyAction::PageDown);
        bindings.insert(KeyCombo::plain(KeyCode::PageUp), KeyAction::PageUp);
        bindings.insert(KeyCombo::plain(KeyCode::PageDown), KeyAction::PageDown);

        // Actions
        bindings.insert(KeyCombo::plain(KeyCode::Enter), KeyAction::Select);
        bindings.insert(KeyCombo::plain(KeyCode::Esc), KeyAction::Back);
        bindings.insert(KeyCombo::plain(KeyCode::Char('/')), KeyAction::Search);
        bindings.insert(KeyCombo::plain(KeyCode::Char('n')), KeyAction::NewItem);
        bindings.insert(KeyCombo::plain(KeyCode::Char('e')), KeyAction::EditItem);
        bindings.insert(KeyCombo::plain(KeyCode::Char('d')), KeyAction::DeleteItem);
        bindings.insert(KeyCombo::plain(KeyCode::Char('y')), KeyAction::CopyContent);
        bindings.insert(KeyCombo::plain(KeyCode::Char('r')), KeyAction::ToggleReveal);
        bindings.insert(KeyCombo::plain(KeyCode::Char('f')), KeyAction::ToggleFavorite);
        bindings.insert(KeyCombo::ctrl(KeyCode::Char('s')), KeyAction::Save);
        bindings.insert(KeyCombo::ctrl(KeyCode::Char('e')), KeyAction::Export);

        // Modes
        bindings.insert(KeyCombo::plain(KeyCode::Char('?')), KeyAction::Help);
        bindings.insert(KeyCombo::plain(KeyCode::Char(',')), KeyAction::Settings);
        bindings.insert(KeyCombo::ctrl(KeyCode::Char('l')), KeyAction::Lock);
        bindings.insert(KeyCombo::plain(KeyCode::Char('q')), KeyAction::Quit);
        bindings.insert(KeyCombo::ctrl(KeyCode::Char('q')), KeyAction::ForceQuit);
        bindings.insert(KeyCombo::ctrl(KeyCode::Char('c')), KeyAction::ForceQuit);

        // History
        bindings.insert(KeyCombo::plain(KeyCode::Char('u')), KeyAction::Undo);
        bindings.insert(KeyCombo::ctrl(KeyCode::Char('r')), KeyAction::Redo);

        // Focus
        bindings.insert(KeyCombo::plain(KeyCode::Char('1')), KeyAction::FocusList);
        bindings.insert(KeyCombo::plain(KeyCode::Char('2')), KeyAction::FocusDetail);
        bindings.insert(KeyCombo::plain(KeyCode::Tab), KeyAction::NextPane);
        bindings.insert(KeyCombo::shift(KeyCode::BackTab), KeyAction::PrevPane);

        Self { bindings }
    }
}

impl KeybindingConfig {
    /// Get action for a key event
    pub fn get_action(&self, event: KeyEvent) -> Option<KeyAction> {
        self.bindings.get(&KeyCombo::from(event)).copied()
    }

    /// Set a custom keybinding
    pub fn set(&mut self, combo: KeyCombo, action: KeyAction) {
        self.bindings.insert(combo, action);
    }

    /// Remove a keybinding
    pub fn remove(&mut self, combo: &KeyCombo) {
        self.bindings.remove(combo);
    }

    /// Get all bindings for an action
    pub fn get_bindings(&self, action: KeyAction) -> Vec<KeyCombo> {
        self.bindings
            .iter()
            .filter(|(_, a)| **a == action)
            .map(|(k, _)| *k)
            .collect()
    }
}

/// Format a key combo for display
pub fn format_key_combo(combo: &KeyCombo) -> String {
    let mut parts = Vec::new();

    if combo.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if combo.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if combo.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }

    let key = match combo.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PgUp".to_string(),
        KeyCode::PageDown => "PgDn".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => "?".to_string(),
    };
    parts.push(&key);

    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bindings() {
        let config = KeybindingConfig::default();

        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(config.get_action(event), Some(KeyAction::MoveDown));

        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(config.get_action(event), Some(KeyAction::MoveUp));
    }

    #[test]
    fn test_ctrl_bindings() {
        let config = KeybindingConfig::default();

        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(config.get_action(event), Some(KeyAction::Save));
    }

    #[test]
    fn test_format_key_combo() {
        let combo = KeyCombo::ctrl(KeyCode::Char('s'));
        assert_eq!(format_key_combo(&combo), "Ctrl+s");

        let combo = KeyCombo::plain(KeyCode::Enter);
        assert_eq!(format_key_combo(&combo), "Enter");
    }
}
