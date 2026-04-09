//! Search dialog widget - fuzzy search with results list
//!
//! A floating search dialog showing query input and matching results.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use uuid::Uuid;

use crate::domain::Item;
use crate::ui::theme::ThemePalette;
use crate::utils::{fuzzy, icons};

/// State for the search dialog
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Current search query
    pub query: String,
    /// Cursor position in query
    pub cursor: usize,
    /// Search results (item IDs with scores)
    pub results: Vec<SearchResultEntry>,
    /// Selected result index
    pub selected: usize,
}

/// A search result entry
#[derive(Debug, Clone)]
pub struct SearchResultEntry {
    pub item_id: Uuid,
    pub title: String,
    pub kind_icon: &'static str,
    pub score: u16,
}

impl SearchState {
    /// Create a new empty search state
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert character at cursor
    pub fn insert(&mut self, c: char) {
        self.query.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let prev = self.query[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.query.remove(prev);
            self.cursor = prev;
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.query[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor < self.query.len() {
            self.cursor = self.query[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.query.len());
        }
    }

    /// Select next result
    pub fn next_result(&mut self) {
        if !self.results.is_empty() && self.selected < self.results.len() - 1 {
            self.selected += 1;
        }
    }

    /// Select previous result  
    pub fn prev_result(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Get the selected item ID
    pub fn selected_item_id(&self) -> Option<Uuid> {
        self.results.get(self.selected).map(|r| r.item_id)
    }

    /// Update search results from items
    pub fn update_results(&mut self, items: &[Item]) {
        self.results.clear();
        self.selected = 0;

        if self.query.is_empty() {
            return;
        }

        // Search through items
        let search_results = fuzzy::search(items, &self.query, |item| &item.title);

        self.results = search_results
            .into_iter()
            .take(10) // Limit results
            .map(|r| SearchResultEntry {
                item_id: r.item.id,
                title: r.item.title.clone(),
                kind_icon: r.item.kind.icon(),
                score: r.score,
            })
            .collect();
    }

    /// Clear the search state
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor = 0;
        self.results.clear();
        self.selected = 0;
    }
}

/// Clickable region info returned from render
#[derive(Debug, Clone)]
pub struct SearchClickRegions {
    pub result_regions: Vec<(usize, crate::input::mouse::ClickRegion)>,
    pub dialog_area: crate::input::mouse::ClickRegion,
}

/// Render the search dialog and return clickable regions
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &SearchState,
    theme: &ThemePalette,
) -> SearchClickRegions {
    // Calculate dialog dimensions
    let dialog_width = area.width.min(60);
    let dialog_height = (state.results.len() as u16 + 5).min(area.height - 4).max(7);
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 3; // Slightly higher

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear background
    frame.render_widget(Clear, dialog_area);

    // Main block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            format!(" {} Search ", icons::ui::SEARCH),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Layout: input + results
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field
            Constraint::Min(1),    // Results
        ])
        .split(inner);

    // Render search input
    render_input(frame, chunks[0], state, theme);

    // Render results and collect clickable regions
    let result_regions = render_results(frame, chunks[1], state, theme);

    SearchClickRegions {
        result_regions,
        dialog_area: crate::input::mouse::ClickRegion::new(
            dialog_area.x,
            dialog_area.y,
            dialog_area.width,
            dialog_area.height,
        ),
    }
}

/// Render the search input field
fn render_input(frame: &mut Frame, area: Rect, state: &SearchState, theme: &ThemePalette) {
    // Build input text with cursor
    let before: String = state.query.chars().take(state.cursor).collect();
    let after: String = state.query.chars().skip(state.cursor).collect();
    let text = format!("{}│{}", before, after);

    let results_text = if state.results.is_empty() && !state.query.is_empty() {
        " (no matches)"
    } else if !state.results.is_empty() {
        ""
    } else {
        ""
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(theme.fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_focused))
                .title(Span::styled(
                    format!(" Query{} ", results_text),
                    Style::default().fg(theme.fg_muted),
                )),
        );

    frame.render_widget(paragraph, area);
}

/// Render search results list and return clickable regions
fn render_results(
    frame: &mut Frame,
    area: Rect,
    state: &SearchState,
    theme: &ThemePalette,
) -> Vec<(usize, crate::input::mouse::ClickRegion)> {
    if state.results.is_empty() {
        let hint = if state.query.is_empty() {
            "Type to search..."
        } else {
            "No items found"
        };

        let paragraph = Paragraph::new(hint)
            .style(Style::default().fg(theme.fg_muted))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
        return Vec::new();
    }

    let items: Vec<ListItem> = state
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_selected = i == state.selected;
            let style = if is_selected {
                Style::default()
                    .fg(theme.selection_fg)
                    .bg(theme.selection_bg)
            } else {
                Style::default().fg(theme.fg)
            };

            let line = Line::from(vec![
                Span::styled(if is_selected { " ▸ " } else { "   " }, style),
                Span::styled(format!("{} ", result.kind_icon), style),
                Span::styled(
                    &result.title,
                    style.add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, area);

    // Return clickable regions for each result
    // Note: List has no block/borders here, area is already the exact content area
    state
        .results
        .iter()
        .enumerate()
        .map(|(i, _)| {
            (
                i,
                crate::input::mouse::ClickRegion::new(area.x, area.y + i as u16, area.width, 1),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_state_input() {
        let mut state = SearchState::new();

        state.insert('t');
        state.insert('e');
        state.insert('s');
        state.insert('t');

        assert_eq!(state.query, "test");
        assert_eq!(state.cursor, 4);

        state.backspace();
        assert_eq!(state.query, "tes");
    }

    #[test]
    fn test_search_state_navigation() {
        let mut state = SearchState::new();
        state.results = vec![
            SearchResultEntry {
                item_id: Uuid::new_v4(),
                title: "Item 1".to_string(),
                kind_icon: "",
                score: 100,
            },
            SearchResultEntry {
                item_id: Uuid::new_v4(),
                title: "Item 2".to_string(),
                kind_icon: "",
                score: 90,
            },
        ];

        assert_eq!(state.selected, 0);
        state.next_result();
        assert_eq!(state.selected, 1);
        state.next_result(); // At end
        assert_eq!(state.selected, 1);
        state.prev_result();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_update_results() {
        use crate::domain::Item;

        let mut state = SearchState::new();
        let items = vec![
            Item::password("Bitcoin Wallet", "secret"),
            Item::password("Ethereum Keys", "secret"),
            Item::password("Bank Account", "secret"),
        ];

        state.query = "bit".to_string();
        state.update_results(&items);

        assert!(!state.results.is_empty());
        assert!(state.results[0].title.to_lowercase().contains("bit"));
    }
}
