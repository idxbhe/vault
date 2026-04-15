//! State update logic - the heart of TEA
//!
//! The update function takes the current state and a message, returning
//! the new state and any effects to execute.

use std::time::Duration;

use uuid::Uuid;

use crate::domain::{Item, ItemKind};
use crate::ui::screens::{
    AddKeyfileAction, AddKeyfileStep, ChangePasswordAction, ChangePasswordStep,
    ManageRecoveryAction, ManageRecoveryStep, ManageRecoveryTarget, RecoveryQuestionDraft,
    RecoverySetupAction, RecoverySetupStep, SecurityActionState, SettingKind, apply_setting,
    get_current_sub_index,
};

use super::effect::Effect;
use super::message::{ConfigUpdate, ItemUpdates, Message, ScrollDirection};
use super::state::{
    AppMode, AppState, FloatingWindow, ItemSnapshot, NotificationLevel, Pane, Screen, UndoEntry,
    VaultState,
};

/// Update the application state based on a message
///
/// Returns the effect(s) to execute as a result of the update.
pub fn update(state: &mut AppState, message: Message) -> Effect {
    // Update last activity time for real user actions only.
    // Timer-driven tick must not reset idle timeout.
    if !matches!(&message, Message::Tick | Message::Noop)
        && let Some(ref mut vs) = state.vault_state
    {
        vs.touch();
    }

    match message {
        // === Navigation ===
        Message::Navigate(screen) => {
            if state.screen == Screen::Settings && screen != Screen::Settings {
                state.settings_state.cancel_edit();
                state.settings_state.security_action = None;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
            }
            state.screen = screen;
            Effect::none()
        }

        Message::FocusPane(pane) => {
            state.ui_state.focused_pane = pane;
            if pane == Pane::Detail {
                state.ui_state.detail_scroll_offset = 0;
                state.ui_state.detail_focus = crate::app::state::DetailFocus::Field(0);
                state.ui_state.notes_scroll_offset = 0;
                state.ui_state.field_scrolls.clear();
            }
            Effect::none()
        }

        // === Vault Operations ===
        Message::UnlockVault { password, keyfile } => {
            let selected_idx = state.login_screen.selected_vault;
            let Some(entry) = state.registry.entries.get(selected_idx) else {
                state.login_screen.error_message = Some("No vault selected".to_string());
                return Effect::none();
            };

            let keyfile_data = if let Some(path) = keyfile {
                match crate::crypto::KeyFile::load(&path) {
                    Ok(kf) => Some(kf.as_bytes().to_vec()),
                    Err(e) => {
                        state.login_screen.error_message =
                            Some(format!("Failed to read keyfile: {}", e));
                        return Effect::none();
                    }
                }
            } else {
                None
            };

            state.ui_state.start_loading("Unlocking vault...");
            Effect::ReadVaultFile {
                path: entry.path.clone(),
                password,
                keyfile: keyfile_data,
            }
        }

        Message::LockVault => {
            if let Some(vs) = state.vault_state.as_ref() {
                if vs.is_dirty {
                    let path = vs.vault_path.clone();
                    let vault = vs.vault.clone();
                    let key = vs.encryption_key;
                    let salt = vs.salt;

                    // Keep unlocked state until write succeeds.
                    state.pending_lock = true;
                    Effect::WriteVaultFile {
                        path,
                        vault,
                        key,
                        salt,
                        has_keyfile: vs.has_keyfile,
                        encryption_method: vs.encryption_method,
                        recovery_metadata: vs.recovery_metadata.clone(),
                    }
                } else {
                    transition_to_locked_state(state);
                    Effect::none()
                }
            } else {
                state.pending_lock = false;
                Effect::none()
            }
        }

        Message::SaveVault => {
            if let Some(vs) = state.vault_state.as_ref() {
                Effect::WriteVaultFile {
                    path: vs.vault_path.clone(),
                    vault: vs.vault.clone(),
                    key: vs.encryption_key,
                    salt: vs.salt,
                    has_keyfile: vs.has_keyfile,
                    encryption_method: vs.encryption_method,
                    recovery_metadata: vs.recovery_metadata.clone(),
                }
            } else {
                Effect::none()
            }
        }

        Message::CloseVault => {
            if state.is_dirty() {
                // Prompt to save first
                state.ui_state.floating_window = Some(FloatingWindow::ConfirmDelete {
                    item_id: Uuid::nil(),
                });
                Effect::none()
            } else {
                transition_to_locked_state(state);
                Effect::none()
            }
        }

        // === Login Flow ===
        Message::StartCreateVault => {
            // Switch login screen to "creating" mode
            state.login_screen.reset_create_form();
            state.login_screen.creating_vault = true;
            state.login_screen.entering_password = false;
            state.login_screen.entering_keyfile_path = false;
            state.login_screen.password_recovery = None;
            state.login_screen.pending_unlock_password = None;
            state.login_screen.error_message = None;
            state.ui_state.input_buffer.clear();

            state.ui_state.input_buffer.masked = false;
            Effect::none()
        }

        Message::EnterPasswordMode => {
            // Switch login screen to "password entry" mode
            state.login_screen.enter_password_mode();
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = true; // Password is masked
            Effect::none()
        }

        Message::StartPasswordRecovery => {
            if !state.login_screen.entering_password {
                return Effect::none();
            }

            let selected_idx = state.login_screen.selected_vault;
            let Some(entry) = state.registry.entries.get(selected_idx) else {
                state.login_screen.error_message = Some("No vault selected".to_string());
                return Effect::none();
            };

            let header = match crate::storage::vault_file::read_header(&entry.path) {
                Ok(header) => header,
                Err(e) => {
                    state.login_screen.error_message =
                        Some(format!("Failed to read vault header: {}", e));
                    return Effect::none();
                }
            };

            let Some(recovery_metadata) = header.recovery_metadata else {
                state.login_screen.error_message =
                    Some("Recovery is not configured for this vault".to_string());
                return Effect::none();
            };

            if !recovery_metadata.is_configured() {
                state.login_screen.error_message =
                    Some("Recovery metadata is incomplete for this vault".to_string());
                return Effect::none();
            }

            state.login_screen.password_recovery =
                Some(crate::ui::screens::login::PasswordRecoverySession::new(
                    entry.name.clone(),
                    entry.path.clone(),
                    recovery_metadata,
                ));
            state.login_screen.error_message = None;
            state.screen = Screen::PasswordRecovery;
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = true;
            Effect::none()
        }

        Message::LoginPrevStep => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                let form = &mut state.login_screen.create_vault_form;
                if form.step == crate::ui::screens::login::CreateVaultStep::Step2 {
                    form.step = crate::ui::screens::login::CreateVaultStep::Step1;
                    form.focused_field = crate::ui::screens::login::CreateVaultField::Name;
                    state.login_screen.error_message = None;
                } else if form.step == crate::ui::screens::login::CreateVaultStep::Step3 {
                    form.step = crate::ui::screens::login::CreateVaultStep::Step2;
                    form.focused_field = crate::ui::screens::login::CreateVaultField::Password;
                    state.login_screen.error_message = None;
                }
            }
            Effect::none()
        }

        Message::CancelInput => {
            if state.screen == Screen::PasswordRecovery {
                state.screen = Screen::Login;
                state.login_screen.password_recovery = None;
                state.login_screen.entering_password = true;
                state.login_screen.entering_keyfile_path = false;
                state.login_screen.error_message = None;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = true;
                return Effect::none();
            }

            if state.screen == Screen::Settings && state.settings_state.security_action.is_some() {
                state.settings_state.security_action = None;
                state.login_screen.error_message = None;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
                return Effect::none();
            }

            if state.login_screen.entering_keyfile_path {
                state.login_screen.entering_keyfile_path = false;
                state.login_screen.entering_password = true;
                state.login_screen.pending_unlock_password = None;
                state.login_screen.error_message = None;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = true;
                return Effect::none();
            }

            // Cancel any input mode and return to vault selection
            state.login_screen.entering_password = false;
            state.login_screen.entering_keyfile_path = false;
            state.login_screen.reset_create_form();
            state.login_screen.pending_unlock_password = None;
            state.login_screen.password_recovery = None;
            state.login_screen.error_message = None;
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = false;
            state.ui_state.floating_window = None;
            Effect::none()
        }

        Message::DeleteSelectedVault => {
            if !state.registry.entries.is_empty() {
                let selected_index = state.login_screen.selected_vault;
                if selected_index < state.registry.entries.len() {
                    let vault_name = state.registry.entries[selected_index].name.clone();
                    state.ui_state.floating_window = Some(FloatingWindow::ConfirmDeleteVault {
                        vault_name,
                        index: selected_index,
                    });
                }
            }
            Effect::none()
        }

        Message::ConfirmDeleteVault(index) => {
            state.ui_state.close_floating_window();

            if index < state.registry.entries.len() {
                let vault_name = state.registry.entries[index].name.clone();

                // Remove from registry
                state.registry.entries.remove(index);

                // Save updated registry
                let effect = match state.registry.save() {
                    Ok(_) => {
                        state.ui_state.notify(
                            format!("Deleted vault: {}", vault_name),
                            NotificationLevel::Info,
                        );
                        Effect::none()
                    }
                    Err(e) => {
                        state.ui_state.notify(
                            format!("Failed to save registry: {}", e),
                            NotificationLevel::Error,
                        );
                        Effect::none()
                    }
                };

                // Adjust selected index if needed
                if state.login_screen.selected_vault >= state.registry.entries.len()
                    && !state.registry.entries.is_empty()
                {
                    state.login_screen.selected_vault = state.registry.entries.len() - 1;
                }

                effect
            } else {
                Effect::none()
            }
        }

        // === Login Screen Navigation ===
        Message::LoginSelectNext => {
            let vault_count = state.registry.entries.len();
            state.login_screen.select_next(vault_count);
            Effect::none()
        }

        Message::LoginSelectPrev => {
            let vault_count = state.registry.entries.len();
            state.login_screen.select_prev(vault_count);
            Effect::none()
        }

        Message::LoginSelectVault(index) => {
            if index < state.registry.entries.len() {
                state.login_screen.selected_vault = index;
                // Clear any previous error when selecting a vault
                state.login_screen.error_message = None;
            }
            Effect::none()
        }

        // === Item Operations ===
        Message::SelectItem(id) => {
            if let Some(ref mut vs) = state.vault_state
                && vs.vault.get_item(id).is_some()
            {
                vs.selected_item_id = Some(id);
                state.ui_state.detail_scroll_offset = 0;
                state.ui_state.detail_focus = crate::app::state::DetailFocus::Field(0);
                state.ui_state.notes_scroll_offset = 0;
                state.ui_state.field_scrolls.clear();
            }
            Effect::none()
        }

        Message::SelectNextItem => {
            if state.screen == Screen::Settings {
                // If ManageRecovery QuestionList is active, navigate in question list
                let is_question_list = matches!(
                    &state.settings_state.security_action,
                    Some(SecurityActionState::ManageRecovery(a))
                        if a.step == ManageRecoveryStep::QuestionList
                );
                if is_question_list {
                    let q_count = state
                        .vault_state
                        .as_ref()
                        .and_then(|vs| vs.recovery_metadata.as_ref())
                        .map(|m| m.questions.len())
                        .unwrap_or(0);
                    if let Some(SecurityActionState::ManageRecovery(ref mut a)) =
                        state.settings_state.security_action
                    {
                        if q_count > 0 && a.selected_idx < q_count - 1 {
                            a.selected_idx += 1;
                        }
                    }
                    return Effect::none();
                }

                let max_items = crate::ui::screens::SettingKind::all().len();
                let max_sub_items = settings_option_count(state, state.settings_state.selected);
                state.settings_state.move_down(max_items, max_sub_items);
                return Effect::none();
            }
            select_adjacent_item(state, 1);
            Effect::none()
        }

        Message::SelectPrevItem => {
            if state.screen == Screen::Settings {
                // If ManageRecovery QuestionList is active, navigate in question list
                let is_question_list = matches!(
                    &state.settings_state.security_action,
                    Some(SecurityActionState::ManageRecovery(a))
                        if a.step == ManageRecoveryStep::QuestionList
                );
                if is_question_list {
                    if let Some(SecurityActionState::ManageRecovery(ref mut a)) =
                        state.settings_state.security_action
                    {
                        a.selected_idx = a.selected_idx.saturating_sub(1);
                    }
                    return Effect::none();
                }

                state.settings_state.move_up();
                return Effect::none();
            }
            select_adjacent_item(state, -1);
            Effect::none()
        }


        Message::CreateItem { kind } => {
            if let Some(ref mut vs) = state.vault_state {
                let item = Item::new("New Item", kind, kind.default_content());
                let id = item.id;

                // Open edit dialog for the new item
                state.ui_state.floating_window = Some(FloatingWindow::edit_item_form(&item));

                vs.vault.add_item(item);
                vs.selected_item_id = Some(id);
                vs.mark_dirty();
            }
            Effect::none()
        }

        Message::UpdateItem { id, updates } => {
            if let Some(ref mut vs) = state.vault_state
                && let Some(item) = vs.vault.get_item(id)
            {
                // Save undo entry before modifying
                let undo_entry = UndoEntry {
                    description: format!("Edit {}", item.title),
                    item_id: id,
                    previous_state: ItemSnapshot::from_item(item),
                };

                // Apply updates
                if let Some(item) = vs.vault.get_item_mut(id) {
                    apply_item_updates(item, updates);
                }

                vs.push_undo(undo_entry);
                vs.mark_dirty();
            }
            Effect::none()
        }

        Message::DeleteItem(id) => {
            // Show confirmation dialog
            state.ui_state.floating_window = Some(FloatingWindow::ConfirmDelete { item_id: id });
            Effect::none()
        }

        Message::ConfirmDeleteItem(id) => {
            state.ui_state.close_floating_window();

            if let Some(ref mut vs) = state.vault_state
                && let Some(item) = vs.vault.get_item(id)
            {
                // Save undo entry
                let undo_entry = UndoEntry {
                    description: format!("Delete {}", item.title),
                    item_id: id,
                    previous_state: ItemSnapshot::from_item(item),
                };

                vs.vault.remove_item(id);
                vs.push_undo(undo_entry);
                vs.mark_dirty();

                // Clear selection if deleted item was selected
                if vs.selected_item_id == Some(id) {
                    vs.selected_item_id = vs.vault.items.first().map(|i| i.id);
                }
            }
            Effect::none()
        }

        Message::ToggleFavorite(id) => {
            if let Some(ref mut vs) = state.vault_state
                && let Some(item) = vs.vault.get_item_mut(id)
            {
                item.favorite = !item.favorite;
                item.touch();
                vs.mark_dirty();
            }
            Effect::none()
        }

        Message::DuplicateItem(id) => {
            if let Some(ref mut vs) = state.vault_state
                && let Some(item) = vs.vault.get_item(id)
            {
                let mut new_item = item.clone();
                new_item.id = Uuid::new_v4();
                new_item.title = format!("{} (Copy)", item.title);
                let new_id = new_item.id;
                vs.vault.add_item(new_item);
                vs.selected_item_id = Some(new_id);
                vs.mark_dirty();
            }
            Effect::none()
        }

        // === History ===
        Message::Undo => {
            if let Some(ref mut vs) = state.vault_state
                && let Some(entry) = vs.undo_stack.pop()
            {
                // Save current state to redo stack
                if let Some(current) = vs.vault.get_item(entry.item_id) {
                    let redo_entry = UndoEntry {
                        description: entry.description.clone(),
                        item_id: entry.item_id,
                        previous_state: ItemSnapshot::from_item(current),
                    };
                    vs.redo_stack.push(redo_entry);
                }

                // Restore previous state
                if let Some(item) = vs.vault.get_item_mut(entry.item_id) {
                    *item = entry.previous_state.item;
                } else {
                    // Item was deleted, restore it
                    vs.vault.add_item(entry.previous_state.item);
                }

                vs.mark_dirty();
                state.ui_state.notify("Undone", NotificationLevel::Info);
            }
            Effect::none()
        }

        Message::Redo => {
            if let Some(ref mut vs) = state.vault_state
                && let Some(entry) = vs.redo_stack.pop()
            {
                // Save current state to undo stack
                if let Some(current) = vs.vault.get_item(entry.item_id) {
                    let undo_entry = UndoEntry {
                        description: entry.description.clone(),
                        item_id: entry.item_id,
                        previous_state: ItemSnapshot::from_item(current),
                    };
                    vs.undo_stack.push(undo_entry);
                }

                // Apply redo state
                if let Some(item) = vs.vault.get_item_mut(entry.item_id) {
                    *item = entry.previous_state.item;
                }

                vs.mark_dirty();
                state.ui_state.notify("Redone", NotificationLevel::Info);
            }
            Effect::none()
        }

        // === Search ===
        Message::OpenSearch => {
            state.ui_state.floating_window = Some(FloatingWindow::new_search());
            state.ui_state.focused_pane = Pane::Search;
            Effect::none()
        }

        Message::CloseSearch => {
            state.ui_state.close_floating_window();
            state.ui_state.focused_pane = Pane::List;
            Effect::none()
        }

        Message::UpdateSearchQuery(query) => {
            if let Some(FloatingWindow::Search {
                state: search_state,
            }) = &mut state.ui_state.floating_window
            {
                search_state.query = query;
                if let Some(ref vs) = state.vault_state {
                    search_state.update_results(&vs.vault.items);
                }
            }
            Effect::none()
        }

        Message::ExecuteSearch => {
            if let Some(FloatingWindow::Search {
                state: search_state,
            }) = &mut state.ui_state.floating_window
                && let Some(ref vs) = state.vault_state
            {
                search_state.update_results(&vs.vault.items);
            }
            Effect::none()
        }

        Message::SelectSearchResult(index) => {
            let selected_id = if let Some(FloatingWindow::Search {
                state: search_state,
            }) = &state.ui_state.floating_window
            {
                search_state.results.get(index).map(|r| r.item_id)
            } else {
                None
            };

            if let Some(id) = selected_id {
                if let Some(ref mut vs) = state.vault_state {
                    vs.selected_item_id = Some(id);
                }
                state.ui_state.close_floating_window();
                state.ui_state.focused_pane = Pane::Detail;
            }
            Effect::none()
        }

        Message::SearchNextResult => {
            if let Some(FloatingWindow::Search {
                state: search_state,
            }) = &mut state.ui_state.floating_window
            {
                search_state.next_result();
            }
            Effect::none()
        }

        Message::SearchPrevResult => {
            if let Some(FloatingWindow::Search {
                state: search_state,
            }) = &mut state.ui_state.floating_window
            {
                search_state.prev_result();
            }
            Effect::none()
        }

        Message::SearchConfirm => {
            let selected_id = if let Some(FloatingWindow::Search {
                state: search_state,
            }) = &state.ui_state.floating_window
            {
                search_state.selected_item_id()
            } else {
                None
            };

            if let Some(id) = selected_id {
                if let Some(ref mut vs) = state.vault_state {
                    vs.selected_item_id = Some(id);
                }
                state.ui_state.close_floating_window();
                state.ui_state.focused_pane = Pane::Detail;
            }
            Effect::none()
        }

        // === Clipboard ===
        Message::CopyToClipboard {
            content,
            is_sensitive,
        } => {
            let delay = if is_sensitive {
                state
                    .clipboard_state
                    .set_secure(state.config.clipboard_timeout_secs);
                Some(Duration::from_secs(state.config.clipboard_timeout_secs))
            } else {
                None
            };

            state
                .ui_state
                .notify("Copied to clipboard", NotificationLevel::Success);

            let mut effects = vec![Effect::SetClipboard {
                content,
                is_sensitive,
            }];

            if let Some(delay) = delay {
                effects.push(Effect::ScheduleClipboardClear { delay });
            }

            Effect::batch(effects)
        }

        Message::CopyCurrentItem => {
            if let Some(item) = state.selected_item()
                && let Some(content) = item.get_copyable_content()
            {
                return update(
                    state,
                    Message::CopyToClipboard {
                        content: content,
                        is_sensitive: true,
                    },
                );
            }
            Effect::none()
        }

        Message::CopyField(index) => {
            if let Some(item) = state.selected_item() {
                let fields = item.get_fields();
                if let Some((_, value, is_sensitive, _)) = fields.get(index) {
                    return update(
                        state,
                        Message::CopyToClipboard {
                            content: value.clone(),
                            is_sensitive: *is_sensitive,
                        },
                    );
                }
            }
            Effect::none()
        }

        Message::FocusDetailNotes => {
            state.ui_state.detail_focus = crate::app::state::DetailFocus::Notes;
            Effect::none()
        }

        Message::EditNotes => {
            if let Some(item) = state.selected_item() {
                let target_form_field = Some(crate::ui::widgets::FormField::Notes);
                let msg = Message::OpenFloatingWindow(FloatingWindow::edit_item_form(item));
                let eff = update(state, msg);

                if let Some(FloatingWindow::EditItem { ref mut form, .. }) = state.ui_state.floating_window {
                    if let Some(target) = target_form_field {
                        if let Some(pos) = form.fields.iter().position(|f| *f == target) {
                            form.focused_field = pos;
                            form.target_field = Some(target);
                            form.cursor = form.values[pos].len();
                        }
                    }
                }
                return eff;
            }
            Effect::none()
        }

        Message::EditField(index) => {
            // Get the item, build edit form, and try to focus the specific field index
            if let Some(item) = state.selected_item() {
                let fields = item.get_fields();
                let target_form_field = fields.get(index).and_then(|f| f.3.clone());

                let msg = Message::OpenFloatingWindow(FloatingWindow::edit_item_form(item));

                // Then we apply it, and then modify the focused field manually in state since EditItem form is created
                let eff = update(state, msg);

                if let Some(FloatingWindow::EditItem { ref mut form, .. }) =
                    state.ui_state.floating_window
                {
                    if let Some(target) = target_form_field.clone() {
                        if let Some(pos) = form.fields.iter().position(|f| *f == target) {
                            form.focused_field = pos;
                            form.target_field = Some(target);
                            form.cursor = form.values[pos].len();
                        }
                    }
                }

                return eff;
            }
            Effect::none()
        }

        Message::ClearClipboard => {
            state.clipboard_state.clear();
            Effect::ClearClipboard
        }

        // === UI ===
        Message::ToggleContentReveal => {
            state.ui_state.content_revealed = !state.ui_state.content_revealed;
            Effect::none()
        }

        Message::OpenFloatingWindow(window) => {
            state.ui_state.floating_window = Some(window);
            Effect::none()
        }

        Message::CloseFloatingWindow => {
            state.ui_state.close_floating_window();
            Effect::none()
        }

        Message::ShowNotification { message, level } => {
            state.ui_state.notify(message, level);
            Effect::none()
        }

        Message::DismissNotification(id) => {
            state.ui_state.notifications.retain(|n| n.id != id);
            Effect::none()
        }

        Message::Scroll(direction) => {
            handle_scroll(state, direction);
            Effect::none()
        }

        // === Input ===
        Message::InputChar(c) => {
            // Clear login error when user starts typing
            if state.login_screen.error_message.is_some() {
                state.login_screen.error_message = None;
            }

            // Check if we're in a form or search
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                if let Some(buf) = state.login_screen.create_vault_form.active_input_mut() {
                    buf.insert(c);
                }
            } else {
                match &mut state.ui_state.floating_window {
                    Some(FloatingWindow::NewItem { form })
                    | Some(FloatingWindow::EditItem { form, .. }) => {
                        form.insert(c);
                    }
                    Some(FloatingWindow::Search {
                        state: search_state,
                    }) => {
                        search_state.insert(c);
                        if let Some(ref vs) = state.vault_state {
                            search_state.update_results(&vs.vault.items);
                        }
                    }
                    _ => {
                        state.ui_state.input_buffer.insert(c);
                    }
                }
            }
            Effect::none()
        }

        Message::InputBackspace => {
            // Clear login error when user starts typing
            if state.login_screen.error_message.is_some() {
                state.login_screen.error_message = None;
            }

            if state.screen == Screen::Login && state.login_screen.creating_vault {
                if let Some(buf) = state.login_screen.create_vault_form.active_input_mut() {
                    buf.backspace();
                }
            } else {
                match &mut state.ui_state.floating_window {
                    Some(FloatingWindow::NewItem { form })
                    | Some(FloatingWindow::EditItem { form, .. }) => {
                        form.backspace();
                    }
                    Some(FloatingWindow::Search {
                        state: search_state,
                    }) => {
                        search_state.backspace();
                        if let Some(ref vs) = state.vault_state {
                            search_state.update_results(&vs.vault.items);
                        }
                    }
                    _ => {
                        state.ui_state.input_buffer.backspace();
                    }
                }
            }
            Effect::none()
        }

        Message::InputDelete => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                if let Some(buf) = state.login_screen.create_vault_form.active_input_mut() {
                    buf.delete();
                }
            } else {
                state.ui_state.input_buffer.delete();
            }
            Effect::none()
        }

        Message::InputLeft => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                let focused_field = state.login_screen.create_vault_form.focused_field;
                if focused_field == crate::ui::screens::login::CreateVaultField::EncryptionMethod {
                    let methods = crate::crypto::EncryptionMethod::all();
                    let current = state.login_screen.create_vault_form.encryption_method;
                    if let Some(idx) = methods.iter().position(|&m| m == current) {
                        let next_idx = if idx == 0 { methods.len() - 1 } else { idx - 1 };
                        state.login_screen.create_vault_form.encryption_method = methods[next_idx];
                    }
                } else if focused_field
                    == crate::ui::screens::login::CreateVaultField::RecoveryQuestionsCount
                {
                    let current = state
                        .login_screen
                        .create_vault_form
                        .recovery_questions_count;
                    state
                        .login_screen
                        .create_vault_form
                        .recovery_questions_count = if current == 0 { 3 } else { current - 1 };
                } else if let Some(buf) = state.login_screen.create_vault_form.active_input_mut() {
                    buf.move_left();
                }
            } else {
                match &mut state.ui_state.floating_window {
                    Some(FloatingWindow::NewItem { form })
                    | Some(FloatingWindow::EditItem { form, .. }) => {
                        form.move_left();
                    }
                    Some(FloatingWindow::Search {
                        state: search_state,
                    }) => {
                        search_state.move_left();
                    }
                    _ => {
                        state.ui_state.input_buffer.move_left();
                    }
                }
            }
            Effect::none()
        }

        Message::InputRight => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                let focused_field = state.login_screen.create_vault_form.focused_field;
                if focused_field == crate::ui::screens::login::CreateVaultField::EncryptionMethod {
                    let methods = crate::crypto::EncryptionMethod::all();
                    let current = state.login_screen.create_vault_form.encryption_method;
                    if let Some(idx) = methods.iter().position(|&m| m == current) {
                        let next_idx = (idx + 1) % methods.len();
                        state.login_screen.create_vault_form.encryption_method = methods[next_idx];
                    }
                } else if focused_field
                    == crate::ui::screens::login::CreateVaultField::RecoveryQuestionsCount
                {
                    let current = state
                        .login_screen
                        .create_vault_form
                        .recovery_questions_count;
                    state
                        .login_screen
                        .create_vault_form
                        .recovery_questions_count = (current + 1) % 4;
                } else if let Some(buf) = state.login_screen.create_vault_form.active_input_mut() {
                    buf.move_right();
                }
            } else {
                match &mut state.ui_state.floating_window {
                    Some(FloatingWindow::NewItem { form })
                    | Some(FloatingWindow::EditItem { form, .. }) => {
                        form.move_right();
                    }
                    Some(FloatingWindow::Search {
                        state: search_state,
                    }) => {
                        search_state.move_right();
                    }
                    _ => {
                        state.ui_state.input_buffer.move_right();
                    }
                }
            }
            Effect::none()
        }

        Message::InputUp => {
            match &mut state.ui_state.floating_window {
                Some(FloatingWindow::NewItem { form })
                | Some(FloatingWindow::EditItem { form, .. }) => {
                    form.move_up();
                }
                _ => {}
            }
            Effect::none()
        }

        Message::InputDown => {
            match &mut state.ui_state.floating_window {
                Some(FloatingWindow::NewItem { form })
                | Some(FloatingWindow::EditItem { form, .. }) => {
                    form.move_down();
                }
                _ => {}
            }
            Effect::none()
        }

        Message::InputHome => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                // Not supported
            } else {
                state.ui_state.input_buffer.home();
            }
            Effect::none()
        }

        Message::InputEnd => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                // Not supported
            } else {
                state.ui_state.input_buffer.end();
            }
            Effect::none()
        }

        Message::InputSubmit => {
            // Context-aware submit handling
            if state.screen == Screen::PasswordRecovery {
                return handle_password_recovery_submit(state);
            }

            if state.screen == Screen::Login && state.login_screen.creating_vault {
                let form = &mut state.login_screen.create_vault_form;
                let current_field = form.focused_field;
                let mut advance_step = false;

                if form.step == crate::ui::screens::login::CreateVaultStep::Step1 {
                    if current_field
                        == crate::ui::screens::login::CreateVaultField::EncryptionMethod
                    {
                        advance_step = true;
                    } else {
                        return update(state, Message::FormNextField);
                    }
                } else if form.step == crate::ui::screens::login::CreateVaultStep::Step2 {
                    let use_keyfile_str = form.use_keyfile.text.clone();
                    let use_keyfile = use_keyfile_str.trim().eq_ignore_ascii_case("y")
                        || use_keyfile_str.trim().eq_ignore_ascii_case("yes");

                    if (!use_keyfile
                        && current_field == crate::ui::screens::login::CreateVaultField::UseKeyfile)
                        || (use_keyfile
                            && current_field
                                == crate::ui::screens::login::CreateVaultField::KeyfilePath)
                    {
                        advance_step = true;
                    } else {
                        return update(state, Message::FormNextField);
                    }
                } else if form.step == crate::ui::screens::login::CreateVaultStep::Step3 {
                    let q_count = form.recovery_questions_count;
                    let is_last = (q_count == 0
                        && current_field
                            == crate::ui::screens::login::CreateVaultField::RecoveryQuestionsCount)
                        || (q_count == 1
                            && current_field
                                == crate::ui::screens::login::CreateVaultField::RecoveryAnswer1)
                        || (q_count == 2
                            && current_field
                                == crate::ui::screens::login::CreateVaultField::RecoveryAnswer2)
                        || (current_field
                            == crate::ui::screens::login::CreateVaultField::RecoveryAnswer3);
                    if is_last {
                        return handle_create_vault_submit(state);
                    } else {
                        return update(state, Message::FormNextField);
                    }
                }

                if advance_step {
                    if form.step == crate::ui::screens::login::CreateVaultStep::Step1 {
                        let vault_name = form.name.text.trim().to_string();
                        if vault_name.is_empty() {
                            state.login_screen.error_message =
                                Some("Vault name cannot be empty".to_string());
                            return Effect::none();
                        }
                        form.step = crate::ui::screens::login::CreateVaultStep::Step2;
                        form.focused_field = crate::ui::screens::login::CreateVaultField::Password;
                        state.login_screen.error_message = None;
                        return Effect::none();
                    } else if form.step == crate::ui::screens::login::CreateVaultStep::Step2 {
                        let password = form.password.text.clone();
                        if password.len() < 4 {
                            state.login_screen.error_message =
                                Some("Password must be at least 4 characters".to_string());
                            return Effect::none();
                        }
                        let confirm = form.confirm_password.text.clone();
                        if password != confirm {
                            state.login_screen.error_message =
                                Some("Passwords do not match".to_string());
                            return Effect::none();
                        }
                        let use_keyfile_str = form.use_keyfile.text.clone();
                        let use_keyfile = use_keyfile_str.trim().eq_ignore_ascii_case("y")
                            || use_keyfile_str.trim().eq_ignore_ascii_case("yes");
                        let keyfile_path = form.keyfile_path.text.trim().to_string();
                        if use_keyfile && keyfile_path.is_empty() {
                            state.login_screen.error_message =
                                Some("Keyfile path cannot be empty if using keyfile".to_string());
                            return Effect::none();
                        }
                        form.step = crate::ui::screens::login::CreateVaultStep::Step3;
                        form.focused_field =
                            crate::ui::screens::login::CreateVaultField::RecoveryQuestionsCount;
                        state.login_screen.error_message = None;
                        return Effect::none();
                    }
                }
            }

            if state.screen == Screen::Settings {
                if state.settings_state.security_action.is_some() {
                    return handle_settings_security_action_submit(state);
                }

                if state.settings_state.editing {
                    let selected = state.settings_state.selected;
                    let chosen = state.settings_state.confirm_edit();
                    apply_setting(state, selected, chosen);
                    return Effect::WriteConfig;
                }

                match SettingKind::all().get(state.settings_state.selected) {
                    Some(SettingKind::ChangeMasterPassword) => {
                        state.settings_state.security_action = Some(
                            SecurityActionState::ChangePassword(ChangePasswordAction::default()),
                        );
                        state.ui_state.input_buffer.clear();
                        state.ui_state.input_buffer.masked = true;
                        state.login_screen.error_message = None;
                        return Effect::none();
                    }
                    Some(SettingKind::AddKeyfile) => {
                        state.settings_state.security_action = Some(
                            SecurityActionState::AddKeyfile(AddKeyfileAction::default()),
                        );
                        state.ui_state.input_buffer.clear();
                        state.ui_state.input_buffer.masked = true;
                        state.login_screen.error_message = None;
                        return Effect::none();
                    }
                    Some(SettingKind::ManageRecovery) => {
                        state.settings_state.security_action = Some(
                            SecurityActionState::ManageRecovery(ManageRecoveryAction::default()),
                        );
                        state.ui_state.input_buffer.clear();
                        state.ui_state.input_buffer.masked = true;
                        state.login_screen.error_message = None;
                        return Effect::none();
                    }
                    Some(SettingKind::ConfigureRecovery) => {
                        state.settings_state.security_action = Some(
                            SecurityActionState::ConfigureRecovery(RecoverySetupAction::default()),
                        );
                        state.ui_state.input_buffer.clear();
                        state.ui_state.input_buffer.masked = true;
                        state.login_screen.error_message = None;
                        return Effect::none();
                    }
                    _ => {}
                }

                let current_sub = get_current_sub_index(state, state.settings_state.selected);
                state.settings_state.start_edit(current_sub);
                return Effect::none();
            }

            if state.screen == Screen::Login {
                if state.login_screen.creating_vault {
                    return handle_create_vault_submit(state);
                } else if state.login_screen.entering_password {
                    // Submit password to unlock vault
                    let password = state.ui_state.input_buffer.text.trim().to_string();

                    if password.is_empty() {
                        state.login_screen.error_message =
                            Some("Password cannot be empty".to_string());
                        return Effect::none();
                    }

                    // Get selected vault path from registry
                    let selected_idx = state.login_screen.selected_vault;
                    if let Some(entry) = state.registry.entries.get(selected_idx) {
                        let path = entry.path.clone();

                        let header = match crate::storage::vault_file::read_header(&path) {
                            Ok(header) => header,
                            Err(e) => {
                                state.login_screen.error_message =
                                    Some(format!("Failed to read vault header: {}", e));
                                return Effect::none();
                            }
                        };

                        if header.has_keyfile {
                            state.login_screen.pending_unlock_password =
                                Some(crate::crypto::SecureString::new(password));
                            state.login_screen.entering_password = false;
                            state.login_screen.entering_keyfile_path = true;
                            state.login_screen.error_message =
                                Some("Vault requires a keyfile. Enter keyfile path.".to_string());
                            state.ui_state.input_buffer.clear();
                            state.ui_state.input_buffer.masked = false;
                            return Effect::none();
                        }

                        state.login_screen.pending_unlock_password = None;
                        state.ui_state.input_buffer.clear();
                        return update(
                            state,
                            Message::UnlockVault {
                                password: crate::crypto::SecureString::new(password),
                                keyfile: None,
                            },
                        );
                    } else {
                        state.login_screen.error_message = Some("No vault selected".to_string());
                        return Effect::none();
                    }
                } else if state.login_screen.entering_keyfile_path {
                    let keyfile_path = state.ui_state.input_buffer.text.trim().to_string();

                    if keyfile_path.is_empty() {
                        state.login_screen.error_message =
                            Some("Keyfile path cannot be empty".to_string());
                        return Effect::none();
                    }

                    let Some(password) = state.login_screen.pending_unlock_password.clone() else {
                        state.login_screen.error_message =
                            Some("Password session expired. Re-enter password.".to_string());
                        state.login_screen.entering_keyfile_path = false;
                        state.login_screen.entering_password = true;
                        state.ui_state.input_buffer.clear();
                        state.ui_state.input_buffer.masked = true;
                        return Effect::none();
                    };

                    state.login_screen.error_message = None;
                    state.ui_state.input_buffer.clear();
                    return update(
                        state,
                        Message::UnlockVault {
                            password,
                            keyfile: Some(std::path::PathBuf::from(keyfile_path)),
                        },
                    );
                }
            }

            // Default: handled by context (search, etc.)
            Effect::none()
        }

        Message::InputCancel => {
            // Cancel input - discard changes in floating window
            match &state.ui_state.floating_window {
                Some(FloatingWindow::NewItem { .. }) | Some(FloatingWindow::EditItem { .. }) => {
                    // Closing form - just close, changes are auto-saved on submit
                    state.ui_state.close_floating_window();
                }
                _ => {
                    // Other contexts - clear input buffer and close
                    state.ui_state.input_buffer.clear();
                    state.ui_state.close_floating_window();
                }
            }
            Effect::none()
        }

        // === Form ===
        Message::FormNextField => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                let q_count = state
                    .login_screen
                    .create_vault_form
                    .recovery_questions_count;
                let use_keyfile_text = state
                    .login_screen
                    .create_vault_form
                    .use_keyfile
                    .text
                    .trim()
                    .to_lowercase();
                let use_keyfile = use_keyfile_text == "yes" || use_keyfile_text == "y";
                state.login_screen.create_vault_form.focused_field =
                    state.login_screen.create_vault_form.focused_field.next(
                        state.login_screen.create_vault_form.step,
                        q_count,
                        use_keyfile,
                    );
            } else {
                match &mut state.ui_state.floating_window {
                    Some(FloatingWindow::NewItem { form })
                    | Some(FloatingWindow::EditItem { form, .. }) => {
                        form.next_field();
                    }
                    _ => {}
                }
            }
            Effect::none()
        }

        Message::FormPrevField => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                let q_count = state
                    .login_screen
                    .create_vault_form
                    .recovery_questions_count;
                let use_keyfile_text = state
                    .login_screen
                    .create_vault_form
                    .use_keyfile
                    .text
                    .trim()
                    .to_lowercase();
                let use_keyfile = use_keyfile_text == "yes" || use_keyfile_text == "y";
                state.login_screen.create_vault_form.focused_field =
                    state.login_screen.create_vault_form.focused_field.prev(
                        state.login_screen.create_vault_form.step,
                        q_count,
                        use_keyfile,
                    );
            } else {
                match &mut state.ui_state.floating_window {
                    Some(FloatingWindow::NewItem { form })
                    | Some(FloatingWindow::EditItem { form, .. }) => {
                        form.prev_field();
                    }
                    _ => {}
                }
            }
            Effect::none()
        }

        Message::FormFocusField(index) => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                // Focus handling for custom fields mapped back from the enum.
                let focused = match index {
                    0 => crate::ui::screens::login::CreateVaultField::Name,
                    1 => crate::ui::screens::login::CreateVaultField::Password,
                    2 => crate::ui::screens::login::CreateVaultField::ConfirmPassword,
                    3 => crate::ui::screens::login::CreateVaultField::UseKeyfile,
                    4 => crate::ui::screens::login::CreateVaultField::KeyfilePath,
                    5 => crate::ui::screens::login::CreateVaultField::RecoveryQuestionsCount,
                    6 => crate::ui::screens::login::CreateVaultField::RecoveryQuestion1,
                    7 => crate::ui::screens::login::CreateVaultField::RecoveryAnswer1,
                    8 => crate::ui::screens::login::CreateVaultField::RecoveryQuestion2,
                    9 => crate::ui::screens::login::CreateVaultField::RecoveryAnswer2,
                    10 => crate::ui::screens::login::CreateVaultField::RecoveryQuestion3,
                    11 => crate::ui::screens::login::CreateVaultField::RecoveryAnswer3,
                    _ => crate::ui::screens::login::CreateVaultField::Name,
                };
                state.login_screen.create_vault_form.focused_field = focused;
            } else {
                match &mut state.ui_state.floating_window {
                    Some(FloatingWindow::NewItem { form })
                    | Some(FloatingWindow::EditItem { form, .. }) => {
                        if index < form.fields.len() {
                            form.focused_field = index;
                            form.cursor = form.values[index].len();
                        }
                    }
                    _ => {}
                }
            }
            Effect::none()
        }

        Message::FormSubmit => {
            if state.screen == Screen::Login && state.login_screen.creating_vault {
                return handle_create_vault_submit(state);
            }

            // Handle form submission
            match state.ui_state.floating_window.take() {
                Some(FloatingWindow::NewItem { form }) => {
                    if let Err(msg) = form.validate() {
                        state.ui_state.notify(msg, NotificationLevel::Error);
                        state.ui_state.floating_window = Some(FloatingWindow::NewItem { form });
                        return Effect::none();
                    }

                    // Create the item from form data
                    if let Some(ref mut vs) = state.vault_state {
                        let item = match create_item_from_form(&form) {
                            Ok(item) => item,
                            Err(msg) => {
                                state.ui_state.notify(msg, NotificationLevel::Error);
                                state.ui_state.floating_window =
                                    Some(FloatingWindow::NewItem { form });
                                return Effect::none();
                            }
                        };
                        let id = item.id;
                        vs.vault.add_item(item);
                        vs.selected_item_id = Some(id);
                        vs.mark_dirty();
                        state
                            .ui_state
                            .notify("Item created and saved", NotificationLevel::Success);

                        // Auto-save to disk
                        return Effect::WriteVaultFile {
                            path: vs.vault_path.clone(),
                            vault: vs.vault.clone(),
                            key: vs.encryption_key,
                            salt: vs.salt,
                            has_keyfile: vs.has_keyfile,
                            encryption_method: vs.encryption_method,
                            recovery_metadata: vs.recovery_metadata.clone(),
                        };
                    }
                }
                Some(FloatingWindow::EditItem { item_id, form }) => {
                    if let Err(msg) = form.validate() {
                        state.ui_state.notify(msg, NotificationLevel::Error);
                        state.ui_state.floating_window =
                            Some(FloatingWindow::EditItem { item_id, form });
                        return Effect::none();
                    }

                    // Update the item from form data
                    if let Some(ref mut vs) = state.vault_state
                        && let Some(item) = vs.vault.get_item(item_id)
                    {
                        // Save undo entry
                        let undo_entry = UndoEntry {
                            description: format!("Edit {}", item.title),
                            item_id,
                            previous_state: ItemSnapshot::from_item(item),
                        };

                        // Apply updates
                        let updates = match create_updates_from_form(&form) {
                            Ok(updates) => updates,
                            Err(msg) => {
                                state.ui_state.notify(msg, NotificationLevel::Error);
                                state.ui_state.floating_window =
                                    Some(FloatingWindow::EditItem { item_id, form });
                                return Effect::none();
                            }
                        };
                        if let Some(item) = vs.vault.get_item_mut(item_id) {
                            apply_item_updates(item, updates);
                        }

                        vs.push_undo(undo_entry);
                        vs.mark_dirty();
                        state
                            .ui_state
                            .notify("Item updated and saved", NotificationLevel::Success);

                        // Auto-save to disk
                        return Effect::WriteVaultFile {
                            path: vs.vault_path.clone(),
                            vault: vs.vault.clone(),
                            key: vs.encryption_key,
                            salt: vs.salt,
                            has_keyfile: vs.has_keyfile,
                            encryption_method: vs.encryption_method,
                            recovery_metadata: vs.recovery_metadata.clone(),
                        };
                    }
                }
                other => {
                    state.ui_state.floating_window = other;
                }
            }
            Effect::none()
        }

        Message::KindSelectorNext => {
            if let Some(FloatingWindow::KindSelector {
                state: ref mut selector,
            }) = state.ui_state.floating_window
            {
                selector.next();
            }
            Effect::none()
        }

        Message::KindSelectorPrev => {
            if let Some(FloatingWindow::KindSelector {
                state: ref mut selector,
            }) = state.ui_state.floating_window
            {
                selector.prev();
            }
            Effect::none()
        }

        Message::KindSelectorSelect(index) => {
            if let Some(FloatingWindow::KindSelector {
                state: ref mut selector,
            }) = state.ui_state.floating_window
            {
                selector.select(index);
            }
            Effect::none()
        }

        Message::KindSelectorConfirm => {
            if let Some(FloatingWindow::KindSelector { state: selector }) =
                state.ui_state.floating_window.take()
            {
                let kind = selector.selected_kind();
                state.ui_state.floating_window = Some(FloatingWindow::new_item_form(kind));
            }
            Effect::none()
        }

        // === Tags ===
        Message::CreateTag(tag) => {
            if let Some(ref mut vs) = state.vault_state {
                vs.vault.add_tag(tag);
                vs.mark_dirty();
            }
            Effect::none()
        }

        Message::DeleteTag(id) => {
            if let Some(ref mut vs) = state.vault_state {
                vs.vault.remove_tag(id);
                // Remove tag from all items
                for item in &mut vs.vault.items {
                    item.tags.retain(|t| *t != id);
                }
                vs.mark_dirty();
            }
            Effect::none()
        }

        Message::ToggleItemTag { item_id, tag_id } => {
            if let Some(ref mut vs) = state.vault_state
                && let Some(item) = vs.vault.get_item_mut(item_id)
            {
                if item.tags.contains(&tag_id) {
                    item.tags.retain(|t| *t != tag_id);
                } else {
                    item.tags.push(tag_id);
                }
                item.touch();
                vs.mark_dirty();
            }
            Effect::none()
        }

        // === Filter ===
        Message::SetKindFilter(kind) => {
            state.ui_state.filter.kind = kind;
            Effect::none()
        }

        Message::NextCategory => {
            let mut kinds = vec![None];
            kinds.extend(ItemKind::all().iter().map(|k| Some(*k)));

            let current_idx = kinds
                .iter()
                .position(|k| *k == state.ui_state.filter.kind)
                .unwrap_or(0);
            let next_idx = (current_idx + 1) % kinds.len();
            state.ui_state.filter.kind = kinds[next_idx];
            Effect::none()
        }

        Message::PrevCategory => {
            let mut kinds = vec![None];
            kinds.extend(ItemKind::all().iter().map(|k| Some(*k)));

            let current_idx = kinds
                .iter()
                .position(|k| *k == state.ui_state.filter.kind)
                .unwrap_or(0);
            let prev_idx = if current_idx == 0 {
                kinds.len() - 1
            } else {
                current_idx - 1
            };
            state.ui_state.filter.kind = kinds[prev_idx];
            Effect::none()
        }

        Message::ToggleTagFilter(tag_id) => {
            if state.ui_state.filter.tags.contains(&tag_id) {
                state.ui_state.filter.tags.retain(|t| *t != tag_id);
            } else {
                state.ui_state.filter.tags.push(tag_id);
            }
            Effect::none()
        }

        Message::ToggleFavoritesFilter => {
            state.ui_state.filter.favorites_only = !state.ui_state.filter.favorites_only;
            Effect::none()
        }

        Message::ClearFilters => {
            state.ui_state.filter.clear();
            Effect::none()
        }

        // === Settings ===
        Message::UpdateConfig(config_update) => {
            apply_config_update(&mut state.config, config_update);
            Effect::WriteConfig
        }

        // === Security Questions ===
        Message::SetupSecurityQuestions(_questions) => {
            // Will be implemented with security module
            Effect::none()
        }

        Message::AttemptRecovery { .. } => {
            // Will be implemented with security module
            Effect::none()
        }

        // === Export ===
        Message::ExportVault { format, path } => {
            if let Some(ref vs) = state.vault_state {
                let encrypted = format == crate::app::ExportFormat::EncryptedJson;
                if !encrypted {
                    state.ui_state.notify(
                        "Warning: JSON export is unencrypted plaintext",
                        NotificationLevel::Warning,
                    );
                }
                let key = if encrypted {
                    Some(vs.encryption_key)
                } else {
                    None
                };

                Effect::ExportVault {
                    path,
                    vault: vs.vault.clone(),
                    encrypted,
                    key,
                    salt: encrypted.then_some(vs.salt),
                    has_keyfile: vs.has_keyfile,
                }
            } else {
                state
                    .ui_state
                    .notify("No vault open to export", NotificationLevel::Warning);
                Effect::none()
            }
        }

        // === System ===
        Message::Tick => {
            // Cleanup expired notifications
            state.ui_state.cleanup_notifications();

            // Check clipboard timeout
            if state.clipboard_state.should_clear() {
                return update(state, Message::ClearClipboard);
            }

            // Check auto-lock
            if state.config.auto_lock_enabled
                && let Some(ref vs) = state.vault_state
            {
                let elapsed = vs.last_activity.elapsed();
                if elapsed.as_secs() >= state.config.auto_lock_timeout_secs {
                    return update(state, Message::LockVault);
                }
            }

            Effect::none()
        }

        Message::Quit => {
            if state.is_dirty() {
                // Prompt to save
                state.ui_state.notify(
                    "Unsaved changes! Press Ctrl+Q again to force quit",
                    NotificationLevel::Warning,
                );
                Effect::none()
            } else {
                state.should_quit = true;
                Effect::Exit
            }
        }

        Message::ForceQuit => {
            state.should_quit = true;
            Effect::Exit
        }

        Message::Noop => Effect::none(),

        Message::AsyncEffectCompleted(_) => {
            // Handled at the application loop level, but returns none here.
            Effect::none()
        }
    }
}

/// Select adjacent item in the list
fn select_adjacent_item(state: &mut AppState, delta: i32) {
    let Some(ref vs) = state.vault_state else {
        return;
    };

    // Get filtered item IDs
    let items: Vec<Uuid> = vs
        .vault
        .items
        .iter()
        .filter(|item| {
            // Kind filter
            if let Some(kind) = state.ui_state.filter.kind {
                let mut matches_kind = item.kind == kind;
                if !matches_kind && kind == ItemKind::Totp && item.kind == ItemKind::Password {
                    if let crate::domain::ItemContent::Password { totp_secret: Some(_), .. } = &item.content {
                        matches_kind = true;
                    }
                }
                if !matches_kind {
                    return false;
                }
            }
            // Tag filter
            if !state.ui_state.filter.tags.is_empty()
                && !state
                    .ui_state
                    .filter
                    .tags
                    .iter()
                    .any(|t| item.tags.contains(t))
            {
                return false;
            }
            // Favorites filter
            if state.ui_state.filter.favorites_only && !item.favorite {
                return false;
            }
            true
        })
        .map(|i| i.id)
        .collect();

    if items.is_empty() {
        return;
    }

    let current_idx = vs
        .selected_item_id
        .and_then(|id| items.iter().position(|i| *i == id))
        .unwrap_or(0);

    let new_idx = if delta > 0 {
        (current_idx + delta as usize).min(items.len() - 1)
    } else {
        current_idx.saturating_sub((-delta) as usize)
    };

    if let Some(id) = items.get(new_idx)
        && let Some(ref mut vs) = state.vault_state
    {
        if vs.selected_item_id != Some(*id) {
            vs.selected_item_id = Some(*id);
            state.ui_state.detail_focus = crate::app::state::DetailFocus::Field(0);
            state.ui_state.notes_scroll_offset = 0;
        }
    }
}

/// Get items filtered by current filter state
#[cfg(test)]
fn get_filtered_items(state: &AppState) -> Vec<&Item> {
    let Some(ref vs) = state.vault_state else {
        return vec![];
    };

    vs.vault
        .items
        .iter()
        .filter(|item| {
            // Kind filter
            if let Some(kind) = state.ui_state.filter.kind {
                if item.kind != kind {
                    return false;
                }
            }

            // Tag filter
            if !state.ui_state.filter.tags.is_empty() {
                if !state
                    .ui_state
                    .filter
                    .tags
                    .iter()
                    .any(|t| item.tags.contains(t))
                {
                    return false;
                }
            }

            // Favorites filter
            if state.ui_state.filter.favorites_only && !item.favorite {
                return false;
            }

            true
        })
        .collect()
}

/// Apply item updates
fn apply_item_updates(item: &mut Item, updates: ItemUpdates) {
    if let Some(title) = updates.title {
        item.title = title;
    }
    if let Some(content) = updates.content {
        item.content = content;
    }
    if let Some(notes) = updates.notes {
        item.notes = notes;
    }
    if let Some(tags) = updates.tags {
        item.tags = tags;
    }
    if let Some(favorite) = updates.favorite {
        item.favorite = favorite;
    }
    item.touch();
}

/// Apply configuration update
fn apply_config_update(config: &mut crate::storage::AppConfig, update: ConfigUpdate) {
    match update {
        ConfigUpdate::SetTheme(theme) => config.theme = theme,
        ConfigUpdate::SetAutoLock(enabled) => config.auto_lock_enabled = enabled,
        ConfigUpdate::SetAutoLockTimeout(secs) => config.auto_lock_timeout_secs = secs,
        ConfigUpdate::SetClipboardTimeout(secs) => config.clipboard_timeout_secs = secs,
        ConfigUpdate::SetShowIcons(show) => config.show_icons = show,
    }
}

/// Handle scroll in current pane
fn handle_scroll(state: &mut AppState, direction: ScrollDirection) {
    let (offset, max) = match state.ui_state.focused_pane {
        Pane::List => {
            let max = state
                .vault_state
                .as_ref()
                .map(|vs| vs.vault.items.len().saturating_sub(1))
                .unwrap_or(0);
            (&mut state.ui_state.list_scroll_offset, max)
        }
        Pane::Detail => {
            let item = state.selected_item();
            let max_fields = item
                .map(|item| item.get_fields().len().saturating_sub(1))
                .unwrap_or(0);

            let has_notes = item.map(|item| item.notes.is_some()).unwrap_or(false);

            match state.ui_state.detail_focus {
                crate::app::state::DetailFocus::Field(idx) => {
                    let offset = idx;
                    let mut new_offset = offset;
                    match direction {
                        ScrollDirection::Up => new_offset = offset.saturating_sub(1),
                        ScrollDirection::Down => {
                            if offset >= max_fields {
                                if has_notes {
                                    state.ui_state.detail_focus = crate::app::state::DetailFocus::Notes;
                                }
                            } else {
                                new_offset = offset + 1;
                            }
                        },
                        ScrollDirection::PageUp => new_offset = offset.saturating_sub(10),
                        ScrollDirection::PageDown => {
                            if offset + 10 > max_fields {
                                if has_notes {
                                    state.ui_state.detail_focus = crate::app::state::DetailFocus::Notes;
                                } else {
                                    new_offset = max_fields;
                                }
                            } else {
                                new_offset = offset + 10;
                            }
                        },
                        ScrollDirection::Top => new_offset = 0,
                        ScrollDirection::Bottom => {
                            if has_notes {
                                state.ui_state.detail_focus = crate::app::state::DetailFocus::Notes;
                            } else {
                                new_offset = max_fields;
                            }
                        },
                    }
                    if matches!(state.ui_state.detail_focus, crate::app::state::DetailFocus::Field(_)) {
                        state.ui_state.detail_focus = crate::app::state::DetailFocus::Field(new_offset.min(max_fields));
                    }
                }
                crate::app::state::DetailFocus::Notes => {
                    let notes_max_scroll = state.selected_item().and_then(|item| item.notes.as_ref()).map(|n| n.lines().count().saturating_sub(1) as u16).unwrap_or(0);
                    let offset = &mut state.ui_state.notes_scroll_offset;
                    match direction {
                        ScrollDirection::Up => {
                            if *offset == 0 {
                                state.ui_state.detail_focus = crate::app::state::DetailFocus::Field(max_fields);
                            } else {
                                *offset = offset.saturating_sub(1);
                            }
                        },
                        ScrollDirection::Down => *offset = (*offset + 1).min(notes_max_scroll),
                        ScrollDirection::PageUp => {
                            if *offset < 10 {
                                *offset = 0;
                                state.ui_state.detail_focus = crate::app::state::DetailFocus::Field(max_fields);
                            } else {
                                *offset = offset.saturating_sub(10);
                            }
                        },
                        ScrollDirection::PageDown => *offset = (*offset + 10).min(notes_max_scroll),
                        ScrollDirection::Top => {
                            *offset = 0;
                            state.ui_state.detail_focus = crate::app::state::DetailFocus::Field(0);
                        },
                        ScrollDirection::Bottom => *offset = notes_max_scroll,
                    }
                }
            }
            return;
        }
        Pane::Search => return,
    };

    match direction {
        ScrollDirection::Up => *offset = offset.saturating_sub(1),
        ScrollDirection::Down => *offset = (*offset + 1).min(max),
        ScrollDirection::PageUp => *offset = offset.saturating_sub(10),
        ScrollDirection::PageDown => *offset = (*offset + 10).min(max),
        ScrollDirection::Top => *offset = 0,
        ScrollDirection::Bottom => *offset = max,
    }
}

fn verify_master_credentials(
    vault_state: &VaultState,
    current_password: &str,
    keyfile_path: Option<&str>,
) -> std::result::Result<Option<Vec<u8>>, String> {
    if current_password.trim().is_empty() {
        return Err("Current password cannot be empty".to_string());
    }

    let keyfile_data = if vault_state.has_keyfile {
        let Some(path) = keyfile_path else {
            return Err("This vault requires a keyfile path".to_string());
        };
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err("Keyfile path cannot be empty".to_string());
        }
        let keyfile = crate::crypto::KeyFile::load(trimmed)
            .map_err(|e| format!("Failed to read keyfile: {}", e))?;
        Some(keyfile.as_bytes().to_vec())
    } else {
        None
    };

    let secure_current = crate::crypto::SecureString::new(current_password.to_string());
    let derived = crate::crypto::derive_key(
        &secure_current,
        keyfile_data.as_deref(),
        &vault_state.salt,
        &crate::crypto::Argon2Params::default(),
    )
    .map_err(|e| format!("Failed to verify credentials: {}", e))?;

    if derived != vault_state.encryption_key {
        return Err("Current password or keyfile is incorrect".to_string());
    }

    Ok(keyfile_data)
}

fn handle_settings_security_action_submit(state: &mut AppState) -> Effect {
    let Some(action_state) = state.settings_state.security_action.clone() else {
        return Effect::none();
    };

    let Some(vault_state) = state.vault_state.as_mut() else {
        state.login_screen.error_message = Some("No vault is open".to_string());
        state.settings_state.security_action = None;
        return Effect::none();
    };

    let input = state.ui_state.input_buffer.text.clone();

    match action_state {
        SecurityActionState::ChangePassword(mut action) => match action.step {
            ChangePasswordStep::CurrentPassword => {
                if input.trim().is_empty() {
                    state.login_screen.error_message =
                        Some("Current password cannot be empty".to_string());
                    return Effect::none();
                }
                action.current_password = Some(input);
                action.step = if vault_state.has_keyfile {
                    ChangePasswordStep::KeyfilePath
                } else {
                    ChangePasswordStep::NewPassword
                };
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = matches!(
                    action.step,
                    ChangePasswordStep::CurrentPassword
                        | ChangePasswordStep::NewPassword
                        | ChangePasswordStep::ConfirmPassword
                );
                state.login_screen.error_message = None;
                state.settings_state.security_action =
                    Some(SecurityActionState::ChangePassword(action));
                Effect::none()
            }
            ChangePasswordStep::KeyfilePath => {
                if input.trim().is_empty() {
                    state.login_screen.error_message =
                        Some("Keyfile path cannot be empty".to_string());
                    return Effect::none();
                }
                action.keyfile_path = input.trim().to_string();
                action.step = ChangePasswordStep::NewPassword;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = true;
                state.login_screen.error_message = None;
                state.settings_state.security_action =
                    Some(SecurityActionState::ChangePassword(action));
                Effect::none()
            }
            ChangePasswordStep::NewPassword => {
                if input.len() < 4 {
                    state.login_screen.error_message =
                        Some("New password must be at least 4 characters".to_string());
                    return Effect::none();
                }
                action.new_password = Some(input);
                action.step = ChangePasswordStep::ConfirmPassword;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = true;
                state.login_screen.error_message = None;
                state.settings_state.security_action =
                    Some(SecurityActionState::ChangePassword(action));
                Effect::none()
            }
            ChangePasswordStep::ConfirmPassword => {
                let Some(current_password) = action.current_password.clone() else {
                    state.login_screen.error_message =
                        Some("Current password is missing".to_string());
                    return Effect::none();
                };
                let Some(new_password) = action.new_password.clone() else {
                    state.login_screen.error_message = Some("New password is missing".to_string());
                    return Effect::none();
                };
                if input != new_password {
                    state.login_screen.error_message =
                        Some("New password confirmation does not match".to_string());
                    state.ui_state.input_buffer.clear();
                    return Effect::none();
                }

                let keyfile_data = match verify_master_credentials(
                    vault_state,
                    &current_password,
                    if vault_state.has_keyfile {
                        Some(action.keyfile_path.as_str())
                    } else {
                        None
                    },
                ) {
                    Ok(data) => data,
                    Err(msg) => {
                        state.login_screen.error_message = Some(msg);
                        return Effect::none();
                    }
                };

                let new_secure_password = crate::crypto::SecureString::new(new_password);
                let new_key = match crate::crypto::derive_key(
                    &new_secure_password,
                    keyfile_data.as_deref(),
                    &vault_state.salt,
                    &crate::crypto::Argon2Params::default(),
                ) {
                    Ok(k) => k,
                    Err(e) => {
                        state.login_screen.error_message =
                            Some(format!("Failed to derive new key: {}", e));
                        return Effect::none();
                    }
                };

                vault_state.encryption_key = new_key;
                vault_state.recovery_metadata = None;
                vault_state.vault.security_questions.clear();
                vault_state.mark_dirty();

                state.settings_state.security_action = None;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
                state.login_screen.error_message =
                    Some("Master password updated. Recovery has been reset.".to_string());
                state.ui_state.notify(
                    "Master password updated. Reconfigure recovery questions.",
                    NotificationLevel::Success,
                );

                Effect::WriteVaultFile {
                    path: vault_state.vault_path.clone(),
                    vault: vault_state.vault.clone(),
                    key: vault_state.encryption_key,
                    salt: vault_state.salt,
                    has_keyfile: vault_state.has_keyfile,
                    encryption_method: vault_state.encryption_method,
                    recovery_metadata: vault_state.recovery_metadata.clone(),
                }
            }
        },
        SecurityActionState::ConfigureRecovery(mut action) => match action.step {
            RecoverySetupStep::CurrentPassword => {
                if input.trim().is_empty() {
                    state.login_screen.error_message =
                        Some("Current password cannot be empty".to_string());
                    return Effect::none();
                }
                action.current_password = Some(input);
                action.step = if vault_state.has_keyfile {
                    RecoverySetupStep::KeyfilePath
                } else {
                    RecoverySetupStep::QuestionCount
                };
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = matches!(
                    action.step,
                    RecoverySetupStep::CurrentPassword | RecoverySetupStep::AnswerText
                );
                state.login_screen.error_message = None;
                state.settings_state.security_action =
                    Some(SecurityActionState::ConfigureRecovery(action));
                Effect::none()
            }
            RecoverySetupStep::KeyfilePath => {
                if input.trim().is_empty() {
                    state.login_screen.error_message =
                        Some("Keyfile path cannot be empty".to_string());
                    return Effect::none();
                }
                action.keyfile_path = input.trim().to_string();
                action.step = RecoverySetupStep::QuestionCount;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
                state.login_screen.error_message = None;
                state.settings_state.security_action =
                    Some(SecurityActionState::ConfigureRecovery(action));
                Effect::none()
            }
            RecoverySetupStep::QuestionCount => {
                let Ok(question_count) = input
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| ())
                    .and_then(|n| if n <= 3 { Ok(n) } else { Err(()) })
                else {
                    state.login_screen.error_message =
                        Some("Enter recovery question count from 0 to 3".to_string());
                    return Effect::none();
                };

                let Some(current_password) = action.current_password.clone() else {
                    state.login_screen.error_message =
                        Some("Current password is missing".to_string());
                    return Effect::none();
                };

                let keyfile_data = match verify_master_credentials(
                    vault_state,
                    &current_password,
                    if vault_state.has_keyfile {
                        Some(action.keyfile_path.as_str())
                    } else {
                        None
                    },
                ) {
                    Ok(data) => data,
                    Err(msg) => {
                        state.login_screen.error_message = Some(msg);
                        return Effect::none();
                    }
                };
                action.keyfile_data = keyfile_data;
                action.question_count = question_count;
                action.questions.clear();
                action.pending_question = None;

                if question_count == 0 {
                    vault_state.recovery_metadata = None;
                    vault_state.vault.security_questions.clear();
                    vault_state.mark_dirty();
                    state.settings_state.security_action = None;
                    state.ui_state.input_buffer.clear();
                    state.ui_state.input_buffer.masked = false;
                    state.login_screen.error_message =
                        Some("Recovery disabled for this vault".to_string());
                    state.ui_state.notify(
                        "Recovery disabled for this vault",
                        NotificationLevel::Success,
                    );
                    return Effect::WriteVaultFile {
                        path: vault_state.vault_path.clone(),
                        vault: vault_state.vault.clone(),
                        key: vault_state.encryption_key,
                        salt: vault_state.salt,
                        has_keyfile: vault_state.has_keyfile,
                        encryption_method: vault_state.encryption_method,
                        recovery_metadata: vault_state.recovery_metadata.clone(),
                    };
                }

                action.step = RecoverySetupStep::QuestionText;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
                state.login_screen.error_message = None;
                state.settings_state.security_action =
                    Some(SecurityActionState::ConfigureRecovery(action));
                Effect::none()
            }
            RecoverySetupStep::QuestionText => {
                let question = input.trim();
                if question.is_empty() {
                    state.login_screen.error_message =
                        Some("Security question cannot be empty".to_string());
                    return Effect::none();
                }
                action.pending_question = Some(question.to_string());
                action.step = RecoverySetupStep::AnswerText;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = true;
                state.login_screen.error_message = None;
                state.settings_state.security_action =
                    Some(SecurityActionState::ConfigureRecovery(action));
                Effect::none()
            }
            RecoverySetupStep::AnswerText => {
                if input.trim().is_empty() {
                    state.login_screen.error_message =
                        Some("Security answer cannot be empty".to_string());
                    return Effect::none();
                }
                let Some(question) = action.pending_question.take() else {
                    state.login_screen.error_message =
                        Some("Question state missing, please try again".to_string());
                    action.step = RecoverySetupStep::QuestionText;
                    state.settings_state.security_action =
                        Some(SecurityActionState::ConfigureRecovery(action));
                    return Effect::none();
                };
                action.questions.push(RecoveryQuestionDraft {
                    question,
                    answer: input,
                });

                if action.questions.len() < action.question_count as usize {
                    action.step = RecoverySetupStep::QuestionText;
                    state.ui_state.input_buffer.clear();
                    state.ui_state.input_buffer.masked = false;
                    state.login_screen.error_message = None;
                    state.settings_state.security_action =
                        Some(SecurityActionState::ConfigureRecovery(action));
                    return Effect::none();
                }

                let Some(current_password) = action.current_password.clone() else {
                    state.login_screen.error_message =
                        Some("Current password is missing".to_string());
                    return Effect::none();
                };
                let secure_current_password =
                    crate::crypto::SecureString::new(current_password.clone());
                let qa_pairs = action
                    .questions
                    .iter()
                    .map(|q| {
                        (
                            q.question.clone(),
                            crate::crypto::SecureString::new(q.answer.clone()),
                        )
                    })
                    .collect::<Vec<_>>();

                let metadata = match crate::domain::RecoveryMetadata::build(
                    qa_pairs,
                    &secure_current_password,
                    vault_state.encryption_method,
                ) {
                    Ok(m) => m,
                    Err(e) => {
                        state.login_screen.error_message =
                            Some(format!("Failed to configure recovery: {}", e));
                        return Effect::none();
                    }
                };

                vault_state.vault.security_questions = metadata.questions.clone();
                vault_state.recovery_metadata = Some(metadata);
                vault_state.mark_dirty();

                state.settings_state.security_action = None;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
                state.login_screen.error_message =
                    Some("Recovery questions saved successfully".to_string());
                state
                    .ui_state
                    .notify("Recovery questions saved", NotificationLevel::Success);

                Effect::WriteVaultFile {
                    path: vault_state.vault_path.clone(),
                    vault: vault_state.vault.clone(),
                    key: vault_state.encryption_key,
                    salt: vault_state.salt,
                    has_keyfile: vault_state.has_keyfile,
                    encryption_method: vault_state.encryption_method,
                    recovery_metadata: vault_state.recovery_metadata.clone(),
                }
            }
        },
        SecurityActionState::AddKeyfile(action) => {
            handle_add_keyfile_submit(state, action)
        }
        SecurityActionState::ManageRecovery(action) => {
            handle_manage_recovery_submit(state, action)
        }
    }
}

// ─────────────────────────────────────────────────────────
// AddKeyfile submit handler
// ─────────────────────────────────────────────────────────

fn handle_add_keyfile_submit(state: &mut AppState, action: AddKeyfileAction) -> Effect {
    let input = state.ui_state.input_buffer.text.clone();

    let Some(vault_state) = state.vault_state.as_mut() else {
        state.login_screen.error_message = Some("No vault is open".to_string());
        state.settings_state.security_action = None;
        return Effect::none();
    };

    match action.step {
        AddKeyfileStep::CurrentPassword => {
            if input.trim().is_empty() {
                state.login_screen.error_message =
                    Some("Current password cannot be empty".to_string());
                state.settings_state.security_action =
                    Some(SecurityActionState::AddKeyfile(action));
                return Effect::none();
            }
            let mut next = action.clone();
            next.current_password = Some(input);

            // If vault already has a keyfile, we need the old keyfile path first
            if vault_state.has_keyfile {
                next.step = AddKeyfileStep::OldKeyfilePath;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
                state.login_screen.error_message = None;
            } else {
                next.step = AddKeyfileStep::NewKeyfilePath;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
                state.login_screen.error_message = None;
            }
            state.settings_state.security_action = Some(SecurityActionState::AddKeyfile(next));
            Effect::none()
        }

        AddKeyfileStep::OldKeyfilePath => {
            if input.trim().is_empty() {
                state.login_screen.error_message =
                    Some("Keyfile path cannot be empty".to_string());
                state.settings_state.security_action =
                    Some(SecurityActionState::AddKeyfile(action));
                return Effect::none();
            }
            let mut next = action.clone();
            next.old_keyfile_path = input.trim().to_string();

            // Verify credentials with old password + old keyfile
            let current_pass = match &next.current_password {
                Some(p) => p.clone(),
                None => {
                    state.login_screen.error_message =
                        Some("Session expired – please restart".to_string());
                    state.settings_state.security_action = None;
                    return Effect::none();
                }
            };

            match verify_master_credentials(vault_state, &current_pass, Some(&next.old_keyfile_path)) {
                Ok(Some(kf_data)) => {
                    next.old_keyfile_data = Some(kf_data);
                }
                Ok(None) => {}
                Err(msg) => {
                    state.login_screen.error_message = Some(msg);
                    state.settings_state.security_action =
                        Some(SecurityActionState::AddKeyfile(next));
                    return Effect::none();
                }
            }

            next.step = AddKeyfileStep::NewKeyfilePath;
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = false;
            state.login_screen.error_message = None;
            state.settings_state.security_action = Some(SecurityActionState::AddKeyfile(next));
            Effect::none()
        }

        AddKeyfileStep::NewKeyfilePath => {
            let new_kf_path = input.trim().to_string();

            // Verify old credentials first (if we haven't already for OldKeyfilePath step)
            let current_pass = match &action.current_password {
                Some(p) => p.clone(),
                None => {
                    state.login_screen.error_message =
                        Some("Session expired – please restart".to_string());
                    state.settings_state.security_action = None;
                    return Effect::none();
                }
            };

            // For vaults without an existing keyfile, verify password alone now
            if !vault_state.has_keyfile {
                match verify_master_credentials(vault_state, &current_pass, None) {
                    Ok(_) => {}
                    Err(msg) => {
                        state.login_screen.error_message = Some(msg);
                        state.settings_state.security_action =
                            Some(SecurityActionState::AddKeyfile(action));
                        return Effect::none();
                    }
                }
            }

            // If a new keyfile path is provided, load it
            let new_kf_data: Option<Vec<u8>> = if new_kf_path.is_empty() {
                None
            } else {
                match crate::crypto::KeyFile::load(&new_kf_path) {
                    Ok(kf) => Some(kf.as_bytes().to_vec()),
                    Err(e) => {
                        state.login_screen.error_message =
                            Some(format!("Failed to read keyfile: {}", e));
                        state.settings_state.security_action =
                            Some(SecurityActionState::AddKeyfile(action));
                        return Effect::none();
                    }
                }
            };

            // Derive new encryption key with password + new keyfile
            let secure_pass = crate::crypto::SecureString::new(current_pass);
            let new_key = match crate::crypto::derive_key(
                &secure_pass,
                new_kf_data.as_deref(),
                &vault_state.salt,
                &crate::crypto::Argon2Params::default(),
            ) {
                Ok(k) => k,
                Err(e) => {
                    state.login_screen.error_message =
                        Some(format!("Failed to derive new key: {}", e));
                    state.settings_state.security_action =
                        Some(SecurityActionState::AddKeyfile(action));
                    return Effect::none();
                }
            };

            vault_state.encryption_key = new_key;
            vault_state.has_keyfile = new_kf_data.is_some();
            vault_state.mark_dirty();

            state.settings_state.security_action = None;
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = false;
            state.login_screen.error_message = None;

            let msg = if vault_state.has_keyfile {
                "Keyfile added and vault re-encrypted"
            } else {
                "Keyfile removed and vault re-encrypted"
            };
            state.ui_state.notify(msg, NotificationLevel::Success);

            Effect::WriteVaultFile {
                path: vault_state.vault_path.clone(),
                vault: vault_state.vault.clone(),
                key: vault_state.encryption_key,
                salt: vault_state.salt,
                has_keyfile: vault_state.has_keyfile,
                encryption_method: vault_state.encryption_method,
                recovery_metadata: vault_state.recovery_metadata.clone(),
            }
        }
    }
}

// ─────────────────────────────────────────────────────────
// ManageRecovery submit handler
// ─────────────────────────────────────────────────────────

fn handle_manage_recovery_submit(state: &mut AppState, action: ManageRecoveryAction) -> Effect {
    let input = state.ui_state.input_buffer.text.clone();

    let Some(vault_state) = state.vault_state.as_mut() else {
        state.login_screen.error_message = Some("No vault is open".to_string());
        state.settings_state.security_action = None;
        return Effect::none();
    };

    match action.step.clone() {
        // ── Step 1: verify master password ──────────────────────────────────
        ManageRecoveryStep::VerifyPassword => {
            if input.trim().is_empty() {
                state.login_screen.error_message =
                    Some("Current password cannot be empty".to_string());
                state.settings_state.security_action =
                    Some(SecurityActionState::ManageRecovery(action));
                return Effect::none();
            }
            let mut next = action.clone();
            next.current_password = Some(input);

            if vault_state.has_keyfile {
                next.step = ManageRecoveryStep::VerifyKeyfile;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
            } else {
                // Verify password-only credentials
                let pass = next.current_password.as_deref().unwrap_or("");
                match verify_master_credentials(vault_state, pass, None) {
                    Ok(_) => {}
                    Err(msg) => {
                        state.login_screen.error_message = Some(msg);
                        state.settings_state.security_action =
                            Some(SecurityActionState::ManageRecovery(next));
                        return Effect::none();
                    }
                }
                next.step = ManageRecoveryStep::QuestionList;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
            }
            state.login_screen.error_message = None;
            state.settings_state.security_action =
                Some(SecurityActionState::ManageRecovery(next));
            Effect::none()
        }

        // ── Step 2: verify keyfile ───────────────────────────────────────────
        ManageRecoveryStep::VerifyKeyfile => {
            if input.trim().is_empty() {
                state.login_screen.error_message =
                    Some("Keyfile path cannot be empty".to_string());
                state.settings_state.security_action =
                    Some(SecurityActionState::ManageRecovery(action));
                return Effect::none();
            }
            let mut next = action.clone();
            next.keyfile_path = input.trim().to_string();

            let pass = next.current_password.as_deref().unwrap_or("");
            match verify_master_credentials(vault_state, pass, Some(&next.keyfile_path)) {
                Ok(Some(kf_data)) => {
                    next.keyfile_data = Some(kf_data);
                }
                Ok(None) => {}
                Err(msg) => {
                    state.login_screen.error_message = Some(msg);
                    state.settings_state.security_action =
                        Some(SecurityActionState::ManageRecovery(next));
                    return Effect::none();
                }
            }

            next.step = ManageRecoveryStep::QuestionList;
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = false;
            state.login_screen.error_message = None;
            state.settings_state.security_action =
                Some(SecurityActionState::ManageRecovery(next));
            Effect::none()
        }

        // ── QuestionList: dispatch based on char key ─────────────────────────
        ManageRecoveryStep::QuestionList => {
            // InputSubmit (Enter) defaults to 'e' (edit question text)
            let char_input = input.chars().next().unwrap_or('\n');
            let questions: Vec<_> = vault_state
                .recovery_metadata
                .as_ref()
                .map(|m| m.questions.iter().map(|q| q.question.clone()).collect())
                .unwrap_or_default();
            let q_count = questions.len();
            let sel_idx = action.selected_idx;

            match char_input {
                // [e] / Enter → Edit question text
                'e' | '\n' => {
                    if q_count == 0 {
                        state.login_screen.error_message =
                            Some("No recovery questions to edit".to_string());
                        state.settings_state.security_action =
                            Some(SecurityActionState::ManageRecovery(action));
                        return Effect::none();
                    }
                    let mut next = action.clone();
                    next.target = ManageRecoveryTarget::EditQuestionText(sel_idx);
                    next.step = ManageRecoveryStep::EditQuestionText;
                    // Pre-fill input with existing question text
                    state.ui_state.input_buffer.text = questions[sel_idx].clone();
                    state.ui_state.input_buffer.cursor = questions[sel_idx].len();
                    state.ui_state.input_buffer.masked = false;
                    state.login_screen.error_message = None;
                    state.settings_state.security_action =
                        Some(SecurityActionState::ManageRecovery(next));
                }

                // [a] → Edit answer (requires collecting all answers + rebuild)
                'a' => {
                    if q_count == 0 {
                        state.login_screen.error_message =
                            Some("No recovery questions configured".to_string());
                        state.settings_state.security_action =
                            Some(SecurityActionState::ManageRecovery(action));
                        return Effect::none();
                    }
                    let mut next = action.clone();
                    next.target = ManageRecoveryTarget::EditAnswer(sel_idx);
                    // We need answers for ALL questions (we'll update sel_idx's answer during rebuild)
                    next.collected_answers = vec![None; q_count];
                    // Start collecting from the first question
                    // (We'll skip sel_idx's answer entry if we're providing new one separately,
                    // but for simplicity we collect ALL answers fresh including the changed one)
                    next.collect_idx = 0;
                    next.step = ManageRecoveryStep::CollectExistingAnswer;
                    state.ui_state.input_buffer.clear();
                    state.ui_state.input_buffer.masked = true;
                    state.login_screen.error_message =
                        Some(format!("Enter answer for: {}", questions[0]));
                    state.settings_state.security_action =
                        Some(SecurityActionState::ManageRecovery(next));
                }

                // [d] → Delete question
                'd' => {
                    if q_count == 0 {
                        state.login_screen.error_message =
                            Some("No recovery questions to delete".to_string());
                        state.settings_state.security_action =
                            Some(SecurityActionState::ManageRecovery(action));
                        return Effect::none();
                    }
                    let mut next = action.clone();
                    next.target = ManageRecoveryTarget::DeleteQuestion(sel_idx);

                    if q_count == 1 {
                        // Only 1 question → ask to confirm disabling recovery entirely
                        next.step = ManageRecoveryStep::ConfirmDisableRecovery;
                        state.ui_state.input_buffer.clear();
                        state.ui_state.input_buffer.masked = false;
                        state.login_screen.error_message =
                            Some("This will disable recovery. Type YES to confirm.".to_string());
                    } else {
                        // Multiple questions → collect remaining answers for rebuild
                        let remaining_count = q_count - 1;
                        next.collected_answers = vec![None; remaining_count];
                        next.collect_idx = 0;
                        next.step = ManageRecoveryStep::CollectExistingAnswer;
                        state.ui_state.input_buffer.clear();
                        state.ui_state.input_buffer.masked = true;
                        let first_remaining = if sel_idx == 0 { &questions[1] } else { &questions[0] };
                        state.login_screen.error_message =
                            Some(format!("Enter answer for: {}", first_remaining));
                    }
                    state.settings_state.security_action =
                        Some(SecurityActionState::ManageRecovery(next));
                }

                // [n] → Add new question (max 3)
                'n' => {
                    if q_count >= 3 {
                        state.login_screen.error_message =
                            Some("Maximum recovery questions (3) reached".to_string());
                        state.settings_state.security_action =
                            Some(SecurityActionState::ManageRecovery(action));
                        return Effect::none();
                    }
                    let mut next = action.clone();
                    next.target = ManageRecoveryTarget::AddQuestion;
                    next.step = ManageRecoveryStep::AddQuestionText;
                    state.ui_state.input_buffer.clear();
                    state.ui_state.input_buffer.masked = false;
                    state.login_screen.error_message = None;
                    state.settings_state.security_action =
                        Some(SecurityActionState::ManageRecovery(next));
                }

                _ => {
                    // Ignore unknown chars in list mode
                    state.settings_state.security_action =
                        Some(SecurityActionState::ManageRecovery(action));
                }
            }
            Effect::none()
        }

        // ── Edit question text ───────────────────────────────────────────────
        ManageRecoveryStep::EditQuestionText => {
            let new_text = input.trim().to_string();
            if new_text.is_empty() {
                state.login_screen.error_message =
                    Some("Question text cannot be empty".to_string());
                state.settings_state.security_action =
                    Some(SecurityActionState::ManageRecovery(action));
                return Effect::none();
            }

            let idx = match &action.target {
                ManageRecoveryTarget::EditQuestionText(i) => *i,
                _ => {
                    state.settings_state.security_action = None;
                    return Effect::none();
                }
            };

            // Update question text directly in metadata (no crypto rebuild needed)
            if let Some(ref mut meta) = vault_state.recovery_metadata {
                if let Some(q) = meta.questions.get_mut(idx) {
                    q.question = new_text.clone();
                }
            }
            if let Some(q) = vault_state.vault.security_questions.get_mut(idx) {
                q.question = new_text;
            }
            vault_state.mark_dirty();

            state.settings_state.security_action = None;
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = false;
            state.login_screen.error_message = None;
            state.ui_state.notify("Question text updated", NotificationLevel::Success);

            Effect::WriteVaultFile {
                path: vault_state.vault_path.clone(),
                vault: vault_state.vault.clone(),
                key: vault_state.encryption_key,
                salt: vault_state.salt,
                has_keyfile: vault_state.has_keyfile,
                encryption_method: vault_state.encryption_method,
                recovery_metadata: vault_state.recovery_metadata.clone(),
            }
        }

        // ── Add question – text step ─────────────────────────────────────────
        ManageRecoveryStep::AddQuestionText => {
            let q_text = input.trim().to_string();
            if q_text.is_empty() {
                state.login_screen.error_message =
                    Some("Question text cannot be empty".to_string());
                state.settings_state.security_action =
                    Some(SecurityActionState::ManageRecovery(action));
                return Effect::none();
            }
            let mut next = action.clone();
            next.new_question_text = Some(q_text);
            next.step = ManageRecoveryStep::AddAnswerText;
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = true;
            state.login_screen.error_message = None;
            state.settings_state.security_action =
                Some(SecurityActionState::ManageRecovery(next));
            Effect::none()
        }

        // ── Add question – answer step ───────────────────────────────────────
        ManageRecoveryStep::AddAnswerText => {
            let a_text = input.trim().to_string();
            if a_text.is_empty() {
                state.login_screen.error_message =
                    Some("Answer cannot be empty".to_string());
                state.settings_state.security_action =
                    Some(SecurityActionState::ManageRecovery(action));
                return Effect::none();
            }
            let q_count = vault_state
                .recovery_metadata
                .as_ref()
                .map(|m| m.questions.len())
                .unwrap_or(0);

            let mut next = action.clone();
            next.new_answer_text = Some(a_text);

            if q_count == 0 {
                // No existing questions → rebuild directly with just the new Q&A
                return manage_recovery_rebuild(state, next);
            }

            // Need to collect existing answers first
            next.collected_answers = vec![None; q_count];
            next.collect_idx = 0;
            next.step = ManageRecoveryStep::CollectExistingAnswer;
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = true;
            let existing_q = vault_state
                .recovery_metadata
                .as_ref()
                .and_then(|m| m.questions.first())
                .map(|q| q.question.clone())
                .unwrap_or_default();
            state.login_screen.error_message =
                Some(format!("Enter existing answer for: {}", existing_q));
            state.settings_state.security_action =
                Some(SecurityActionState::ManageRecovery(next));
            Effect::none()
        }

        // ── Collect existing answers (for rebuild) ───────────────────────────
        ManageRecoveryStep::CollectExistingAnswer => {
            if input.trim().is_empty() {
                state.login_screen.error_message =
                    Some("Answer cannot be empty".to_string());
                state.settings_state.security_action =
                    Some(SecurityActionState::ManageRecovery(action));
                return Effect::none();
            }

            let mut next = action.clone();

            // Store the answer we just collected
            let questions: Vec<String> = vault_state
                .recovery_metadata
                .as_ref()
                .map(|m| m.questions.iter().map(|q| q.question.clone()).collect())
                .unwrap_or_default();

            match &next.target {
                ManageRecoveryTarget::DeleteQuestion(del_idx) => {
                    let del_idx = *del_idx;
                    // Map collect_idx to actual question index (skip deleted)
                    let _actual_idx = collect_idx_to_actual(next.collect_idx, del_idx, questions.len());
                    if let Some(slot) = next.collected_answers.get_mut(next.collect_idx) {
                        *slot = Some(input.clone());
                    }
                    next.collect_idx += 1;

                    // Check if we've collected all remaining answers
                    let remaining = questions.len() - 1;
                    if next.collect_idx >= remaining {
                        return manage_recovery_rebuild(state, next);
                    }

                    // Continue collecting next answer
                    let next_actual = collect_idx_to_actual(next.collect_idx, del_idx, questions.len());
                    let next_q = questions.get(next_actual).cloned().unwrap_or_default();
                    state.login_screen.error_message =
                        Some(format!("Enter answer for: {}", next_q));
                }
                ManageRecoveryTarget::EditAnswer(_) | ManageRecoveryTarget::AddQuestion => {
                    if let Some(slot) = next.collected_answers.get_mut(next.collect_idx) {
                        *slot = Some(input.clone());
                    }
                    next.collect_idx += 1;

                    // Check if we've collected all answers
                    if next.collect_idx >= next.collected_answers.len() {
                        return manage_recovery_rebuild(state, next);
                    }

                    // Continue collecting
                    let next_q = questions
                        .get(next.collect_idx)
                        .cloned()
                        .unwrap_or_default();
                    state.login_screen.error_message =
                        Some(format!("Enter answer for: {}", next_q));
                }
                _ => {
                    state.settings_state.security_action = None;
                    return Effect::none();
                }
            }

            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = true;
            state.settings_state.security_action =
                Some(SecurityActionState::ManageRecovery(next));
            Effect::none()
        }

        // ── Confirm disable recovery (single question delete) ────────────────
        ManageRecoveryStep::ConfirmDisableRecovery => {
            if input.trim().to_uppercase() == "YES" {
                vault_state.recovery_metadata = None;
                vault_state.vault.security_questions.clear();
                vault_state.mark_dirty();

                state.settings_state.security_action = None;
                state.ui_state.input_buffer.clear();
                state.ui_state.input_buffer.masked = false;
                state.login_screen.error_message = None;
                state.ui_state.notify("Recovery disabled for this vault", NotificationLevel::Success);

                Effect::WriteVaultFile {
                    path: vault_state.vault_path.clone(),
                    vault: vault_state.vault.clone(),
                    key: vault_state.encryption_key,
                    salt: vault_state.salt,
                    has_keyfile: vault_state.has_keyfile,
                    encryption_method: vault_state.encryption_method,
                    recovery_metadata: vault_state.recovery_metadata.clone(),
                }
            } else {
                state.login_screen.error_message =
                    Some("Type YES (uppercase) to confirm".to_string());
                state.ui_state.input_buffer.clear();
                state.settings_state.security_action =
                    Some(SecurityActionState::ManageRecovery(action));
                Effect::none()
            }
        }
    }
}

/// Map a "remaining index" (skipping del_idx) to the actual question index.
fn collect_idx_to_actual(collect_idx: usize, del_idx: usize, total: usize) -> usize {
    let mut actual = 0usize;
    let mut remaining_seen = 0usize;
    for i in 0..total {
        if i == del_idx {
            continue;
        }
        if remaining_seen == collect_idx {
            actual = i;
            break;
        }
        remaining_seen += 1;
    }
    actual
}

/// Rebuild RecoveryMetadata from the collected answers and target operation, then save.
fn manage_recovery_rebuild(state: &mut AppState, action: ManageRecoveryAction) -> Effect {
    let Some(vault_state) = state.vault_state.as_mut() else {
        state.settings_state.security_action = None;
        return Effect::none();
    };

    let current_password = match &action.current_password {
        Some(p) => p.clone(),
        None => {
            state.login_screen.error_message = Some("Session expired".to_string());
            state.settings_state.security_action = None;
            return Effect::none();
        }
    };

    let existing_questions: Vec<crate::domain::SecurityQuestion> = vault_state
        .recovery_metadata
        .as_ref()
        .map(|m| m.questions.clone())
        .unwrap_or_default();

    // Build new Q&A pairs based on target operation
    let new_qa_pairs: Vec<(String, crate::crypto::SecureString)> = match &action.target {
        ManageRecoveryTarget::EditAnswer(edit_idx) => {
            let _edit_idx = *edit_idx;
            existing_questions
                .iter()
                .enumerate()
                .map(|(i, q)| {
                    let answer_text = action
                        .collected_answers
                        .get(i)
                        .and_then(|a| a.as_deref())
                        .unwrap_or("")
                        .to_string();
                    (
                        q.question.clone(),
                        crate::crypto::SecureString::new(answer_text),
                    )
                })
                .collect()
        }

        ManageRecoveryTarget::DeleteQuestion(del_idx) => {
            let del_idx = *del_idx;
            let mut pairs = Vec::new();
            let mut remaining_idx = 0usize;
            for (i, q) in existing_questions.iter().enumerate() {
                if i == del_idx {
                    continue;
                }
                let answer_text = action
                    .collected_answers
                    .get(remaining_idx)
                    .and_then(|a| a.as_deref())
                    .unwrap_or("")
                    .to_string();
                pairs.push((
                    q.question.clone(),
                    crate::crypto::SecureString::new(answer_text),
                ));
                remaining_idx += 1;
            }
            pairs
        }

        ManageRecoveryTarget::AddQuestion => {
            let new_q = action.new_question_text.clone().unwrap_or_default();
            let new_a = action.new_answer_text.clone().unwrap_or_default();

            let mut pairs: Vec<(String, crate::crypto::SecureString)> = existing_questions
                .iter()
                .enumerate()
                .map(|(i, q)| {
                    let answer_text = action
                        .collected_answers
                        .get(i)
                        .and_then(|a| a.as_deref())
                        .unwrap_or("")
                        .to_string();
                    (
                        q.question.clone(),
                        crate::crypto::SecureString::new(answer_text),
                    )
                })
                .collect();
            pairs.push((new_q, crate::crypto::SecureString::new(new_a)));
            pairs
        }

        _ => {
            state.settings_state.security_action = None;
            return Effect::none();
        }
    };

    if new_qa_pairs.is_empty() {
        // This shouldn't happen, but guard
        state.settings_state.security_action = None;
        state.login_screen.error_message = Some("No Q&A pairs to rebuild with".to_string());
        return Effect::none();
    }

    let secure_pass = crate::crypto::SecureString::new(current_password);
    let metadata = match crate::domain::RecoveryMetadata::build(
        new_qa_pairs,
        &secure_pass,
        vault_state.encryption_method,
    ) {
        Ok(m) => m,
        Err(e) => {
            state.login_screen.error_message =
                Some(format!("Failed to rebuild recovery: {}", e));
            state.settings_state.security_action =
                Some(SecurityActionState::ManageRecovery(action));
            return Effect::none();
        }
    };

    vault_state.vault.security_questions = metadata.questions.clone();
    vault_state.recovery_metadata = Some(metadata);
    vault_state.mark_dirty();

    state.settings_state.security_action = None;
    state.ui_state.input_buffer.clear();
    state.ui_state.input_buffer.masked = false;
    state.login_screen.error_message = None;
    state.ui_state.notify("Recovery questions updated", NotificationLevel::Success);

    Effect::WriteVaultFile {
        path: vault_state.vault_path.clone(),
        vault: vault_state.vault.clone(),
        key: vault_state.encryption_key,
        salt: vault_state.salt,
        has_keyfile: vault_state.has_keyfile,
        encryption_method: vault_state.encryption_method,
        recovery_metadata: vault_state.recovery_metadata.clone(),
    }
}

fn settings_option_count(_state: &AppState, setting_index: usize) -> usize {
    use crate::ui::screens::SettingKind;

    let Some(setting) = SettingKind::all().get(setting_index) else {
        return 0;
    };

    match setting {
        SettingKind::Theme => crate::storage::ThemeChoice::all().len(),
        SettingKind::AutoLock | SettingKind::ShowIcons | SettingKind::MouseEnabled => 2,
        SettingKind::AutoLockTimeout => 5,
        SettingKind::ClipboardTimeout => 5,
        SettingKind::IconColor => crate::storage::IconColorChoice::all().len(),
        SettingKind::ChangeMasterPassword
        | SettingKind::AddKeyfile
        | SettingKind::ManageRecovery
        | SettingKind::ConfigureRecovery => 0,
    }
}

fn handle_password_recovery_submit(state: &mut AppState) -> Effect {
    let answer_text = state.ui_state.input_buffer.text.clone();
    if answer_text.trim().is_empty() {
        state.login_screen.error_message = Some("Answer cannot be empty".to_string());
        return Effect::none();
    }

    let submit_result = {
        let Some(session) = state.login_screen.password_recovery.as_mut() else {
            return Effect::none();
        };

        if session.is_locked_out() {
            state.login_screen.error_message =
                Some("Recovery is locked due to too many failed attempts".to_string());
            return Effect::none();
        }

        match session.submit_answer(crate::crypto::SecureString::new(answer_text)) {
            Ok(is_correct) => Ok((
                is_correct,
                session.is_complete(),
                session.is_locked_out(),
                session.remaining_attempts(),
                session.latest_hint.clone(),
                session.recovered_password.clone(),
            )),
            Err(e) => Err(e.to_string()),
        }
    };

    state.ui_state.input_buffer.clear();
    state.ui_state.input_buffer.masked = true;

    match submit_result {
        Ok((true, true, _, _, _, Some(password))) => {
            state.login_screen.error_message =
                Some("Recovery complete. Master password revealed below.".to_string());
            state.ui_state.notify(
                "Recovery complete: password fully revealed",
                NotificationLevel::Success,
            );
            state.ui_state.notify(
                format!("Recovered password: {}", password),
                NotificationLevel::Info,
            );
        }
        Ok((true, false, _, _, Some(_), _)) => {
            state.login_screen.error_message =
                Some("Correct answer. More characters are now revealed.".to_string());
        }
        Ok((false, _, true, _, _, _)) => {
            state.login_screen.error_message = Some(
                "Incorrect answer. Recovery locked after maximum failed attempts.".to_string(),
            );
        }
        Ok((false, _, false, remaining, _, _)) => {
            state.login_screen.error_message = Some(format!(
                "Incorrect answer. {} attempts remaining.",
                remaining
            ));
        }
        Ok((_, _, _, _, _, None)) => {
            state.login_screen.error_message =
                Some("Recovery state is inconsistent for this vault".to_string());
        }
        Ok(_) => {
            state.login_screen.error_message = Some("Recovery progress updated".to_string());
        }
        Err(e) => {
            state.login_screen.error_message = Some(format!("Recovery failed: {}", e));
        }
    }

    Effect::none()
}

fn handle_create_vault_submit(state: &mut AppState) -> Effect {
    let form = &state.login_screen.create_vault_form;

    let vault_name = form.name.text.trim().to_string();
    if vault_name.is_empty() {
        state.login_screen.error_message = Some("Vault name cannot be empty".to_string());
        return Effect::none();
    }

    state.ui_state.start_loading("Creating vault...");

    let password = form.password.text.clone();
    if password.len() < 4 {
        state.login_screen.error_message =
            Some("Password must be at least 4 characters".to_string());
        return Effect::none();
    }

    let confirm = form.confirm_password.text.clone();
    if password != confirm {
        state.login_screen.error_message = Some("Passwords do not match".to_string());
        return Effect::none();
    }

    let use_keyfile_str = form.use_keyfile.text.clone();
    let use_keyfile = use_keyfile_str.trim().eq_ignore_ascii_case("y")
        || use_keyfile_str.trim().eq_ignore_ascii_case("yes");

    let keyfile_path = form.keyfile_path.text.trim().to_string();
    if use_keyfile && keyfile_path.is_empty() {
        state.login_screen.error_message =
            Some("Keyfile path cannot be empty if using keyfile".to_string());
        return Effect::none();
    }

    let q_count = form.recovery_questions_count;

    if q_count > 3 {
        state.login_screen.error_message = Some("Maximum 3 security questions allowed".to_string());
        state.ui_state.stop_loading();
        return Effect::none();
    }

    let mut draft_qs = Vec::new();

    if q_count > 0 {
        let q1 = form.question1.text.trim().to_string();
        let a1 = form.answer1.text.trim().to_string();
        if q1.is_empty() || a1.is_empty() {
            state.login_screen.error_message =
                Some("Question 1 and its answer cannot be empty".to_string());
            state.ui_state.stop_loading();
            return Effect::none();
        }
        draft_qs.push((q1, a1));
    }

    if q_count > 1 {
        let q2 = form.question2.text.trim().to_string();
        let a2 = form.answer2.text.trim().to_string();
        if q2.is_empty() || a2.is_empty() {
            state.login_screen.error_message =
                Some("Question 2 and its answer cannot be empty".to_string());
            state.ui_state.stop_loading();
            return Effect::none();
        }
        draft_qs.push((q2, a2));
    }

    if q_count > 2 {
        let q3 = form.question3.text.trim().to_string();
        let a3 = form.answer3.text.trim().to_string();
        if q3.is_empty() || a3.is_empty() {
            state.login_screen.error_message =
                Some("Question 3 and its answer cannot be empty".to_string());
            state.ui_state.stop_loading();
            return Effect::none();
        }
        draft_qs.push((q3, a3));
    }

    state.login_screen.error_message = None;

    let secure_password = crate::crypto::SecureString::new(password);

    Effect::CreateVault {
        vault_name,
        password: secure_password,
        use_keyfile,
        keyfile_path,
        encryption_method: form.encryption_method,
        draft_qs,
    }
}

fn transition_to_locked_state(state: &mut AppState) {
    state.vault_state = None;
    state.pending_lock = false;
    state.mode = AppMode::Locked;
    state.screen = Screen::Login;
    state.ui_state = Default::default();
    state.ui_state.input_buffer.masked = true;
    state.clipboard_state.clear();
    state.login_screen.entering_password = false;
    state.login_screen.entering_keyfile_path = false;
    state.login_screen.reset_create_form();
    state.login_screen.password_recovery = None;
    state.login_screen.pending_unlock_password = None;
    state.login_screen.error_message = None;
    state.settings_state.security_action = None;
    state.settings_state.cancel_edit();
}

/// Create a new item from form data
fn create_item_from_form(
    form: &crate::ui::widgets::EditFormState,
) -> std::result::Result<Item, String> {
    use crate::domain::ItemContent;
    use crate::ui::widgets::FormField;

    let title = form
        .get_value(&FormField::Title)
        .unwrap_or("Untitled")
        .to_string();
    let notes = if matches!(form.kind, crate::domain::ItemKind::SecureNote) {
        None
    } else {
        form.get_value(&FormField::Notes)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    };

    let content = match form.kind {
        crate::domain::ItemKind::Generic | crate::domain::ItemKind::SecureNote => {
            let value = form
                .get_value(&FormField::Content)
                .unwrap_or("")
                .to_string();
            if form.kind == crate::domain::ItemKind::SecureNote {
                ItemContent::SecureNote { content: value }
            } else {
                ItemContent::Generic { value }
            }
        }
        crate::domain::ItemKind::CryptoSeed => ItemContent::CryptoSeed {
            seed_phrase: form
                .get_value(&FormField::SeedPhrase)
                .unwrap_or("")
                .to_string(),
            derivation_path: form
                .get_value(&FormField::DerivationPath)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            network: form
                .get_value(&FormField::Network)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
        },
        crate::domain::ItemKind::Password => ItemContent::Password {
            username: form
                .get_value(&FormField::Username)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            password: form
                .get_value(&FormField::Password)
                .unwrap_or("")
                .to_string(),
            url: form
                .get_value(&FormField::Url)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            totp_secret: form
                .get_value(&FormField::TotpSecret)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
        },
        crate::domain::ItemKind::ApiKey => ItemContent::ApiKey {
            key: form.get_value(&FormField::ApiKey).unwrap_or("").to_string(),
            service: form
                .get_value(&FormField::Service)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            expires_at: None,
        },
        crate::domain::ItemKind::Custom => ItemContent::Custom {
            fields: parse_custom_fields(
                form.get_value(&FormField::CustomFields).unwrap_or_default(),
            )?,
        },
        crate::domain::ItemKind::Totp => ItemContent::Totp {
            issuer: form
                .get_value(&FormField::Issuer)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            account_name: form
                .get_value(&FormField::AccountName)
                .unwrap_or("")
                .to_string(),
            secret: form
                .get_value(&FormField::TotpSecret)
                .unwrap_or("")
                .to_string(),
        },
    };

    let mut item = Item::new(&title, form.kind, content);
    item.notes = notes;
    Ok(item)
}

/// Create item updates from form data
fn create_updates_from_form(
    form: &crate::ui::widgets::EditFormState,
) -> std::result::Result<ItemUpdates, String> {
    use crate::domain::ItemContent;
    use crate::ui::widgets::FormField;

    let title = form.get_value(&FormField::Title).map(|s| s.to_string());
    let notes = if matches!(form.kind, crate::domain::ItemKind::SecureNote) {
        Some(None)
    } else {
        Some(
            form.get_value(&FormField::Notes)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
        )
    };

    let content = match form.kind {
        crate::domain::ItemKind::Generic | crate::domain::ItemKind::SecureNote => {
            let value = form
                .get_value(&FormField::Content)
                .unwrap_or("")
                .to_string();
            if form.kind == crate::domain::ItemKind::SecureNote {
                Some(ItemContent::SecureNote { content: value })
            } else {
                Some(ItemContent::Generic { value })
            }
        }
        crate::domain::ItemKind::CryptoSeed => Some(ItemContent::CryptoSeed {
            seed_phrase: form
                .get_value(&FormField::SeedPhrase)
                .unwrap_or("")
                .to_string(),
            derivation_path: form
                .get_value(&FormField::DerivationPath)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            network: form
                .get_value(&FormField::Network)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
        }),
        crate::domain::ItemKind::Password => Some(ItemContent::Password {
            username: form
                .get_value(&FormField::Username)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            password: form
                .get_value(&FormField::Password)
                .unwrap_or("")
                .to_string(),
            url: form
                .get_value(&FormField::Url)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            totp_secret: form
                .get_value(&FormField::TotpSecret)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
        }),
        crate::domain::ItemKind::ApiKey => Some(ItemContent::ApiKey {
            key: form.get_value(&FormField::ApiKey).unwrap_or("").to_string(),
            service: form
                .get_value(&FormField::Service)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            expires_at: None,
        }),
        crate::domain::ItemKind::Custom => Some(ItemContent::Custom {
            fields: parse_custom_fields(
                form.get_value(&FormField::CustomFields).unwrap_or_default(),
            )?,
        }),
        crate::domain::ItemKind::Totp => Some(ItemContent::Totp {
            issuer: form
                .get_value(&FormField::Issuer)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            account_name: form
                .get_value(&FormField::AccountName)
                .unwrap_or("")
                .to_string(),
            secret: form
                .get_value(&FormField::TotpSecret)
                .unwrap_or("")
                .to_string(),
        }),
    };

    Ok(ItemUpdates {
        title,
        content,
        notes,
        tags: None,
        favorite: None,
    })
}

fn parse_custom_fields(
    input: &str,
) -> std::result::Result<Vec<crate::domain::CustomField>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    trimmed
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(parse_single_custom_field)
        .collect()
}

fn parse_single_custom_field(
    input: &str,
) -> std::result::Result<crate::domain::CustomField, String> {
    let Some((raw_type, key_value)) = input.split_once(':') else {
        return Err(format!(
            "Invalid custom field format '{input}'. Use type:key=value"
        ));
    };
    let field_type = parse_custom_field_type(raw_type.trim())?;

    let Some((raw_key, raw_value)) = key_value.split_once('=') else {
        return Err(format!(
            "Invalid custom field format '{input}'. Use type:key=value"
        ));
    };

    let key = raw_key.trim();
    if key.is_empty() {
        return Err("Custom field key cannot be empty".to_string());
    }

    Ok(crate::domain::CustomField {
        key: key.to_string(),
        value: raw_value.trim().to_string(),
        field_type,
    })
}

fn parse_custom_field_type(
    raw: &str,
) -> std::result::Result<crate::domain::CustomFieldType, String> {
    match raw.to_ascii_lowercase().as_str() {
        "text" => Ok(crate::domain::CustomFieldType::Text),
        "secret" => Ok(crate::domain::CustomFieldType::Secret),
        "url" => Ok(crate::domain::CustomFieldType::Url),
        "number" => Ok(crate::domain::CustomFieldType::Number),
        _ => Err(format!(
            "Unsupported custom field type '{raw}'. Use text, secret, url, or number"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::SecureString;
    use crate::domain::{Item, Vault};
    use crate::storage::{AppConfig, VaultRegistry};
    use std::path::PathBuf;
    use std::time::{Duration, Instant};
    use tempfile::tempdir;

    fn test_state() -> AppState {
        let config = AppConfig::default();
        let registry = VaultRegistry::default();
        let mut state = AppState::new(config, registry);

        // Setup unlocked vault
        let mut vault = Vault::new("Test Vault");
        vault.add_item(Item::password("GitHub", "secret123"));
        vault.add_item(Item::password("AWS", "aws-key"));

        state.vault_state = Some(VaultState::new(
            vault,
            PathBuf::from("/test/vault.vlt"),
            [0u8; 32],
            [0u8; 32], // salt
            false,
            crate::crypto::EncryptionMethod::Aes256Gcm,
            None,
        ));
        state.mode = AppMode::Unlocked;
        state.screen = Screen::Main;

        state
    }

    fn login_state_with_vault(path: PathBuf, name: &str) -> AppState {
        let config = AppConfig::default();
        let mut registry = VaultRegistry::default();
        registry.add_or_update(&path, name);
        let mut state = AppState::new(config, registry);
        state.screen = Screen::Login;
        state
    }

    #[test]
    fn test_navigate() {
        let mut state = test_state();
        let effect = update(&mut state, Message::Navigate(Screen::Settings));

        assert_eq!(state.screen, Screen::Settings);
        assert!(effect.is_none());
    }

    #[test]
    fn test_select_item() {
        let mut state = test_state();
        let item_id = state.vault_state.as_ref().unwrap().vault.items[0].id;

        let effect = update(&mut state, Message::SelectItem(item_id));

        assert_eq!(
            state.vault_state.as_ref().unwrap().selected_item_id,
            Some(item_id)
        );
        assert!(effect.is_none());
    }

    #[test]
    fn test_toggle_favorite() {
        let mut state = test_state();
        let item_id = state.vault_state.as_ref().unwrap().vault.items[0].id;

        // Initially not favorite
        assert!(!state.vault_state.as_ref().unwrap().vault.items[0].favorite);

        update(&mut state, Message::ToggleFavorite(item_id));

        assert!(state.vault_state.as_ref().unwrap().vault.items[0].favorite);
        assert!(state.is_dirty());
    }

    #[test]
    fn test_undo_redo() {
        let mut state = test_state();
        let item_id = state.vault_state.as_ref().unwrap().vault.items[0].id;

        // Make a change
        update(
            &mut state,
            Message::UpdateItem {
                id: item_id,
                updates: ItemUpdates::new().title("Changed Title"),
            },
        );

        assert_eq!(
            state.vault_state.as_ref().unwrap().vault.items[0].title,
            "Changed Title"
        );
        assert!(state.vault_state.as_ref().unwrap().can_undo());

        // Undo
        update(&mut state, Message::Undo);
        assert_eq!(
            state.vault_state.as_ref().unwrap().vault.items[0].title,
            "GitHub"
        );
        assert!(state.vault_state.as_ref().unwrap().can_redo());

        // Redo
        update(&mut state, Message::Redo);
        assert_eq!(
            state.vault_state.as_ref().unwrap().vault.items[0].title,
            "Changed Title"
        );
    }

    #[test]
    fn test_search() {
        let mut state = test_state();

        update(&mut state, Message::OpenSearch);
        assert!(state.ui_state.has_floating_window());

        update(&mut state, Message::UpdateSearchQuery("Git".to_string()));
        // Check search results inside the floating window
        if let Some(FloatingWindow::Search {
            state: search_state,
        }) = &state.ui_state.floating_window
        {
            assert_eq!(search_state.results.len(), 1);
        } else {
            panic!("Expected Search floating window");
        }

        update(&mut state, Message::UpdateSearchQuery("".to_string()));
        if let Some(FloatingWindow::Search {
            state: search_state,
        }) = &state.ui_state.floating_window
        {
            assert!(search_state.results.is_empty());
        }
    }

    #[test]
    fn test_filter() {
        let mut state = test_state();

        // Add a favorite item
        let item_id = state.vault_state.as_ref().unwrap().vault.items[0].id;
        update(&mut state, Message::ToggleFavorite(item_id));

        // Toggle favorites filter
        update(&mut state, Message::ToggleFavoritesFilter);
        assert!(state.ui_state.filter.is_active());

        let filtered = get_filtered_items(&state);
        assert_eq!(filtered.len(), 1);

        // Clear filter
        update(&mut state, Message::ClearFilters);
        assert!(!state.ui_state.filter.is_active());
    }

    #[test]
    fn test_toggle_reveal() {
        let mut state = test_state();
        assert!(!state.ui_state.content_revealed);

        update(&mut state, Message::ToggleContentReveal);
        assert!(state.ui_state.content_revealed);

        update(&mut state, Message::ToggleContentReveal);
        assert!(!state.ui_state.content_revealed);
    }

    #[test]
    fn test_tick_does_not_refresh_activity() {
        let mut state = test_state();
        let old_activity = Instant::now() - Duration::from_secs(10);
        state.vault_state.as_mut().unwrap().last_activity = old_activity;

        update(&mut state, Message::Tick);

        let current = state.vault_state.as_ref().unwrap().last_activity;
        assert_eq!(current, old_activity);
    }

    #[test]
    fn test_lock_vault_dirty_defers_lock_until_save() {
        let mut state = test_state();
        state.vault_state.as_mut().unwrap().is_dirty = true;

        let effect = update(&mut state, Message::LockVault);

        assert!(matches!(effect, Effect::WriteVaultFile { .. }));
        assert!(state.pending_lock);
        assert!(state.vault_state.is_some());
        assert_eq!(state.mode, AppMode::Unlocked);
    }

    #[test]
    fn test_lock_vault_clean_locks_immediately() {
        let mut state = test_state();
        state.vault_state.as_mut().unwrap().is_dirty = false;

        let effect = update(&mut state, Message::LockVault);

        assert!(effect.is_none());
        assert!(!state.pending_lock);
        assert!(state.vault_state.is_none());
        assert_eq!(state.mode, AppMode::Locked);
        assert_eq!(state.screen, Screen::Login);
    }

    #[test]
    fn test_save_vault_keeps_dirty_until_save_result() {
        let mut state = test_state();
        state.vault_state.as_mut().unwrap().is_dirty = true;

        let effect = update(&mut state, Message::SaveVault);

        assert!(matches!(effect, Effect::WriteVaultFile { .. }));
        assert!(state.vault_state.as_ref().unwrap().is_dirty);
    }

    #[test]
    fn test_unlock_vault_message_emits_read_effect() {
        let path = PathBuf::from("/tmp/test-unlock.vault");
        let mut state = login_state_with_vault(path.clone(), "Test");

        let effect = update(
            &mut state,
            Message::UnlockVault {
                password: SecureString::from_str("password123"),
                keyfile: None,
            },
        );

        match effect {
            Effect::ReadVaultFile {
                path: p,
                password,
                keyfile,
            } => {
                assert_eq!(p, path);
                assert_eq!(password.as_str(), "password123");
                assert!(keyfile.is_none());
            }
            other => panic!("Expected ReadVaultFile effect, got {:?}", other),
        }
    }

    #[test]
    fn test_password_submit_switches_to_keyfile_mode_for_keyfile_vault() {
        let dir = tempdir().unwrap();
        let vault_path = dir.path().join("keyfile-required.vault");
        let keyfile_bytes = vec![1u8; 32];
        let password = SecureString::from_str("password123");
        let vault = Vault::new("Keyfile Vault");
        let vault_file = crate::storage::VaultFile::new(&vault, &password, Some(&keyfile_bytes))
            .expect("create vault with keyfile");
        vault_file.write(&vault_path).expect("write vault");

        let mut state = login_state_with_vault(vault_path, "Keyfile Vault");
        state.login_screen.entering_password = true;
        state.ui_state.input_buffer.text = "password123".to_string();
        state.ui_state.input_buffer.cursor = state.ui_state.input_buffer.text.len();
        state.ui_state.input_buffer.masked = true;

        let effect = update(&mut state, Message::InputSubmit);

        assert!(effect.is_none());
        assert!(!state.login_screen.entering_password);
        assert!(state.login_screen.entering_keyfile_path);
        assert!(state.login_screen.pending_unlock_password.is_some());
        assert!(!state.ui_state.input_buffer.masked);
    }

    #[test]
    fn test_keyfile_submit_emits_read_effect_with_loaded_keyfile() {
        let dir = tempdir().unwrap();
        let vault_path = dir.path().join("needs-keyfile.vault");
        let keyfile_path = dir.path().join("unlock.key");
        let keyfile_bytes = vec![7u8; 32];
        std::fs::write(&keyfile_path, &keyfile_bytes).expect("write keyfile");

        let password = SecureString::from_str("password123");
        let vault = Vault::new("Keyfile Vault");
        let vault_file = crate::storage::VaultFile::new(&vault, &password, Some(&keyfile_bytes))
            .expect("create vault with keyfile");
        vault_file.write(&vault_path).expect("write vault");

        let mut state = login_state_with_vault(vault_path.clone(), "Keyfile Vault");
        state.login_screen.entering_keyfile_path = true;
        state.login_screen.pending_unlock_password = Some(SecureString::from_str("password123"));
        state.ui_state.input_buffer.text = keyfile_path.to_string_lossy().to_string();
        state.ui_state.input_buffer.cursor = state.ui_state.input_buffer.text.len();
        state.ui_state.input_buffer.masked = false;

        let effect = update(&mut state, Message::InputSubmit);

        match effect {
            Effect::ReadVaultFile {
                path,
                password,
                keyfile,
            } => {
                assert_eq!(path, vault_path);
                assert_eq!(password.as_str(), "password123");
                assert_eq!(keyfile, Some(keyfile_bytes));
            }
            other => panic!("Expected ReadVaultFile effect, got {:?}", other),
        }
    }

    #[test]
    fn test_cancel_input_from_keyfile_mode_returns_to_password_mode() {
        let mut state = login_state_with_vault(PathBuf::from("/tmp/test.vault"), "Test");
        state.login_screen.entering_password = false;
        state.login_screen.entering_keyfile_path = true;
        state.login_screen.pending_unlock_password = Some(SecureString::from_str("password123"));
        state.ui_state.input_buffer.masked = false;
        state.ui_state.input_buffer.text = "/tmp/keyfile".to_string();

        update(&mut state, Message::CancelInput);

        assert!(state.login_screen.entering_password);
        assert!(!state.login_screen.entering_keyfile_path);
        assert!(state.login_screen.pending_unlock_password.is_none());
        assert!(state.ui_state.input_buffer.masked);
        assert!(state.ui_state.input_buffer.text.is_empty());
    }

    #[test]
    fn test_parse_custom_fields_success() {
        let parsed =
            parse_custom_fields("text:username=alice;secret:token=abc123;number:port=443").unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].key, "username");
        assert_eq!(parsed[1].field_type, crate::domain::CustomFieldType::Secret);
        assert_eq!(parsed[2].value, "443");
    }

    #[test]
    fn test_parse_custom_fields_rejects_invalid_type() {
        let err = parse_custom_fields("unknown:key=value")
            .expect_err("expected unsupported type to return an error");
        assert!(err.contains("Unsupported custom field type"));
    }

    #[test]
    fn test_create_custom_item_from_form() {
        let mut form =
            crate::ui::widgets::EditFormState::new(crate::domain::ItemKind::Custom, true);
        let title_idx = form
            .fields
            .iter()
            .position(|f| *f == crate::ui::widgets::FormField::Title)
            .unwrap();
        let fields_idx = form
            .fields
            .iter()
            .position(|f| *f == crate::ui::widgets::FormField::CustomFields)
            .unwrap();

        form.values[title_idx] = "Infra".to_string();
        form.values[fields_idx] = "text:role=admin;secret:token=xyz".to_string();

        let item = create_item_from_form(&form).expect("custom form should be valid");
        match item.content {
            crate::domain::ItemContent::Custom { fields } => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[1].field_type, crate::domain::CustomFieldType::Secret);
            }
            other => panic!("expected custom content, got {:?}", other),
        }
    }
}
