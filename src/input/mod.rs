//! Input handling
//!
//! Keyboard and mouse event processing with Vim-style keybindings.

pub mod keybindings;
pub mod mouse;
pub mod router;

pub use keybindings::{format_key_combo, KeyAction, KeyCombo, KeybindingConfig};
pub use mouse::{ClickRegion, ClickableElement, LayoutRegions, MouseAction, UiRegion};
pub use router::route_event;
