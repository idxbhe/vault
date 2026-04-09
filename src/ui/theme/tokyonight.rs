//! Tokyo Night theme variants
//! https://github.com/enkia/tokyo-night-vscode-theme

use ratatui::style::Color;

use super::palette::{Theme, ThemePalette};

/// Tokyo Night (dark theme)
pub struct TokyoNightNight;

impl Theme for TokyoNightNight {
    fn name(&self) -> &'static str {
        "Tokyo Night"
    }

    fn palette(&self) -> ThemePalette {
        ThemePalette {
            // Base colors
            bg: Color::Rgb(26, 27, 38),        // bg
            bg_alt: Color::Rgb(22, 22, 30),    // bg_dark
            fg: Color::Rgb(192, 202, 245),     // fg
            fg_muted: Color::Rgb(86, 95, 137), // comment

            // Accent colors
            primary: Color::Rgb(122, 162, 247),   // blue
            secondary: Color::Rgb(187, 154, 247), // purple
            accent: Color::Rgb(255, 158, 100),    // orange

            // Semantic colors
            success: Color::Rgb(158, 206, 106), // green
            warning: Color::Rgb(224, 175, 104), // yellow
            error: Color::Rgb(247, 118, 142),   // red
            info: Color::Rgb(125, 207, 255),    // cyan

            // UI elements
            border: Color::Rgb(41, 46, 66),            // bg_highlight
            border_focused: Color::Rgb(122, 162, 247), // blue
            selection_bg: Color::Rgb(52, 59, 88),      // bg_visual
            selection_fg: Color::Rgb(192, 202, 245),   // fg

            // Special
            sensitive_mask: Color::Rgb(86, 95, 137), // comment
        }
    }
}

/// Tokyo Night Storm (slightly lighter dark theme)
pub struct TokyoNightStorm;

impl Theme for TokyoNightStorm {
    fn name(&self) -> &'static str {
        "Tokyo Night Storm"
    }

    fn palette(&self) -> ThemePalette {
        ThemePalette {
            // Base colors
            bg: Color::Rgb(36, 40, 59),        // bg
            bg_alt: Color::Rgb(31, 35, 53),    // bg_dark
            fg: Color::Rgb(192, 202, 245),     // fg
            fg_muted: Color::Rgb(86, 95, 137), // comment

            // Accent colors
            primary: Color::Rgb(122, 162, 247),   // blue
            secondary: Color::Rgb(187, 154, 247), // purple
            accent: Color::Rgb(255, 158, 100),    // orange

            // Semantic colors
            success: Color::Rgb(158, 206, 106), // green
            warning: Color::Rgb(224, 175, 104), // yellow
            error: Color::Rgb(247, 118, 142),   // red
            info: Color::Rgb(125, 207, 255),    // cyan

            // UI elements
            border: Color::Rgb(59, 66, 97),            // bg_highlight
            border_focused: Color::Rgb(122, 162, 247), // blue
            selection_bg: Color::Rgb(63, 71, 106),     // bg_visual
            selection_fg: Color::Rgb(192, 202, 245),   // fg

            // Special
            sensitive_mask: Color::Rgb(86, 95, 137), // comment
        }
    }
}

/// Tokyo Night Day (light theme)
pub struct TokyoNightDay;

impl Theme for TokyoNightDay {
    fn name(&self) -> &'static str {
        "Tokyo Night Day"
    }

    fn is_light(&self) -> bool {
        true
    }

    fn palette(&self) -> ThemePalette {
        ThemePalette {
            // Base colors
            bg: Color::Rgb(212, 216, 232),       // bg
            bg_alt: Color::Rgb(199, 203, 219),   // bg_dark
            fg: Color::Rgb(59, 66, 97),          // fg
            fg_muted: Color::Rgb(149, 157, 193), // comment

            // Accent colors
            primary: Color::Rgb(52, 84, 138),   // blue
            secondary: Color::Rgb(92, 75, 163), // purple
            accent: Color::Rgb(150, 84, 0),     // orange

            // Semantic colors
            success: Color::Rgb(56, 113, 62), // green
            warning: Color::Rgb(143, 111, 0), // yellow
            error: Color::Rgb(143, 76, 90),   // red
            info: Color::Rgb(0, 110, 128),    // cyan

            // UI elements
            border: Color::Rgb(175, 180, 200),       // bg_highlight
            border_focused: Color::Rgb(52, 84, 138), // blue
            selection_bg: Color::Rgb(153, 158, 182), // bg_visual
            selection_fg: Color::Rgb(59, 66, 97),    // fg

            // Special
            sensitive_mask: Color::Rgb(149, 157, 193), // comment
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokyonight_themes() {
        assert!(!TokyoNightNight.is_light());
        assert!(!TokyoNightStorm.is_light());
        assert!(TokyoNightDay.is_light());

        assert_eq!(TokyoNightNight.name(), "Tokyo Night");

        let palette = TokyoNightNight.palette();
        assert!(matches!(palette.primary, Color::Rgb(122, 162, 247)));
    }
}
