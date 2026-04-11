use ratatui::{
    Frame,
    layout::{Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::domain::ItemKind;
use crate::ui::theme::ThemePalette;

pub struct CategoryBarClickRegions {
    pub option_regions: Vec<(Option<ItemKind>, crate::input::mouse::ClickRegion)>,
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    selected_kind: Option<ItemKind>,
    theme: &ThemePalette,
) -> CategoryBarClickRegions {
    let mut option_regions = Vec::new();

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border));

    let inner_area = block.inner(area);

    let mut kinds = vec![None];
    kinds.extend(ItemKind::all().iter().map(|k| Some(*k)));

    let mut spans = Vec::new();
    let mut current_x = inner_area.x;

    for kind in kinds {
        let display_name = match kind {
            Some(k) => format!(" {} {} ", k.icon(), k.display_name()),
            None => " All ".to_string(),
        };

        let is_selected = kind == selected_kind;

        let style = if is_selected {
            Style::default()
                .fg(theme.selection_fg)
                .bg(theme.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg_muted)
        };

        let span_len = display_name.chars().count() as u16;

        option_regions.push((
            kind,
            crate::input::mouse::ClickRegion::new(
                current_x,
                inner_area.y,
                span_len,
                1,
            )
        ));

        spans.push(Span::styled(display_name, style));
        spans.push(Span::raw("  "));
        current_x += span_len + 2;
    }

    let hints_text = " <,> Prev | <.> Next ";
    let hints_span = Span::styled(hints_text, Style::default().fg(theme.fg_muted));

    let hints_len = hints_text.chars().count() as u16;
    let space_left = inner_area.width.saturating_sub(current_x - inner_area.x).saturating_sub(hints_len);

    if space_left > 0 {
        spans.push(Span::raw(" ".repeat(space_left as usize)));
    }
    spans.push(hints_span);

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);

    CategoryBarClickRegions { option_regions }
}
