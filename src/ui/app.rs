//! Main UI application wrapper
//!
//! Coordinates state, update, and rendering.

use std::path::PathBuf;

use ratatui::Frame;

use crate::app::state::NotificationLevel;
use crate::app::{AppMode, AppState, Effect, Message, Screen, VaultState, update};
use crate::crypto::EncryptionMethod;
use crate::domain::Vault;
use crate::ui::screens::{
    ExportScreen, LoginScreen, MainScreen, SettingsScreen, render_export, render_login,
    render_main, render_settings,
};
use crate::ui::theme::get_theme;

/// Main application UI wrapper
pub struct App {
    /// Application state
    state: AppState,
    /// Main screen state
    main_screen: MainScreen,
    /// Export screen state
    export_screen: ExportScreen,
}

impl App {
    /// Create a new App with initial state
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            main_screen: MainScreen::new(),
            export_screen: ExportScreen::new(),
        }
    }

    /// Get a reference to the application state
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get a mutable reference to the application state
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    /// Process a message through TEA update
    pub fn update(&mut self, message: Message) -> Effect {
        update(&mut self.state, message)
    }

    /// Render the current screen
    pub fn render(&mut self, frame: &mut Frame) {
        let theme = get_theme(self.state.config.theme);

        // Clear layout regions at start of each render
        self.state.ui_state.clear_layout_regions();

        match self.state.screen {
            Screen::Login | Screen::PasswordRecovery | Screen::SecurityQuestions => {
                render_login(frame, &mut self.state, &theme);
            }
            Screen::Main => {
                render_main(frame, &mut self.state, &mut self.main_screen, &theme);
            }
            Screen::Settings => {
                render_settings(frame, &self.state, &self.state.settings_state, &theme);
            }
            Screen::Export => {
                render_export(frame, &self.state, &self.export_screen, &theme);
            }
        }
    }

    /// Get the login screen state (for external manipulation)
    pub fn login_screen_mut(&mut self) -> &mut LoginScreen {
        &mut self.state.login_screen
    }

    /// Get the main screen state
    pub fn main_screen_mut(&mut self) -> &mut MainScreen {
        &mut self.main_screen
    }

    /// Get the settings screen state
    pub fn settings_screen_mut(&mut self) -> &mut SettingsScreen {
        &mut self.state.settings_state
    }

    /// Get the export screen state
    pub fn export_screen_mut(&mut self) -> &mut ExportScreen {
        &mut self.export_screen
    }

    /// Handle successful vault load from effect
    pub fn handle_vault_loaded(
        &mut self,
        vault: Vault,
        path: PathBuf,
        key: [u8; 32],
        salt: [u8; 32],
        has_keyfile: bool,
        encryption_method: EncryptionMethod,
        recovery_metadata: Option<crate::domain::RecoveryMetadata>,
    ) {
        // Stop loading indicator
        self.state.ui_state.stop_loading();

        // Create vault state with salt
        let vault_state = VaultState::new(
            vault,
            path.clone(),
            key,
            salt,
            has_keyfile,
            encryption_method,
            recovery_metadata,
        );

        // Update app state
        self.state.vault_state = Some(vault_state);
        self.state.mode = AppMode::Unlocked;
        self.state.screen = Screen::Main;
        self.state.pending_lock = false;

        // Reset login screen state
        self.state.login_screen.entering_password = false;
        self.state.login_screen.entering_keyfile_path = false;
        self.state.login_screen.creating_vault = false;
        self.state.login_screen.pending_unlock_password = None;
        self.state.login_screen.error_message = None;
        self.state.ui_state.input_buffer.clear();

        // Show success notification
        self.state
            .ui_state
            .notify("Vault unlocked successfully", NotificationLevel::Success);
    }

    /// Handle vault creation success
    pub fn handle_vault_created(
        &mut self,
        vault: Vault,
        path: PathBuf,
        key: [u8; 32],
        salt: [u8; 32],
        has_keyfile: bool,
        encryption_method: EncryptionMethod,
        recovery_metadata: Option<crate::domain::RecoveryMetadata>,
    ) {
        // Stop loading indicator
        self.state.ui_state.stop_loading();

        // Same as loaded, but with different message
        let vault_state = VaultState::new(
            vault,
            path.clone(),
            key,
            salt,
            has_keyfile,
            encryption_method,
            recovery_metadata,
        );

        self.state.vault_state = Some(vault_state);
        self.state.mode = AppMode::Unlocked;
        self.state.screen = Screen::Main;
        self.state.pending_lock = false;

        // Reset login screen state
        self.state.login_screen.entering_password = false;
        self.state.login_screen.entering_keyfile_path = false;
        self.state.login_screen.creating_vault = false;
        self.state.login_screen.pending_unlock_password = None;
        self.state.login_screen.error_message = None;
        self.state.ui_state.input_buffer.clear();

        self.state
            .ui_state
            .notify("Vault created successfully", NotificationLevel::Success);
    }

    /// Handle effect error
    pub fn handle_effect_error(&mut self, error: String) {
        // Stop loading indicator
        self.state.ui_state.stop_loading();

        // Cancel any deferred lock if the save path failed.
        self.state.pending_lock = false;

        // Show error on login screen if applicable
        if self.state.screen == Screen::Login {
            self.state.login_screen.error_message = Some(error.clone());
        }

        // Also show notification
        self.state.ui_state.notify(&error, NotificationLevel::Error);

        // Log at debug level to avoid console spam for expected errors (like wrong password)
        tracing::debug!("Effect error: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppConfig, VaultRegistry};

    #[test]
    fn test_app_creation() {
        let config = AppConfig::default();
        let registry = VaultRegistry::default();
        let state = AppState::new(config, registry);

        let app = App::new(state);
        assert!(matches!(app.state().screen, Screen::Login));
    }

    #[test]
    fn test_app_update() {
        let config = AppConfig::default();
        let registry = VaultRegistry::default();
        let state = AppState::new(config, registry);

        let mut app = App::new(state);

        // Test navigation
        let effect = app.update(Message::Navigate(Screen::Settings));
        assert!(matches!(effect, Effect::None));
        assert!(matches!(app.state().screen, Screen::Settings));
    }
}
