//! Reusable UI widgets
//!
//! Components that can be composed to build screens.

pub mod button;
pub mod category_bar;
pub mod edit_form;
pub mod help;
pub mod item_detail;
pub mod item_list;
pub mod kind_selector;
pub mod notification;
pub mod search_dialog;
pub mod statusline;

pub use button::{ButtonRegion, ButtonStyle, render_button_row, render_keyboard_hints};
pub use category_bar::{CategoryBarClickRegions, render as render_category_bar};
pub use edit_form::{EditFormState, FormClickRegions, FormField, render as render_edit_form};
pub use help::render as render_help;
pub use item_detail::render as render_item_detail;
pub use item_list::{ItemListState, render as render_item_list};
pub use kind_selector::{
    KindSelectorClickRegions, KindSelectorState, render as render_kind_selector,
};
pub use search_dialog::{SearchClickRegions, SearchState, render as render_search_dialog};
pub use statusline::render as render_statusline;
