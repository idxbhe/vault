//! Statusline widget - Lualine-style status bar
//!
//! Shows mode, vault info, item count, and keybinding hints.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{AppMode, AppState, FloatingWindow, NotificationLevel, Pane};
use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// Render the statusline at the bottom of the screen
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemePalette) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // Mode indicator
            Constraint::Min(20),    // Middle info
            Constraint::Length(20), // Right info
        ])
        .split(area);

    render_mode_indicator(frame, chunks[0], state, theme);
    render_middle_section(frame, chunks[1], state, theme);
    render_right_section(frame, chunks[2], state, theme);
}

/// Render the mode indicator (leftmost section)
fn render_mode_indicator(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemePalette) {
    let (icon, label, bg_color) = match state.mode {
        AppMode::Locked => (icons::mode::NORMAL, "LOCKED", theme.warning),
        AppMode::Unlocked => match state.ui_state.focused_pane {
            Pane::List => (icons::mode::NORMAL, "LIST", theme.primary),
            Pane::Detail => (icons::mode::NORMAL, "DETAIL", theme.secondary),
            Pane::Search => (icons::mode::SEARCH, "SEARCH", theme.accent),
        },
        AppMode::Creating => (icons::mode::INSERT, "CREATE", theme.success),
        AppMode::Exporting => (icons::mode::COMMAND, "EXPORT", theme.info),
    };

    let mode = Paragraph::new(Line::from(vec![Span::styled(
        format!(" {} {} ", icon, label),
        Style::default()
            .fg(theme.bg)
            .bg(bg_color)
            .add_modifier(Modifier::BOLD),
    )]));

    frame.render_widget(mode, area);
}

/// Render the middle section (vault info, hints)
fn render_middle_section(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemePalette) {
    let mut spans = vec![Span::styled(" ", Style::default().bg(theme.bg_alt))];

    // Vault name if unlocked
    if let Some(ref vs) = state.vault_state {
        spans.push(Span::styled(
            format!("{} {} ", icons::ui::VAULT, vs.vault.name),
            Style::default().fg(theme.fg).bg(theme.bg_alt),
        ));

        // Dirty indicator
        if vs.is_dirty {
            spans.push(Span::styled(
                "[+] ",
                Style::default()
                    .fg(theme.warning)
                    .bg(theme.bg_alt)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Filter indicator
        if state.ui_state.filter.is_active() {
            spans.push(Span::styled(
                format!("{} ", icons::ui::TAG),
                Style::default().fg(theme.accent).bg(theme.bg_alt),
            ));
        }

        // Search indicator (show if search dialog is open)
        if let Some(FloatingWindow::Search {
            state: ref search_state,
        }) = state.ui_state.floating_window
        {
            if !search_state.query.is_empty() {
                spans.push(Span::styled(
                    format!("/{} ", search_state.query),
                    Style::default().fg(theme.accent).bg(theme.bg_alt),
                ));
            }
        }
    }

    // Keybinding hints
    let hints = get_context_hints(state);
    if !hints.is_empty() {
        spans.push(Span::styled(
            "│ ",
            Style::default().fg(theme.fg_muted).bg(theme.bg_alt),
        ));
        spans.push(Span::styled(
            hints,
            Style::default().fg(theme.fg_muted).bg(theme.bg_alt),
        ));
    }

    // Fill remaining space
    let total_len: usize = spans.iter().map(|s| s.content.len()).sum();
    if area.width as usize > total_len {
        spans.push(Span::styled(
            " ".repeat(area.width as usize - total_len),
            Style::default().bg(theme.bg_alt),
        ));
    }

    let middle = Paragraph::new(Line::from(spans));
    frame.render_widget(middle, area);
}

/// Render the right section (item count, position)
fn render_right_section(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemePalette) {
    let item_info = if let Some(ref vs) = state.vault_state {
        format!("{} items", vs.vault.items.len())
    } else {
        String::new()
    };

    let (icon, notif_color, notif_text) = match state.ui_state.notifications.last() {
        Some(notif) => {
            let (icon, color) = match notif.level {
                NotificationLevel::Info => (icons::ui::INFO, theme.info),
                NotificationLevel::Success => (icons::ui::SUCCESS, theme.success),
                NotificationLevel::Warning => (icons::ui::WARNING, theme.warning),
                NotificationLevel::Error => (icons::ui::ERROR, theme.error),
            };
            (icon, color, notif.message.as_str())
        }
        None => (icons::ui::INFO, theme.fg_muted, ""),
    };

    let mut right_spans = vec![Span::styled(" ", Style::default().bg(theme.bg_alt))];
    if notif_text.is_empty() {
        right_spans.push(Span::styled(
            item_info,
            Style::default().fg(theme.fg_muted).bg(theme.bg_alt),
        ));
    } else {
        let mut max_notif_chars = area.width.saturating_sub(6) as usize; // icon + spacing/padding
        if !item_info.is_empty() {
            max_notif_chars = max_notif_chars.saturating_sub(item_info.len() + 3); // " | " + item info
        }
        max_notif_chars = max_notif_chars.max(8);

        right_spans.push(Span::styled(
            format!("{} ", icon),
            Style::default()
                .fg(notif_color)
                .bg(theme.bg_alt)
                .add_modifier(Modifier::BOLD),
        ));
        right_spans.push(Span::styled(
            truncate_status_text(notif_text, max_notif_chars),
            Style::default().fg(notif_color).bg(theme.bg_alt),
        ));
        if !item_info.is_empty() {
            right_spans.push(Span::styled(
                format!(" | {}", item_info),
                Style::default().fg(theme.fg_muted).bg(theme.bg_alt),
            ));
        }
    }

    let right = Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right);

    frame.render_widget(right, area);
}

fn truncate_status_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Get context-sensitive keybinding hints
fn get_context_hints(state: &AppState) -> String {
    match state.mode {
        AppMode::Locked => "Enter:unlock n:new q:quit".to_string(),
        AppMode::Unlocked => {
            if state.ui_state.has_floating_window() {
                "Esc:close".to_string()
            } else {
                match state.ui_state.focused_pane {
                    Pane::List => "j/k:nav /:search n:new y:copy".to_string(),
                    Pane::Detail => "j/k:scroll r:reveal e:edit".to_string(),
                    Pane::Search => "Enter:select Esc:cancel".to_string(),
                }
            }
        }
        AppMode::Creating => "Enter:confirm Esc:cancel".to_string(),
        AppMode::Exporting => "Enter:export Esc:cancel".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppConfig, VaultRegistry};

    #[test]
    fn test_context_hints() {
        let config = AppConfig::default();
        let registry = VaultRegistry::default();
        let state = AppState::new(config, registry);

        let hints = get_context_hints(&state);
        assert!(hints.contains("unlock"));
    }
}
