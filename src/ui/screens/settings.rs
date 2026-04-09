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

/// Settings screen state
#[derive(Debug, Default)]
pub struct SettingsScreen {
    /// Currently selected setting index
    pub selected: usize,
    /// Whether editing the selected setting
    pub editing: bool,
    /// Sub-selection for lists (like theme chooser)
    pub sub_selection: usize,
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
    let hints = if screen_state.editing {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_screen_navigation() {
        let mut screen = SettingsScreen::new();
        assert_eq!(screen.selected, 0);

        screen.move_down(6, 0);
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

        screen.move_down(6, 5);
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
