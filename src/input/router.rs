//! Event routing - context-aware message dispatch
//!
//! Routes input events to appropriate messages based on current app state.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::app::{AppState, FloatingWindow, Message, Pane, Screen, ScrollDirection};

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

/// Route keys in vault delete confirmation
fn route_confirm_delete_vault_key(event: KeyEvent, index: usize) -> Message {
    match event.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            Message::ConfirmDeleteVault(index)
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Message::CloseFloatingWindow,
        _ => Message::Noop,
    }
}

/// Route a key event to a message
fn route_key_event(state: &AppState, event: KeyEvent, keybindings: &KeybindingConfig) -> Message {
    // Handle floating windows first
    if let Some(ref window) = state.ui_state.floating_window {
        return route_floating_window_key(state, event, window);
    }

    // Handle screen-specific input
    match state.screen {
        Screen::Login => route_login_key(state, event),
        Screen::PasswordRecovery => route_password_recovery_key(event),
        Screen::Main => route_main_key(state, event, keybindings),
        Screen::Settings => route_settings_key(state, event),
        _ => Message::Noop,
    }
}

/// Route keys in the login screen
fn route_login_key(state: &AppState, event: KeyEvent) -> Message {
    // Check login screen mode
    let login_state = &state.login_screen;

    // If creating a new vault, treat it as a form
    if login_state.creating_vault {
        return match event.code {
            KeyCode::Char(c) if !event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::InputChar(c)
            }
            KeyCode::Backspace => Message::InputBackspace,
            KeyCode::Delete => Message::InputDelete,
            KeyCode::Left => {
                // If it's a Back button, or just general left navigation, we map to LoginPrevStep
                // but let's just make Esc the dedicated key.
                Message::InputLeft
            }
            KeyCode::Right => Message::InputRight,
            KeyCode::Home => Message::InputHome,
            KeyCode::End => Message::InputEnd,
            KeyCode::Tab => Message::FormNextField,
            KeyCode::BackTab => Message::FormPrevField,
            KeyCode::Down => Message::FormNextField,
            KeyCode::Up => Message::FormPrevField,
            KeyCode::Enter => Message::InputSubmit,
            KeyCode::Esc => {
                if login_state.create_vault_form.step
                    != crate::ui::screens::login::CreateVaultStep::Step1
                {
                    Message::LoginPrevStep
                } else {
                    Message::CancelInput
                }
            }
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::ForceQuit
            }
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::ForceQuit
            }
            _ => Message::Noop,
        };
    }

    // If entering keyfile path, all input goes to keyfile path field
    if login_state.entering_keyfile_path {
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
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::ForceQuit
            }
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::ForceQuit
            }
            _ => Message::Noop,
        };
    }

    // If entering password, all input goes to password field
    if login_state.entering_password {
        return match event.code {
            KeyCode::Char('f') if !event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::StartPasswordRecovery
            }
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
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::ForceQuit
            }
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                Message::ForceQuit
            }
            _ => Message::Noop,
        };
    }

    // Otherwise, we're in vault selection mode - special keybindings
    match event.code {
        KeyCode::Char('n') | KeyCode::Char('i') => {
            // Start creating a new vault
            Message::StartCreateVault
        }
        KeyCode::Char('d') => {
            // Delete selected vault
            Message::DeleteSelectedVault
        }
        KeyCode::Char('c') | KeyCode::Char('q')
            if event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            Message::ForceQuit
        }
        KeyCode::Char('q') => Message::Quit,
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

/// Route keys in forgot-password recovery mode
fn route_password_recovery_key(event: KeyEvent) -> Message {
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
        KeyCode::Enter => Message::InputSubmit,
        KeyCode::Esc => Message::CancelInput,
        KeyCode::Char('q') | KeyCode::Char('c')
            if event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            Message::ForceQuit
        }
        _ => Message::Noop,
    }
}

/// Route keys in the main screen
fn route_main_key(state: &AppState, event: KeyEvent, keybindings: &KeybindingConfig) -> Message {
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
                match state.ui_state.focused_pane {
                    Pane::Detail => Message::CopyField(state.ui_state.detail_selected_field),
                    _ => Message::OpenFloatingWindow(FloatingWindow::edit_item_form(item)),
                }
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
        KeyAction::NewItem => Message::OpenFloatingWindow(FloatingWindow::new_kind_selector()),
        KeyAction::EditItem => {
            if let Some(item) = state.selected_item() {
                match state.ui_state.focused_pane {
                    Pane::Detail => Message::EditField(state.ui_state.detail_selected_field),
                    _ => Message::OpenFloatingWindow(FloatingWindow::edit_item_form(item)),
                }
            } else {
                Message::Noop
            }
        }
        KeyAction::DeleteItem => {
            if let Some(id) = state
                .vault_state
                .as_ref()
                .and_then(|vs| vs.selected_item_id)
            {
                Message::DeleteItem(id)
            } else {
                Message::Noop
            }
        }
        KeyAction::CopyContent => match state.ui_state.focused_pane {
            Pane::Detail => Message::CopyField(state.ui_state.detail_selected_field),
            _ => Message::CopyCurrentItem,
        },
        KeyAction::ToggleReveal => Message::ToggleContentReveal,
        KeyAction::ToggleFavorite => {
            if let Some(id) = state
                .vault_state
                .as_ref()
                .and_then(|vs| vs.selected_item_id)
            {
                Message::ToggleFavorite(id)
            } else {
                Message::Noop
            }
        }
        KeyAction::Save => Message::SaveVault,
        KeyAction::Export => {
            // Secure-by-default quick export
            let path = std::path::PathBuf::from("vault_export.vault");
            Message::ExportVault {
                format: crate::app::ExportFormat::EncryptedJson,
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

        // Category
        KeyAction::NextCategory => Message::NextCategory,
        KeyAction::PrevCategory => Message::PrevCategory,

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
    _state: &AppState,
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
        FloatingWindow::ConfirmDeleteVault { index, .. } => route_confirm_delete_vault_key(event, *index),
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
        KeyCode::Char('n') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            Message::SearchNextResult
        }
        KeyCode::Char('p') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            Message::SearchPrevResult
        }
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
        KeyCode::Tab | KeyCode::Down => Message::FormNextField,
        KeyCode::BackTab | KeyCode::Up => Message::FormPrevField,
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
fn route_settings_key(state: &AppState, event: KeyEvent) -> Message {
    if state.settings_state.security_action.is_some() {
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
            _ => Message::Noop,
        };
    }

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
        MouseEventKind::ScrollUp => {
            // Context-aware scroll
            let (x, y) = (event.column, event.row);
            if let Some(region) = state.ui_state.layout_regions.find_region(x, y) {
                match region {
                    crate::input::mouse::UiRegion::Detail => Message::Scroll(ScrollDirection::Up),
                    crate::input::mouse::UiRegion::List => {
                        // Scroll list = select previous item
                        Message::SelectPrevItem
                    }
                    _ => Message::Scroll(ScrollDirection::Up),
                }
            } else {
                Message::Scroll(ScrollDirection::Up)
            }
        }
        MouseEventKind::ScrollDown => {
            // Context-aware scroll
            let (x, y) = (event.column, event.row);
            if let Some(region) = state.ui_state.layout_regions.find_region(x, y) {
                match region {
                    crate::input::mouse::UiRegion::Detail => Message::Scroll(ScrollDirection::Down),
                    crate::input::mouse::UiRegion::List => {
                        // Scroll list = select next item
                        Message::SelectNextItem
                    }
                    _ => Message::Scroll(ScrollDirection::Down),
                }
            } else {
                Message::Scroll(ScrollDirection::Down)
            }
        }
        MouseEventKind::Down(button) => {
            // Handle clicks based on current screen and region
            if button != crossterm::event::MouseButton::Left {
                return Message::Noop;
            }

            let click_x = event.column;
            let click_y = event.row;

            // First, check for clickable elements (most specific)
            if let Some(element) = state
                .ui_state
                .layout_regions
                .find_clickable(click_x, click_y)
            {
                // Register click and check for double-click
                let is_double_click = state
                    .ui_state
                    .layout_regions
                    .register_click(click_x, click_y);
                return handle_clickable_element(state, element, click_x, click_y, is_double_click);
            }

            // Fall back to region-based handling
            if let Some(region) = state.ui_state.layout_regions.find_region(click_x, click_y) {
                return handle_click_in_region(state, region, click_x, click_y);
            }

            // Click outside any known region - check if we should close floating window
            if state.ui_state.floating_window.is_some() {
                return Message::CloseFloatingWindow;
            }

            Message::Noop
        }
        _ => Message::Noop,
    }
}

/// Handle click on a specific clickable element
fn handle_clickable_element(
    state: &AppState,
    element: &crate::input::mouse::ClickableElement,
    _x: u16,
    _y: u16,
    is_double_click: bool,
) -> Message {
    use crate::input::mouse::ClickableElement;

    match element {
        ClickableElement::VaultEntry(index) => {
            // On login screen, clicking a vault entry selects it
            if state.login_screen.entering_password || state.login_screen.creating_vault {
                // If already in password/create mode, ignore vault clicks
                Message::Noop
            } else if is_double_click {
                // Double-click opens the vault (enters password mode)
                Message::EnterPasswordMode
            } else {
                // Single click selects
                Message::LoginSelectVault(*index)
            }
        }

        ClickableElement::ListItem(uuid) => {
            // In main screen, clicking an item selects it
            Message::SelectItem(*uuid)
        }

        ClickableElement::FormField(index) => {
            // Clicking a form field focuses it
            Message::FormFocusField(*index)
        }

        ClickableElement::DetailField(index) => {
            // Clicking a detail field copies it
            Message::CopyField(*index)
        }

        ClickableElement::KindOption(index) => {
            // Clicking a kind option selects it
            Message::KindSelectorSelect(*index)
        }

        ClickableElement::CategoryOption(kind) => {
            // Clicking a category sets the kind filter
            Message::SetKindFilter(*kind)
        }

        ClickableElement::CategoryScrollLeft => Message::PrevCategory,
        ClickableElement::CategoryScrollRight => Message::NextCategory,

        ClickableElement::SearchResult(index) => {
            // Clicking a search result selects it
            Message::SelectSearchResult(*index)
        }

        ClickableElement::Button(action) => {
            // Handle button clicks by action name
            match action.as_str() {
                // Login screen buttons
                "new-vault" => Message::StartCreateVault,
                "select-vault" => Message::EnterPasswordMode,
                "delete-vault" => Message::DeleteSelectedVault,
                "quit" => Message::Quit,
                "unlock" => Message::InputSubmit, // Trigger unlock attempt
                "back" => Message::CancelInput,   // Fixed: was InputCancel, now CancelInput
                "forgot-password" => Message::StartPasswordRecovery,
                "submit-recovery" => Message::InputSubmit,
                "prev-step" => Message::LoginPrevStep, // Go back in vault creation
                "save-vault" => Message::InputSubmit,  // Trigger vault creation
                "cancel" => {
                    // In vault creation mode, cancel should exit to login
                    if state.login_screen.creating_vault {
                        Message::CancelInput
                    } else {
                        Message::InputCancel
                    }
                }

                // Item detail buttons
                "reveal" => Message::ToggleContentReveal,
                "copy" => match state.ui_state.focused_pane {
                    Pane::Detail => Message::CopyField(state.ui_state.detail_selected_field),
                    _ => Message::CopyCurrentItem,
                },
                "edit" => {
                    if let Some(item) = state.selected_item() {
                        Message::OpenFloatingWindow(
                            crate::app::state::FloatingWindow::edit_item_form(item),
                        )
                    } else {
                        Message::Noop
                    }
                }
                "delete" => {
                    if let Some(item) = state.selected_item() {
                        Message::DeleteItem(item.id)
                    } else {
                        Message::Noop
                    }
                }

                // Form buttons
                "form-save" => Message::FormSubmit,
                "form-cancel" => Message::CloseFloatingWindow,

                // Confirm delete buttons
                "confirm-delete" => {
                    // Get item ID from confirm delete window
                    if let Some(crate::app::state::FloatingWindow::ConfirmDelete { item_id }) =
                        &state.ui_state.floating_window
                    {
                        Message::ConfirmDeleteItem(*item_id)
                    } else {
                        Message::Noop
                    }
                }
                "cancel-delete" => Message::CloseFloatingWindow,
                "confirm-delete-vault" => {
                    if let Some(crate::app::state::FloatingWindow::ConfirmDeleteVault { index, .. }) =
                        &state.ui_state.floating_window
                    {
                        Message::ConfirmDeleteVault(*index)
                    } else {
                        Message::Noop
                    }
                }
                "cancel-delete-vault" => Message::CloseFloatingWindow,

                // Legacy buttons
                "submit" | "save" => Message::FormSubmit,
                "confirm" => Message::InputSubmit,
                "enter_password" => Message::EnterPasswordMode,

                _ => Message::Noop,
            }
        }

        ClickableElement::CloseArea => {
            // Clicking close area closes the floating window
            Message::CloseFloatingWindow
        }
    }
}

/// Handle a click within a specific UI region
fn handle_click_in_region(
    state: &AppState,
    region: crate::input::mouse::UiRegion,
    _x: u16,
    _y: u16,
) -> Message {
    use crate::input::mouse::UiRegion;

    match state.screen {
        Screen::Login => {
            // Clicks in login screen handled by clickable elements now
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
                UiRegion::FloatingWindow => {
                    // Click inside floating window - don't close it
                    Message::Noop
                }
                _ => Message::Noop,
            }
        }
        _ => Message::Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AppMode, FloatingWindow, Screen, VaultState};
    use crate::domain::{Item, Vault};
    use crate::input::mouse::ClickableElement;
    use crate::storage::{AppConfig, VaultRegistry};
    use std::path::PathBuf;

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

        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::InputChar('b')));

        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::InputSubmit));

        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::CancelInput));
    }

    #[test]
    fn test_force_quit() {
        let state = test_state();

        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::ForceQuit));

        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::ForceQuit));
    }

    #[test]
    fn test_keyfile_path_mode_accepts_text_input() {
        let mut state = test_state();
        state.login_screen.entering_keyfile_path = true;

        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::InputChar('b')));

        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let msg = route_key_event(&state, event, &KeybindingConfig::default());
        assert!(matches!(msg, Message::CancelInput));
    }

    #[test]
    fn test_delete_button_and_confirm_button_mapping() {
        let mut state = test_state();
        let mut vault = Vault::new("Test");
        let item = Item::password("GitHub", "secret123");
        let item_id = item.id;
        vault.add_item(item);

        state.vault_state = Some(VaultState::new(
            vault,
            PathBuf::from("/tmp/test.vault"),
            [0u8; 32],
            [0u8; 32],
            false,
            crate::crypto::EncryptionMethod::Aes256Gcm,
            None,
        ));
        state.vault_state.as_mut().unwrap().selected_item_id = Some(item_id);
        state.mode = AppMode::Unlocked;
        state.screen = Screen::Main;

        let msg = handle_clickable_element(
            &state,
            &ClickableElement::Button("delete".to_string()),
            0,
            0,
            false,
        );
        assert!(matches!(msg, Message::DeleteItem(id) if id == item_id));

        state.ui_state.floating_window = Some(FloatingWindow::ConfirmDelete { item_id });
        let msg = handle_clickable_element(
            &state,
            &ClickableElement::Button("confirm-delete".to_string()),
            0,
            0,
            false,
        );
        assert!(matches!(msg, Message::ConfirmDeleteItem(id) if id == item_id));
    }
}
