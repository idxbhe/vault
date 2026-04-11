use ratatui::{
    Frame,
    layout::{Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::domain::ItemKind;
use crate::ui::theme::ThemePalette;
use crate::input::mouse::ClickRegion;

pub struct CategoryBarClickRegions {
    pub option_regions: Vec<(Option<ItemKind>, ClickRegion)>,
    pub scroll_left: Option<ClickRegion>,
    pub scroll_right: Option<ClickRegion>,
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    selected_kind: Option<ItemKind>,
    scroll_offset: &mut u16,
    theme: &ThemePalette,
) -> CategoryBarClickRegions {
    let mut option_regions = Vec::new();
    let mut scroll_left = None;
    let mut scroll_right = None;

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border));

    let inner_area = block.inner(area);
    if inner_area.width == 0 {
        return CategoryBarClickRegions {
            option_regions,
            scroll_left,
            scroll_right,
        };
    }

    let mut kinds = vec![None];
    kinds.extend(ItemKind::all().iter().map(|k| Some(*k)));

    // Generate formatted names and their widths
    let mut items = Vec::new();
    let mut total_width = 0;

    for kind in &kinds {
        let display_name = match kind {
            Some(k) => format!(" {} {} ", k.icon(), k.display_name()),
            None => " All ".to_string(),
        };
        // Add 2 spaces after each item except the last one (but we'll just add it to all for simplicity and adjust later)
        let width = display_name.chars().count() as u16 + 2;
        items.push((kind.clone(), display_name, total_width, width));
        total_width += width;
    }

    // Auto-scroll logic
    // We want to ensure the selected item is fully visible.
    // If scrolling is needed, we have `<` and `>` buttons which take 3 characters each.
    if let Some((_, _, start_x, width)) = items.iter().find(|(k, _, _, _)| *k == selected_kind) {
        let left_padding = if *scroll_offset > 0 { 3 } else { 0 };
        // Assume right padding might be needed if total width > inner_area.width
        let right_padding = if total_width > inner_area.width { 3 } else { 0 };
        let visible_window_width = inner_area.width.saturating_sub(left_padding + right_padding);

        if *start_x < *scroll_offset {
            *scroll_offset = *start_x;
        } else if *start_x + *width > *scroll_offset + visible_window_width {
            *scroll_offset = (*start_x + *width).saturating_sub(visible_window_width);
        }
    }

    // Bound check scroll_offset
    let max_scroll = total_width.saturating_sub(inner_area.width.saturating_sub(3)); // allow space for `<` if scrolled to end
    if *scroll_offset > max_scroll {
        *scroll_offset = max_scroll;
    }
    if total_width <= inner_area.width {
        *scroll_offset = 0;
    }

    let draw_left = *scroll_offset > 0;
    // draw_right if there is more content beyond what's visible
    let mut available_width = inner_area.width;
    if draw_left {
        available_width = available_width.saturating_sub(3);
    }
    let draw_right = total_width > *scroll_offset + available_width;
    if draw_right {
        available_width = available_width.saturating_sub(3);
    }
    // Recheck left because reducing width for right might require a left shift, but let's assume it's fine.

    let mut spans = Vec::new();
    let mut current_x = inner_area.x;

    if draw_left {
        spans.push(Span::styled(" < ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
        scroll_left = Some(ClickRegion::new(current_x, inner_area.y, 3, 1));
        current_x += 3;
    }

    let mut remaining_width = available_width;

    for (kind, display_name, item_start, width) in items {
        let item_end = item_start + width;

        // Skip items that are completely to the left of the scroll window
        if item_end <= *scroll_offset {
            continue;
        }

        // Break if we have no more width
        if remaining_width == 0 {
            break;
        }

        let mut item_chars: Vec<char> = display_name.chars().chain(std::iter::repeat(' ').take(2)).collect();

        // Left truncation if item crosses the left boundary of the scroll window
        if item_start < *scroll_offset {
            let chop = (*scroll_offset - item_start) as usize;
            if chop < item_chars.len() {
                item_chars.drain(0..chop);
            } else {
                item_chars.clear();
            }
        }

        // Right truncation if item crosses the right boundary of the visible area
        if item_chars.len() as u16 > remaining_width {
            item_chars.truncate(remaining_width as usize);
        }

        if item_chars.is_empty() {
            continue;
        }

        let visible_str: String = item_chars.into_iter().collect();
        let visible_len = visible_str.chars().count() as u16;

        let is_selected = kind == selected_kind;
        let style = if is_selected {
            Style::default()
                .fg(theme.selection_fg)
                .bg(theme.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg_muted)
        };

        spans.push(Span::styled(visible_str, style));

        option_regions.push((
            kind,
            ClickRegion::new(
                current_x,
                inner_area.y,
                visible_len,
                1,
            )
        ));

        current_x += visible_len;
        remaining_width -= visible_len;
    }

    if draw_right {
        if remaining_width > 0 {
            spans.push(Span::raw(" ".repeat(remaining_width as usize)));
            current_x += remaining_width;
        }
        spans.push(Span::styled(" > ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
        scroll_right = Some(ClickRegion::new(current_x, inner_area.y, 3, 1));
    }

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);

    CategoryBarClickRegions { option_regions, scroll_left, scroll_right }
}
