//! Main screen - the primary vault view with list and detail panes
//!
//! Layout: [List | Detail] with statusline at bottom.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::{AppState, Pane};
use crate::ui::theme::ThemePalette;
use crate::ui::widgets::{item_detail, item_list, statusline};

/// State for the main screen
#[derive(Debug, Default)]
pub struct MainScreen {
    /// Item list widget state
    pub list_state: item_list::ItemListState,
}

impl MainScreen {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Render the main screen
pub fn render(
    frame: &mut Frame,
    state: &mut AppState,
    screen_state: &mut MainScreen,
    theme: &ThemePalette,
) {
    let area = frame.area();

    // Main layout: content area + statusline
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Statusline
        ])
        .split(area);

    // Content layout: list | detail
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // List
            Constraint::Percentage(65), // Detail
        ])
        .split(main_chunks[0]);

    // Register pane regions for mouse clicks
    state.ui_state.register_region(
        crate::input::mouse::UiRegion::List,
        crate::input::mouse::ClickRegion::new(
            content_chunks[0].x,
            content_chunks[0].y,
            content_chunks[0].width,
            content_chunks[0].height,
        ),
    );
    state.ui_state.register_region(
        crate::input::mouse::UiRegion::Detail,
        crate::input::mouse::ClickRegion::new(
            content_chunks[1].x,
            content_chunks[1].y,
            content_chunks[1].width,
            content_chunks[1].height,
        ),
    );

    // Render list pane
    let is_focused = state.ui_state.focused_pane == Pane::List;
    item_list::render(
        frame,
        content_chunks[0],
        state,
        &mut screen_state.list_state,
        is_focused,
        theme,
    );

    // Render detail pane
    item_detail::render(
        frame,
        content_chunks[1],
        state,
        state.ui_state.focused_pane == Pane::Detail,
        theme,
    );

    // Render statusline
    statusline::render(frame, main_chunks[1], state, theme);

    // Render floating windows if any
    if let Some(ref window) = state.ui_state.floating_window {
        render_floating_window(frame, area, state, window, theme);
    }

    // Render notifications
    render_notifications(frame, area, state, theme);
}

/// Render floating windows
fn render_floating_window(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    window: &crate::app::FloatingWindow,
    theme: &ThemePalette,
) {
    use ratatui::{
        style::Style,
        text::Span,
        widgets::{Block, Borders, Clear, Paragraph},
    };
    use crate::app::FloatingWindow;
    use crate::ui::widgets::{edit_form, kind_selector, search_dialog};

    // Calculate centered floating area
    let width = area.width.min(60);
    let height = area.height.min(10);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let float_area = Rect::new(x, y, width, height);

    match window {
        FloatingWindow::Search { state: search_state } => {
            search_dialog::render(frame, area, search_state, theme);
        }

        FloatingWindow::Help => {
            use crate::ui::widgets::help;
            help::render(frame, area, theme);
        }

        FloatingWindow::ConfirmDelete { item_id } => {
            // Clear background
            frame.render_widget(Clear, float_area);
            
            let item_name = state
                .vault_state
                .as_ref()
                .and_then(|vs| vs.vault.get_item(*item_id))
                .map(|i| i.title.as_str())
                .unwrap_or("item");

            let text = format!(
                "Delete \"{}\"?\n\n  [y] Yes, delete    [n] No, cancel",
                item_name
            );

            let paragraph = Paragraph::new(text)
                .style(Style::default().fg(theme.fg))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::default().fg(theme.error))
                        .title(Span::styled(
                            " Confirm Delete ",
                            Style::default().fg(theme.error),
                        ))
                        .style(Style::default().bg(theme.bg)),
                );

            frame.render_widget(paragraph, float_area);
        }

        FloatingWindow::KindSelector { state: selector_state } => {
            kind_selector::render(frame, area, selector_state, theme);
        }

        FloatingWindow::NewItem { form } => {
            edit_form::render(frame, area, form, theme);
        }

        FloatingWindow::EditItem { form, .. } => {
            edit_form::render(frame, area, form, theme);
        }

        FloatingWindow::TagFilter => {
            // Clear background
            frame.render_widget(Clear, float_area);
            
            let paragraph = Paragraph::new("Tag filter coming soon...")
                .style(Style::default().fg(theme.fg_muted))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .border_style(Style::default().fg(theme.border))
                        .title(Span::styled(
                            " Filter by Tag ",
                            Style::default().fg(theme.accent),
                        ))
                        .style(Style::default().bg(theme.bg)),
                );

            frame.render_widget(paragraph, float_area);
        }
    }
}

/// Render notifications in the top-right corner
fn render_notifications(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &ThemePalette,
) {
    use crate::ui::widgets::notification;
    notification::render(frame, area, &state.ui_state.notifications, theme);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_screen_creation() {
        let screen = MainScreen::new();
        assert!(screen.list_state.visible_items.is_empty());
    }
}
