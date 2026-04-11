//! Item detail widget - displays full item information
//!
//! Shows item content with masking, notes, tags, and metadata.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use chrono::{DateTime, Utc};

use crate::app::AppState;
use crate::domain::{Item, Tag};
use crate::ui::theme::ThemePalette;
use crate::utils::{icons, mask};

/// Render the item detail view
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    focused: bool,
    theme: &ThemePalette,
) {
    let border_color = if focused {
        theme.border_focused
    } else {
        theme.border
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(" Details ", Style::default().fg(theme.accent)));

    let item_opt = state.selected_item().cloned();
    let Some(item) = item_opt else {
        render_empty(frame, area, block, theme);
        return;
    };

    let tags = state
        .vault_state
        .as_ref()
        .map(|vs| &vs.vault.tags[..])
        .unwrap_or(&[]);

    let revealed = state.ui_state.content_revealed;

    // Split area for content and action buttons (hints now embedded in buttons)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Main content
            Constraint::Length(1), // Action buttons with embedded hints
        ])
        .split(block.inner(area));

    let selected_field_idx = state.ui_state.detail_selected_field;
    let is_focused = state.ui_state.focused_pane == crate::app::Pane::Detail;
    let lines = build_detail_lines(&item, tags, revealed, theme, selected_field_idx, is_focused);

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((state.ui_state.detail_scroll_offset as u16, 0));

    frame.render_widget(block, area);
    frame.render_widget(paragraph, chunks[0]);

    // Render action buttons (now includes keyboard hints in labels)
    render_action_buttons(frame, chunks[1], state, revealed, theme);
}

/// Build the detail lines for an item
fn build_detail_lines<'a>(
    item: &Item,
    tags: &[Tag],
    revealed: bool,
    theme: &'a ThemePalette,
    selected_field_idx: usize,
    is_focused: bool,
) -> Vec<Line<'a>> {
    let mut lines = vec![];

    // Header: icon + title + favorite
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", item.kind.icon()),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            item.title.clone(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        if item.favorite {
            Span::styled(
                format!(" {}", icons::ui::STAR),
                Style::default().fg(theme.warning),
            )
        } else {
            Span::raw("")
        },
    ]));

    lines.push(Line::from(""));

    // Content section based on item type
    lines.extend(build_content_section(
        item,
        revealed,
        theme,
        selected_field_idx,
        is_focused,
    ));

    // Notes section
    if let Some(ref notes) = item.notes {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "╭─ Notes",
            Style::default().fg(theme.fg_muted),
        )));
        for line in notes.lines() {
            lines.push(Line::from(Span::styled(
                format!("│ {}", line),
                Style::default().fg(theme.fg),
            )));
        }
        lines.push(Line::from(Span::styled(
            "╰─",
            Style::default().fg(theme.fg_muted),
        )));
    }

    // Tags section
    if !item.tags.is_empty() {
        lines.push(Line::from(""));
        let tag_spans: Vec<Span> = item
            .tags
            .iter()
            .filter_map(|tag_id| tags.iter().find(|t| t.id == *tag_id))
            .map(|tag| {
                let color = tag
                    .color
                    .as_ref()
                    .and_then(|c| parse_hex_color(c))
                    .unwrap_or(theme.accent);
                Span::styled(
                    format!(" {} {} ", icons::ui::TAG, tag.name),
                    Style::default().fg(theme.bg).bg(color),
                )
            })
            .collect();

        if !tag_spans.is_empty() {
            let mut spans = vec![Span::styled("Tags: ", Style::default().fg(theme.fg_muted))];
            for tag_span in tag_spans {
                spans.push(tag_span);
                spans.push(Span::raw(" "));
            }
            lines.push(Line::from(spans));
        }
    }

    // Metadata section
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format_metadata(item),
        Style::default().fg(theme.fg_muted),
    )));

    lines
}

/// Build content section based on item type
fn build_content_section<'a>(
    item: &Item,
    revealed: bool,
    theme: &'a ThemePalette,
    selected_field_idx: usize,
    is_focused: bool,
) -> Vec<Line<'a>> {
    let mut lines = vec![];
    let fields = item.get_fields();

    for (idx, (label, value, is_sensitive, _)) in fields.iter().enumerate() {
        let is_selected = is_focused && idx == selected_field_idx;

        let display_value = if *is_sensitive && !revealed {
            mask::mask_content(value)
        } else {
            value.to_string()
        };

        let bg_color = if is_selected {
            theme.selection_bg
        } else {
            theme.bg
        };

        let mut spans = vec![];
        if is_selected {
            spans.push(Span::styled(
                " > ",
                Style::default()
                    .fg(theme.accent)
                    .bg(bg_color)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled("   ", Style::default().bg(bg_color)));
        }

        spans.push(Span::styled(
            format!("{}: ", label),
            Style::default()
                .fg(if is_selected {
                    theme.fg
                } else {
                    theme.fg_muted
                })
                .bg(bg_color),
        ));

        let mut timer_span = None;
        if label == "TOTP Code" && revealed {
            let remaining = 30 - (chrono::Utc::now().timestamp() % 30);
            timer_span = Some(Span::styled(
                format!(" ({}s)", remaining),
                Style::default()
                    .fg(theme.fg_muted)
                    .bg(bg_color),
            ));
        }

        spans.push(Span::styled(
            display_value,
            Style::default()
                .fg(if *is_sensitive && !revealed {
                    theme.sensitive_mask
                } else {
                    theme.fg
                })
                .bg(bg_color),
        ));

        if let Some(timer) = timer_span {
            spans.push(timer);
        }

        lines.push(Line::from(spans));
    }

    lines
}

/// Render action buttons with embedded keyboard hints at bottom of detail pane
fn render_action_buttons(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    revealed: bool,
    theme: &ThemePalette,
) {
    use crate::ui::widgets::{ButtonStyle, render_button_row};

    let buttons = vec![
        (
            "reveal".to_string(),
            if revealed { "Hide" } else { "Reveal" },
            Some("r"),
            ButtonStyle::Primary,
        ),
        (
            "copy".to_string(),
            "Copy",
            Some("y"),
            ButtonStyle::Secondary,
        ),
        (
            "edit".to_string(),
            "Edit",
            Some("e"),
            ButtonStyle::Secondary,
        ),
        (
            "delete".to_string(),
            "Delete",
            Some("d"),
            ButtonStyle::Danger,
        ),
    ];

    let button_regions = render_button_row(frame, area, &buttons, theme);

    // Register button regions
    for button_region in button_regions {
        state.ui_state.layout_regions.register_clickable(
            button_region.region,
            crate::input::mouse::ClickableElement::Button(button_region.name),
        );
    }
}

/// Format metadata line
fn format_metadata(item: &Item) -> String {
    format!(
        "Created: {} • Updated: {}",
        format_datetime(item.created_at),
        format_datetime(item.updated_at)
    )
}

/// Format datetime for display
fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M").to_string()
}

/// Parse hex color string to ratatui Color
fn parse_hex_color(hex: &str) -> Option<ratatui::style::Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(ratatui::style::Color::Rgb(r, g, b))
}

/// Render empty state when no item is selected
fn render_empty(frame: &mut Frame, area: Rect, block: Block, theme: &ThemePalette) {
    let help = vec![
        Line::from(""),
        Line::from(Span::styled(
            "No item selected",
            Style::default().fg(theme.fg_muted),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Select an item from the list",
            Style::default().fg(theme.fg_muted),
        )),
        Line::from(Span::styled(
            "or press 'n' to create a new one",
            Style::default().fg(theme.fg_muted),
        )),
    ];

    let paragraph = Paragraph::new(help)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_datetime() {
        let dt = chrono::Utc::now();
        let formatted = format_datetime(dt);
        assert!(formatted.contains("-"));
        assert!(formatted.contains(":"));
    }

    #[test]
    fn test_parse_hex_color() {
        let color = parse_hex_color("#ff5500").unwrap();
        assert!(matches!(color, ratatui::style::Color::Rgb(255, 85, 0)));

        let color = parse_hex_color("00ff00").unwrap();
        assert!(matches!(color, ratatui::style::Color::Rgb(0, 255, 0)));

        assert!(parse_hex_color("invalid").is_none());
        assert!(parse_hex_color("#fff").is_none());
    }
}
