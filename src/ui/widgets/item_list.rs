//! Item list widget - displays vault items with filtering and selection
//!
//! A scrollable list of items with icons, favorites, and tag indicators.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use uuid::Uuid;

use crate::app::{AppState, FilterState};
use crate::domain::{Item, ItemKind};
use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// State for the item list widget
#[derive(Debug, Default)]
pub struct ItemListState {
    /// List widget state for scrolling
    pub list_state: ListState,
    /// Currently visible items (after filtering)
    pub visible_items: Vec<Uuid>,
}

impl ItemListState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update visible items based on filter
    pub fn update_visible(&mut self, items: &[Item], filter: &FilterState, search_query: &str) {
        self.visible_items = items
            .iter()
            .filter(|item| {
                // Apply filters
                if let Some(kind) = filter.kind {
                    if item.kind != kind {
                        return false;
                    }
                }
                if !filter.tags.is_empty() && !filter.tags.iter().any(|t| item.tags.contains(t)) {
                    return false;
                }
                if filter.favorites_only && !item.favorite {
                    return false;
                }

                // Apply search
                if !search_query.is_empty() {
                    let query = search_query.to_lowercase();
                    let title_match = item.title.to_lowercase().contains(&query);
                    let notes_match = item
                        .notes
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&query))
                        .unwrap_or(false);
                    if !title_match && !notes_match {
                        return false;
                    }
                }

                true
            })
            .map(|i| i.id)
            .collect();
    }

    /// Select an item by ID
    pub fn select(&mut self, id: Uuid) {
        if let Some(idx) = self.visible_items.iter().position(|i| *i == id) {
            self.list_state.select(Some(idx));
        }
    }

    /// Get the currently selected item ID
    pub fn selected(&self) -> Option<Uuid> {
        self.list_state
            .selected()
            .and_then(|idx| self.visible_items.get(idx).copied())
    }

    /// Select next item
    pub fn select_next(&mut self) {
        if self.visible_items.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.visible_items.len() - 1);
        self.list_state.select(Some(next));
    }

    /// Select previous item
    pub fn select_prev(&mut self) {
        if self.visible_items.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let prev = current.saturating_sub(1);
        self.list_state.select(Some(prev));
    }

    /// Select first item
    pub fn select_first(&mut self) {
        if !self.visible_items.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Select last item
    pub fn select_last(&mut self) {
        if !self.visible_items.is_empty() {
            self.list_state.select(Some(self.visible_items.len() - 1));
        }
    }
}

/// Render the item list
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    list_state: &mut ItemListState,
    focused: bool,
    theme: &ThemePalette,
) {
    let Some(ref vault_state) = state.vault_state else {
        render_empty(frame, area, "No vault loaded", theme);
        return;
    };

    // Update visible items (no search filter at list level - search uses floating dialog)
    list_state.update_visible(
        &vault_state.vault.items,
        &state.ui_state.filter,
        "", // Search is now handled via floating window
    );

    // Sync selection with vault state
    if let Some(id) = vault_state.selected_item_id {
        list_state.select(id);
    }

    if list_state.visible_items.is_empty() {
        let msg = if state.ui_state.filter.is_active() {
            "No items match filter"
        } else {
            "No items yet. Press 'n' to create one."
        };
        render_empty(frame, area, msg, theme);
        return;
    }

    // Build list items
    let items: Vec<ListItem> = list_state
        .visible_items
        .iter()
        .filter_map(|id| state.vault_state.as_ref().and_then(|vs| vs.vault.get_item(*id)))
        .enumerate()
        .map(|(idx, item)| {
            let selected = list_state.list_state.selected() == Some(idx);
            create_list_item(item, selected, &state.vault_state.as_ref().unwrap().vault.tags, theme)
        })
        .collect();

    // Build block with title
    let vault_name = state.vault_state.as_ref().map(|vs| vs.vault.name.as_str()).unwrap_or("Vault");
    let title = build_title(vault_name, list_state.visible_items.len(), theme);
    let border_color = if focused {
        theme.border_focused
    } else {
        theme.border
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let list = List::new(items)
        .block(block.clone())
        .highlight_style(theme.selected_style())
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut list_state.list_state);
    
    // Register clickable elements for each visible item
    // Use block.inner() to get the exact inner area after borders and title
    let inner = block.inner(area);
    
    for (i, item_id) in list_state.visible_items.iter().enumerate() {
        let item_y = inner.y + i as u16;
        if item_y < inner.y + inner.height { // Stay within inner bounds
            state.ui_state.layout_regions.register_clickable(
                crate::input::mouse::ClickRegion::new(inner.x, item_y, inner.width, 1),
                crate::input::mouse::ClickableElement::ListItem(*item_id),
            );
        }
    }
}

/// Create a list item for an item
fn create_list_item<'a>(
    item: &Item,
    _selected: bool,
    _tags: &[crate::domain::Tag],
    theme: &ThemePalette,
) -> ListItem<'a> {
    let icon = get_item_icon(item.kind);
    let fav_icon = if item.favorite {
        icons::ui::STAR
    } else {
        ""
    };

    let mut spans = vec![
        Span::styled(
            format!("{} ", icon),
            Style::default().fg(theme.accent),
        ),
        Span::styled(item.title.clone(), Style::default().fg(theme.fg)),
    ];

    // Add favorite indicator
    if !fav_icon.is_empty() {
        spans.push(Span::styled(
            format!(" {}", fav_icon),
            Style::default().fg(theme.warning),
        ));
    }

    // Add tag count indicator if has tags
    if !item.tags.is_empty() {
        spans.push(Span::styled(
            format!(" {} {}", icons::ui::TAG, item.tags.len()),
            Style::default().fg(theme.fg_muted),
        ));
    }

    ListItem::new(Line::from(spans))
}

/// Get icon for item kind
fn get_item_icon(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Generic => icons::item::GENERIC,
        ItemKind::CryptoSeed => icons::item::CRYPTO_SEED,
        ItemKind::Password => icons::item::PASSWORD,
        ItemKind::SecureNote => icons::item::SECURE_NOTE,
        ItemKind::ApiKey => icons::item::API_KEY,
    }
}

/// Build the title spans
fn build_title<'a>(vault_name: &str, count: usize, theme: &ThemePalette) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!(" {} ", icons::ui::VAULT),
            Style::default().fg(theme.accent),
        ),
        Span::styled(
            vault_name.to_string(),
            Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({}) ", count),
            Style::default().fg(theme.fg_muted),
        ),
    ])
}

/// Render empty state
fn render_empty(frame: &mut Frame, area: Rect, message: &str, theme: &ThemePalette) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Items ",
            Style::default().fg(theme.accent),
        ));

    let paragraph = ratatui::widgets::Paragraph::new(message)
        .style(Style::default().fg(theme.fg_muted))
        .alignment(ratatui::layout::Alignment::Center)
        .block(block);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Item;

    #[test]
    fn test_item_list_navigation() {
        let mut state = ItemListState::new();
        let items = vec![
            Item::password("Item 1", "pass1"),
            Item::password("Item 2", "pass2"),
            Item::password("Item 3", "pass3"),
        ];

        state.update_visible(&items, &FilterState::default(), "");
        assert_eq!(state.visible_items.len(), 3);

        state.select_first();
        assert_eq!(state.list_state.selected(), Some(0));

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(1));

        state.select_last();
        assert_eq!(state.list_state.selected(), Some(2));

        state.select_prev();
        assert_eq!(state.list_state.selected(), Some(1));
    }

    #[test]
    fn test_item_list_filtering() {
        let mut state = ItemListState::new();
        let items = vec![
            Item::password("GitHub", "pass1"),
            Item::password("GitLab", "pass2"),
            Item::crypto_seed("Bitcoin", "seed words"),
        ];

        // No filter - all items
        state.update_visible(&items, &FilterState::default(), "");
        assert_eq!(state.visible_items.len(), 3);

        // Search filter
        state.update_visible(&items, &FilterState::default(), "git");
        assert_eq!(state.visible_items.len(), 2);

        // Kind filter
        let mut filter = FilterState::default();
        filter.kind = Some(ItemKind::CryptoSeed);
        state.update_visible(&items, &filter, "");
        assert_eq!(state.visible_items.len(), 1);
    }
}
