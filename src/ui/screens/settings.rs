//! Settings screen
//!
//! Allows users to configure application settings like theme, auto-lock, etc.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::app::AppState;
use crate::storage::ThemeChoice;
use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// Step state for change-password workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangePasswordStep {
    CurrentPassword,
    KeyfilePath,
    NewPassword,
    ConfirmPassword,
}

/// Change-password workflow state.
#[derive(Debug, Clone)]
pub struct ChangePasswordAction {
    pub step: ChangePasswordStep,
    pub current_password: Option<String>,
    pub keyfile_path: String,
    pub keyfile_data: Option<Vec<u8>>,
    pub new_password: Option<String>,
}

impl Default for ChangePasswordAction {
    fn default() -> Self {
        Self {
            step: ChangePasswordStep::CurrentPassword,
            current_password: None,
            keyfile_path: String::new(),
            keyfile_data: None,
            new_password: None,
        }
    }
}

/// Step state for recovery setup workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverySetupStep {
    CurrentPassword,
    KeyfilePath,
    QuestionCount,
    QuestionText,
    AnswerText,
}

/// Recovery question draft for settings workflow.
#[derive(Debug, Clone, Default)]
pub struct RecoveryQuestionDraft {
    pub question: String,
    pub answer: String,
}

/// Recovery setup workflow state.
#[derive(Debug, Clone)]
pub struct RecoverySetupAction {
    pub step: RecoverySetupStep,
    pub current_password: Option<String>,
    pub keyfile_path: String,
    pub keyfile_data: Option<Vec<u8>>,
    pub question_count: u8,
    pub questions: Vec<RecoveryQuestionDraft>,
    pub pending_question: Option<String>,
}

impl Default for RecoverySetupAction {
    fn default() -> Self {
        Self {
            step: RecoverySetupStep::CurrentPassword,
            current_password: None,
            keyfile_path: String::new(),
            keyfile_data: None,
            question_count: 0,
            questions: Vec::new(),
            pending_question: None,
        }
    }
}

/// Active security action workflow from settings.
#[derive(Debug, Clone)]
pub enum SecurityActionState {
    ChangePassword(ChangePasswordAction),
    ConfigureRecovery(RecoverySetupAction),
}

/// Settings screen state
#[derive(Debug, Default)]
pub struct SettingsScreen {
    /// Currently selected setting index
    pub selected: usize,
    /// Whether editing the selected setting
    pub editing: bool,
    /// Sub-selection for lists (like theme chooser)
    pub sub_selection: usize,
    /// Active security workflow (if any)
    pub security_action: Option<SecurityActionState>,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self::default()
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.editing {
            self.sub_selection = self.sub_selection.saturating_sub(1);
        } else {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, max_items: usize, max_sub_items: usize) {
        if self.editing {
            if self.sub_selection < max_sub_items.saturating_sub(1) {
                self.sub_selection += 1;
            }
        } else if self.selected < max_items.saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Enter edit mode
    pub fn start_edit(&mut self, current_sub_index: usize) {
        self.editing = true;
        self.sub_selection = current_sub_index;
    }

    /// Exit edit mode
    pub fn cancel_edit(&mut self) {
        self.editing = false;
    }

    /// Confirm selection and exit edit mode
    pub fn confirm_edit(&mut self) -> usize {
        self.editing = false;
        self.sub_selection
    }
}

/// Setting item types
#[derive(Debug, Clone)]
pub enum SettingKind {
    Theme,
    AutoLock,
    AutoLockTimeout,
    ClipboardTimeout,
    ShowIcons,
    MouseEnabled,
    ChangeMasterPassword,
    ConfigureRecovery,
}

impl SettingKind {
    pub fn all() -> &'static [SettingKind] {
        &[
            SettingKind::Theme,
            SettingKind::AutoLock,
            SettingKind::AutoLockTimeout,
            SettingKind::ClipboardTimeout,
            SettingKind::ShowIcons,
            SettingKind::MouseEnabled,
            SettingKind::ChangeMasterPassword,
            SettingKind::ConfigureRecovery,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingKind::Theme => "Theme",
            SettingKind::AutoLock => "Auto-Lock",
            SettingKind::AutoLockTimeout => "Auto-Lock Timeout",
            SettingKind::ClipboardTimeout => "Clipboard Timeout",
            SettingKind::ShowIcons => "Show Icons",
            SettingKind::MouseEnabled => "Mouse Support",
            SettingKind::ChangeMasterPassword => "Change Master Password",
            SettingKind::ConfigureRecovery => "Configure Recovery",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SettingKind::Theme => "󰏘",
            SettingKind::AutoLock => icons::ui::VAULT_LOCKED,
            SettingKind::AutoLockTimeout => icons::ui::CLOCK,
            SettingKind::ClipboardTimeout => icons::ui::COPY,
            SettingKind::ShowIcons => "",
            SettingKind::MouseEnabled => "󰍽",
            SettingKind::ChangeMasterPassword => "󰌾",
            SettingKind::ConfigureRecovery => "󱞁",
        }
    }
}

/// Render the settings screen
pub fn render(
    frame: &mut Frame,
    state: &AppState,
    screen_state: &SettingsScreen,
    theme: &ThemePalette,
) {
    let area = frame.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Settings
            Constraint::Length(2), // Footer
        ])
        .margin(1)
        .split(area);

    // Header
    render_header(frame, chunks[0], theme);

    // Settings list
    render_settings_list(frame, chunks[1], state, screen_state, theme);

    // Footer hints
    render_footer(frame, chunks[2], screen_state, theme);

    // Security action workflow popup takes priority
    if screen_state.security_action.is_some() {
        render_security_action_popup(frame, area, state, screen_state, theme);
        return;
    }

    // If editing, show selection popup
    if screen_state.editing {
        render_edit_popup(frame, area, state, screen_state, theme);
    }
}

fn render_header(frame: &mut Frame, area: Rect, theme: &ThemePalette) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", icons::ui::SETTINGS),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Settings",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.border)),
    );

    frame.render_widget(header, area);
}

fn render_settings_list(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    screen_state: &SettingsScreen,
    theme: &ThemePalette,
) {
    let settings = SettingKind::all();

    let items: Vec<ListItem> = settings
        .iter()
        .enumerate()
        .map(|(i, setting)| {
            let value = get_setting_value(state, setting);
            let selected = i == screen_state.selected && !screen_state.editing;

            let style = if selected {
                Style::default()
                    .fg(theme.selection_fg)
                    .bg(theme.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            let line = Line::from(vec![
                Span::styled(format!(" {} ", setting.icon()), style.fg(theme.accent)),
                Span::styled(format!("{:20}", setting.label()), style),
                Span::styled(value, style.fg(theme.fg_muted)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(" Options ")
            .title_style(Style::default().fg(theme.fg_muted)),
    );

    let mut list_state = ListState::default();
    list_state.select(Some(screen_state.selected));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_footer(
    frame: &mut Frame,
    area: Rect,
    screen_state: &SettingsScreen,
    theme: &ThemePalette,
) {
    let hints = if screen_state.security_action.is_some() {
        "Type input  Enter: Next/Save  Esc: Cancel"
    } else if screen_state.editing {
        "j/k: Select  Enter: Confirm  Esc: Cancel"
    } else {
        "j/k: Navigate  Enter: Edit  Esc: Back"
    };

    let footer = Paragraph::new(hints)
        .style(Style::default().fg(theme.fg_muted))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

fn render_edit_popup(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    screen_state: &SettingsScreen,
    theme: &ThemePalette,
) {
    let settings = SettingKind::all();
    let setting = &settings[screen_state.selected];

    let options = get_setting_options(state, setting);
    if options.is_empty() {
        return;
    }

    // Calculate popup size
    let max_width = options.iter().map(|s| s.len()).max().unwrap_or(20) + 6;
    let width = (max_width as u16).min(area.width.saturating_sub(4));
    let height = (options.len() as u16 + 2).min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let popup_area = Rect::new(x, y, width, height);

    // Clear background
    frame.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let selected = i == screen_state.sub_selection;
            let style = if selected {
                Style::default()
                    .fg(theme.selection_fg)
                    .bg(theme.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            let prefix = if selected { " › " } else { "   " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style.fg(theme.accent)),
                Span::styled(opt.clone(), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.accent))
            .title(format!(" {} ", setting.label()))
            .title_style(Style::default().fg(theme.accent))
            .style(Style::default().bg(theme.bg)),
    );

    let mut list_state = ListState::default();
    list_state.select(Some(screen_state.sub_selection));

    frame.render_stateful_widget(list, popup_area, &mut list_state);
}

fn get_setting_value(state: &AppState, setting: &SettingKind) -> String {
    match setting {
        SettingKind::Theme => state.config.theme.display_name().to_string(),
        SettingKind::AutoLock => {
            if state.config.auto_lock_enabled {
                "Enabled".to_string()
            } else {
                "Disabled".to_string()
            }
        }
        SettingKind::AutoLockTimeout => format!("{}s", state.config.auto_lock_timeout_secs),
        SettingKind::ClipboardTimeout => format!("{}s", state.config.clipboard_timeout_secs),
        SettingKind::ShowIcons => {
            if state.config.show_icons {
                "Yes".to_string()
            } else {
                "No".to_string()
            }
        }
        SettingKind::MouseEnabled => {
            if state.config.mouse_enabled {
                "Enabled".to_string()
            } else {
                "Disabled".to_string()
            }
        }
        SettingKind::ChangeMasterPassword => "Action".to_string(),
        SettingKind::ConfigureRecovery => "Action".to_string(),
    }
}

fn get_setting_options(_state: &AppState, setting: &SettingKind) -> Vec<String> {
    match setting {
        SettingKind::Theme => ThemeChoice::all()
            .iter()
            .map(|t| t.display_name().to_string())
            .collect(),
        SettingKind::AutoLock | SettingKind::ShowIcons | SettingKind::MouseEnabled => {
            vec!["Enabled".to_string(), "Disabled".to_string()]
        }
        SettingKind::AutoLockTimeout => vec![
            "60s".to_string(),
            "120s".to_string(),
            "300s".to_string(),
            "600s".to_string(),
            "1800s".to_string(),
        ],
        SettingKind::ClipboardTimeout => vec![
            "10s".to_string(),
            "30s".to_string(),
            "60s".to_string(),
            "120s".to_string(),
            "Never".to_string(),
        ],
        SettingKind::ChangeMasterPassword | SettingKind::ConfigureRecovery => vec![],
    }
}

/// Get the current sub-selection index for a setting
pub fn get_current_sub_index(state: &AppState, setting_index: usize) -> usize {
    let settings = SettingKind::all();
    if setting_index >= settings.len() {
        return 0;
    }

    match &settings[setting_index] {
        SettingKind::Theme => ThemeChoice::all()
            .iter()
            .position(|t| *t == state.config.theme)
            .unwrap_or(0),
        SettingKind::AutoLock => {
            if state.config.auto_lock_enabled {
                0
            } else {
                1
            }
        }
        SettingKind::ShowIcons => {
            if state.config.show_icons {
                0
            } else {
                1
            }
        }
        SettingKind::MouseEnabled => {
            if state.config.mouse_enabled {
                0
            } else {
                1
            }
        }
        SettingKind::AutoLockTimeout => match state.config.auto_lock_timeout_secs {
            60 => 0,
            120 => 1,
            300 => 2,
            600 => 3,
            _ => 4,
        },
        SettingKind::ClipboardTimeout => match state.config.clipboard_timeout_secs {
            10 => 0,
            30 => 1,
            60 => 2,
            120 => 3,
            _ => 4,
        },
        SettingKind::ChangeMasterPassword | SettingKind::ConfigureRecovery => 0,
    }
}

/// Apply a setting change
pub fn apply_setting(state: &mut AppState, setting_index: usize, option_index: usize) {
    let settings = SettingKind::all();
    if setting_index >= settings.len() {
        return;
    }

    match &settings[setting_index] {
        SettingKind::Theme => {
            if let Some(theme) = ThemeChoice::all().get(option_index) {
                state.config.theme = *theme;
            }
        }
        SettingKind::AutoLock => {
            state.config.auto_lock_enabled = option_index == 0;
        }
        SettingKind::ShowIcons => {
            state.config.show_icons = option_index == 0;
        }
        SettingKind::MouseEnabled => {
            state.config.mouse_enabled = option_index == 0;
        }
        SettingKind::AutoLockTimeout => {
            state.config.auto_lock_timeout_secs = match option_index {
                0 => 60,
                1 => 120,
                2 => 300,
                3 => 600,
                _ => 1800,
            };
        }
        SettingKind::ClipboardTimeout => {
            state.config.clipboard_timeout_secs = match option_index {
                0 => 10,
                1 => 30,
                2 => 60,
                3 => 120,
                _ => 0, // Never
            };
        }
        SettingKind::ChangeMasterPassword | SettingKind::ConfigureRecovery => {}
    }
}

fn render_security_action_popup(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    screen_state: &SettingsScreen,
    theme: &ThemePalette,
) {
    let Some(action) = screen_state.security_action.as_ref() else {
        return;
    };

    let (title, prompt, detail, force_mask) = match action {
        SecurityActionState::ChangePassword(change) => match change.step {
            ChangePasswordStep::CurrentPassword => (
                " Change Master Password ",
                "Enter current master password",
                "Step 1/4",
                true,
            ),
            ChangePasswordStep::KeyfilePath => (
                " Change Master Password ",
                "Enter keyfile path for verification",
                "Step 2/4",
                false,
            ),
            ChangePasswordStep::NewPassword => (
                " Change Master Password ",
                "Enter new master password",
                "Step 3/4",
                true,
            ),
            ChangePasswordStep::ConfirmPassword => (
                " Change Master Password ",
                "Confirm new master password",
                "Step 4/4",
                true,
            ),
        },
        SecurityActionState::ConfigureRecovery(recovery) => match recovery.step {
            RecoverySetupStep::CurrentPassword => (
                " Configure Recovery ",
                "Enter current master password",
                "Step 1/5",
                true,
            ),
            RecoverySetupStep::KeyfilePath => (
                " Configure Recovery ",
                "Enter keyfile path for verification",
                "Step 2/5",
                false,
            ),
            RecoverySetupStep::QuestionCount => (
                " Configure Recovery ",
                "Number of questions (0-3)",
                "Step 3/5",
                false,
            ),
            RecoverySetupStep::QuestionText => (
                " Configure Recovery ",
                "Enter security question text",
                "Step 4/5",
                false,
            ),
            RecoverySetupStep::AnswerText => (
                " Configure Recovery ",
                "Enter security answer",
                "Step 5/5",
                true,
            ),
        },
    };

    let popup_width = area.width.min(76).saturating_sub(2);
    let popup_height = 10;
    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(title)
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // prompt
            Constraint::Length(3), // input
            Constraint::Length(1), // detail
            Constraint::Length(1), // hint
        ])
        .split(inner);

    let prompt_para = Paragraph::new(prompt)
        .style(Style::default().fg(theme.fg_muted))
        .alignment(Alignment::Center);
    frame.render_widget(prompt_para, chunks[0]);

    let display_text = if force_mask {
        "•".repeat(state.ui_state.input_buffer.text.chars().count())
    } else {
        state.ui_state.input_buffer.text.clone()
    };
    let input_para = Paragraph::new(display_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_focused))
            .title(Span::styled(" Input ", Style::default().fg(theme.accent))),
    );
    frame.render_widget(input_para, chunks[1]);

    let detail_line = if let Some(err) = state.login_screen.error_message.as_ref() {
        format!("{} • {}", detail, err)
    } else {
        detail.to_string()
    };
    let detail_para = Paragraph::new(detail_line)
        .style(Style::default().fg(theme.fg))
        .alignment(Alignment::Center);
    frame.render_widget(detail_para, chunks[2]);

    let hint_para = Paragraph::new("Enter: Next/Save    Esc: Cancel")
        .style(Style::default().fg(theme.fg_muted))
        .alignment(Alignment::Center);
    frame.render_widget(hint_para, chunks[3]);

    let cursor_x = chunks[1].x + 1 + state.ui_state.input_buffer.cursor as u16;
    let cursor_y = chunks[1].y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_screen_navigation() {
        let mut screen = SettingsScreen::new();
        assert_eq!(screen.selected, 0);

        screen.move_down(SettingKind::all().len(), 0);
        assert_eq!(screen.selected, 1);

        screen.move_up();
        assert_eq!(screen.selected, 0);
    }

    #[test]
    fn test_settings_screen_edit_mode() {
        let mut screen = SettingsScreen::new();
        assert!(!screen.editing);

        screen.start_edit(2);
        assert!(screen.editing);
        assert_eq!(screen.sub_selection, 2);

        screen.move_down(SettingKind::all().len(), 5);
        assert_eq!(screen.sub_selection, 3);

        let result = screen.confirm_edit();
        assert!(!screen.editing);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_setting_kind_labels() {
        assert_eq!(SettingKind::Theme.label(), "Theme");
        assert_eq!(SettingKind::AutoLock.label(), "Auto-Lock");
    }
}
