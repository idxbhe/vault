//! Help screen widget
//!
//! Shows keybinding help as a floating overlay.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// Help section with title and keybindings
struct HelpSection {
    title: &'static str,
    bindings: Vec<(&'static str, &'static str)>,
}

/// Render the help overlay
pub fn render(frame: &mut Frame, area: Rect, theme: &ThemePalette) {
    // Calculate centered area
    let width = 60.min(area.width.saturating_sub(4));
    let height = 24.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let help_area = Rect::new(x, y, width, height);

    // Clear background
    frame.render_widget(Clear, help_area);

    // Create help block
    let block = Block::default()
        .title(format!(" {} Help ", icons::ui::HELP))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_focused))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(help_area);
    frame.render_widget(block, help_area);

    // Define help sections
    let sections = vec![
        HelpSection {
            title: "Navigation",
            bindings: vec![
                ("j / ↓", "Move down"),
                ("k / ↑", "Move up"),
                ("h / ←", "Focus list / Previous"),
                ("l / → / Enter", "Focus detail / Select"),
                ("Tab", "Switch pane"),
                ("g g", "Jump to top"),
                ("G", "Jump to bottom"),
            ],
        },
        HelpSection {
            title: "Actions",
            bindings: vec![
                ("/", "Search"),
                ("n / i", "New item"),
                ("e", "Edit item"),
                ("d", "Delete item"),
                ("y", "Copy content"),
                ("r", "Toggle reveal"),
                ("f", "Toggle favorite"),
            ],
        },
        HelpSection {
            title: "System",
            bindings: vec![
                ("u", "Undo"),
                ("Ctrl+r", "Redo"),
                ("Ctrl+l", "Lock vault"),
                ("Ctrl+s", "Save vault"),
                ("?", "This help"),
                ("Esc", "Close / Back"),
                (":q / Ctrl+q", "Quit"),
            ],
        },
    ];

    // Render sections in columns
    let col_width = inner.width / 3;
    let y_offset = inner.y;

    for (col, section) in sections.into_iter().enumerate() {
        let section_x = inner.x + (col as u16 * col_width);
        let section_width = col_width.saturating_sub(1);

        // Section title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            format!(" {} ", section.title),
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]));
        frame.render_widget(title, Rect::new(section_x, y_offset, section_width, 1));

        // Keybindings
        for (i, (key, desc)) in section.bindings.iter().enumerate() {
            let binding_y = y_offset + 1 + i as u16;
            if binding_y >= inner.y + inner.height {
                break;
            }

            let binding = Paragraph::new(Line::from(vec![
                Span::styled(
                    format!(" {:12}", key),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(desc.to_string(), Style::default().fg(theme.fg_muted)),
            ]));

            frame.render_widget(binding, Rect::new(section_x, binding_y, section_width, 1));
        }
    }

    // Footer
    let footer_y = help_area.y + help_area.height - 2;
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Press ", Style::default().fg(theme.fg_muted)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" or ", Style::default().fg(theme.fg_muted)),
        Span::styled(
            "?",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to close ", Style::default().fg(theme.fg_muted)),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(footer, Rect::new(help_area.x, footer_y, help_area.width, 1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_section_structure() {
        // Basic structure test
        let section = HelpSection {
            title: "Test",
            bindings: vec![("a", "Action A"), ("b", "Action B")],
        };
        assert_eq!(section.title, "Test");
        assert_eq!(section.bindings.len(), 2);
    }
}
