//! Catppuccin theme variants
//! https://github.com/catppuccin/catppuccin

use ratatui::style::Color;

use super::palette::{Theme, ThemePalette};

/// Catppuccin Latte (light theme)
pub struct CatppuccinLatte;

impl Theme for CatppuccinLatte {
    fn name(&self) -> &'static str {
        "Catppuccin Latte"
    }

    fn is_light(&self) -> bool {
        true
    }

    fn palette(&self) -> ThemePalette {
        ThemePalette {
            // Base colors
            bg: Color::Rgb(239, 241, 245),       // Base
            bg_alt: Color::Rgb(230, 233, 239),   // Mantle
            fg: Color::Rgb(76, 79, 105),         // Text
            fg_muted: Color::Rgb(140, 143, 161), // Subtext0

            // Accent colors
            primary: Color::Rgb(30, 102, 245),   // Blue
            secondary: Color::Rgb(136, 57, 239), // Mauve
            accent: Color::Rgb(220, 138, 120),   // Peach

            // Semantic colors
            success: Color::Rgb(64, 160, 43),  // Green
            warning: Color::Rgb(223, 142, 29), // Yellow
            error: Color::Rgb(210, 15, 57),    // Red
            info: Color::Rgb(32, 159, 181),    // Teal

            // UI elements
            border: Color::Rgb(188, 192, 204),        // Surface1
            border_focused: Color::Rgb(30, 102, 245), // Blue
            selection_bg: Color::Rgb(188, 192, 204),  // Surface1
            selection_fg: Color::Rgb(76, 79, 105),    // Text

            // Special
            sensitive_mask: Color::Rgb(140, 143, 161), // Subtext0
        }
    }
}

/// Catppuccin Frappé (medium-light dark theme)
pub struct CatppuccinFrappe;

impl Theme for CatppuccinFrappe {
    fn name(&self) -> &'static str {
        "Catppuccin Frappé"
    }

    fn palette(&self) -> ThemePalette {
        ThemePalette {
            // Base colors
            bg: Color::Rgb(48, 52, 70),          // Base
            bg_alt: Color::Rgb(41, 44, 60),      // Mantle
            fg: Color::Rgb(198, 208, 245),       // Text
            fg_muted: Color::Rgb(165, 173, 206), // Subtext0

            // Accent colors
            primary: Color::Rgb(140, 170, 238),   // Blue
            secondary: Color::Rgb(202, 158, 230), // Mauve
            accent: Color::Rgb(239, 159, 118),    // Peach

            // Semantic colors
            success: Color::Rgb(166, 209, 137), // Green
            warning: Color::Rgb(229, 200, 144), // Yellow
            error: Color::Rgb(231, 130, 132),   // Red
            info: Color::Rgb(129, 200, 190),    // Teal

            // UI elements
            border: Color::Rgb(81, 87, 109),           // Surface1
            border_focused: Color::Rgb(140, 170, 238), // Blue
            selection_bg: Color::Rgb(81, 87, 109),     // Surface1
            selection_fg: Color::Rgb(198, 208, 245),   // Text

            // Special
            sensitive_mask: Color::Rgb(115, 121, 148), // Overlay0
        }
    }
}

/// Catppuccin Macchiato (medium-dark theme)
pub struct CatppuccinMacchiato;

impl Theme for CatppuccinMacchiato {
    fn name(&self) -> &'static str {
        "Catppuccin Macchiato"
    }

    fn palette(&self) -> ThemePalette {
        ThemePalette {
            // Base colors
            bg: Color::Rgb(36, 39, 58),          // Base
            bg_alt: Color::Rgb(30, 32, 48),      // Mantle
            fg: Color::Rgb(202, 211, 245),       // Text
            fg_muted: Color::Rgb(165, 173, 203), // Subtext0

            // Accent colors
            primary: Color::Rgb(138, 173, 244),   // Blue
            secondary: Color::Rgb(198, 160, 246), // Mauve
            accent: Color::Rgb(245, 169, 127),    // Peach

            // Semantic colors
            success: Color::Rgb(166, 218, 149), // Green
            warning: Color::Rgb(238, 212, 159), // Yellow
            error: Color::Rgb(237, 135, 150),   // Red
            info: Color::Rgb(139, 213, 202),    // Teal

            // UI elements
            border: Color::Rgb(73, 77, 100),           // Surface1
            border_focused: Color::Rgb(138, 173, 244), // Blue
            selection_bg: Color::Rgb(73, 77, 100),     // Surface1
            selection_fg: Color::Rgb(202, 211, 245),   // Text

            // Special
            sensitive_mask: Color::Rgb(110, 115, 141), // Overlay0
        }
    }
}

/// Catppuccin Mocha (dark theme) - Default
pub struct CatppuccinMocha;

impl Theme for CatppuccinMocha {
    fn name(&self) -> &'static str {
        "Catppuccin Mocha"
    }

    fn palette(&self) -> ThemePalette {
        ThemePalette {
            // Base colors
            bg: Color::Rgb(30, 30, 46),          // Base
            bg_alt: Color::Rgb(24, 24, 37),      // Mantle
            fg: Color::Rgb(205, 214, 244),       // Text
            fg_muted: Color::Rgb(166, 173, 200), // Subtext0

            // Accent colors
            primary: Color::Rgb(137, 180, 250),   // Blue
            secondary: Color::Rgb(203, 166, 247), // Mauve
            accent: Color::Rgb(250, 179, 135),    // Peach

            // Semantic colors
            success: Color::Rgb(166, 227, 161), // Green
            warning: Color::Rgb(249, 226, 175), // Yellow
            error: Color::Rgb(243, 139, 168),   // Red
            info: Color::Rgb(148, 226, 213),    // Teal

            // UI elements
            border: Color::Rgb(69, 71, 90),            // Surface1
            border_focused: Color::Rgb(137, 180, 250), // Blue
            selection_bg: Color::Rgb(69, 71, 90),      // Surface1
            selection_fg: Color::Rgb(205, 214, 244),   // Text

            // Special
            sensitive_mask: Color::Rgb(108, 112, 134), // Overlay0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catppuccin_themes() {
        assert!(CatppuccinLatte.is_light());
        assert!(!CatppuccinMocha.is_light());

        assert_eq!(CatppuccinMocha.name(), "Catppuccin Mocha");

        let palette = CatppuccinMocha.palette();
        assert!(matches!(palette.bg, Color::Rgb(30, 30, 46)));
    }
}
