use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use chrono::{DateTime, Utc};

use crate::app::AppState;
use crate::domain::{Item, Tag};
use crate::ui::theme::ThemePalette;
use crate::utils::{icons, mask};

#[derive(Clone, Debug)]
enum ViewComponent {
    HeaderAndStandard(Vec<usize>),
    Standard(Vec<usize>),
    SpecializedBox(usize),
    NotesBox,
    Metadata,
    Buttons,
}

fn estimate_height(text: &str, width: u16) -> u16 {
    if text.is_empty() {
        return 3; 
    }
    let w = width.saturating_sub(4) as f32; // account for borders and padding
    if w <= 0.0 {
        return 3;
    }
    let mut lines = 0;
    for line in text.lines() {
        let chars = line.chars().count() as f32;
        let l = (chars / w).ceil() as usize;
        lines += l.max(1);
    }
    (lines as u16 + 2).min(16).max(4) // capped between 4 and 16 lines
}

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

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let item_opt = state.selected_item().cloned();
    let Some(item) = item_opt else {
        render_empty(frame, inner_area, Block::default(), theme);
        return;
    };

    let tags_vec: Vec<Tag> = state
        .vault_state
        .as_ref()
        .map(|vs| vs.vault.tags.clone())
        .unwrap_or_default();
    let tags = &tags_vec[..];

    let revealed = state.ui_state.content_revealed;

    let fields = item.get_fields();
    use crate::ui::widgets::FormField;

    let mut components = Vec::new();
    let mut current_standard = Vec::new();
    let mut is_first = true;

    for (idx, (_, _, _, f)) in fields.iter().enumerate() {
        let is_box = matches!(
            f,
            Some(FormField::SeedPhrase) | Some(FormField::Content) | Some(FormField::ApiKey)
        );

        if is_box {
            if is_first || !current_standard.is_empty() {
                components.push(if is_first {
                    ViewComponent::HeaderAndStandard(current_standard.clone())
                } else {
                    ViewComponent::Standard(current_standard.clone())
                });
                current_standard.clear();
                is_first = false;
            }
            components.push(ViewComponent::SpecializedBox(idx));
        } else {
            current_standard.push(idx);
        }
    }

    if is_first || !current_standard.is_empty() {
        components.push(if is_first {
            ViewComponent::HeaderAndStandard(current_standard)
        } else {
            ViewComponent::Standard(current_standard)
        });
    }

    if item.notes.is_some() {
        components.push(ViewComponent::NotesBox);
    }

    components.push(ViewComponent::Metadata);
    components.push(ViewComponent::Buttons);

    let mut constraints = Vec::new();
    for comp in &components {
        match comp {
            ViewComponent::HeaderAndStandard(fields_idx) => {
                let mut lines = 2; // Icon+Title, Blank
                if !item.tags.is_empty() {
                    lines += 2; // Tags, Blank
                }
                lines += fields_idx.len() as u16; // Fields
                constraints.push(Constraint::Length(lines.max(1)));
            }
            ViewComponent::Standard(fields_idx) => {
                constraints.push(Constraint::Length((fields_idx.len() as u16).max(1)));
            }
            ViewComponent::SpecializedBox(idx) => {
                let (_, value, _, _) = &fields[*idx];
                let h = estimate_height(value, inner_area.width);
                constraints.push(Constraint::Length(h));
            }
            ViewComponent::NotesBox => {
                let h = estimate_height(item.notes.as_deref().unwrap_or(""), inner_area.width);
                constraints.push(Constraint::Length(h));
            }
            ViewComponent::Metadata => {
                constraints.push(Constraint::Length(3)); // Blank, Created, Updated
            }
            ViewComponent::Buttons => {
                constraints.push(Constraint::Length(1));
            }
        }
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    let is_focused = state.ui_state.focused_pane == crate::app::Pane::Detail;

    for (chunk_idx, comp) in components.iter().enumerate() {
        if chunk_idx >= chunks.len() { break; }
        let chunk = chunks[chunk_idx];
        
        match comp {
            ViewComponent::HeaderAndStandard(fields_idx) => {
                let lines = build_standard_lines(true, &item, tags, fields_idx, revealed, theme, state.ui_state.detail_focus, is_focused);
                let p = Paragraph::new(lines).wrap(Wrap { trim: false });
                frame.render_widget(p, chunk);
                
                // Clicking logic
                let mut local_y = chunk.y;
                local_y += 2; // Title + Blank
                if !item.tags.is_empty() { local_y += 2; }
                for &idx in fields_idx {
                    if local_y < inner_area.y + inner_area.height {
                        state.ui_state.layout_regions.register_clickable(
                            crate::input::mouse::ClickRegion::new(chunk.x, local_y, chunk.width, 1),
                            crate::input::mouse::ClickableElement::DetailField(idx),
                        );
                    }
                    local_y += 1;
                }
            }
            ViewComponent::Standard(fields_idx) => {
                let lines = build_standard_lines(false, &item, tags, fields_idx, revealed, theme, state.ui_state.detail_focus, is_focused);
                let p = Paragraph::new(lines).wrap(Wrap { trim: false });
                frame.render_widget(p, chunk);
                
                let mut local_y = chunk.y;
                for &idx in fields_idx {
                    if local_y < inner_area.y + inner_area.height {
                        state.ui_state.layout_regions.register_clickable(
                            crate::input::mouse::ClickRegion::new(chunk.x, local_y, chunk.width, 1),
                            crate::input::mouse::ClickableElement::DetailField(idx),
                        );
                    }
                    local_y += 1;
                }
            }
            ViewComponent::SpecializedBox(idx) => {
                let (label, value, is_sensitive, _) = &fields[*idx];
                let field_is_focused = is_focused && state.ui_state.detail_focus == crate::app::state::DetailFocus::Field(*idx);
                let field_border_color = if field_is_focused { theme.border_focused } else { theme.border };

                let field_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(field_border_color))
                    .title(
                        ratatui::text::Line::from(format!(" {} ", label))
                            .alignment(ratatui::layout::Alignment::Center)
                            .style(Style::default().fg(theme.fg_muted)),
                    )
                    .padding(Padding::horizontal(1));

                let display_value = if *is_sensitive {
                    if !revealed {
                        mask::mask_content(value)
                    } else {
                        value.to_string()
                    }
                } else {
                    value.to_string()
                };

                let scroll_val = state.ui_state.field_scrolls.get(idx).copied().unwrap_or(0);
                let p = Paragraph::new(display_value)
                    .block(field_block)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(if *is_sensitive && !revealed { theme.sensitive_mask } else { theme.fg }))
                    .wrap(Wrap { trim: false })
                    .scroll((scroll_val, 0));

                frame.render_widget(p, chunk);

                state.ui_state.layout_regions.register_clickable(
                    crate::input::mouse::ClickRegion::new(chunk.x, chunk.y, chunk.width, chunk.height),
                    crate::input::mouse::ClickableElement::DetailField(*idx),
                );
            }
            ViewComponent::NotesBox => {
                let notes_is_focused = is_focused && state.ui_state.detail_focus == crate::app::state::DetailFocus::Notes;
                let notes_border_color = if notes_is_focused { theme.border_focused } else { theme.border };

                let notes_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(notes_border_color))
                    .title(
                        ratatui::text::Line::from(" Notes ")
                            .alignment(ratatui::layout::Alignment::Center)
                            .style(Style::default().fg(theme.fg_muted)),
                    )
                    .padding(Padding::horizontal(1));

                let np = Paragraph::new(item.notes.as_deref().unwrap_or(""))
                    .block(notes_block)
                    .style(Style::default().fg(theme.fg))
                    .wrap(Wrap { trim: false })
                    .scroll((state.ui_state.notes_scroll_offset, 0));

                frame.render_widget(np, chunk);

                state.ui_state.layout_regions.register_clickable(
                    crate::input::mouse::ClickRegion::new(chunk.x, chunk.y, chunk.width, chunk.height),
                    crate::input::mouse::ClickableElement::DetailNotes,
                );
            }
            ViewComponent::Metadata => {
                let lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(format!("Created: {}", format_datetime(item.created_at)), Style::default().fg(theme.fg_muted))),
                    Line::from(Span::styled(format!("Updated: {}", format_datetime(item.updated_at)), Style::default().fg(theme.fg_muted))),
                ];
                let p = Paragraph::new(lines).alignment(Alignment::Left);
                frame.render_widget(p, chunk);
            }
            ViewComponent::Buttons => {
                render_action_buttons(frame, chunk, state, revealed, theme);
            }
        }
    }
}

fn build_standard_lines<'a>(
    include_header: bool,
    item: &Item,
    tags: &[Tag],
    fields_idx: &[usize],
    revealed: bool,
    theme: &'a ThemePalette,
    detail_focus: crate::app::state::DetailFocus,
    is_focused: bool,
) -> Vec<Line<'a>> {
    let mut lines = vec![];

    if include_header {
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", item.kind.icon()), Style::default().fg(theme.accent)),
            Span::styled(item.title.clone(), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
            if item.favorite { Span::styled(format!(" {}", icons::ui::STAR), Style::default().fg(theme.warning)) } else { Span::raw("") },
        ]));
        lines.push(Line::from(""));

        if !item.tags.is_empty() {
            let tag_spans: Vec<Span> = item.tags.iter().filter_map(|tag_id| tags.iter().find(|t| t.id == *tag_id)).map(|tag| {
                let color = tag.color.as_ref().and_then(|c| parse_hex_color(c)).unwrap_or(theme.accent);
                Span::styled(format!(" {} {} ", icons::ui::TAG, tag.name), Style::default().fg(theme.bg).bg(color))
            }).collect();

            if !tag_spans.is_empty() {
                let mut spans = vec![Span::styled("Tags: ", Style::default().fg(theme.fg_muted))];
                for tag_span in tag_spans {
                    spans.push(tag_span);
                    spans.push(Span::raw(" "));
                }
                lines.push(Line::from(spans));
            }
            lines.push(Line::from(""));
        }
    }

    let fields = item.get_fields();
    for &idx in fields_idx {
        let (label, value, is_sensitive, _) = &fields[idx];
        let is_selected = is_focused && detail_focus == crate::app::state::DetailFocus::Field(idx);
        let display_value = if *is_sensitive { if !revealed { mask::mask_content(value) } else { value.to_string() } } else { value.to_string() };
        let bg_color = if is_selected { theme.selection_bg } else { theme.bg };
        
        let mut dv = display_value;
        if dv.is_empty() { dv = "-".to_string(); }

        let mut spans = vec![];
        if is_selected {
            spans.push(Span::styled(" > ", Style::default().fg(theme.accent).bg(bg_color).add_modifier(Modifier::BOLD)));
        } else {
            spans.push(Span::styled("   ", Style::default().bg(bg_color)));
        }

        spans.push(Span::styled(format!("{}: ", label), Style::default().fg(if is_selected { theme.fg } else { theme.fg_muted }).bg(bg_color)));

        let mut timer_span = None;
        if label == "TOTP Code" {
            let remaining = 30 - (chrono::Utc::now().timestamp() % 30);
            timer_span = Some(Span::styled(format!(" ({}s)", remaining), Style::default().fg(theme.fg_muted).bg(bg_color)));
        }

        spans.push(Span::styled(dv, Style::default().fg(if *is_sensitive && !revealed { theme.sensitive_mask } else { theme.fg }).bg(bg_color)));

        if let Some(timer) = timer_span { spans.push(timer); }
        lines.push(Line::from(spans));
    }

    lines
}

fn render_action_buttons(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    revealed: bool,
    theme: &ThemePalette,
) {
    use crate::ui::widgets::{ButtonStyle, render_button_row};

    let buttons = vec![
        ("reveal".to_string(), if revealed { "Hide" } else { "Reveal" }, Some("r"), ButtonStyle::Primary),
        ("copy".to_string(), "Copy", Some("y"), ButtonStyle::Secondary),
        ("edit".to_string(), "Edit", Some("e"), ButtonStyle::Secondary),
        ("delete".to_string(), "Delete", Some("d"), ButtonStyle::Danger),
    ];

    let button_regions = render_button_row(frame, area, &buttons, theme);
    for button_region in button_regions {
        state.ui_state.layout_regions.register_clickable(
            button_region.region,
            crate::input::mouse::ClickableElement::Button(button_region.name),
        );
    }
}

fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M").to_string()
}

fn parse_hex_color(hex: &str) -> Option<ratatui::style::Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(ratatui::style::Color::Rgb(r, g, b))
}

fn render_empty(frame: &mut Frame, area: Rect, _block: Block, theme: &ThemePalette) {
    let help = vec![
        Line::from(""),
        Line::from(Span::styled("No item selected", Style::default().fg(theme.fg_muted))),
        Line::from(""),
        Line::from(Span::styled("Select an item from the list", Style::default().fg(theme.fg_muted))),
        Line::from(Span::styled("or press 'n' to create a new one", Style::default().fg(theme.fg_muted))),
    ];
    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
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
    }
}
