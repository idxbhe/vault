# 🏛️ TEA Pattern dalam Rust

Implementasi The Elm Architecture (TEA) untuk aplikasi Vault.

## Apa itu TEA?

**The Elm Architecture** adalah pola arsitektur yang berasal dari bahasa pemrograman Elm. Pola ini memberikan cara yang terstruktur dan predictable untuk mengelola state dalam aplikasi.

### Komponen Utama

```
┌─────────────────────────────────────────────────────────────┐
│                                                              │
│   ┌───────────┐      ┌───────────┐      ┌───────────┐       │
│   │   Model   │ ───▶ │   View    │ ───▶ │  Screen   │       │
│   │  (State)  │      │ (Render)  │      │ (Output)  │       │
│   └───────────┘      └───────────┘      └───────────┘       │
│         ▲                                     │              │
│         │                                     │              │
│         │                                     ▼              │
│   ┌───────────┐      ┌───────────┐      ┌───────────┐       │
│   │  Update   │ ◀─── │  Message  │ ◀─── │   User    │       │
│   │           │      │  (Event)  │      │  (Input)  │       │
│   └───────────┘      └───────────┘      └───────────┘       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Alur Data

1. **Model (State)**: Satu-satunya sumber kebenaran
2. **View**: Pure function yang render state ke UI
3. **Message**: Event yang mendeskripsikan perubahan
4. **Update**: Pure function yang menghasilkan state baru

## Implementasi dalam Rust

### 1. Model (State)

```rust
// src/app/state.rs

/// Application state - single source of truth
pub struct AppState {
    // Core state
    pub mode: AppMode,
    pub screen: Screen,
    pub vault_state: Option<VaultState>,
    
    // UI state  
    pub ui_state: UIState,
    pub login_screen: LoginScreen,
    
    // Configuration
    pub config: AppConfig,
    pub registry: VaultRegistry,
    
    // Runtime state
    pub clipboard_state: ClipboardState,
    pub should_quit: bool,
}

/// Vault state when unlocked
pub struct VaultState {
    pub vault: Vault,
    pub vault_path: PathBuf,
    pub encryption_key: [u8; 32],
    pub selected_item_id: Option<Uuid>,
    pub is_dirty: bool,
    pub undo_stack: Vec<UndoEntry>,
    pub redo_stack: Vec<UndoEntry>,
    pub last_activity: Instant,
}

/// UI-specific state
pub struct UIState {
    pub focused_pane: Pane,
    pub floating_window: Option<FloatingWindow>,
    pub content_revealed: bool,
    pub input_buffer: InputBuffer,
    pub notifications: Vec<Notification>,
    pub detail_scroll_offset: usize,
}
```

**Prinsip:**
- State immutable secara konseptual
- Semua state dalam satu struct
- Mudah di-serialize untuk debugging

### 2. Message

```rust
// src/app/message.rs

/// All possible events in the application
pub enum Message {
    // === Navigation ===
    Navigate(Screen),
    FocusPane(Pane),
    SelectItem(Uuid),
    SelectNextItem,
    SelectPrevItem,
    
    // === Vault Operations ===
    CreateVault { name: String, password: String },
    OpenVault { path: PathBuf },
    UnlockVault { password: SecureString, keyfile: Option<Vec<u8>> },
    SaveVault,
    LockVault,
    CloseVault,
    
    // === Item Operations ===
    CreateItem { kind: ItemKind },
    UpdateItem { id: Uuid, updates: ItemUpdates },
    DeleteItem(Uuid),
    ConfirmDeleteItem(Uuid),
    
    // === Input ===
    InputChar(char),
    InputBackspace,
    InputDelete,
    InputLeft,
    InputRight,
    InputHome,
    InputEnd,
    InputSubmit,
    InputCancel,
    
    // === UI ===
    OpenFloatingWindow(FloatingWindow),
    CloseFloatingWindow,
    ToggleContentReveal,
    Scroll(ScrollDirection),
    
    // === Search ===
    OpenSearch,
    UpdateSearchQuery(String),
    SearchNextResult,
    SearchPrevResult,
    SearchConfirm,
    
    // === Clipboard ===
    CopyToClipboard { content: String, is_sensitive: bool },
    CopyCurrentItem,
    ClearClipboard,
    
    // === History ===
    Undo,
    Redo,
    
    // === System ===
    Tick,
    Quit,
    ForceQuit,
    Noop,
}
```

**Prinsip:**
- Enum dengan semua kemungkinan action
- Membawa data yang diperlukan
- Exhaustive pattern matching

### 3. Update Function

```rust
// src/app/update.rs

/// Pure function that handles state transitions
pub fn update(state: &mut AppState, msg: Message) -> Effect {
    // Update last activity for auto-lock
    if let Some(ref mut vs) = state.vault_state {
        vs.last_activity = Instant::now();
    }
    
    match msg {
        // === Navigation ===
        Message::Navigate(screen) => {
            state.screen = screen;
            Effect::none()
        }
        
        Message::FocusPane(pane) => {
            state.ui_state.focused_pane = pane;
            Effect::none()
        }
        
        // === Item Operations ===
        Message::CreateItem { kind } => {
            if let Some(ref mut vs) = state.vault_state {
                let item = Item::new("New Item", kind, kind.default_content());
                let id = item.id;
                
                // Open edit dialog
                state.ui_state.floating_window = 
                    Some(FloatingWindow::edit_item_form(&item));
                
                vs.vault.add_item(item);
                vs.selected_item_id = Some(id);
                vs.mark_dirty();
            }
            Effect::none()
        }
        
        Message::DeleteItem(id) => {
            // Show confirmation
            state.ui_state.floating_window = 
                Some(FloatingWindow::ConfirmDelete { item_id: id });
            Effect::none()
        }
        
        Message::ConfirmDeleteItem(id) => {
            state.ui_state.close_floating_window();
            
            if let Some(ref mut vs) = state.vault_state {
                if let Some(item) = vs.vault.get_item(id) {
                    // Save for undo
                    let undo_entry = UndoEntry {
                        description: format!("Delete {}", item.title),
                        item_id: id,
                        previous_state: ItemSnapshot::from_item(item),
                    };
                    
                    vs.vault.remove_item(id);
                    vs.push_undo(undo_entry);
                    vs.mark_dirty();
                }
            }
            Effect::none()
        }
        
        // === Effects ===
        Message::SaveVault => {
            if let Some(ref mut vs) = state.vault_state {
                vs.is_dirty = false;
                Effect::WriteVaultFile {
                    path: vs.vault_path.clone(),
                    vault: vs.vault.clone(),
                    key: vs.encryption_key,
                }
            } else {
                Effect::none()
            }
        }
        
        // ... more handlers
        
        Message::Noop => Effect::none(),
    }
}
```

**Prinsip:**
- Pattern matching exhaustive
- Return Effect untuk side effects
- State mutation in-place (Rust optimization)

### 4. Effect

```rust
// src/app/effect.rs

/// Side effects that need to be executed
pub enum Effect {
    /// No effect
    None,
    
    /// Multiple effects
    Batch(Vec<Effect>),
    
    // === File I/O ===
    ReadVaultFile {
        path: PathBuf,
        password: SecureString,
        keyfile: Option<Vec<u8>>,
    },
    WriteVaultFile {
        path: PathBuf,
        vault: Vault,
        key: [u8; 32],
    },
    ExportVault {
        path: PathBuf,
        vault: Vault,
        encrypted: bool,
        key: Option<[u8; 32]>,
    },
    
    // === Clipboard ===
    SetClipboard {
        content: String,
        is_sensitive: bool,
    },
    ClearClipboard,
    ScheduleClipboardClear { delay: Duration },
    
    // === Timer ===
    ScheduleAutoLock { delay: Duration },
    CancelAutoLock,
    
    // === System ===
    Exit,
}

impl Effect {
    pub fn none() -> Self {
        Self::None
    }
    
    pub fn batch(effects: Vec<Effect>) -> Self {
        let effects: Vec<Effect> = effects
            .into_iter()
            .filter(|e| !matches!(e, Effect::None))
            .collect();
        
        match effects.len() {
            0 => Effect::None,
            1 => effects.into_iter().next().unwrap(),
            _ => Effect::Batch(effects),
        }
    }
}
```

**Prinsip:**
- Describes what, not how
- Composable with Batch
- No actual I/O in update function

### 5. Runtime (Effect Executor)

```rust
// src/app/runtime.rs

/// Executes effects and returns results
pub struct Runtime {
    message_tx: Sender<Message>,
    clipboard_clear_at: Option<Instant>,
    auto_lock_at: Option<Instant>,
}

impl Runtime {
    pub fn execute(&mut self, effect: Effect) -> EffectResult {
        match effect {
            Effect::None => EffectResult::Success,
            
            Effect::Batch(effects) => {
                for effect in effects {
                    let result = self.execute(effect);
                    if let EffectResult::Error(_) = result {
                        return result;
                    }
                }
                EffectResult::Success
            }
            
            Effect::ReadVaultFile { path, password, keyfile } => {
                match read_vault_file(&path, &password, keyfile.as_deref()) {
                    Ok((vault, key)) => EffectResult::VaultLoaded { vault, path, key },
                    Err(e) => EffectResult::Error(e),
                }
            }
            
            Effect::WriteVaultFile { path, vault, key } => {
                match write_vault_file(&path, &vault, &key) {
                    Ok(()) => EffectResult::VaultSaved,
                    Err(e) => EffectResult::Error(e),
                }
            }
            
            Effect::SetClipboard { content, is_sensitive } => {
                match set_clipboard(&content) {
                    Ok(()) => {
                        if is_sensitive {
                            self.schedule_clipboard_clear(Duration::from_secs(30));
                        }
                        EffectResult::Success
                    }
                    Err(e) => EffectResult::Error(e),
                }
            }
            
            // ... more handlers
        }
    }
}

/// Result of effect execution
pub enum EffectResult {
    Success,
    VaultLoaded { vault: Vault, path: PathBuf, key: [u8; 32] },
    VaultSaved,
    ExportCompleted { path: PathBuf },
    ConfigLoaded(AppConfig),
    Error(String),
}
```

### 6. View

```rust
// src/ui/screens/main.rs

/// Pure function that renders state to terminal
pub fn render(frame: &mut Frame, state: &AppState, theme: &ThemePalette) {
    let area = frame.area();
    
    // Layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(area);
    
    // Render list panel
    render_item_list(frame, chunks[0], state, theme);
    
    // Render detail panel
    render_item_detail(frame, chunks[1], state, theme);
    
    // Render floating window if any
    if let Some(ref window) = state.ui_state.floating_window {
        render_floating_window(frame, area, window, state, theme);
    }
    
    // Render notifications
    render_notifications(frame, area, &state.ui_state.notifications, theme);
}
```

**Prinsip:**
- Pure function (no side effects)
- Reads state, produces UI
- No state modification

### 7. Main Loop

```rust
// src/main.rs

fn main() -> Result<()> {
    // Setup
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let mut runtime = Runtime::new();
    let keybindings = KeybindingConfig::default();
    
    // Main loop
    loop {
        // 1. Render
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;
        
        // 2. Handle input
        if event::poll(TICK_RATE)? {
            let event = event::read()?;
            let msg = route_event(app.state(), event, &keybindings);
            
            // 3. Update state
            let effect = update(app.state_mut(), msg);
            
            // 4. Execute effect
            if !effect.is_none() {
                let result = runtime.execute(effect);
                
                // 5. Handle result
                handle_effect_result(&mut app, result);
            }
        }
        
        // 6. Check timers
        runtime.tick();
        
        // 7. Check quit
        if app.state().should_quit {
            break;
        }
    }
    
    // Cleanup
    restore_terminal(terminal)?;
    Ok(())
}
```

## Keuntungan TEA

### 1. Predictability

State hanya berubah melalui messages:

```rust
// BAD: Direct mutation
state.vault_state.as_mut().unwrap().selected_item_id = Some(id);

// GOOD: Via message
update(state, Message::SelectItem(id));
```

### 2. Debugging

Semua state changes dapat di-log:

```rust
pub fn update(state: &mut AppState, msg: Message) -> Effect {
    tracing::debug!("Processing message: {:?}", msg);
    // ...
}
```

### 3. Testing

Update function mudah di-test:

```rust
#[test]
fn test_create_item() {
    let mut state = AppState::default();
    state.vault_state = Some(VaultState::new(...));
    
    let effect = update(&mut state, Message::CreateItem { 
        kind: ItemKind::Generic 
    });
    
    assert!(state.vault_state.unwrap().is_dirty);
    assert!(effect.is_none());
}
```

### 4. Time-Travel Debugging

State snapshots memungkinkan replay:

```rust
// Save state history
let history: Vec<AppState> = vec![];

// Replay
for (state, msg) in history.iter().zip(messages.iter()) {
    update(&mut state.clone(), msg.clone());
}
```

## Perbedaan dengan Elm

| Aspect | Elm | Rust (Vault) |
|--------|-----|--------------|
| Immutability | Enforced | Convention |
| Side Effects | Cmd monad | Effect enum |
| Concurrency | None | Single-threaded |
| State Mutation | Copy-on-write | In-place |

## Best Practices

1. **Keep State Minimal**: Hanya simpan data yang diperlukan
2. **Messages Descriptive**: Nama message menjelaskan intent
3. **Effects Explicit**: Semua side effects melalui Effect
4. **View Pure**: Tidak ada mutation di render
5. **Test Update**: Focus testing pada update function
