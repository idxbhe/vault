//! Event routing - context-aware message dispatch
//!
//! Routes input events to appropriate messages based on current app state.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::app::{
    AppMode, AppState, FloatingWindow, Message, Pane, Screen, ScrollDirection,
};

use super::keybindings::{KeyAction, KeybindingConfig};

/// Route an input event to a message based on current state
pub fn route_event(state: &AppState, event: Event, keybindings: &KeybindingConfig) -> Message {
    match event {
        Event::Key(key_event) => route_key_event(state, key_event, keybindings),
        Event::Mouse(mouse_event) => route_mouse_event(state, mouse_event),
        Event::Resize(_, _) => Message::Noop, // Handled by terminal
        _ => Message::Noop,
    }
}

/// Route a key event to a message
fn route_key_event(
    state: &AppState,
    event: KeyEvent,
    keybindings: &KeybindingConfig,
) -> Message {
    // Handle floating windows first
    if let Some(ref window) = state.ui_state.floating_window {
        return route_floating_window_key(state, event, window);
    }

    // Handle screen-specific input
    match state.screen {
        Screen::Login => route_login_key(state, event),
        Screen::Main => route_main_key(state, event, keybindings),
        Screen::Settings => route_settings_key(event),
        _ => Message::Noop,
    }
}

/// Route keys in the login screen
fn route_login_key(state: &AppState, event: KeyEvent) -> Message {
    // Check login screen mode
    let login_state = &state.login_screen;
    
    // If creating a new vault, all input goes to vault name field
    if login_state.creating_vault {
        return match event.code {
            KeyCode::Char(c) if !event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::InputChar(c)
            }
            KeyCode::Backspace => Message::InputBackspace,
            KeyCode::Delete => Message::InputDelete,
            KeyCode::Left => Message::InputLeft,
            KeyCode::Right => Message::InputRight,
            KeyCode::Home => Message::InputHome,
            KeyCode::End => Message::InputEnd,
            KeyCode::Enter => Message::InputSubmit,
            KeyCode::Esc => Message::CancelInput,
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => Message::ForceQuit,
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => Message::ForceQuit,
            _ => Message::Noop,
        };
    }
    
    // If entering password, all input goes to password field
    if login_state.entering_password {
        return match event.code {
            KeyCode::Char(c) if !event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::InputChar(c)
            }
            KeyCode::Backspace => Message::InputBackspace,
            KeyCode::Delete => Message::InputDelete,
            KeyCode::Left => Message::InputLeft,
            KeyCode::Right => Message::InputRight,
            KeyCode::Home => Message::InputHome,
            KeyCode::End => Message::InputEnd,
            KeyCode::Enter => Message::InputSubmit,
            KeyCode::Esc => Message::CancelInput,
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => Message::ForceQuit,
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => Message::ForceQuit,
            _ => Message::Noop,
        };
    }
    
    // Otherwise, we're in vault selection mode - special keybindings
    match event.code {
        KeyCode::Char('n') | KeyCode::Char('i') => {
            // Start creating a new vault
            Message::StartCreateVault
        }
        KeyCode::Char('q') => Message::Quit,
        KeyCode::Char('c') | KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            Message::ForceQuit
        }
        KeyCode::Tab | KeyCode::Char('j') | KeyCode::Down => Message::LoginSelectNext,
        KeyCode::BackTab | KeyCode::Char('k') | KeyCode::Up => Message::LoginSelectPrev,
        KeyCode::Enter => {
            // If we have vaults, enter password mode for selected vault
            if !state.registry.entries.is_empty() {
                Message::EnterPasswordMode
            } else {
                Message::Noop
            }
        }
        _ => Message::Noop,
    }
}

/// Route keys in the main screen
fn route_main_key(
    state: &AppState,
    event: KeyEvent,
    keybindings: &KeybindingConfig,
) -> Message {
    // Check for mapped actions first
    if let Some(action) = keybindings.get_action(event) {
        return action_to_message(state, action);
    }

    // Handle unmapped keys based on context
    match state.ui_state.focused_pane {
        Pane::Search => route_search_input(event),
        _ => Message::Noop,
    }
}

/// Convert a key action to a message
fn action_to_message(state: &AppState, action: KeyAction) -> Message {
    match action {
        // Navigation
        KeyAction::MoveUp => match state.ui_state.focused_pane {
            Pane::List => Message::SelectPrevItem,
            Pane::Detail => Message::Scroll(ScrollDirection::Up),
            _ => Message::Noop,
        },
        KeyAction::MoveDown => match state.ui_state.focused_pane {
            Pane::List => Message::SelectNextItem,
            Pane::Detail => Message::Scroll(ScrollDirection::Down),
            _ => Message::Noop,
        },
        KeyAction::MoveLeft => Message::FocusPane(Pane::List),
        KeyAction::MoveRight => Message::FocusPane(Pane::Detail),
        KeyAction::JumpToTop => Message::Scroll(ScrollDirection::Top),
        KeyAction::JumpToBottom => Message::Scroll(ScrollDirection::Bottom),
        KeyAction::PageUp => Message::Scroll(ScrollDirection::PageUp),
        KeyAction::PageDown => Message::Scroll(ScrollDirection::PageDown),

        // Actions
        KeyAction::Select => {
            if let Some(item) = state.selected_item() {
                Message::OpenFloatingWindow(FloatingWindow::edit_item_form(item))
            } else {
                Message::Noop
            }
        }
        KeyAction::Back => {
            if state.ui_state.has_floating_window() {
                Message::CloseFloatingWindow
            } else {
                Message::LockVault
            }
        }
        KeyAction::Search => Message::OpenSearch,
        KeyAction::NewItem => {
            Message::OpenFloatingWindow(FloatingWindow::new_kind_selector())
        }
        KeyAction::EditItem => {
            if let Some(item) = state.selected_item() {
                Message::OpenFloatingWindow(FloatingWindow::edit_item_form(item))
            } else {
                Message::Noop
            }
        }
        KeyAction::DeleteItem => {
            if let Some(id) = state.vault_state.as_ref().and_then(|vs| vs.selected_item_id) {
                Message::DeleteItem(id)
            } else {
                Message::Noop
            }
        }
        KeyAction::CopyContent => Message::CopyCurrentItem,
        KeyAction::ToggleReveal => Message::ToggleContentReveal,
        KeyAction::ToggleFavorite => {
            if let Some(id) = state.vault_state.as_ref().and_then(|vs| vs.selected_item_id) {
                Message::ToggleFavorite(id)
            } else {
                Message::Noop
            }
        }
        KeyAction::Save => Message::SaveVault,
        KeyAction::Export => {
            // Quick export to JSON in current directory
            let path = std::path::PathBuf::from("vault_export.json");
            Message::ExportVault {
                format: crate::app::ExportFormat::Json,
                path,
            }
        }

        // Modes
        KeyAction::Help => Message::OpenFloatingWindow(FloatingWindow::Help),
        KeyAction::Settings => Message::Navigate(Screen::Settings),
        KeyAction::Lock => Message::LockVault,
        KeyAction::Quit => Message::Quit,
        KeyAction::ForceQuit => Message::ForceQuit,

        // History
        KeyAction::Undo => Message::Undo,
        KeyAction::Redo => Message::Redo,

        // Focus
        KeyAction::FocusList => Message::FocusPane(Pane::List),
        KeyAction::FocusDetail => Message::FocusPane(Pane::Detail),
        KeyAction::NextPane => {
            let next = match state.ui_state.focused_pane {
                Pane::List => Pane::Detail,
                Pane::Detail => Pane::List,
                Pane::Search => Pane::List,
            };
            Message::FocusPane(next)
        }
        KeyAction::PrevPane => {
            let prev = match state.ui_state.focused_pane {
                Pane::List => Pane::Detail,
                Pane::Detail => Pane::List,
                Pane::Search => Pane::List,
            };
            Message::FocusPane(prev)
        }
    }
}

/// Route keys in floating windows
fn route_floating_window_key(
    state: &AppState,
    event: KeyEvent,
    window: &FloatingWindow,
) -> Message {
    match window {
        FloatingWindow::Search { .. } => route_search_key(event),
        FloatingWindow::ConfirmDelete { item_id } => route_confirm_delete_key(event, *item_id),
        FloatingWindow::Help => route_help_key(event),
        FloatingWindow::KindSelector { .. } => route_kind_selector_key(event),
        FloatingWindow::NewItem { .. } | FloatingWindow::EditItem { .. } => {
            route_edit_form_key(event)
        }
        FloatingWindow::TagFilter => route_tag_filter_key(event),
    }
}

/// Route keys in search dialog
fn route_search_key(event: KeyEvent) -> Message {
    match event.code {
        KeyCode::Char(c) if !event.modifiers.contains(KeyModifiers::CONTROL) => {
            Message::InputChar(c)
        }
        KeyCode::Backspace => Message::InputBackspace,
        KeyCode::Delete => Message::InputDelete,
        KeyCode::Left => Message::InputLeft,
        KeyCode::Right => Message::InputRight,
        KeyCode::Home => Message::InputHome,
        KeyCode::End => Message::InputEnd,
        KeyCode::Enter => Message::SearchConfirm,
        KeyCode::Esc => Message::CloseSearch,
        KeyCode::Down | KeyCode::Tab => Message::SearchNextResult,
        KeyCode::Up | KeyCode::BackTab => Message::SearchPrevResult,
        KeyCode::Char('n') if event.modifiers.contains(KeyModifiers::CONTROL) => Message::SearchNextResult,
        KeyCode::Char('p') if event.modifiers.contains(KeyModifiers::CONTROL) => Message::SearchPrevResult,
        _ => Message::Noop,
    }
}

/// Route keys for text input in search pane
fn route_search_input(event: KeyEvent) -> Message {
    match event.code {
        KeyCode::Char(c) => Message::InputChar(c),
        KeyCode::Backspace => Message::InputBackspace,
        _ => Message::Noop,
    }
}

/// Route keys in delete confirmation
fn route_confirm_delete_key(event: KeyEvent, item_id: uuid::Uuid) -> Message {
    match event.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            Message::ConfirmDeleteItem(item_id)
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Message::CloseFloatingWindow,
        _ => Message::Noop,
    }
}

/// Route keys in help overlay
fn route_help_key(event: KeyEvent) -> Message {
    match event.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
            Message::CloseFloatingWindow
        }
        _ => Message::Noop,
    }
}

/// Route keys in edit form
fn route_edit_form_key(event: KeyEvent) -> Message {
    match event.code {
        KeyCode::Char(c) if !event.modifiers.contains(KeyModifiers::CONTROL) => {
            Message::InputChar(c)
        }
        KeyCode::Backspace => Message::InputBackspace,
        KeyCode::Delete => Message::InputDelete,
        KeyCode::Left => Message::InputLeft,
        KeyCode::Right => Message::InputRight,
        KeyCode::Home => Message::InputHome,
        KeyCode::End => Message::InputEnd,
        KeyCode::Enter => Message::FormSubmit,
        KeyCode::Esc => Message::InputCancel,
        KeyCode::Tab => Message::FormNextField,
        KeyCode::BackTab => Message::FormPrevField,
        _ => Message::Noop,
    }
}

/// Route keys in kind selector
fn route_kind_selector_key(event: KeyEvent) -> Message {
    match event.code {
        KeyCode::Down | KeyCode::Char('j') => Message::KindSelectorNext,
        KeyCode::Up | KeyCode::Char('k') => Message::KindSelectorPrev,
        KeyCode::Enter => Message::KindSelectorConfirm,
        KeyCode::Esc => Message::CloseFloatingWindow,
        _ => Message::Noop,
    }
}

/// Route keys in tag filter
fn route_tag_filter_key(event: KeyEvent) -> Message {
    match event.code {
        KeyCode::Esc => Message::CloseFloatingWindow,
        KeyCode::Down | KeyCode::Char('j') => Message::SelectNextItem,
        KeyCode::Up | KeyCode::Char('k') => Message::SelectPrevItem,
        KeyCode::Enter | KeyCode::Char(' ') => Message::InputSubmit, // Toggle selected tag
        _ => Message::Noop,
    }
}

/// Route keys in settings screen  
fn route_settings_key(event: KeyEvent) -> Message {
    match event.code {
        KeyCode::Esc | KeyCode::Char('q') => Message::Navigate(Screen::Main),
        KeyCode::Down | KeyCode::Char('j') => Message::SelectNextItem,
        KeyCode::Up | KeyCode::Char('k') => Message::SelectPrevItem,
        KeyCode::Enter | KeyCode::Char(' ') => Message::InputSubmit,
        _ => Message::Noop,
    }
}

/// Route a mouse event to a message
fn route_mouse_event(state: &AppState, event: MouseEvent) -> Message {
    // Check if mouse is enabled in config
    if !state.config.mouse_enabled {
        return Message::Noop;
    }

    match event.kind {
        MouseEventKind::ScrollUp => Message::Scroll(ScrollDirection::Up),
        MouseEventKind::ScrollDown => Message::Scroll(ScrollDirection::Down),
        MouseEventKind::Down(button) => {
            // Handle clicks based on current screen and region
            if button != crossterm::event::MouseButton::Left {
                return Message::Noop;
            }

            let click_x = event.column;
            let click_y = event.row;
            
            // Find which UI region was clicked
            if let Some(region) = state.ui_state.layout_regions.find_region(click_x, click_y) {
                return handle_click_in_region(state, region, click_x, click_y);
            }
            
            Message::Noop
        }
        _ => Message::Noop,
    }
}

/// Handle a click within a specific UI region
fn handle_click_in_region(
    state: &AppState,
    region: crate::input::mouse::UiRegion,
    x: u16,
    y: u16,
) -> Message {
    use crate::input::mouse::UiRegion;
    
    match state.screen {
        Screen::Login => {
            // On login screen, List region contains vault items
            if region == UiRegion::List {
                // Calculate which vault item was clicked
                // We need to find the item index from y-coordinate
                // This requires knowing the list's inner y position
                // For now, we'll use a simple heuristic based on stored regions
                
                // Count how many regions we have (one per vault item)
                // The vault index equals the number of List regions above this click
                let vault_index = calculate_vault_index_from_click(state, y);
                if vault_index < state.registry.entries.len() {
                    return Message::LoginSelectVault(vault_index);
                }
            }
            Message::Noop
        }
        Screen::Main => {
            // Handle main screen clicks
            match region {
                UiRegion::List => {
                    // Focus the list pane when clicked
                    Message::FocusPane(Pane::List)
                }
                UiRegion::Detail => {
                    // Focus the detail pane when clicked
                    Message::FocusPane(Pane::Detail)
                }
                _ => Message::Noop,
            }
        }
        _ => Message::Noop,
    }
}

/// Calculate vault index from click y-coordinate
/// This is a helper for login screen vault list clicks
fn calculate_vault_index_from_click(state: &AppState, click_y: u16) -> usize {
    // The vault items start at a specific y coordinate
    // We registered each item with its y position
    // Count how many items are above or at this y position
    let mut count = 0;
    for (i, _entry) in state.registry.entries.iter().enumerate() {
        // Each item is at base_y + index
        // For simplicity, we'll check if click_y matches expected position
        // The first item is typically at y=4 (after header and border)
        // This is a heuristic - ideally we'd store item positions
        let expected_y = 4 + i as u16; // Approximate
        if click_y >= expected_y && i < state.registry.entries.len() {
            count = i;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppConfig, VaultRegistry};

    fn test_state() -> AppState {
        let config = AppConfig::default();
        let registry = VaultRegistry::default();
        AppState::new(config, registry)
    }

    #[test]
    fn test_login_routing() {
        let mut state = test_state();
        
        // Test vault selection mode (default) - 'a' should be ignored, 'n' should start create
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::Noop));
        
        let event = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::StartCreateVault));
        
        // Now test password entry mode - 'a' should be input
        state.login_screen.entering_password = true;
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::InputChar('a')));
        
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::InputSubmit));
    }

    #[test]
    fn test_force_quit() {
        let state = test_state();
        
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::ForceQuit));
    }
}
