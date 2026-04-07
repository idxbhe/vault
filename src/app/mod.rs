//! Application core - TEA (The Elm Architecture) implementation
//!
//! This module implements the core application logic using TEA pattern:
//! - **State**: Application state (AppState, VaultState, UIState)
//! - **Message**: All possible actions/events
//! - **Update**: Pure function (state, message) -> (state, effect)
//! - **Effect**: Side effects (I/O, clipboard, timers)
//! - **Runtime**: Effect executor and timer management

pub mod effect;
pub mod message;
pub mod runtime;
pub mod state;
pub mod update;

pub use effect::{Effect, EffectResult};
pub use message::{ConfigUpdate, ExportFormat, ItemUpdates, Message, ScrollDirection};
pub use runtime::{write_vault_file, Runtime};
pub use state::{
    AppMode, AppState, ClipboardState, FilterState, FloatingWindow, InputBuffer, Notification,
    NotificationLevel, Pane, Screen, UIState, UndoEntry, VaultState,
};
pub use update::update;
