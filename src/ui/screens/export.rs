//! Export screen
//!
//! Allows users to export vault data in various formats.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// Export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    EncryptedJson,
    Csv,
}

impl ExportFormat {
    pub fn all() -> &'static [ExportFormat] {
        &[
            ExportFormat::Json,
            ExportFormat::EncryptedJson,
            ExportFormat::Csv,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ExportFormat::Json => "JSON (Plaintext)",
            ExportFormat::EncryptedJson => "JSON (Encrypted)",
            ExportFormat::Csv => "CSV (Plaintext)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ExportFormat::Json => "Export all data as readable JSON. WARNING: Not encrypted!",
            ExportFormat::EncryptedJson => "Export data as encrypted JSON. Requires password to import.",
            ExportFormat::Csv => "Export as CSV for spreadsheet import. WARNING: Not encrypted!",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::EncryptedJson => "vault.json",
            ExportFormat::Csv => "csv",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ExportFormat::Json => "󰘦",
            ExportFormat::EncryptedJson => icons::ui::VAULT_LOCKED,
            ExportFormat::Csv => "󱎏",
        }
    }

    pub fn is_encrypted(&self) -> bool {
        matches!(self, ExportFormat::EncryptedJson)
    }
}

/// Export screen state
#[derive(Debug, Default)]
pub struct ExportScreen {
    /// Selected format index
    pub selected: usize,
    /// Export path input
    pub path_input: String,
    /// Whether path input is focused
    pub path_focused: bool,
    /// Export status message
    pub status: Option<ExportStatus>,
}

#[derive(Debug, Clone)]
pub enum ExportStatus {
    Success(String),
    Error(String),
    InProgress,
}

impl ExportScreen {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get selected format
    pub fn selected_format(&self) -> ExportFormat {
        ExportFormat::all()
            .get(self.selected)
            .copied()
            .unwrap_or(ExportFormat::Json)
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if !self.path_focused {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if !self.path_focused && self.selected < ExportFormat::all().len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Toggle between format selection and path input
    pub fn toggle_focus(&mut self) {
        self.path_focused = !self.path_focused;
    }

    /// Add character to path input
    pub fn input_char(&mut self, c: char) {
        if self.path_focused {
            self.path_input.push(c);
        }
    }

    /// Remove character from path input
    pub fn delete_char(&mut self) {
        if self.path_focused {
            self.path_input.pop();
        }
    }

    /// Get default export path
    pub fn default_path(&self, vault_name: &str) -> String {
        let format = self.selected_format();
        format!("{}.{}", vault_name.replace(' ', "_"), format.extension())
    }
}

/// Render the export screen
pub fn render(
    frame: &mut Frame,
    state: &AppState,
    screen_state: &ExportScreen,
    theme: &ThemePalette,
) {
    let area = frame.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(10), // Format selection
            Constraint::Length(5),  // Path input
            Constraint::Min(3),     // Description / Status
            Constraint::Length(2),  // Footer
        ])
        .margin(2)
        .split(area);

    // Header
    render_header(frame, chunks[0], theme);

    // Format selection
    render_format_list(frame, chunks[1], screen_state, theme);

    // Path input
    render_path_input(frame, chunks[2], state, screen_state, theme);

    // Description / Status
    render_description(frame, chunks[3], screen_state, theme);

    // Footer
    render_footer(frame, chunks[4], screen_state, theme);
}

fn render_header(frame: &mut Frame, area: Rect, theme: &ThemePalette) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " 󰈔 ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Export Vault",
            Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
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

fn render_format_list(
    frame: &mut Frame,
    area: Rect,
    screen_state: &ExportScreen,
    theme: &ThemePalette,
) {
    let formats = ExportFormat::all();

    let items: Vec<ListItem> = formats
        .iter()
        .enumerate()
        .map(|(i, format)| {
            let selected = i == screen_state.selected && !screen_state.path_focused;

            let style = if selected {
                Style::default()
                    .fg(theme.selection_fg)
                    .bg(theme.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            let warning = if !format.is_encrypted() {
                Span::styled(
                    format!(" {} ", icons::ui::WARNING),
                    style.fg(theme.warning),
                )
            } else {
                Span::styled(
                    format!(" {} ", icons::ui::CHECK),
                    style.fg(theme.success),
                )
            };

            let line = Line::from(vec![
                Span::styled(format!(" {} ", format.icon()), style.fg(theme.accent)),
                Span::styled(format.label().to_string(), style),
                warning,
            ]);

            ListItem::new(line)
        })
        .collect();

    let border_color = if !screen_state.path_focused {
        theme.border_focused
    } else {
        theme.border
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(" Format ")
            .title_style(Style::default().fg(theme.fg_muted)),
    );

    let mut list_state = ListState::default();
    list_state.select(Some(screen_state.selected));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_path_input(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    screen_state: &ExportScreen,
    theme: &ThemePalette,
) {
    let vault_name = state
        .vault_state
        .as_ref()
        .map(|vs| vs.vault.name.as_str())
        .unwrap_or("vault");

    let path = if screen_state.path_input.is_empty() {
        screen_state.default_path(vault_name)
    } else {
        screen_state.path_input.clone()
    };

    let border_color = if screen_state.path_focused {
        theme.border_focused
    } else {
        theme.border
    };

    let display_path = if screen_state.path_focused {
        format!("{}|", path)
    } else {
        path
    };

    let input = Paragraph::new(display_path)
        .style(Style::default().fg(theme.fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(" Export Path ")
                .title_style(Style::default().fg(theme.fg_muted)),
        );

    frame.render_widget(input, area);
}

fn render_description(
    frame: &mut Frame,
    area: Rect,
    screen_state: &ExportScreen,
    theme: &ThemePalette,
) {
    let format = screen_state.selected_format();

    // Show status if available, otherwise show description
    let content = if let Some(ref status) = screen_state.status {
        match status {
            ExportStatus::Success(msg) => Paragraph::new(msg.clone())
                .style(Style::default().fg(theme.success))
                .alignment(Alignment::Center),
            ExportStatus::Error(msg) => Paragraph::new(msg.clone())
                .style(Style::default().fg(theme.error))
                .alignment(Alignment::Center),
            ExportStatus::InProgress => Paragraph::new("Exporting...")
                .style(Style::default().fg(theme.info))
                .alignment(Alignment::Center),
        }
    } else {
        let warning = if !format.is_encrypted() {
            format!(
                "\n\n{} Warning: This format is NOT encrypted!",
                icons::ui::WARNING
            )
        } else {
            String::new()
        };

        Paragraph::new(format!("{}{}", format.description(), warning))
            .style(Style::default().fg(theme.fg_muted))
            .alignment(Alignment::Center)
    };

    frame.render_widget(content, area);
}

fn render_footer(frame: &mut Frame, area: Rect, screen_state: &ExportScreen, theme: &ThemePalette) {
    let hints = if screen_state.path_focused {
        "Tab: Switch focus  Enter: Export  Esc: Cancel"
    } else {
        "j/k: Select format  Tab: Edit path  Enter: Export  Esc: Cancel"
    };

    let footer = Paragraph::new(hints)
        .style(Style::default().fg(theme.fg_muted))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_all() {
        let formats = ExportFormat::all();
        assert_eq!(formats.len(), 3);
    }

    #[test]
    fn test_export_format_encrypted() {
        assert!(!ExportFormat::Json.is_encrypted());
        assert!(ExportFormat::EncryptedJson.is_encrypted());
        assert!(!ExportFormat::Csv.is_encrypted());
    }

    #[test]
    fn test_export_screen_navigation() {
        let mut screen = ExportScreen::new();
        assert_eq!(screen.selected, 0);
        assert!(!screen.path_focused);

        screen.move_down();
        assert_eq!(screen.selected, 1);

        screen.toggle_focus();
        assert!(screen.path_focused);

        screen.input_char('t');
        screen.input_char('e');
        screen.input_char('s');
        screen.input_char('t');
        assert_eq!(screen.path_input, "test");

        screen.delete_char();
        assert_eq!(screen.path_input, "tes");
    }

    #[test]
    fn test_default_path() {
        let screen = ExportScreen::new();
        let path = screen.default_path("My Vault");
        assert_eq!(path, "My_Vault.json");
    }
}
