//! Reusable UI widgets
//!
//! Components that can be composed to build screens.

pub mod button;
pub mod edit_form;
pub mod help;
pub mod item_detail;
pub mod item_list;
pub mod kind_selector;
pub mod notification;
pub mod search_dialog;
pub mod statusline;

pub use button::{render_button_row, render_keyboard_hints, ButtonRegion, ButtonStyle};
pub use edit_form::{render as render_edit_form, EditFormState, FormClickRegions, FormField};
pub use help::render as render_help;
pub use item_detail::render as render_item_detail;
pub use item_list::{render as render_item_list, ItemListState};
pub use kind_selector::{render as render_kind_selector, KindSelectorClickRegions, KindSelectorState};
pub use search_dialog::{render as render_search_dialog, SearchClickRegions, SearchState};
pub use statusline::render as render_statusline;
