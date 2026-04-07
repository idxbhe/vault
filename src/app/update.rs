//! State update logic - the heart of TEA
//!
//! The update function takes the current state and a message, returning
//! the new state and any effects to execute.

use std::time::Duration;

use uuid::Uuid;

use crate::domain::Item;

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
    // Update last activity time if vault is unlocked
    if let Some(ref mut vs) = state.vault_state {
        vs.touch();
    }

    match message {
        // === Navigation ===
        Message::Navigate(screen) => {
            state.screen = screen;
            Effect::none()
        }

        Message::FocusPane(pane) => {
            state.ui_state.focused_pane = pane;
            Effect::none()
        }

        // === Vault Operations ===
        Message::CreateVault {
            name,
            path,
            password,
            keyfile,
        } => {
            // This will be handled by effect executor
            state.mode = AppMode::Creating;
            state.ui_state.start_loading("Creating vault...");
            let keyfile_data = keyfile.map(|_| vec![]); // Will be populated by effect
            Effect::ReadVaultFile {
                path,
                password,
                keyfile: keyfile_data,
            }
        }

        Message::OpenVault { path } => {
            // Store path for later, wait for password
            state.ui_state.input_buffer.clear();
            Effect::none()
        }

        Message::UnlockVault { password, keyfile } => {
            // Will be handled by effect executor to actually decrypt
            Effect::none()
        }

        Message::LockVault => {
            if let Some(ref vs) = state.vault_state {
                // Save before locking if dirty
                let effect = if state.is_dirty() {
                    Effect::WriteVaultFile {
                        path: vs.vault_path.clone(),
                        vault: vs.vault.clone(),
                        key: vs.encryption_key,
                        salt: vs.salt,
                    }
                } else {
                    Effect::none()
                };

                // Clear vault state
                state.vault_state = None;
                state.mode = AppMode::Locked;
                state.screen = Screen::Login;
                state.ui_state = Default::default();

                effect
            } else {
                Effect::none()
            }
        }

        Message::SaveVault => {
            if let Some(ref mut vs) = state.vault_state {
                vs.is_dirty = false;
                Effect::WriteVaultFile {
                    path: vs.vault_path.clone(),
                    vault: vs.vault.clone(),
                    key: vs.encryption_key,
                    salt: vs.salt,
                }
            } else {
                Effect::none()
            }
        }

        Message::CloseVault => {
            let effect = if state.is_dirty() {
                // Prompt to save first
                state.ui_state.floating_window =
                    Some(FloatingWindow::ConfirmDelete { item_id: Uuid::nil() });
                Effect::none()
            } else {
                state.vault_state = None;
                state.mode = AppMode::Locked;
                state.screen = Screen::Login;
                Effect::none()
            };
            effect
        }
        
        // === Login Flow ===
        Message::StartCreateVault => {
            // Switch login screen to "creating" mode
            state.login_screen.creating_vault = true;
            state.login_screen.entering_password = false;
            state.login_screen.create_step = 0;
            state.login_screen.new_vault_name.clear();
            state.login_screen.new_vault_password.clear();
            state.login_screen.error_message = None;
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = false; // Vault name is not masked
            Effect::none()
        }
        
        Message::EnterPasswordMode => {
            // Switch login screen to "password entry" mode
            state.login_screen.entering_password = true;
            state.login_screen.creating_vault = false;
            state.login_screen.error_message = None; // Clear any previous error
            state.ui_state.input_buffer.clear();
            state.ui_state.input_buffer.masked = true; // Password is masked
            Effect::none()
        }
        
        Message::CancelInput => {
            // Cancel any input mode and return to vault selection
            state.login_screen.entering_password = false;
            state.login_screen.creating_vault = false;
            state.login_screen.create_step = 0;
            state.login_screen.new_vault_name.clear();
            state.login_screen.new_vault_password.clear();
            state.login_screen.error_message = None;
            state.ui_state.input_buffer.clear();
            state.ui_state.floating_window = None;
            Effect::none()
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
            if let Some(ref mut vs) = state.vault_state {
                if vs.vault.get_item(id).is_some() {
                    vs.selected_item_id = Some(id);
                    state.ui_state.detail_scroll_offset = 0;
                }
            }
            Effect::none()
        }

        Message::SelectNextItem => {
            select_adjacent_item(state, 1);
            Effect::none()
        }

        Message::SelectPrevItem => {
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
            if let Some(ref mut vs) = state.vault_state {
                if let Some(item) = vs.vault.get_item(id) {
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

            if let Some(ref mut vs) = state.vault_state {
                if let Some(item) = vs.vault.get_item(id) {
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
            }
            Effect::none()
        }

        Message::ToggleFavorite(id) => {
            if let Some(ref mut vs) = state.vault_state {
                if let Some(item) = vs.vault.get_item_mut(id) {
                    item.favorite = !item.favorite;
                    item.touch();
                    vs.mark_dirty();
                }
            }
            Effect::none()
        }

        Message::DuplicateItem(id) => {
            if let Some(ref mut vs) = state.vault_state {
                if let Some(item) = vs.vault.get_item(id) {
                    let mut new_item = item.clone();
                    new_item.id = Uuid::new_v4();
                    new_item.title = format!("{} (Copy)", item.title);
                    let new_id = new_item.id;
                    vs.vault.add_item(new_item);
                    vs.selected_item_id = Some(new_id);
                    vs.mark_dirty();
                }
            }
            Effect::none()
        }

        // === History ===
        Message::Undo => {
            if let Some(ref mut vs) = state.vault_state {
                if let Some(entry) = vs.undo_stack.pop() {
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
            }
            Effect::none()
        }

        Message::Redo => {
            if let Some(ref mut vs) = state.vault_state {
                if let Some(entry) = vs.redo_stack.pop() {
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
            if let Some(FloatingWindow::Search { state: search_state }) = &mut state.ui_state.floating_window {
                search_state.query = query;
                if let Some(ref vs) = state.vault_state {
                    search_state.update_results(&vs.vault.items);
                }
            }
            Effect::none()
        }

        Message::ExecuteSearch => {
            if let Some(FloatingWindow::Search { state: search_state }) = &mut state.ui_state.floating_window {
                if let Some(ref vs) = state.vault_state {
                    search_state.update_results(&vs.vault.items);
                }
            }
            Effect::none()
        }

        Message::SelectSearchResult(index) => {
            let selected_id = if let Some(FloatingWindow::Search { state: search_state }) = &state.ui_state.floating_window {
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
            if let Some(FloatingWindow::Search { state: search_state }) = &mut state.ui_state.floating_window {
                search_state.next_result();
            }
            Effect::none()
        }

        Message::SearchPrevResult => {
            if let Some(FloatingWindow::Search { state: search_state }) = &mut state.ui_state.floating_window {
                search_state.prev_result();
            }
            Effect::none()
        }

        Message::SearchConfirm => {
            let selected_id = if let Some(FloatingWindow::Search { state: search_state }) = &state.ui_state.floating_window {
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
                state.clipboard_state.set_secure(state.config.clipboard_timeout_secs);
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
            if let Some(item) = state.selected_item() {
                if let Some(content) = item.get_copyable_content() {
                    return update(
                        state,
                        Message::CopyToClipboard {
                            content: content.to_string(),
                            is_sensitive: true,
                        },
                    );
                }
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
            match &mut state.ui_state.floating_window {
                Some(FloatingWindow::NewItem { form }) | Some(FloatingWindow::EditItem { form, .. }) => {
                    form.insert(c);
                }
                Some(FloatingWindow::Search { state: search_state }) => {
                    search_state.insert(c);
                    if let Some(ref vs) = state.vault_state {
                        search_state.update_results(&vs.vault.items);
                    }
                }
                _ => {
                    state.ui_state.input_buffer.insert(c);
                }
            }
            Effect::none()
        }

        Message::InputBackspace => {
            // Clear login error when user starts typing
            if state.login_screen.error_message.is_some() {
                state.login_screen.error_message = None;
            }
            
            match &mut state.ui_state.floating_window {
                Some(FloatingWindow::NewItem { form }) | Some(FloatingWindow::EditItem { form, .. }) => {
                    form.backspace();
                }
                Some(FloatingWindow::Search { state: search_state }) => {
                    search_state.backspace();
                    if let Some(ref vs) = state.vault_state {
                        search_state.update_results(&vs.vault.items);
                    }
                }
                _ => {
                    state.ui_state.input_buffer.backspace();
                }
            }
            Effect::none()
        }

        Message::InputDelete => {
            state.ui_state.input_buffer.delete();
            Effect::none()
        }

        Message::InputLeft => {
            match &mut state.ui_state.floating_window {
                Some(FloatingWindow::NewItem { form }) | Some(FloatingWindow::EditItem { form, .. }) => {
                    form.move_left();
                }
                Some(FloatingWindow::Search { state: search_state }) => {
                    search_state.move_left();
                }
                _ => {
                    state.ui_state.input_buffer.move_left();
                }
            }
            Effect::none()
        }

        Message::InputRight => {
            match &mut state.ui_state.floating_window {
                Some(FloatingWindow::NewItem { form }) | Some(FloatingWindow::EditItem { form, .. }) => {
                    form.move_right();
                }
                Some(FloatingWindow::Search { state: search_state }) => {
                    search_state.move_right();
                }
                _ => {
                    state.ui_state.input_buffer.move_right();
                }
            }
            Effect::none()
        }

        Message::InputHome => {
            state.ui_state.input_buffer.home();
            Effect::none()
        }

        Message::InputEnd => {
            state.ui_state.input_buffer.end();
            Effect::none()
        }

        Message::InputSubmit => {
            // Context-aware submit handling
            if state.screen == Screen::Login {
                if state.login_screen.creating_vault {
                    let input = state.ui_state.input_buffer.text.clone();
                    
                    match state.login_screen.create_step {
                        0 => {
                            // Step 0: Vault name
                            if input.is_empty() {
                                state.login_screen.error_message = Some("Vault name cannot be empty".to_string());
                                return Effect::none();
                            }
                            // Save name and move to password step
                            state.login_screen.new_vault_name = input;
                            state.login_screen.create_step = 1;
                            state.login_screen.error_message = None;
                            state.ui_state.input_buffer.clear();
                            state.ui_state.input_buffer.masked = true;
                            return Effect::none();
                        }
                        1 => {
                            // Step 1: Password
                            if input.len() < 4 {
                                state.login_screen.error_message = Some("Password must be at least 4 characters".to_string());
                                return Effect::none();
                            }
                            // Save password and move to confirm step
                            state.login_screen.new_vault_password = input;
                            state.login_screen.create_step = 2;
                            state.login_screen.error_message = None;
                            state.ui_state.input_buffer.clear();
                            return Effect::none();
                        }
                        2 => {
                            // Step 2: Confirm password
                            if input != state.login_screen.new_vault_password {
                                state.login_screen.error_message = Some("Passwords do not match".to_string());
                                state.ui_state.input_buffer.clear();
                                return Effect::none();
                            }
                            
                            // Create the vault!
                            let vault_name = state.login_screen.new_vault_name.clone();
                            let password = state.login_screen.new_vault_password.clone();
                            
                            // Determine vault path using directories crate
                            let vault_filename = format!("{}.vault", vault_name.to_lowercase().replace(' ', "_"));
                            let vault_path = directories::ProjectDirs::from("com", "vault", "vault")
                                .map(|dirs| dirs.data_dir().to_path_buf())
                                .unwrap_or_else(|| std::path::PathBuf::from(".").join(".vault"))
                                .join(&vault_filename);
                            
                            // Create vault directory if needed
                            if let Some(parent) = vault_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            
                            // Create new vault
                            let vault = crate::domain::Vault::new(&vault_name);
                            let secure_password = crate::crypto::SecureString::new(password.clone());
                            
                            // Try to create vault file
                            match crate::storage::VaultFile::new(&vault, &secure_password, None) {
                                Ok(vault_file) => {
                                    // Extract salt before writing
                                    let salt = vault_file.encrypted_payload.salt;
                                    
                                    if let Err(e) = vault_file.write(&vault_path) {
                                        state.login_screen.error_message = Some(format!("Failed to save vault: {}", e));
                                        return Effect::none();
                                    }
                                    
                                    // Get the encryption key
                                    let (_, key) = vault_file.decrypt_with_key(&secure_password, None)
                                        .expect("Just created, should decrypt");
                                    
                                    // Add to registry
                                    state.registry.add_or_update(&vault_path, &vault_name);
                                    let _ = state.registry.save();
                                    
                                    // Reset login screen state
                                    state.login_screen.creating_vault = false;
                                    state.login_screen.create_step = 0;
                                    state.login_screen.new_vault_name.clear();
                                    state.login_screen.new_vault_password.clear();
                                    state.ui_state.input_buffer.clear();
                                    
                                    // Set up vault state and transition to main (with salt)
                                    state.vault_state = Some(crate::app::VaultState::new(vault, vault_path, key, salt));
                                    state.mode = crate::app::AppMode::Unlocked;
                                    state.screen = Screen::Main;
                                    
                                    state.ui_state.notify("Vault created successfully!", NotificationLevel::Success);
                                }
                                Err(e) => {
                                    state.login_screen.error_message = Some(format!("Failed to create vault: {}", e));
                                }
                            }
                            return Effect::none();
                        }
                        _ => {}
                    }
                    return Effect::none();
                } else if state.login_screen.entering_password {
                    // Submit password to unlock vault
                    let password = state.ui_state.input_buffer.text.trim().to_string();
                    
                    if password.is_empty() {
                        state.login_screen.error_message = Some("Password cannot be empty".to_string());
                        return Effect::none();
                    }
                    
                    // Get selected vault path from registry
                    let selected_idx = state.login_screen.selected_vault;
                    if let Some(entry) = state.registry.entries.get(selected_idx) {
                        let path = entry.path.clone();
                        
                        // Clear input buffer (password entered)
                        state.ui_state.input_buffer.clear();
                        
                        // Set loading state
                        state.ui_state.start_loading("Unlocking vault...");
                        
                        // Trigger vault decryption
                        return Effect::ReadVaultFile {
                            path,
                            password: crate::crypto::SecureString::new(password),
                            keyfile: None, // TODO: Support keyfile selection
                        };
                    } else {
                        state.login_screen.error_message = Some("No vault selected".to_string());
                        return Effect::none();
                    }
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
            match &mut state.ui_state.floating_window {
                Some(FloatingWindow::NewItem { form }) | Some(FloatingWindow::EditItem { form, .. }) => {
                    form.next_field();
                }
                _ => {}
            }
            Effect::none()
        }

        Message::FormPrevField => {
            match &mut state.ui_state.floating_window {
                Some(FloatingWindow::NewItem { form }) | Some(FloatingWindow::EditItem { form, .. }) => {
                    form.prev_field();
                }
                _ => {}
            }
            Effect::none()
        }

        Message::FormSubmit => {
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
                        let item = create_item_from_form(&form);
                        let id = item.id;
                        vs.vault.add_item(item);
                        vs.selected_item_id = Some(id);
                        vs.mark_dirty();
                        state.ui_state.notify("Item created and saved", NotificationLevel::Success);
                        
                        // Auto-save to disk
                        vs.is_dirty = false;
                        return Effect::WriteVaultFile {
                            path: vs.vault_path.clone(),
                            vault: vs.vault.clone(),
                            key: vs.encryption_key,
                            salt: vs.salt,
                        };
                    }
                }
                Some(FloatingWindow::EditItem { item_id, form }) => {
                    if let Err(msg) = form.validate() {
                        state.ui_state.notify(msg, NotificationLevel::Error);
                        state.ui_state.floating_window = Some(FloatingWindow::EditItem { item_id, form });
                        return Effect::none();
                    }
                    
                    // Update the item from form data
                    if let Some(ref mut vs) = state.vault_state {
                        if let Some(item) = vs.vault.get_item(item_id) {
                            // Save undo entry
                            let undo_entry = UndoEntry {
                                description: format!("Edit {}", item.title),
                                item_id,
                                previous_state: ItemSnapshot::from_item(item),
                            };
                            
                            // Apply updates
                            let updates = create_updates_from_form(&form);
                            if let Some(item) = vs.vault.get_item_mut(item_id) {
                                apply_item_updates(item, updates);
                            }
                            
                            vs.push_undo(undo_entry);
                            vs.mark_dirty();
                            state.ui_state.notify("Item updated and saved", NotificationLevel::Success);
                            
                            // Auto-save to disk
                            vs.is_dirty = false;
                            return Effect::WriteVaultFile {
                                path: vs.vault_path.clone(),
                                vault: vs.vault.clone(),
                                key: vs.encryption_key,
                                salt: vs.salt,
                            };
                        }
                    }
                }
                other => {
                    state.ui_state.floating_window = other;
                }
            }
            Effect::none()
        }

        Message::KindSelectorNext => {
            if let Some(FloatingWindow::KindSelector { state: ref mut selector }) = state.ui_state.floating_window {
                selector.next();
            }
            Effect::none()
        }

        Message::KindSelectorPrev => {
            if let Some(FloatingWindow::KindSelector { state: ref mut selector }) = state.ui_state.floating_window {
                selector.prev();
            }
            Effect::none()
        }

        Message::KindSelectorConfirm => {
            if let Some(FloatingWindow::KindSelector { state: selector }) = state.ui_state.floating_window.take() {
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
            if let Some(ref mut vs) = state.vault_state {
                if let Some(item) = vs.vault.get_item_mut(item_id) {
                    if item.tags.contains(&tag_id) {
                        item.tags.retain(|t| *t != tag_id);
                    } else {
                        item.tags.push(tag_id);
                    }
                    item.touch();
                    vs.mark_dirty();
                }
            }
            Effect::none()
        }

        // === Filter ===
        Message::SetKindFilter(kind) => {
            state.ui_state.filter.kind = kind;
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
                }
            } else {
                state.ui_state.notify("No vault open to export", NotificationLevel::Warning);
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
            if state.config.auto_lock_enabled {
                if let Some(ref vs) = state.vault_state {
                    let elapsed = vs.last_activity.elapsed();
                    if elapsed.as_secs() >= state.config.auto_lock_timeout_secs {
                        return update(state, Message::LockVault);
                    }
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
    }
}

/// Select adjacent item in the list
fn select_adjacent_item(state: &mut AppState, delta: i32) {
    let Some(ref vs) = state.vault_state else {
        return;
    };
    
    // Get filtered item IDs
    let items: Vec<Uuid> = vs.vault
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
                if !state.ui_state.filter.tags.iter().any(|t| item.tags.contains(t)) {
                    return false;
                }
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

    if let Some(id) = items.get(new_idx) {
        if let Some(ref mut vs) = state.vault_state {
            vs.selected_item_id = Some(*id);
        }
    }
}

/// Get items filtered by current filter state
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
            // Detail scroll max would be based on content height
            (&mut state.ui_state.detail_scroll_offset, 100)
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

/// Create a new item from form data
fn create_item_from_form(form: &crate::ui::widgets::EditFormState) -> Item {
    use crate::domain::ItemContent;
    use crate::ui::widgets::FormField;

    let title = form.get_value(&FormField::Title).unwrap_or("Untitled").to_string();
    let notes = form.get_value(&FormField::Notes).filter(|s| !s.is_empty()).map(|s| s.to_string());

    let content = match form.kind {
        crate::domain::ItemKind::Generic | crate::domain::ItemKind::SecureNote => {
            let value = form.get_value(&FormField::Content).unwrap_or("").to_string();
            if form.kind == crate::domain::ItemKind::SecureNote {
                ItemContent::SecureNote { content: value }
            } else {
                ItemContent::Generic { value }
            }
        }
        crate::domain::ItemKind::CryptoSeed => {
            ItemContent::CryptoSeed {
                seed_phrase: form.get_value(&FormField::SeedPhrase).unwrap_or("").to_string(),
                derivation_path: form.get_value(&FormField::DerivationPath).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                network: form.get_value(&FormField::Network).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            }
        }
        crate::domain::ItemKind::Password => {
            ItemContent::Password {
                username: form.get_value(&FormField::Username).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                password: form.get_value(&FormField::Password).unwrap_or("").to_string(),
                url: form.get_value(&FormField::Url).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                totp_secret: None,
            }
        }
        crate::domain::ItemKind::ApiKey => {
            ItemContent::ApiKey {
                key: form.get_value(&FormField::ApiKey).unwrap_or("").to_string(),
                service: form.get_value(&FormField::Service).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                expires_at: None,
            }
        }
    };

    let mut item = Item::new(&title, form.kind, content);
    item.notes = notes;
    item
}

/// Create item updates from form data
fn create_updates_from_form(form: &crate::ui::widgets::EditFormState) -> ItemUpdates {
    use crate::domain::ItemContent;
    use crate::ui::widgets::FormField;

    let title = form.get_value(&FormField::Title).map(|s| s.to_string());
    let notes = Some(form.get_value(&FormField::Notes).filter(|s| !s.is_empty()).map(|s| s.to_string()));

    let content = match form.kind {
        crate::domain::ItemKind::Generic | crate::domain::ItemKind::SecureNote => {
            let value = form.get_value(&FormField::Content).unwrap_or("").to_string();
            if form.kind == crate::domain::ItemKind::SecureNote {
                Some(ItemContent::SecureNote { content: value })
            } else {
                Some(ItemContent::Generic { value })
            }
        }
        crate::domain::ItemKind::CryptoSeed => {
            Some(ItemContent::CryptoSeed {
                seed_phrase: form.get_value(&FormField::SeedPhrase).unwrap_or("").to_string(),
                derivation_path: form.get_value(&FormField::DerivationPath).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                network: form.get_value(&FormField::Network).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            })
        }
        crate::domain::ItemKind::Password => {
            Some(ItemContent::Password {
                username: form.get_value(&FormField::Username).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                password: form.get_value(&FormField::Password).unwrap_or("").to_string(),
                url: form.get_value(&FormField::Url).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                totp_secret: None,
            })
        }
        crate::domain::ItemKind::ApiKey => {
            Some(ItemContent::ApiKey {
                key: form.get_value(&FormField::ApiKey).unwrap_or("").to_string(),
                service: form.get_value(&FormField::Service).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                expires_at: None,
            })
        }
    };

    ItemUpdates {
        title,
        content,
        notes,
        tags: None,
        favorite: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Item, ItemKind, Vault};
    use crate::storage::{AppConfig, VaultRegistry};
    use std::path::PathBuf;

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
            [0u8; 32],  // salt
        ));
        state.mode = AppMode::Unlocked;
        state.screen = Screen::Main;

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
        if let Some(FloatingWindow::Search { state: search_state }) = &state.ui_state.floating_window {
            assert_eq!(search_state.results.len(), 1);
        } else {
            panic!("Expected Search floating window");
        }

        update(&mut state, Message::UpdateSearchQuery("".to_string()));
        if let Some(FloatingWindow::Search { state: search_state }) = &state.ui_state.floating_window {
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
}
