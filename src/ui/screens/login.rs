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
use crate::crypto::SecureString;
use crate::domain::RecoveryMetadata;
use crate::storage::VaultRegistryEntry;
use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// Wizard step for creating a vault
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreateVaultStep {
    #[default]
    Step1,
    Step2,
    Step3,
}

/// Form field index for the Create Vault form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreateVaultField {
    #[default]
    Name,
    EncryptionMethod,
    Password,
    ConfirmPassword,
    UseKeyfile,
    KeyfilePath,
    RecoveryQuestionsCount,
    RecoveryQuestion1,
    RecoveryAnswer1,
    RecoveryQuestion2,
    RecoveryAnswer2,
    RecoveryQuestion3,
    RecoveryAnswer3,
}

impl CreateVaultField {
    pub fn next(self, step: CreateVaultStep, q_count: usize, use_keyfile: bool) -> Self {
        match step {
            CreateVaultStep::Step1 => match self {
                Self::Name => Self::EncryptionMethod,
                Self::EncryptionMethod => Self::Name,
                _ => Self::Name,
            },
            CreateVaultStep::Step2 => match self {
                Self::Password => Self::ConfirmPassword,
                Self::ConfirmPassword => Self::UseKeyfile,
                Self::UseKeyfile => {
                    if use_keyfile {
                        Self::KeyfilePath
                    } else {
                        Self::Password
                    }
                }
                Self::KeyfilePath => Self::Password,
                _ => Self::Password,
            },
            CreateVaultStep::Step3 => match self {
                Self::RecoveryQuestionsCount => {
                    if q_count > 0 {
                        Self::RecoveryQuestion1
                    } else {
                        Self::RecoveryQuestionsCount
                    }
                }
                Self::RecoveryQuestion1 => Self::RecoveryAnswer1,
                Self::RecoveryAnswer1 => {
                    if q_count > 1 {
                        Self::RecoveryQuestion2
                    } else {
                        Self::RecoveryQuestionsCount
                    }
                }
                Self::RecoveryQuestion2 => Self::RecoveryAnswer2,
                Self::RecoveryAnswer2 => {
                    if q_count > 2 {
                        Self::RecoveryQuestion3
                    } else {
                        Self::RecoveryQuestionsCount
                    }
                }
                Self::RecoveryQuestion3 => Self::RecoveryAnswer3,
                Self::RecoveryAnswer3 => Self::RecoveryQuestionsCount,
                _ => Self::RecoveryQuestionsCount,
            },
        }
    }

    pub fn prev(self, step: CreateVaultStep, q_count: usize, use_keyfile: bool) -> Self {
        match step {
            CreateVaultStep::Step1 => match self {
                Self::EncryptionMethod => Self::Name,
                Self::Name => Self::EncryptionMethod,
                _ => Self::Name,
            },
            CreateVaultStep::Step2 => match self {
                Self::Password => {
                    if use_keyfile {
                        Self::KeyfilePath
                    } else {
                        Self::UseKeyfile
                    }
                }
                Self::ConfirmPassword => Self::Password,
                Self::UseKeyfile => Self::ConfirmPassword,
                Self::KeyfilePath => Self::UseKeyfile,
                _ => Self::Password,
            },
            CreateVaultStep::Step3 => match self {
                Self::RecoveryQuestionsCount => {
                    if q_count > 2 {
                        Self::RecoveryAnswer3
                    } else if q_count > 1 {
                        Self::RecoveryAnswer2
                    } else if q_count > 0 {
                        Self::RecoveryAnswer1
                    } else {
                        Self::RecoveryQuestionsCount
                    }
                }
                Self::RecoveryQuestion1 => Self::RecoveryQuestionsCount,
                Self::RecoveryAnswer1 => Self::RecoveryQuestion1,
                Self::RecoveryQuestion2 => Self::RecoveryAnswer1,
                Self::RecoveryAnswer2 => Self::RecoveryQuestion2,
                Self::RecoveryQuestion3 => Self::RecoveryAnswer2,
                Self::RecoveryAnswer3 => Self::RecoveryQuestion3,
                _ => Self::RecoveryQuestionsCount,
            },
        }
    }
}

/// State for the create vault form
#[derive(Debug, Clone, Default)]
pub struct CreateVaultFormState {
    pub step: CreateVaultStep,
    pub focused_field: CreateVaultField,
    pub encryption_method: crate::crypto::EncryptionMethod,
    pub name: crate::app::state::InputBuffer,
    pub password: crate::app::state::InputBuffer,
    pub confirm_password: crate::app::state::InputBuffer,
    pub use_keyfile: crate::app::state::InputBuffer,
    pub keyfile_path: crate::app::state::InputBuffer,
    pub recovery_questions_count: usize,
    pub question1: crate::app::state::InputBuffer,
    pub answer1: crate::app::state::InputBuffer,
    pub question2: crate::app::state::InputBuffer,
    pub answer2: crate::app::state::InputBuffer,
    pub question3: crate::app::state::InputBuffer,
    pub answer3: crate::app::state::InputBuffer,

    /// Offset to track scrolling in long steps (e.g. Step 3)
    pub scroll_offset: u16,
}

impl CreateVaultFormState {
    pub fn new() -> Self {
        let mut state = Self::default();
        state.password.masked = true;
        state.confirm_password.masked = true;
        state.answer1.masked = true;
        state.answer2.masked = true;
        state.answer3.masked = true;
        state
    }

    pub fn active_input_mut(&mut self) -> Option<&mut crate::app::state::InputBuffer> {
        match self.focused_field {
            CreateVaultField::Name => Some(&mut self.name),
            CreateVaultField::Password => Some(&mut self.password),
            CreateVaultField::ConfirmPassword => Some(&mut self.confirm_password),
            CreateVaultField::UseKeyfile => Some(&mut self.use_keyfile),
            CreateVaultField::KeyfilePath => Some(&mut self.keyfile_path),
            CreateVaultField::RecoveryQuestion1 => Some(&mut self.question1),
            CreateVaultField::RecoveryAnswer1 => Some(&mut self.answer1),
            CreateVaultField::RecoveryQuestion2 => Some(&mut self.question2),
            CreateVaultField::RecoveryAnswer2 => Some(&mut self.answer2),
            CreateVaultField::RecoveryQuestion3 => Some(&mut self.question3),
            CreateVaultField::RecoveryAnswer3 => Some(&mut self.answer3),
            _ => None,
        }
    }

    pub fn active_input(&self) -> Option<&crate::app::state::InputBuffer> {
        match self.focused_field {
            CreateVaultField::Name => Some(&self.name),
            CreateVaultField::Password => Some(&self.password),
            CreateVaultField::ConfirmPassword => Some(&self.confirm_password),
            CreateVaultField::UseKeyfile => Some(&self.use_keyfile),
            CreateVaultField::KeyfilePath => Some(&self.keyfile_path),
            CreateVaultField::RecoveryQuestion1 => Some(&self.question1),
            CreateVaultField::RecoveryAnswer1 => Some(&self.answer1),
            CreateVaultField::RecoveryQuestion2 => Some(&self.question2),
            CreateVaultField::RecoveryAnswer2 => Some(&self.answer2),
            CreateVaultField::RecoveryQuestion3 => Some(&self.question3),
            CreateVaultField::RecoveryAnswer3 => Some(&self.answer3),
            _ => None,
        }
    }
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
#[derive(Debug)]
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
    /// Form state for creating a new vault
    pub create_vault_form: CreateVaultFormState,
    /// Pending password for keyfile-required unlock flow
    pub pending_unlock_password: Option<SecureString>,
    /// Active forgot-password recovery session
    pub password_recovery: Option<PasswordRecoverySession>,
}

impl Default for LoginScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl LoginScreen {
    pub fn new() -> Self {
        let mut screen = Self {
            selected_vault: 0,
            entering_password: false,
            entering_keyfile_path: false,
            error_message: None,
            creating_vault: false,
            create_vault_form: CreateVaultFormState::new(),
            pending_unlock_password: None,
            password_recovery: None,
        };
        screen.reset_create_form();
        screen
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
    pub fn reset_create_form(&mut self) {
        self.creating_vault = false;
        self.create_vault_form = CreateVaultFormState::new();
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

    // Center the content (use full area for modals, logic moved inside specific render functions)
    let content_width = area.width.min(60);
    let horizontal_padding = (area.width.saturating_sub(content_width)) / 2;

    let centered_area = Rect {
        x: area.x + horizontal_padding,
        y: area.y,
        width: content_width,
        height: area.height,
    };

    if creating_vault {
        // use full area so the modal can center itself on the screen
        render_create_vault_form(frame, area, state, theme);
    } else if entering_keyfile_path {
        // use full area so the modal can center itself on the screen
        render_keyfile_form(frame, area, state, selected_vault, theme);
    } else if entering_password {
        // use full area so the modal can center itself on the screen
        render_password_form(frame, area, state, selected_vault, theme);
    } else {
        // The list view is okay using just centered column area
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

    // Calculate a centered, fixed-size area for the modal
    let modal_width = 50;
    let modal_height = 10;
    let modal_area = Rect {
        x: area.x + (area.width.saturating_sub(modal_width)) / 2,
        y: area.y + (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // Render an outer block for the modal
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(Clear, modal_area);
    frame.render_widget(modal_block.clone(), modal_area);

    let inner_area = modal_block.inner(modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title padding
            Constraint::Length(3), // Password input
            Constraint::Length(2), // Error message
            Constraint::Min(0),    // Padding
        ])
        .split(inner_area);

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

    // Calculate a centered, fixed-size area for the modal
    let modal_width = 50;
    let modal_height = 10;
    let modal_area = Rect {
        x: area.x + (area.width.saturating_sub(modal_width)) / 2,
        y: area.y + (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // Render an outer block for the modal
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(Clear, modal_area);
    frame.render_widget(modal_block.clone(), modal_area);

    let inner_area = modal_block.inner(modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title padding
            Constraint::Length(3), // Keyfile path input
            Constraint::Length(2), // Error message
            Constraint::Min(0),    // Padding
        ])
        .split(inner_area);

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
    // We cannot mutably borrow state here if it is going to violate borrow rules,
    // but the function signature accepts `state: &mut AppState`.
    // Let's defer any mutation until the end, or clone what we need.
    // Actually, Ratatui render functions should generally take `&AppState`.
    // Wait, the signature is `state: &mut AppState`. Let's just fix the logic
    // for calculating offset and stop mutating `form.scroll_offset` inside render,
    // or keep it but be careful.
    let error_message = state.login_screen.error_message.clone();
    let form = &mut state.login_screen.create_vault_form;

    let mut fields_to_show = Vec::new();

    // Determine what to show based on current step
    match form.step {
        CreateVaultStep::Step1 => {
            fields_to_show.push(("Vault Name", Some(&form.name), CreateVaultField::Name));
            // For encryption method we'll handle rendering slightly differently, but add it to fields
            fields_to_show.push((
                "Encryption Method",
                None,
                CreateVaultField::EncryptionMethod,
            ));
        }
        CreateVaultStep::Step2 => {
            fields_to_show.push(("Password", Some(&form.password), CreateVaultField::Password));
            fields_to_show.push((
                "Confirm Password",
                Some(&form.confirm_password),
                CreateVaultField::ConfirmPassword,
            ));
            fields_to_show.push((
                "Use Keyfile (y/n)",
                Some(&form.use_keyfile),
                CreateVaultField::UseKeyfile,
            ));

            let use_keyfile_text = form.use_keyfile.text.trim().to_lowercase();
            if use_keyfile_text == "yes" || use_keyfile_text == "y" {
                fields_to_show.push((
                    "Keyfile Path",
                    Some(&form.keyfile_path),
                    CreateVaultField::KeyfilePath,
                ));
            }
        }
        CreateVaultStep::Step3 => {
            fields_to_show.push((
                "Number of Recovery Questions (0-3)",
                None,
                CreateVaultField::RecoveryQuestionsCount,
            ));

            let q_count = form.recovery_questions_count;

            if q_count > 0 {
                fields_to_show.push((
                    "Recovery Question 1",
                    Some(&form.question1),
                    CreateVaultField::RecoveryQuestion1,
                ));
                fields_to_show.push((
                    "Recovery Answer 1",
                    Some(&form.answer1),
                    CreateVaultField::RecoveryAnswer1,
                ));
            }
            if q_count > 1 {
                fields_to_show.push((
                    "Recovery Question 2",
                    Some(&form.question2),
                    CreateVaultField::RecoveryQuestion2,
                ));
                fields_to_show.push((
                    "Recovery Answer 2",
                    Some(&form.answer2),
                    CreateVaultField::RecoveryAnswer2,
                ));
            }
            if q_count > 2 {
                fields_to_show.push((
                    "Recovery Question 3",
                    Some(&form.question3),
                    CreateVaultField::RecoveryQuestion3,
                ));
                fields_to_show.push((
                    "Recovery Answer 3",
                    Some(&form.answer3),
                    CreateVaultField::RecoveryAnswer3,
                ));
            }
        }
    }

    // Update scroll offset to keep focused field in view
    let focused_idx = fields_to_show
        .iter()
        .position(|(_, _, field_enum)| form.focused_field == *field_enum);

    // Total fields + nav + error padding
    let _total_rows = fields_to_show.len() + 2;

    // Calculate form dimensions
    let form_width = area.width.min(70);

    // Precise height calculation
    let mut total_required_height = 0;
    for (_, _, field_enum) in &fields_to_show {
        if *field_enum == CreateVaultField::EncryptionMethod {
            total_required_height += 4;
        } else {
            total_required_height += 3;
        }
    }
    // Add space for nav row (1), error row (1), padding and borders
    total_required_height += 4;

    let max_height = area.height.saturating_sub(2);
    let form_height = total_required_height.min(max_height);
    let x = (area.width.saturating_sub(form_width)) / 2;
    let y = (area.height.saturating_sub(form_height)) / 2;

    let form_area = Rect::new(x, y, form_width, form_height);

    // Check if we need scrolling
    let mut max_visible_rows: usize = fields_to_show.len();

    if total_required_height <= max_height {
        form.scroll_offset = 0;
    } else {
        // Find how many fields can actually fit in the available space
        let available_field_height = form_area.height.saturating_sub(2); // subtract borders

        let mut current_h = 0;
        max_visible_rows = 0;
        // Start counting from current offset
        let mut i = form.scroll_offset as usize;
        while i < fields_to_show.len() {
            let h = match fields_to_show[i].2 {
                CreateVaultField::EncryptionMethod => 4,
                _ => 3,
            };
            if current_h + h <= available_field_height {
                current_h += h;
                max_visible_rows += 1;
                i += 1;
            } else {
                break;
            }
        }

        if let Some(idx) = focused_idx {
            if idx < form.scroll_offset as usize {
                // Scroll up
                form.scroll_offset = idx as u16;
            } else if idx >= (form.scroll_offset as usize + max_visible_rows) {
                // Scroll down
                // We need to calculate the offset such that the focused index is at the bottom of the visible list
                let mut reverse_h = 0;
                let mut new_offset = idx;

                while new_offset > 0 {
                    let h = match fields_to_show[new_offset].2 {
                        CreateVaultField::EncryptionMethod => 4,
                        _ => 3,
                    };
                    if reverse_h + h <= available_field_height {
                        reverse_h += h;
                        new_offset -= 1;
                    } else {
                        break;
                    }
                }
                form.scroll_offset = (new_offset + 1) as u16;
            }
        }

        // Bound scroll offset to prevent scrolling past the end
        let max_scroll = fields_to_show.len().saturating_sub(max_visible_rows) as u16;
        form.scroll_offset = form.scroll_offset.min(max_scroll);
    }

    // Clear background
    frame.render_widget(Clear, form_area);

    let step_str = match form.step {
        CreateVaultStep::Step1 => "Step 1/3",
        CreateVaultStep::Step2 => "Step 2/3",
        CreateVaultStep::Step3 => "Step 3/3",
    };

    let title = format!(" {} Create Vault [{}] ", icons::ui::VAULT, step_str);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.bg));

    frame.render_widget(block.clone(), form_area);
    let inner_area = block.inner(form_area);

    // Apply scroll offset to determine visible items
    let mut constraints = vec![];
    let start_idx = form.scroll_offset as usize;
    let max_visible_fields = max_visible_rows.min(fields_to_show.len());
    let end_idx = (start_idx + max_visible_fields).min(fields_to_show.len());

    for i in start_idx..end_idx {
        let field = fields_to_show[i].2;
        if field == CreateVaultField::EncryptionMethod {
            constraints.push(Constraint::Length(4));
        } else {
            constraints.push(Constraint::Length(3));
        }
    }

    // Fill empty space if we have less fields than required
    constraints.push(Constraint::Min(0));

    // Always add Nav row and Error row at the end
    constraints.push(Constraint::Length(1)); // Nav row
    constraints.push(Constraint::Length(2)); // Error padding

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    let visible_fields = fields_to_show
        .iter()
        .enumerate()
        .skip(start_idx)
        .take(max_visible_fields);

    let mut current_layout_idx = 0;

    for (_, (label, buffer_opt, field_enum)) in visible_fields {
        let is_focused = form.focused_field == *field_enum;
        let border_style = if is_focused {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.border)
        };

        let title_style = if is_focused {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg_muted)
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(if is_focused {
                ratatui::widgets::BorderType::Double
            } else {
                ratatui::widgets::BorderType::Plain
            })
            .border_style(border_style)
            .title(Span::styled(format!(" {} ", label), title_style));

        if let Some(buffer) = buffer_opt {
            let input_para = Paragraph::new(buffer.display())
                .style(Style::default().fg(theme.fg))
                .block(input_block.clone());
            frame.render_widget(input_para, layout[current_layout_idx]);

            if is_focused {
                let cursor_x = layout[current_layout_idx].x + 1 + buffer.cursor as u16;
                let cursor_y = layout[current_layout_idx].y + 1;
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        } else if *field_enum == CreateVaultField::EncryptionMethod {
            let method_text = if is_focused {
                format!("< {} >", form.encryption_method.display_name())
            } else {
                form.encryption_method.display_name().to_string()
            };

            let desc_text = format!(
                "Security: {} | Speed: {}",
                form.encryption_method.security_level(),
                form.encryption_method.decryption_speed()
            );

            let method_para = Paragraph::new(vec![
                Line::from(method_text).alignment(Alignment::Center),
                Line::from(Span::styled(
                    desc_text,
                    Style::default()
                        .fg(theme.fg_muted)
                        .add_modifier(Modifier::ITALIC),
                ))
                .alignment(Alignment::Center),
            ])
            .style(if is_focused {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            })
            .block(input_block.clone());

            frame.render_widget(method_para, layout[current_layout_idx]);
        } else if *field_enum == CreateVaultField::RecoveryQuestionsCount {
            let desc_para = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Security questions provide a fallback method to recover the password if forgotten.",
                    Style::default().fg(theme.fg_muted).add_modifier(Modifier::ITALIC),
                )),
                Line::from(Span::styled(
                    "Note: Answers must be entered exactly as provided during recovery.",
                    Style::default().fg(theme.warning).add_modifier(Modifier::ITALIC),
                ))
            ]);
            let area = layout[current_layout_idx];
            let desc_area = Rect::new(area.x, area.y, area.width, 2);
            let input_area = Rect::new(area.x, area.y + 2, area.width, 3);

            frame.render_widget(desc_para, desc_area);

            let count_text = if is_focused {
                format!("< {} >", form.recovery_questions_count)
            } else {
                form.recovery_questions_count.to_string()
            };

            let count_para = Paragraph::new(count_text)
                .alignment(Alignment::Center)
                .style(if is_focused {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                })
                .block(input_block.clone());
            frame.render_widget(count_para, input_area);
        }

        let field_idx = match field_enum {
            CreateVaultField::Name => 0,
            CreateVaultField::EncryptionMethod => 1,
            CreateVaultField::Password => 2,
            CreateVaultField::ConfirmPassword => 3,
            CreateVaultField::UseKeyfile => 4,
            CreateVaultField::KeyfilePath => 5,
            CreateVaultField::RecoveryQuestionsCount => 6,
            CreateVaultField::RecoveryQuestion1 => 7,
            CreateVaultField::RecoveryAnswer1 => 8,
            CreateVaultField::RecoveryQuestion2 => 9,
            CreateVaultField::RecoveryAnswer2 => 10,
            CreateVaultField::RecoveryQuestion3 => 11,
            CreateVaultField::RecoveryAnswer3 => 12,
        };
        state.ui_state.layout_regions.register_clickable(
            crate::input::mouse::ClickRegion::new(
                layout[current_layout_idx].x,
                layout[current_layout_idx].y,
                layout[current_layout_idx].width,
                layout[current_layout_idx].height,
            ),
            crate::input::mouse::ClickableElement::FormField(field_idx),
        );
        current_layout_idx += 1;
    }

    let error_layout_idx = layout.len() - 1;

    // Render error message
    if let Some(ref error) = error_message {
        let error_rect = layout[error_layout_idx];
        let error_text = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(
                format!("{} ", icons::ui::ERROR),
                ratatui::style::Style::default().fg(theme.error),
            ),
            ratatui::text::Span::styled(
                error.clone(),
                ratatui::style::Style::default().fg(theme.error),
            ),
        ]))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(error_text, error_rect);
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

    // Calculate a centered, fixed-size area for the modal
    let modal_width = 60;
    let modal_height = 16;
    let modal_area = Rect {
        x: area.x + (area.width.saturating_sub(modal_width)) / 2,
        y: area.y + (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // Render an outer block for the modal
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(Clear, modal_area);
    frame.render_widget(modal_block.clone(), modal_area);

    let inner_area = modal_block.inner(modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // title padding
            Constraint::Length(2), // question progress
            Constraint::Length(3), // answer input
            Constraint::Length(3), // hint/password reveal
            Constraint::Length(2), // attempts
            Constraint::Min(0),
        ])
        .split(inner_area);

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
        let mut btns = vec![];

        let form = &state.login_screen.create_vault_form;

        let primary_label = if form.step == CreateVaultStep::Step3
            && form.focused_field == CreateVaultField::RecoveryAnswer3
        {
            "Create"
        } else if form.step == CreateVaultStep::Step3
            && form.focused_field == CreateVaultField::RecoveryQuestionsCount {
                let q_count = form.recovery_questions_count;
                if q_count == 0 {
                    "Create"
                } else {
                    "Next"
                }
        } else if form.step == CreateVaultStep::Step3
            && form.focused_field == CreateVaultField::RecoveryAnswer1 {
                let q_count = form.recovery_questions_count;
                if q_count == 1 {
                    "Create"
                } else {
                    "Next"
                }
        } else if form.step == CreateVaultStep::Step3
            && form.focused_field == CreateVaultField::RecoveryAnswer2 {
                let q_count = form.recovery_questions_count;
                if q_count == 2 {
                    "Create"
                } else {
                    "Next"
                }
        } else {
            "Next"
        };

        let secondary_label = if form.step == CreateVaultStep::Step1 {
            "Cancel"
        } else {
            "Back"
        };

        btns.push((
            "save-vault".to_string(),
            primary_label,
            Some("Enter"),
            ButtonStyle::Primary,
        ));
        btns.push((
            "cancel".to_string(),
            secondary_label,
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
