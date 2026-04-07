//! Notification toast widget
//!
//! Shows temporary notifications/toasts at the top-right of the screen.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{Notification, NotificationLevel};
use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// Maximum width of notification toasts
const MAX_NOTIFICATION_WIDTH: u16 = 50;

/// Height per notification (including border)
const NOTIFICATION_HEIGHT: u16 = 3;

/// Maximum visible notifications
const MAX_VISIBLE_NOTIFICATIONS: usize = 3;

/// Render notifications as toasts
pub fn render(frame: &mut Frame, area: Rect, notifications: &[Notification], theme: &ThemePalette) {
    if notifications.is_empty() {
        return;
    }

    // Show at most MAX_VISIBLE_NOTIFICATIONS
    let visible: Vec<_> = notifications.iter().take(MAX_VISIBLE_NOTIFICATIONS).collect();

    // Position notifications at top-right corner
    let start_y = area.y + 1;
    let width = MAX_NOTIFICATION_WIDTH.min(area.width.saturating_sub(2));
    let start_x = area.x + area.width.saturating_sub(width + 1);

    for (i, notification) in visible.iter().enumerate() {
        let y = start_y + (i as u16 * NOTIFICATION_HEIGHT);
        if y + NOTIFICATION_HEIGHT > area.y + area.height {
            break;
        }

        let notification_area = Rect::new(start_x, y, width, NOTIFICATION_HEIGHT);
        render_notification(frame, notification_area, notification, theme);
    }
}

/// Render a single notification toast
fn render_notification(
    frame: &mut Frame,
    area: Rect,
    notification: &Notification,
    theme: &ThemePalette,
) {
    let (icon, border_color, bg_color) = match notification.level {
        NotificationLevel::Info => (icons::ui::INFO, theme.info, theme.bg_alt),
        NotificationLevel::Success => (icons::ui::SUCCESS, theme.success, theme.bg_alt),
        NotificationLevel::Warning => (icons::ui::WARNING, theme.warning, theme.bg_alt),
        NotificationLevel::Error => (icons::ui::ERROR, theme.error, theme.bg_alt),
    };

    // Clear the background
    frame.render_widget(Clear, area);

    // Create the notification block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg_color));

    // Truncate message if too long
    let max_msg_len = (area.width as usize).saturating_sub(6); // Account for icon, spaces, borders
    let message = if notification.message.len() > max_msg_len {
        format!("{}...", &notification.message[..max_msg_len.saturating_sub(3)])
    } else {
        notification.message.clone()
    };

    let content = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", icon),
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(message, Style::default().fg(theme.fg)),
    ]))
    .block(block)
    .alignment(Alignment::Left);

    frame.render_widget(content, area);
}

/// Calculate the area needed for notifications overlay
pub fn notification_area(frame_area: Rect, notification_count: usize) -> Rect {
    let count = notification_count.min(MAX_VISIBLE_NOTIFICATIONS);
    let height = (count as u16 * NOTIFICATION_HEIGHT).min(frame_area.height);
    let width = MAX_NOTIFICATION_WIDTH.min(frame_area.width);

    Rect::new(
        frame_area.x + frame_area.width.saturating_sub(width + 1),
        frame_area.y + 1,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_notification(level: NotificationLevel, message: &str) -> Notification {
        Notification {
            id: Uuid::new_v4(),
            message: message.to_string(),
            level,
            expires_at: Utc::now(),
        }
    }

    #[test]
    fn test_notification_area_calculation() {
        let frame_area = Rect::new(0, 0, 100, 50);

        let area = notification_area(frame_area, 1);
        assert_eq!(area.height, NOTIFICATION_HEIGHT);
        assert!(area.width <= MAX_NOTIFICATION_WIDTH);

        let area = notification_area(frame_area, 5);
        assert_eq!(area.height, 3 * NOTIFICATION_HEIGHT); // Capped at MAX_VISIBLE
    }

    #[test]
    fn test_notification_levels() {
        let info = create_notification(NotificationLevel::Info, "Info message");
        let success = create_notification(NotificationLevel::Success, "Success!");
        let warning = create_notification(NotificationLevel::Warning, "Warning...");
        let error = create_notification(NotificationLevel::Error, "Error occurred");

        assert!(matches!(info.level, NotificationLevel::Info));
        assert!(matches!(success.level, NotificationLevel::Success));
        assert!(matches!(warning.level, NotificationLevel::Warning));
        assert!(matches!(error.level, NotificationLevel::Error));
    }

    #[test]
    fn test_message_truncation() {
        let long_message = "A".repeat(100);
        let notification = create_notification(NotificationLevel::Info, &long_message);

        // Verify notification was created with full message
        assert_eq!(notification.message.len(), 100);
    }
}
