//! Login screen - vault selection and password entry
//!
//! The entry point for the application where users select or create a vault.

use std::path::PathBuf;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::{AppState, Screen};
use crate::crypto::{EncryptionMethod, SecureString};
use crate::domain::RecoveryMetadata;
use crate::storage::VaultRegistryEntry;
use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// Current step in the create-vault wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreateVaultStep {
    #[default]
    Name,
    Password,
    ConfirmPassword,
    EncryptionMethod,
    KeyfileOption,
    KeyfilePath,
    RecoveryQuestionCount,
    RecoveryQuestion,
    RecoveryAnswer,
    ConfirmCreate,
}

/// Draft security question during vault creation.
#[derive(Debug, Clone, Default)]
pub struct SecurityQuestionDraft {
    pub question: String,
    pub answer: String,
}

/// Ongoing forgot-password recovery session.
#[derive(Debug, Clone)]
pub struct PasswordRecoverySession {
    pub vault_name: String,
    pub vault_path: PathBuf,
    pub metadata: RecoveryMetadata,
    pub current_question: usize,
    pub failed_attempts: u32,
    pub provided_answers: Vec<SecureString>,
    pub latest_hint: Option<String>,
    pub recovered_password: Option<String>,
}

impl PasswordRecoverySession {
    pub fn new(vault_name: String, vault_path: PathBuf, metadata: RecoveryMetadata) -> Self {
        Self {
            vault_name,
            vault_path,
            metadata,
            current_question: 0,
            failed_attempts: 0,
            provided_answers: Vec::new(),
            latest_hint: None,
            recovered_password: None,
        }
    }

    pub fn current_question_text(&self) -> Option<&str> {
        self.metadata
            .questions
            .get(self.current_question)
            .map(|q| q.question.as_str())
    }

    pub fn is_complete(&self) -> bool {
        self.current_question >= self.metadata.questions.len() && self.recovered_password.is_some()
    }

    pub fn is_locked_out(&self) -> bool {
        self.failed_attempts >= self.metadata.max_attempts
    }

    pub fn remaining_attempts(&self) -> u32 {
        self.metadata
            .max_attempts
            .saturating_sub(self.failed_attempts)
    }

    pub fn submit_answer(&mut self, answer: SecureString) -> crate::Result<bool> {
        if self.current_question >= self.metadata.questions.len() {
            return Ok(false);
        }

        let is_correct = self.metadata.questions[self.current_question].verify(&answer)?;
        if !is_correct {
            self.failed_attempts += 1;
            return Ok(false);
        }

        self.provided_answers.push(answer);
        let revealed = self.metadata.reveal_for_answers(&self.provided_answers)?;
        self.latest_hint = Some(revealed.clone());
        self.current_question += 1;

        if self.current_question >= self.metadata.questions.len() {
            self.recovered_password = Some(revealed);
        }

        Ok(true)
    }
}

/// Login screen state
#[derive(Debug, Default)]
pub struct LoginScreen {
    /// Currently selected vault index
    pub selected_vault: usize,
    /// Whether we're in password entry mode
    pub entering_password: bool,
    /// Whether we're entering keyfile path for unlock
    pub entering_keyfile_path: bool,
    /// Error message to display
    pub error_message: Option<String>,
    /// Whether to show create vault form
    pub creating_vault: bool,
    /// Current step in create vault flow
    pub create_step: CreateVaultStep,
    /// Vault name being created
    pub new_vault_name: String,
    /// Password for new vault (stored temporarily)
    pub new_vault_password: String,
    /// Selected encryption method for new vault
    pub new_vault_encryption_method: EncryptionMethod,
    /// Whether new vault requires keyfile for unlock
    pub new_vault_use_keyfile: bool,
    /// Keyfile path for create flow (optional until needed)
    pub new_vault_keyfile_path: String,
    /// Number of recovery security questions to configure (0-3)
    pub new_vault_recovery_question_count: u8,
    /// Collected security questions for the new vault
    pub new_vault_recovery_questions: Vec<SecurityQuestionDraft>,
    /// Temporary holder for question text before answer step
    pub pending_recovery_question: Option<String>,
    /// Pending password for keyfile-required unlock flow
    pub pending_unlock_password: Option<SecureString>,
    /// Active forgot-password recovery session
    pub password_recovery: Option<PasswordRecoverySession>,
}

impl LoginScreen {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select next vault in list
    pub fn select_next(&mut self, vault_count: usize) {
        if vault_count > 0 {
            self.selected_vault = (self.selected_vault + 1) % vault_count;
        }
    }

    /// Select previous vault in list
    pub fn select_prev(&mut self, vault_count: usize) {
        if vault_count > 0 {
            self.selected_vault = self
                .selected_vault
                .checked_sub(1)
                .unwrap_or(vault_count - 1);
        }
    }

    /// Enter password mode for selected vault
    pub fn enter_password_mode(&mut self) {
        self.entering_password = true;
        self.entering_keyfile_path = false;
        self.creating_vault = false;
        self.create_step = CreateVaultStep::Name;
        self.pending_unlock_password = None;
        self.password_recovery = None;
        self.error_message = None;
    }

    /// Exit password mode
    pub fn exit_password_mode(&mut self) {
        self.entering_password = false;
        self.entering_keyfile_path = false;
        self.password_recovery = None;
        self.pending_unlock_password = None;
    }

    /// Show error message
    pub fn show_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Reset create-vault draft fields.
    pub fn reset_create_draft(&mut self) {
        self.creating_vault = false;
        self.create_step = CreateVaultStep::Name;
        self.new_vault_name.clear();
        self.new_vault_password.clear();
        self.new_vault_encryption_method = EncryptionMethod::Aes256Gcm;
        self.new_vault_use_keyfile = false;
        self.new_vault_keyfile_path.clear();
        self.new_vault_recovery_question_count = 0;
        self.new_vault_recovery_questions.clear();
        self.pending_recovery_question = None;
    }
}

/// Render the login screen
pub fn render(frame: &mut Frame, state: &mut AppState, theme: &ThemePalette) {
    let area = frame.area();

    // Clear background
    frame.render_widget(Clear, area);

    // Main layout: header, content, buttons (merged with hints)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Header with logo
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Buttons with embedded keyboard hints
        ])
        .split(area);

    render_header(frame, chunks[0], theme);

    // Render content (this needs mutable state for region registration)
    let entering_password = state.login_screen.entering_password;
    let entering_keyfile_path = state.login_screen.entering_keyfile_path;
    let creating_vault = state.login_screen.creating_vault;
    let is_loading = state.ui_state.is_loading();
    render_content(frame, chunks[1], state, theme);

    // Render loading overlay if loading
    if is_loading {
        render_loading_overlay(frame, area, state, theme);
    }

    // Render buttons (now includes keyboard hints in labels)
    render_footer(
        frame,
        chunks[2],
        state,
        entering_password,
        entering_keyfile_path,
        creating_vault,
        theme,
    );
}

/// Render the header with logo/title
fn render_header(frame: &mut Frame, area: Rect, theme: &ThemePalette) {
    let logo = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", icons::ui::VAULT),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "VAULT",
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            "Secure Secret Manager",
            Style::default().fg(theme.fg_muted),
        )),
    ];

    let header = Paragraph::new(logo).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(theme.bg)),
    );

    frame.render_widget(header, area);
}

/// Render the main content area
fn render_content(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &ThemePalette) {
    if state.screen == Screen::PasswordRecovery {
        render_password_recovery_form(frame, area, state, theme);
        return;
    }

    let entering_password = state.login_screen.entering_password;
    let entering_keyfile_path = state.login_screen.entering_keyfile_path;
    let creating_vault = state.login_screen.creating_vault;
    let selected_vault = state.login_screen.selected_vault;

    // Center the content
    let content_width = area.width.min(60);
    let horizontal_padding = (area.width.saturating_sub(content_width)) / 2;

    let centered_area = Rect {
        x: area.x + horizontal_padding,
        y: area.y,
        width: content_width,
        height: area.height,
    };

    if creating_vault {
        render_create_vault_form(frame, centered_area, state, theme);
    } else if entering_keyfile_path {
        render_keyfile_form(frame, centered_area, state, selected_vault, theme);
    } else if entering_password {
        render_password_form(frame, centered_area, state, selected_vault, theme);
    } else {
        render_vault_list(frame, centered_area, state, selected_vault, theme);
    }
}

/// Render the vault selection list
fn render_vault_list(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    selected_vault: usize,
    theme: &ThemePalette,
) {
    let entries = &state.registry.entries;

    if entries.is_empty() {
        // No vaults - show welcome message
        let welcome = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Welcome to Vault!",
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "No vaults found. Press 'n' to create a new vault.",
                Style::default().fg(theme.fg_muted),
            )),
        ];

        let paragraph = Paragraph::new(welcome)
            .alignment(Alignment::Center)
            .block(create_block("Getting Started", theme));

        frame.render_widget(paragraph, area);
    } else {
        // Show vault list
        let items: Vec<ListItem> = entries
            .iter()
            .enumerate()
            .map(|(i, entry)| create_vault_list_item(entry, i == selected_vault, theme))
            .collect();

        let block = create_block("Select Vault", theme);
        let list = List::new(items).block(block.clone()).highlight_style(
            Style::default()
                .bg(theme.selection_bg)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_widget(list, area);

        // Register clickable elements for each vault item
        // Use block.inner() to get the exact inner area after borders and title
        let inner = block.inner(area);

        for (i, _) in entries.iter().enumerate() {
            let item_y = inner.y + i as u16;
            if item_y < inner.y + inner.height {
                // Stay within inner bounds
                state.ui_state.layout_regions.register_clickable(
                    crate::input::mouse::ClickRegion::new(inner.x, item_y, inner.width, 1),
                    crate::input::mouse::ClickableElement::VaultEntry(i),
                );
            }
        }

        // Also register the list region for context
        state.ui_state.register_region(
            crate::input::mouse::UiRegion::List,
            crate::input::mouse::ClickRegion::new(area.x, area.y, area.width, area.height),
        );
    }
}

/// Create a list item for a vault entry
fn create_vault_list_item<'a>(
    entry: &VaultRegistryEntry,
    selected: bool,
    theme: &ThemePalette,
) -> ListItem<'a> {
    let icon = if entry.is_default {
        icons::ui::STAR
    } else {
        icons::ui::VAULT
    };

    let style = if selected {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.fg)
    };

    let selector = if selected { "▸ " } else { "  " };

    let line = Line::from(vec![
        Span::styled(selector.to_string(), style),
        Span::styled(format!("{} ", icon), Style::default().fg(theme.accent)),
        Span::styled(entry.name.clone(), style),
    ]);

    ListItem::new(line)
}

/// Render the password entry form
fn render_password_form(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    selected_vault: usize,
    theme: &ThemePalette,
) {
    let vault_name = state
        .registry
        .entries
        .get(selected_vault)
        .map(|e| e.name.as_str())
        .unwrap_or("Unknown");

    let error_message = state.login_screen.error_message.clone();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Password input
            Constraint::Length(2), // Error message
            Constraint::Min(0),    // Padding
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", icons::ui::VAULT_LOCKED),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            format!("Unlock \"{}\"", vault_name),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(title, chunks[0]);

    // Password input
    let password_display = state.ui_state.input_buffer.display();
    let input = Paragraph::new(password_display)
        .style(Style::default().fg(theme.fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_focused))
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(Span::styled(
                    " Password ",
                    Style::default().fg(theme.accent),
                )),
        );

    frame.render_widget(input, chunks[1]);

    // Render cursor (display() returns masked chars, but cursor position is still correct)
    let cursor_x = chunks[1].x + 1 + state.ui_state.input_buffer.cursor as u16;
    let cursor_y = chunks[1].y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));

    // Error message
    if let Some(ref error) = error_message {
        let error_text = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} ", icons::ui::ERROR),
                Style::default().fg(theme.error),
            ),
            Span::styled(error.clone(), Style::default().fg(theme.error)),
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(error_text, chunks[2]);
    }
}

/// Render the keyfile path entry form
fn render_keyfile_form(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    selected_vault: usize,
    theme: &ThemePalette,
) {
    let vault_name = state
        .registry
        .entries
        .get(selected_vault)
        .map(|e| e.name.as_str())
        .unwrap_or("Unknown");

    let error_message = state.login_screen.error_message.clone();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Keyfile path input
            Constraint::Length(2), // Error message
            Constraint::Min(0),    // Padding
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", icons::item::API_KEY),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            format!("Keyfile for \"{}\"", vault_name),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(title, chunks[0]);

    let input = Paragraph::new(state.ui_state.input_buffer.display())
        .style(Style::default().fg(theme.fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_focused))
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(Span::styled(
                    " Keyfile Path ",
                    Style::default().fg(theme.accent),
                )),
        );

    frame.render_widget(input, chunks[1]);

    let cursor_x = chunks[1].x + 1 + state.ui_state.input_buffer.cursor as u16;
    let cursor_y = chunks[1].y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));

    if let Some(ref error) = error_message {
        let error_text = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} ", icons::ui::ERROR),
                Style::default().fg(theme.error),
            ),
            Span::styled(error.clone(), Style::default().fg(theme.error)),
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(error_text, chunks[2]);
    }
}

/// Render the create vault form (multi-step)
fn render_create_vault_form(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    theme: &ThemePalette,
) {
    let step = state.login_screen.create_step;
    let error_message = state.login_screen.error_message.clone();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(2), // Step indicator
            Constraint::Length(3), // Input field
            Constraint::Length(2), // Error message
            Constraint::Min(0),    // Padding
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", icons::ui::VAULT),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            "Create New Vault",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(title, chunks[0]);

    // Step indicator
    let step_text = match step {
        CreateVaultStep::Name => "Step 1/10: Enter vault name",
        CreateVaultStep::Password => "Step 2/10: Enter password",
        CreateVaultStep::ConfirmPassword => "Step 3/10: Confirm password",
        CreateVaultStep::EncryptionMethod => "Step 4/10: Encryption method",
        CreateVaultStep::KeyfileOption => "Step 5/10: Require keyfile? (yes/no)",
        CreateVaultStep::KeyfilePath => "Step 6/10: Keyfile path",
        CreateVaultStep::RecoveryQuestionCount => "Step 7/10: Recovery question count (0-3)",
        CreateVaultStep::RecoveryQuestion => "Step 8/10: Enter recovery question",
        CreateVaultStep::RecoveryAnswer => "Step 9/10: Enter recovery answer",
        CreateVaultStep::ConfirmCreate => "Step 10/10: Review and create vault",
    };
    let step_indicator =
        Paragraph::new(Span::styled(step_text, Style::default().fg(theme.fg_muted)))
            .alignment(Alignment::Center);
    frame.render_widget(step_indicator, chunks[1]);

    // Input field based on step
    let (field_title, field_value) = match step {
        CreateVaultStep::Name => (" Vault Name ", state.ui_state.input_buffer.text.clone()),
        CreateVaultStep::Password => (" Password ", state.ui_state.input_buffer.display()),
        CreateVaultStep::ConfirmPassword => {
            (" Confirm Password ", state.ui_state.input_buffer.display())
        }
        CreateVaultStep::EncryptionMethod => (
            " Encryption Method ",
            state
                .login_screen
                .new_vault_encryption_method
                .profile_label(),
        ),
        CreateVaultStep::KeyfileOption => (
            " Require Keyfile (yes/no) ",
            state.ui_state.input_buffer.text.clone(),
        ),
        CreateVaultStep::KeyfilePath => {
            (" Keyfile Path ", state.ui_state.input_buffer.text.clone())
        }
        CreateVaultStep::RecoveryQuestionCount => (
            " Recovery Questions (0-3) ",
            state.ui_state.input_buffer.text.clone(),
        ),
        CreateVaultStep::RecoveryQuestion => (
            " Security Question ",
            state.ui_state.input_buffer.text.clone(),
        ),
        CreateVaultStep::RecoveryAnswer => {
            (" Security Answer ", state.ui_state.input_buffer.display())
        }
        CreateVaultStep::ConfirmCreate => (
            " Review ",
            format!(
                "Vault: {} | Keyfile: {} | Recovery Questions: {}",
                state.login_screen.new_vault_name,
                if state.login_screen.new_vault_use_keyfile {
                    "Yes"
                } else {
                    "No"
                },
                state.login_screen.new_vault_recovery_question_count
            ),
        ),
    };

    let input = Paragraph::new(field_value)
        .style(Style::default().fg(theme.fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_focused))
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(Span::styled(field_title, Style::default().fg(theme.accent))),
        );

    frame.render_widget(input, chunks[2]);

    // Render cursor
    if step != CreateVaultStep::EncryptionMethod && step != CreateVaultStep::ConfirmCreate {
        let cursor_x = chunks[2].x + 1 + state.ui_state.input_buffer.cursor as u16;
        let cursor_y = chunks[2].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    // Error message
    if let Some(ref error) = error_message {
        let error_text = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} ", icons::ui::ERROR),
                Style::default().fg(theme.error),
            ),
            Span::styled(error.clone(), Style::default().fg(theme.error)),
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(error_text, chunks[3]);
    }
}

fn render_password_recovery_form(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    theme: &ThemePalette,
) {
    let Some(session) = state.login_screen.password_recovery.as_ref() else {
        let paragraph = Paragraph::new("No active recovery session")
            .alignment(Alignment::Center)
            .block(create_block("Password Recovery", theme));
        frame.render_widget(paragraph, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Length(2), // question progress
            Constraint::Length(3), // answer input
            Constraint::Length(3), // hint/password reveal
            Constraint::Length(2), // attempts
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", icons::ui::VAULT_LOCKED),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            format!("Recover \"{}\"", session.vault_name),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let question_text = if session.is_complete() {
        "Recovery complete".to_string()
    } else if session.is_locked_out() {
        "Recovery locked".to_string()
    } else {
        format!(
            "Question {}/{}: {}",
            session.current_question + 1,
            session.metadata.questions.len(),
            session.current_question_text().unwrap_or("-")
        )
    };
    let question_para = Paragraph::new(question_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.fg_muted));
    frame.render_widget(question_para, chunks[1]);

    let input = Paragraph::new(state.ui_state.input_buffer.display())
        .style(Style::default().fg(theme.fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_focused))
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(Span::styled(" Answer ", Style::default().fg(theme.accent))),
        );
    frame.render_widget(input, chunks[2]);

    if !session.is_complete() && !session.is_locked_out() {
        let cursor_x = chunks[2].x + 1 + state.ui_state.input_buffer.cursor as u16;
        let cursor_y = chunks[2].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    let reveal_text = session
        .recovered_password
        .as_ref()
        .map(|p| format!("Recovered password: {}", p))
        .or_else(|| {
            session
                .latest_hint
                .as_ref()
                .map(|h| format!("Current hint: {}", h))
        })
        .unwrap_or_else(|| "Current hint: ••••••••".to_string());
    let reveal_para = Paragraph::new(reveal_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.fg));
    frame.render_widget(reveal_para, chunks[3]);

    let attempts = format!(
        "Attempts remaining: {} / {}",
        session.remaining_attempts(),
        session.metadata.max_attempts
    );
    let attempts_para = Paragraph::new(attempts)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.warning));
    frame.render_widget(attempts_para, chunks[4]);
}

/// Render the footer with keybinding hints
/// Render action buttons with embedded keyboard hints
fn render_footer(
    frame: &mut Frame,
    area: Rect,
    state: &mut crate::app::AppState,
    entering_password: bool,
    entering_keyfile_path: bool,
    creating_vault: bool,
    theme: &ThemePalette,
) {
    use crate::ui::widgets::{ButtonStyle, render_button_row};

    let buttons = if state.screen == Screen::PasswordRecovery {
        vec![
            (
                "submit-recovery".to_string(),
                "Submit Answer",
                Some("Enter"),
                ButtonStyle::Primary,
            ),
            (
                "back".to_string(),
                "Back",
                Some("Esc"),
                ButtonStyle::Secondary,
            ),
        ]
    } else if creating_vault {
        let step = state.login_screen.create_step;
        let mut btns = vec![];

        // Add back button if not on first step
        if step != CreateVaultStep::Name {
            btns.push((
                "prev-step".to_string(),
                "Back",
                None,
                ButtonStyle::Secondary,
            ));
        }

        btns.push((
            "save-vault".to_string(),
            if step == CreateVaultStep::ConfirmCreate {
                "Create"
            } else {
                "Next"
            },
            Some("Enter"),
            ButtonStyle::Primary,
        ));
        btns.push((
            "cancel".to_string(),
            "Cancel",
            Some("Esc"),
            ButtonStyle::Secondary,
        ));
        btns
    } else if entering_password || entering_keyfile_path {
        let mut btns = vec![
            (
                "unlock".to_string(),
                "Unlock",
                Some("Enter"),
                ButtonStyle::Primary,
            ),
            (
                "back".to_string(),
                "Back",
                Some("Esc"),
                ButtonStyle::Secondary,
            ),
        ];
        if entering_password {
            btns.push((
                "forgot-password".to_string(),
                "Forgot Password",
                Some("f"),
                ButtonStyle::Secondary,
            ));
        }
        btns
    } else {
        vec![
            (
                "select-vault".to_string(),
                "Select",
                Some("Enter"),
                ButtonStyle::Primary,
            ),
            (
                "new-vault".to_string(),
                "New Vault",
                Some("n"),
                ButtonStyle::Secondary,
            ),
            (
                "delete-vault".to_string(),
                "Delete",
                Some("d"),
                ButtonStyle::Danger,
            ),
            (
                "quit".to_string(),
                "Quit",
                Some("q"),
                ButtonStyle::Secondary,
            ),
        ]
    };

    let button_regions = render_button_row(frame, area, &buttons, theme);

    // Register button regions
    for button_region in button_regions {
        state.ui_state.layout_regions.register_clickable(
            button_region.region,
            crate::input::mouse::ClickableElement::Button(button_region.name),
        );
    }
}

/// Create a styled block
fn create_block<'a>(title: &'a str, theme: &ThemePalette) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.bg))
}

/// Render loading overlay
fn render_loading_overlay(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemePalette) {
    // Semi-transparent overlay
    let overlay = Block::default().style(Style::default().bg(theme.bg_alt));
    frame.render_widget(overlay, area);

    // Center the loading message
    let loading_width = 50;
    let loading_height = 5;
    let loading_x = (area.width.saturating_sub(loading_width)) / 2;
    let loading_y = (area.height.saturating_sub(loading_height)) / 2;

    let loading_area = Rect {
        x: area.x + loading_x,
        y: area.y + loading_y,
        width: loading_width,
        height: loading_height,
    };

    // Loading box
    let loading_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg));

    frame.render_widget(Clear, loading_area);
    frame.render_widget(loading_block, loading_area);

    // Loading content
    let inner = loading_area.inner(ratatui::layout::Margin::new(2, 1));

    let spinner = state.ui_state.spinner_char();
    let message = state
        .ui_state
        .loading_message
        .as_deref()
        .unwrap_or("Loading...");

    let loading_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(message, Style::default().fg(theme.fg)),
        ]),
    ];

    let loading_para = Paragraph::new(loading_text).alignment(Alignment::Center);

    frame.render_widget(loading_para, inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_screen_navigation() {
        let mut screen = LoginScreen::new();

        screen.select_next(3);
        assert_eq!(screen.selected_vault, 1);

        screen.select_next(3);
        assert_eq!(screen.selected_vault, 2);

        screen.select_next(3);
        assert_eq!(screen.selected_vault, 0); // Wrap around

        screen.select_prev(3);
        assert_eq!(screen.selected_vault, 2); // Wrap around backwards
    }

    #[test]
    fn test_password_mode() {
        let mut screen = LoginScreen::new();

        assert!(!screen.entering_password);

        screen.enter_password_mode();
        assert!(screen.entering_password);

        screen.exit_password_mode();
        assert!(!screen.entering_password);
    }

    #[test]
    fn test_error_handling() {
        let mut screen = LoginScreen::new();

        screen.show_error("Invalid password");
        assert_eq!(screen.error_message, Some("Invalid password".to_string()));

        screen.clear_error();
        assert!(screen.error_message.is_none());
    }
}
