//! Main screen - the primary vault view with list and detail panes
//!
//! Layout: [List | Detail] with statusline at bottom.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
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
    let detail_focused = state.ui_state.focused_pane == Pane::Detail;
    item_detail::render(frame, content_chunks[1], state, detail_focused, theme);

    // Render statusline
    statusline::render(frame, main_chunks[1], state, theme);

    // Render floating windows if any
    // Clone the window to avoid borrow conflict
    if let Some(window) = state.ui_state.floating_window.clone() {
        render_floating_window(frame, area, state, &window, theme);
    }
}

/// Render floating windows
fn render_floating_window(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    window: &crate::app::FloatingWindow,
    theme: &ThemePalette,
) {
    use crate::app::FloatingWindow;
    use crate::input::mouse::{ClickRegion, ClickableElement};
    use crate::ui::widgets::{edit_form, kind_selector, search_dialog};
    use ratatui::{
        style::Style,
        text::Span,
        widgets::{Block, Borders, Clear, Paragraph},
    };

    // Calculate centered floating area (for simple windows)
    let width = area.width.min(60);
    let height = area.height.min(10);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let float_area = Rect::new(x, y, width, height);

    match window {
        FloatingWindow::Search {
            state: search_state,
        } => {
            let click_regions = search_dialog::render(frame, area, search_state, theme);

            // Register floating window region
            state.ui_state.register_region(
                crate::input::mouse::UiRegion::FloatingWindow,
                click_regions.dialog_area,
            );

            // Register clickable search results
            for (index, region) in click_regions.result_regions {
                state
                    .ui_state
                    .layout_regions
                    .register_clickable(region, ClickableElement::SearchResult(index));
            }
        }

        FloatingWindow::Help => {
            use crate::ui::widgets::help;
            help::render(frame, area, theme);
            // Help window just needs the floating region for close-on-outside
            state.ui_state.register_region(
                crate::input::mouse::UiRegion::FloatingWindow,
                ClickRegion::new(
                    (area.width.saturating_sub(70)) / 2,
                    (area.height.saturating_sub(20)) / 2,
                    area.width.min(70),
                    area.height.min(20),
                ),
            );
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

            let text = format!("Delete \"{}\"?", item_name);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(theme.error))
                .title(Span::styled(
                    " Confirm Delete ",
                    Style::default().fg(theme.error),
                ))
                .style(Style::default().bg(theme.bg));

            let inner = block.inner(float_area);
            frame.render_widget(block, float_area);

            // Split inner area for message and buttons
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(2),    // Message
                    Constraint::Length(1), // Buttons
                ])
                .split(inner);

            // Render message
            let paragraph = Paragraph::new(text)
                .style(Style::default().fg(theme.fg))
                .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(paragraph, chunks[0]);

            // Render buttons with embedded keyboard hints
            use crate::ui::widgets::{ButtonStyle, render_button_row};
            let buttons = vec![
                (
                    "confirm-delete".to_string(),
                    "Yes, Delete",
                    Some("y"),
                    ButtonStyle::Danger,
                ),
                (
                    "cancel-delete".to_string(),
                    "No, Cancel",
                    Some("Esc"),
                    ButtonStyle::Secondary,
                ),
            ];

            let button_regions = render_button_row(frame, chunks[1], &buttons, theme);

            // Register button regions
            for button_region in button_regions {
                state.ui_state.layout_regions.register_clickable(
                    button_region.region,
                    ClickableElement::Button(button_region.name),
                );
            }

            // Register floating window region
            state.ui_state.register_region(
                crate::input::mouse::UiRegion::FloatingWindow,
                ClickRegion::new(
                    float_area.x,
                    float_area.y,
                    float_area.width,
                    float_area.height,
                ),
            );
        }

        FloatingWindow::KindSelector {
            state: selector_state,
        } => {
            let click_regions = kind_selector::render(frame, area, selector_state, theme);

            // Register floating window region
            state.ui_state.register_region(
                crate::input::mouse::UiRegion::FloatingWindow,
                click_regions.popup_area,
            );

            // Register clickable kind options
            for (index, region) in click_regions.option_regions {
                state
                    .ui_state
                    .layout_regions
                    .register_clickable(region, ClickableElement::KindOption(index));
            }
        }

        FloatingWindow::NewItem { form } => {
            let click_regions = edit_form::render(frame, area, form, theme);

            // Register floating window region
            state.ui_state.register_region(
                crate::input::mouse::UiRegion::FloatingWindow,
                click_regions.form_area,
            );

            // Register clickable form fields
            for (index, region) in click_regions.field_regions {
                state
                    .ui_state
                    .layout_regions
                    .register_clickable(region, ClickableElement::FormField(index));
            }

            // Register form buttons
            for button_region in click_regions.button_regions {
                state.ui_state.layout_regions.register_clickable(
                    button_region.region,
                    ClickableElement::Button(button_region.name),
                );
            }
        }

        FloatingWindow::EditItem { form, .. } => {
            let click_regions = edit_form::render(frame, area, form, theme);

            // Register floating window region
            state.ui_state.register_region(
                crate::input::mouse::UiRegion::FloatingWindow,
                click_regions.form_area,
            );

            // Register clickable form fields
            for (index, region) in click_regions.field_regions {
                state
                    .ui_state
                    .layout_regions
                    .register_clickable(region, ClickableElement::FormField(index));
            }

            // Register form buttons
            for button_region in click_regions.button_regions {
                state.ui_state.layout_regions.register_clickable(
                    button_region.region,
                    ClickableElement::Button(button_region.name),
                );
            }
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

            state.ui_state.register_region(
                crate::input::mouse::UiRegion::FloatingWindow,
                ClickRegion::new(
                    float_area.x,
                    float_area.y,
                    float_area.width,
                    float_area.height,
                ),
            );
        }
    }
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
