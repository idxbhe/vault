//! Item detail widget - displays full item information
//!
//! Shows item content with masking, notes, tags, and metadata.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use chrono::{DateTime, Utc};

use crate::app::AppState;
use crate::domain::{Item, ItemContent, Tag};
use crate::ui::theme::ThemePalette;
use crate::utils::{icons, mask};

/// Render the item detail view
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
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
        .title(Span::styled(
            " Details ",
            Style::default().fg(theme.accent),
        ));

    let Some(item) = state.selected_item() else {
        render_empty(frame, area, block, theme);
        return;
    };

    let tags = state
        .vault_state
        .as_ref()
        .map(|vs| &vs.vault.tags[..])
        .unwrap_or(&[]);

    let revealed = state.ui_state.content_revealed;
    let lines = build_detail_lines(item, tags, revealed, theme);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((state.ui_state.detail_scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}

/// Build the detail lines for an item
fn build_detail_lines<'a>(
    item: &Item,
    tags: &[Tag],
    revealed: bool,
    theme: &'a ThemePalette,
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
            Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
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
    lines.extend(build_content_section(item, revealed, theme));

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

    // Action hints
    lines.push(Line::from(""));
    lines.push(build_action_hints(revealed, theme));

    lines
}

/// Build content section based on item type
fn build_content_section<'a>(
    item: &Item,
    revealed: bool,
    theme: &'a ThemePalette,
) -> Vec<Line<'a>> {
    let mut lines = vec![];

    match &item.content {
        ItemContent::Generic { value } => {
            lines.push(build_field_line("Value", value, revealed, theme));
        }

        ItemContent::CryptoSeed {
            seed_phrase,
            derivation_path,
            network,
        } => {
            lines.push(build_field_line("Seed Phrase", seed_phrase, revealed, theme));
            if let Some(path) = derivation_path {
                lines.push(build_field_line("Derivation Path", path, false, theme));
            }
            if let Some(net) = network {
                lines.push(build_field_line("Network", net, false, theme));
            }
        }

        ItemContent::Password {
            username,
            password,
            url,
            totp_secret,
        } => {
            if let Some(user) = username {
                lines.push(build_field_line("Username", user, false, theme));
            }
            lines.push(build_field_line("Password", password, revealed, theme));
            if let Some(u) = url {
                lines.push(build_field_line("URL", u, false, theme));
            }
            if let Some(_totp) = totp_secret {
                lines.push(Line::from(vec![
                    Span::styled("TOTP: ", Style::default().fg(theme.fg_muted)),
                    Span::styled(
                        if revealed { "configured" } else { "••••••" },
                        Style::default().fg(theme.fg),
                    ),
                ]));
            }
        }

        ItemContent::SecureNote { content } => {
            lines.push(Line::from(Span::styled(
                "Content:",
                Style::default().fg(theme.fg_muted),
            )));
            let display = if revealed {
                content.clone()
            } else {
                mask::mask_content(content)
            };
            for line in display.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", line),
                    Style::default().fg(if revealed {
                        theme.fg
                    } else {
                        theme.sensitive_mask
                    }),
                )));
            }
        }

        ItemContent::ApiKey {
            key,
            service,
            expires_at,
        } => {
            if let Some(svc) = service {
                lines.push(build_field_line("Service", svc, false, theme));
            }
            lines.push(build_field_line("API Key", key, revealed, theme));
            if let Some(exp) = expires_at {
                lines.push(build_field_line("Expires", &format_datetime(*exp), false, theme));
            }
        }
    }

    lines
}

/// Build a single field line with optional masking
fn build_field_line<'a>(
    label: &'a str,
    value: &str,
    sensitive: bool,
    theme: &'a ThemePalette,
) -> Line<'a> {
    let display_value = if sensitive {
        mask::mask_content(value)
    } else {
        value.to_string()
    };

    Line::from(vec![
        Span::styled(
            format!("{}: ", label),
            Style::default().fg(theme.fg_muted),
        ),
        Span::styled(
            display_value,
            Style::default().fg(if sensitive {
                theme.sensitive_mask
            } else {
                theme.fg
            }),
        ),
    ])
}

/// Build action hints line
fn build_action_hints<'a>(revealed: bool, theme: &'a ThemePalette) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            " r ",
            Style::default().fg(theme.bg).bg(theme.accent),
        ),
        Span::styled(
            if revealed { " hide " } else { " reveal " },
            Style::default().fg(theme.fg_muted),
        ),
        Span::styled(
            " y ",
            Style::default().fg(theme.bg).bg(theme.accent),
        ),
        Span::styled(" copy ", Style::default().fg(theme.fg_muted)),
        Span::styled(
            " e ",
            Style::default().fg(theme.bg).bg(theme.accent),
        ),
        Span::styled(" edit ", Style::default().fg(theme.fg_muted)),
        Span::styled(
            " d ",
            Style::default().fg(theme.bg).bg(theme.error),
        ),
        Span::styled(" delete ", Style::default().fg(theme.fg_muted)),
    ])
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
