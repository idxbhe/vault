//! Vault - TUI Vault Manager
//!
//! A secure terminal-based vault for storing sensitive data.

use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use vault::app::{AppState, Effect, EffectResult, Message, NotificationLevel};
use vault::input::{KeybindingConfig, route_event};
use vault::storage::{AppConfig, VaultRegistry};
use vault::ui::app::App;

/// Tick rate for the application (100ms for responsiveness)
const TICK_RATE: Duration = Duration::from_millis(100);

fn main() -> Result<()> {
    // Initialize logging (errors only, no spam)
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("vault=error".parse()?))
        .init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize application state
    let config = AppConfig::load_or_default();
    let registry = VaultRegistry::load().unwrap_or_default();
    let state = AppState::new(config, registry);

    // Create UI app wrapper
    let mut app = App::new(state);

    // Create keybindings
    let keybindings = KeybindingConfig::default();

    // Create message channel for async effects
    let (tx, rx) = mpsc::channel::<Message>();
    let mut runtime = vault::app::Runtime::new(tx);

    // Run the application
    let result = run_app(&mut terminal, &mut app, &keybindings, &mut runtime, rx);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }

    result
}

/// Main application loop
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    keybindings: &KeybindingConfig,
    runtime: &mut vault::app::Runtime,
    rx: std::sync::mpsc::Receiver<Message>,
) -> Result<()> {
    let mut last_tick = Instant::now();

    loop {
        // Render
        terminal.draw(|f| {
            app.render(f);
        })?;

        // Handle events with timeout
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            let evt = event::read()?;

            // Convert event to message
            let message = route_event(app.state(), evt, keybindings);

            // Process message through TEA update
            if !matches!(message, Message::Noop) {
                let effect = app.update(message);

                // Execute effects and handle results
                match effect {
                    Effect::Exit => {
                        tracing::info!("Quit requested");
                        return Ok(());
                    }
                    Effect::None => {}
                    effect => {
                        // Force a render before executing potentially blocking effects
                        // like ReadVaultFile to ensure "loading" overlays are shown.
                        if app.state().ui_state.is_loading() {
                            terminal.draw(|f| {
                                app.render(f);
                            })?;
                        }

                        let result = runtime.execute(effect);
                        handle_effect_result(app, result);
                    }
                }
            }
        }

        // Process any async messages
        while let Ok(msg) = rx.try_recv() {
            if let Message::AsyncEffectCompleted(boxed_result) = msg {
                handle_effect_result(app, *boxed_result);
            }
        }

        // Tick (for timers, animations, etc.)
        if last_tick.elapsed() >= TICK_RATE {
            let tick_effect = app.update(Message::Tick);
            if !matches!(tick_effect, Effect::None) {
                let result = runtime.execute(tick_effect);
                handle_effect_result(app, result);
            }

            // Check runtime timers
            runtime.tick();

            // Advance spinner animation if loading
            app.state_mut().ui_state.tick_spinner();

            last_tick = Instant::now();
        }
    }
}

/// Handle the result of executing an effect
fn handle_effect_result(app: &mut App, result: EffectResult) {
    match result {
        EffectResult::Success => {
            tracing::debug!("Effect completed successfully");
        }

        EffectResult::VaultLoaded {
            vault,
            path,
            key,
            salt,
            has_keyfile,
            encryption_method,
            recovery_metadata,
        } => {
            app.handle_vault_loaded(
                vault,
                path,
                key,
                salt,
                has_keyfile,
                encryption_method,
                recovery_metadata,
            );
        }

        EffectResult::VaultCreated {
            vault,
            path,
            vault_name,
            key,
            salt,
            has_keyfile,
            encryption_method,
            recovery_metadata,
            keyfile_message,
        } => {
            app.state_mut().registry.add_or_update(&path, &vault_name);
            if let Err(e) = app.state_mut().registry.save() {
                app.state_mut().ui_state.notify(
                    format!("Vault created, but failed to update registry: {}", e),
                    vault::app::NotificationLevel::Warning,
                );
            }
            if let Some(msg) = keyfile_message {
                app.state_mut()
                    .ui_state
                    .notify(msg, vault::app::NotificationLevel::Info);
            }

            app.handle_vault_created(
                vault,
                path,
                key,
                salt,
                has_keyfile,
                encryption_method,
                recovery_metadata,
            );
        }

        EffectResult::VaultSaved => {
            let pending_lock = {
                let state = app.state_mut();
                if let Some(vs) = state.vault_state.as_mut() {
                    vs.is_dirty = false;
                }
                state.pending_lock
            };

            if pending_lock {
                let _ = app.update(Message::LockVault);
            } else {
                app.state_mut()
                    .ui_state
                    .notify("Vault saved successfully!", NotificationLevel::Success);
            }
            tracing::info!("Vault saved");
        }

        EffectResult::ExportCompleted { path } => {
            app.state_mut().ui_state.notify(
                format!("Exported to {}", path.display()),
                NotificationLevel::Success,
            );
            tracing::info!("Vault exported to {:?}", path);
        }

        EffectResult::ConfigLoaded(config) => {
            app.state_mut().config = config;
            tracing::info!("Config loaded");
        }

        EffectResult::RegistryLoaded(registry) => {
            app.state_mut().registry = registry;
            tracing::info!("Registry loaded");
        }

        EffectResult::KeyfileLoaded { path, data: _ } => {
            tracing::info!("Keyfile loaded from {:?}", path);
            // Keyfile data will be used in vault unlock flow
        }

        EffectResult::Error(error) => {
            app.handle_effect_error(error);
        }
    }
}
