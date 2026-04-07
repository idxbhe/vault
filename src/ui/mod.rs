//! User interface components
//!
//! Contains all UI rendering logic following the Elm Architecture pattern.

pub mod app;
pub mod screens;
pub mod theme;
pub mod widgets;

pub use app::App;
pub use screens::{render_login, render_main, LoginScreen, MainScreen};
pub use theme::{get_theme, ThemePalette};
pub use widgets::{render_item_list, render_statusline, ItemListState};

// Re-export ThemeChoice from storage for convenience
pub use crate::storage::ThemeChoice;
