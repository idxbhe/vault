//! Button widget - renders clickable buttons with different styles

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::ui::theme::ThemePalette;

/// Button style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Primary action (accent color)
    Primary,
    /// Danger action (error color)
    Danger,
    /// Secondary/normal action
    Secondary,
}

/// A clickable button region
#[derive(Debug, Clone)]
pub struct ButtonRegion {
    pub name: String,
    pub region: crate::input::mouse::ClickRegion,
}

/// Render a row of buttons and return their clickable regions
///
/// Buttons are rendered as solid background blocks with optional keyboard hints: ` Label (key) `
/// Returns the regions for mouse click registration
pub fn render_button_row<'a>(
    frame: &mut Frame,
    area: Rect,
    buttons: &[(String, &str, Option<&str>, ButtonStyle)], // (name, label, optional_key, style)
    theme: &ThemePalette,
) -> Vec<ButtonRegion> {
    if buttons.is_empty() || area.height == 0 {
        return Vec::new();
    }

    let mut spans = Vec::new();
    let mut regions = Vec::new();
    let mut rendered_buttons = Vec::with_capacity(buttons.len());
    let mut total_width: u16 = 0;

    for (index, (name, label, key, btn_style)) in buttons.iter().enumerate() {
        let button_text = if let Some(k) = key {
            format!(" {} ({}) ", label, k)
        } else {
            format!(" {} ", label)
        };
        let button_width = button_text.len() as u16;

        if index > 0 {
            total_width = total_width.saturating_add(2); // Double space between buttons
        }
        total_width = total_width.saturating_add(button_width);

        rendered_buttons.push((name.clone(), button_text, button_width, *btn_style));
    }

    let center_padding = area.width.saturating_sub(total_width) / 2;
    let mut x_offset = area.x.saturating_add(center_padding);

    for (index, (name, button_text, button_width, btn_style)) in rendered_buttons.iter().enumerate()
    {
        if index > 0 {
            spans.push(Span::raw("  "));
            x_offset = x_offset.saturating_add(2);
        }

        // Choose colors based on style
        let (fg, bg) = match *btn_style {
            ButtonStyle::Primary => (theme.bg, theme.accent),
            ButtonStyle::Danger => (theme.bg, theme.error),
            ButtonStyle::Secondary => (theme.bg, theme.info), // Changed from bg_alt to info for better visibility
        };

        spans.push(Span::styled(
            button_text.clone(),
            Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        ));

        // Register clickable region for this button
        regions.push(ButtonRegion {
            name: name.clone(),
            region: crate::input::mouse::ClickRegion::new(x_offset, area.y, *button_width, 1),
        });

        x_offset = x_offset.saturating_add(*button_width);
    }

    // Render the button row with center alignment
    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);

    regions
}

/// Render keyboard hints (shortcuts) in a compact format
///
/// Renders hints as: `key: action  key: action` in muted style
pub fn render_keyboard_hints(
    frame: &mut Frame,
    area: Rect,
    hints: &[(&str, &str)], // (key, action)
    theme: &ThemePalette,
) {
    if hints.is_empty() || area.height == 0 {
        return;
    }

    let mut spans = Vec::new();

    for (i, (key, action)) in hints.iter().enumerate() {
        // Add separator between hints
        if i > 0 {
            spans.push(Span::styled("  ", Style::default()));
        }

        // Key in slightly brighter color
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(theme.fg_muted)
                .add_modifier(Modifier::BOLD),
        ));

        spans.push(Span::styled(": ", Style::default().fg(theme.fg_muted)));

        // Action in muted color
        spans.push(Span::styled(*action, Style::default().fg(theme.fg_muted)));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_styles() {
        assert_eq!(ButtonStyle::Primary, ButtonStyle::Primary);
        assert_ne!(ButtonStyle::Primary, ButtonStyle::Danger);
    }
}
