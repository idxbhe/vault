//! User interface components
//!
//! Contains all UI rendering logic following the Elm Architecture pattern.

pub mod app;
pub mod screens;
pub mod theme;
pub mod widgets;

pub use app::App;
pub use screens::{LoginScreen, MainScreen, render_login, render_main};
pub use theme::{ThemePalette, get_theme};
pub use widgets::{ItemListState, render_item_list, render_statusline};

// Re-export ThemeChoice from storage for convenience
pub use crate::storage::ThemeChoice;
