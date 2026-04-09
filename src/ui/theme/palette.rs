//! Theme palette definitions

use ratatui::style::{Color, Modifier, Style};

/// Color palette for a theme
#[derive(Debug, Clone)]
pub struct ThemePalette {
    // Base colors
    pub bg: Color,
    pub bg_alt: Color,
    pub fg: Color,
    pub fg_muted: Color,

    // Accent colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,

    // Semantic colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // UI elements
    pub border: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,

    // Special
    pub sensitive_mask: Color,
}

impl ThemePalette {
    /// Get a style for normal text
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }

    /// Get a style for muted text
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.fg_muted).bg(self.bg)
    }

    /// Get a style for highlighted/selected items
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.selection_fg)
            .bg(self.selection_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get a style for borders
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Get a style for focused borders
    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    /// Get a style for primary accent elements
    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// Get a style for secondary elements
    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    /// Get a style for success messages
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Get a style for warning messages
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Get a style for error messages
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Get a style for info messages
    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// Get a style for masked/sensitive content
    pub fn sensitive_style(&self) -> Style {
        Style::default().fg(self.sensitive_mask)
    }

    /// Get a style for titles
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Get a style for the status line
    pub fn statusline_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg_alt)
    }
}

/// Trait for theme implementations
pub trait Theme {
    /// Get the theme name
    fn name(&self) -> &'static str;

    /// Get the color palette
    fn palette(&self) -> ThemePalette;

    /// Check if this is a light theme
    fn is_light(&self) -> bool {
        false
    }
}
