//! Theming system
//!
//! Provides color palettes for Catppuccin and TokyoNight themes.

pub mod catppuccin;
pub mod palette;
pub mod tokyonight;

pub use catppuccin::{CatppuccinFrappe, CatppuccinLatte, CatppuccinMacchiato, CatppuccinMocha};
pub use palette::{Theme, ThemePalette};
pub use tokyonight::{TokyoNightDay, TokyoNightNight, TokyoNightStorm};

use crate::storage::ThemeChoice;

/// Get the theme palette for a theme choice
pub fn get_theme(choice: ThemeChoice) -> ThemePalette {
    match choice {
        ThemeChoice::CatppuccinLatte => CatppuccinLatte.palette(),
        ThemeChoice::CatppuccinFrappe => CatppuccinFrappe.palette(),
        ThemeChoice::CatppuccinMacchiato => CatppuccinMacchiato.palette(),
        ThemeChoice::CatppuccinMocha => CatppuccinMocha.palette(),
        ThemeChoice::TokyoNightNight => TokyoNightNight.palette(),
        ThemeChoice::TokyoNightStorm => TokyoNightStorm.palette(),
        ThemeChoice::TokyoNightDay => TokyoNightDay.palette(),
    }
}
