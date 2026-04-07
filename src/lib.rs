//! # Vault - TUI Vault Manager
//!
//! A terminal-based secure data storage application for sensitive information
//! like seed phrases, passwords, and API keys.
//!
//! ## Architecture
//!
//! This application follows The Elm Architecture (TEA) pattern:
//! - **Model**: Application state (`app::state`)
//! - **Update**: State transitions via messages (`app::update`)
//! - **View**: UI rendering (`ui`)
//!
//! ## Modules
//!
//! - `app` - Application core (state, messages, effects)
//! - `domain` - Business logic and data models
//! - `crypto` - Cryptographic operations
//! - `storage` - File I/O and persistence
//! - `ui` - User interface components
//! - `input` - Input handling and keybindings
//! - `clipboard` - Secure clipboard operations
//! - `utils` - Shared utilities

pub mod app;
pub mod clipboard;
pub mod crypto;
pub mod domain;
pub mod input;
pub mod storage;
pub mod ui;
pub mod utils;

pub use utils::error::{Error, Result};
