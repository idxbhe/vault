//! Kind selector widget - for selecting item type when creating new items
//!
//! A popup menu showing available item kinds with icons.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::domain::ItemKind;
use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// State for kind selector
#[derive(Debug, Clone)]
pub struct KindSelectorState {
    /// Available kinds
    pub kinds: Vec<ItemKind>,
    /// Currently selected index
    pub selected: usize,
}

impl Default for KindSelectorState {
    fn default() -> Self {
        Self {
            kinds: vec![
                ItemKind::Password,
                ItemKind::CryptoSeed,
                ItemKind::ApiKey,
                ItemKind::SecureNote,
                ItemKind::Custom,
                ItemKind::Generic,
            ],
            selected: 0,
        }
    }
}

impl KindSelectorState {
    /// Move selection up
    pub fn prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn next(&mut self) {
        if self.selected < self.kinds.len() - 1 {
            self.selected += 1;
        }
    }

    /// Select by index (for mouse clicks)
    pub fn select(&mut self, index: usize) {
        if index < self.kinds.len() {
            self.selected = index;
        }
    }

    /// Get the currently selected kind
    pub fn selected_kind(&self) -> ItemKind {
        self.kinds[self.selected]
    }
}

/// Get icon for item kind
fn kind_icon(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Generic => icons::item::GENERIC,
        ItemKind::CryptoSeed => icons::item::CRYPTO,
        ItemKind::Password => icons::item::PASSWORD,
        ItemKind::SecureNote => icons::item::NOTE,
        ItemKind::ApiKey => icons::item::API_KEY,
        ItemKind::Totp => icons::item::TOTP,
        ItemKind::Custom => icons::item::CUSTOM,
    }
}

/// Clickable region info returned from render
#[derive(Debug, Clone)]
pub struct KindSelectorClickRegions {
    pub option_regions: Vec<(usize, crate::input::mouse::ClickRegion)>,
    pub popup_area: crate::input::mouse::ClickRegion,
}

/// Render the kind selector popup and return clickable regions
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &KindSelectorState,
    theme: &ThemePalette,
) -> KindSelectorClickRegions {
    // Calculate popup dimensions
    let popup_width = 35u16;
    let popup_height = (state.kinds.len() as u16 + 2).min(area.height - 4);
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear background
    frame.render_widget(Clear, popup_area);

    // Create list items
    let items: Vec<ListItem> = state
        .kinds
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            let icon = kind_icon(*kind);
            let name = kind.display_name();
            let style = if i == state.selected {
                Style::default()
                    .fg(theme.selection_fg)
                    .bg(theme.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", icon), style),
                Span::styled(name, style),
            ]))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            " Select Item Type ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.bg));

    let list = List::new(items)
        .block(block.clone())
        .highlight_style(Style::default());

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));

    frame.render_stateful_widget(list, popup_area, &mut list_state);

    // Calculate clickable regions for each kind option
    // Use block.inner() to get the exact inner area after borders and title
    let inner = block.inner(popup_area);

    let option_regions: Vec<(usize, crate::input::mouse::ClickRegion)> = state
        .kinds
        .iter()
        .enumerate()
        .map(|(i, _)| {
            (
                i,
                crate::input::mouse::ClickRegion::new(inner.x, inner.y + i as u16, inner.width, 1),
            )
        })
        .collect();

    KindSelectorClickRegions {
        option_regions,
        popup_area: crate::input::mouse::ClickRegion::new(
            popup_area.x,
            popup_area.y,
            popup_area.width,
            popup_area.height,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_navigation() {
        let mut state = KindSelectorState::default();
        assert_eq!(state.selected, 0);

        state.next();
        assert_eq!(state.selected, 1);

        state.next();
        state.next();
        state.next();
        state.next();
        assert_eq!(state.selected, 5);

        state.next(); // At end, should stay
        assert_eq!(state.selected, 5);

        state.prev();
        assert_eq!(state.selected, 4);
    }

    #[test]
    fn test_selected_kind() {
        let state = KindSelectorState::default();
        assert_eq!(state.selected_kind(), ItemKind::Password);
    }
}
