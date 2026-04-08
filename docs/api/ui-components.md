# 🎨 UI Components

Referensi API untuk widget dan komponen UI.

## Overview

UI components adalah widget Ratatui yang dapat dikomposisi untuk membangun interface. Semua komponen menggunakan theme palette untuk styling konsisten.

## Widget Hierarchy

```
App UI
├── Screens (full-screen views)
│   ├── LoginScreen
│   ├── MainScreen
│   ├── SettingsScreen
│   └── ExportScreen
├── Widgets (reusable components)
│   ├── VaultList
│   ├── ItemDetail
│   ├── SearchPopup
│   ├── PasswordInput
│   ├── Statusline
│   ├── Notification
│   ├── ConfirmDialog
│   ├── KindSelector
│   └── ItemForm
└── Theme (styling)
    ├── ThemePalette
    ├── Catppuccin variants
    └── TokyoNight variants
```

## Screens

### LoginScreen

Screen untuk vault selection dan unlock.

```rust
pub struct LoginScreen {
    pub vault_entries: Vec<VaultRegistryEntry>,
    pub selected_index: usize,
    pub password_input: PasswordInput,
    pub entering_password: bool,
    pub creating_vault: bool,
    pub create_step: u8,
    pub new_vault_name: String,
    pub new_vault_password: String,
}
```

**Modes:**
- Vault selection (default)
- Password entry
- Create vault wizard

**Rendering:**

```rust
impl LoginScreen {
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect, theme: &ThemePalette) {
        // Layout: centered card
        let card = centered_rect(60, 70, area);
        
        // Render based on mode
        if self.creating_vault {
            self.render_create_vault_form(f, card, theme);
        } else if self.entering_password {
            self.render_password_form(f, card, theme);
        } else {
            self.render_vault_list(f, card, theme);
        }
    }
}
```

### MainScreen

Screen utama dengan split layout.

```rust
pub struct MainScreen {
    // Layout is controlled by AppState
}
```

**Layout:**

```
┌─────────────────┬──────────────────────┐
│                 │                      │
│   Vault List    │    Item Detail       │
│   (30%)         │    (70%)             │
│                 │                      │
├─────────────────┴──────────────────────┤
│              Statusline                │
└────────────────────────────────────────┘
```

**Rendering:**

```rust
pub fn render_main_screen<B: Backend>(
    f: &mut Frame<B>,
    state: &AppState,
    theme: &ThemePalette,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());
    
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(chunks[0]);
    
    render_vault_list(f, main_chunks[0], state, theme);
    render_item_detail(f, main_chunks[1], state, theme);
    render_statusline(f, chunks[1], state, theme);
}
```

### SettingsScreen

Screen untuk konfigurasi.

```rust
pub struct SettingsScreen {
    pub selected_setting: usize,
    pub settings: Vec<SettingItem>,
}

pub struct SettingItem {
    pub key: String,
    pub label: String,
    pub value: SettingValue,
}

pub enum SettingValue {
    Bool(bool),
    Number(u64),
    Choice { options: Vec<String>, selected: usize },
    Theme(ThemeChoice),
}
```

## Widgets

### VaultList

List widget untuk menampilkan items.

```rust
pub struct VaultList<'a> {
    items: &'a [Item],
    selected: Option<usize>,
    show_favorites_only: bool,
    search_filter: Option<&'a str>,
}

impl<'a> VaultList<'a> {
    pub fn new(items: &'a [Item]) -> Self {
        Self {
            items,
            selected: None,
            show_favorites_only: false,
            search_filter: None,
        }
    }
    
    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }
    
    pub fn favorites_only(mut self, only: bool) -> Self {
        self.show_favorites_only = only;
        self
    }
}
```

**Rendering:**

```rust
impl Widget for VaultList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self.items.iter()
            .filter(|i| !self.show_favorites_only || i.favorite)
            .map(|item| {
                let icon = item.kind.icon();
                let title = &item.title;
                let star = if item.favorite { " " } else { "" };
                ListItem::new(format!("{} {}{}", icon, title, star))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .title(" Items ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded))
            .highlight_style(Style::default()
                .bg(theme.selection_bg)
                .add_modifier(Modifier::BOLD));
        
        // Render with state for selection
        StatefulWidget::render(list, area, buf, &mut state);
    }
}
```

### ItemDetail

Widget untuk menampilkan detail item.

```rust
pub struct ItemDetail<'a> {
    item: Option<&'a Item>,
    revealed: bool,
    focused: bool,
}

impl<'a> ItemDetail<'a> {
    pub fn new(item: Option<&'a Item>) -> Self {
        Self {
            item,
            revealed: false,
            focused: false,
        }
    }
    
    pub fn revealed(mut self, revealed: bool) -> Self {
        self.revealed = revealed;
        self
    }
    
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}
```

**Content Rendering:**

```rust
fn render_content(&self, area: Rect, buf: &mut Buffer, theme: &ThemePalette) {
    let item = match self.item {
        Some(i) => i,
        None => {
            render_empty_state(area, buf, theme);
            return;
        }
    };
    
    let content = match &item.content {
        ItemContent::Generic { value } => {
            if self.revealed {
                value.clone()
            } else {
                mask_content(value)
            }
        }
        ItemContent::Password { username, password, url, .. } => {
            // Format password entry
            format_password_content(username, password, url, self.revealed)
        }
        // ... other content types
    };
    
    let paragraph = Paragraph::new(content)
        .block(detail_block(self.focused, theme))
        .wrap(Wrap { trim: true });
    
    paragraph.render(area, buf);
}
```

### PasswordInput

Widget untuk input password dengan masking.

```rust
pub struct PasswordInput {
    value: String,
    cursor: usize,
    masked: bool,
}

impl PasswordInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            masked: true,
        }
    }
    
    pub fn input(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += 1;
    }
    
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
        }
    }
    
    pub fn value(&self) -> &str {
        &self.value
    }
    
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }
    
    pub fn toggle_mask(&mut self) {
        self.masked = !self.masked;
    }
}
```

**Rendering:**

```rust
impl Widget for PasswordInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display = if self.masked {
            "•".repeat(self.value.len())
        } else {
            self.value.clone()
        };
        
        let input = Paragraph::new(display)
            .block(Block::default()
                .title(" Password ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded));
        
        input.render(area, buf);
        
        // Render cursor
        let cursor_x = area.x + 1 + self.cursor as u16;
        buf.set_string(cursor_x, area.y + 1, "▏", cursor_style);
    }
}
```

### SearchPopup

Floating window untuk pencarian.

```rust
pub struct SearchPopup<'a> {
    query: &'a str,
    results: &'a [SearchResult],
    selected: usize,
}

pub struct SearchResult {
    pub item_id: Uuid,
    pub title: String,
    pub match_ranges: Vec<(usize, usize)>,
    pub score: u32,
}
```

**Rendering:**

```rust
impl Widget for SearchPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Centered floating window
        let popup_area = centered_rect(50, 60, area);
        
        // Clear background
        Clear.render(popup_area, buf);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Input
                Constraint::Min(0),     // Results
            ])
            .split(popup_area);
        
        // Search input
        let input = Paragraph::new(format!(" {}", self.query))
            .block(Block::default()
                .title(" Search ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded));
        input.render(chunks[0], buf);
        
        // Results list with highlighted matches
        let items: Vec<ListItem> = self.results.iter()
            .map(|r| {
                let spans = highlight_matches(&r.title, &r.match_ranges);
                ListItem::new(Line::from(spans))
            })
            .collect();
        
        let list = List::new(items)
            .highlight_style(Style::default().bg(theme.selection_bg));
        
        StatefulWidget::render(list, chunks[1], buf, &mut ListState::default()
            .with_selected(Some(self.selected)));
    }
}
```

### Statusline

Bottom status bar (Lualine-style) yang juga menampilkan notifikasi terakhir secara inline.

```rust
pub struct Statusline<'a> {
    mode: &'a str,
    vault_name: Option<&'a str>,
    item_count: usize,
    dirty: bool,
    hints: &'a [(&'a str, &'a str)],
}
```

**Rendering:**

```rust
impl Widget for Statusline<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Left section: mode indicator
        let mode_span = Span::styled(
            format!(" {} ", self.mode),
            Style::default()
                .bg(theme.primary)
                .fg(theme.bg)
                .add_modifier(Modifier::BOLD),
        );
        
        // Middle section: vault info
        let vault_info = match self.vault_name {
            Some(name) => format!(
                "  {} │ {} items{}",
                name,
                self.item_count,
                if self.dirty { " [+]" } else { "" }
            ),
            None => String::new(),
        };
        
        // Right section: keybinding hints + latest notification (inline)
        let hints: Vec<Span> = self.hints.iter()
            .flat_map(|(key, action)| {
                vec![
                    Span::styled(key, Style::default().fg(theme.primary)),
                    Span::raw(format!(" {} ", action)),
                ]
            })
            .collect();
        
        // Render with proper spacing
        let line = Line::from(vec![
            mode_span,
            Span::raw(vault_info),
            // ... spacing ...
            Span::from(hints),
        ]);
        
        Paragraph::new(line)
            .style(Style::default().bg(theme.bg_alt))
            .render(area, buf);
    }
}
```

### Notification

Model notifikasi internal. Saat ini notifikasi tidak dirender sebagai popup/toast; pesan terakhir tampil inline di statusline.

```rust
pub struct NotificationWidget<'a> {
    notifications: &'a [Notification],
}

pub struct Notification {
    pub id: usize,
    pub message: String,
    pub level: NotificationLevel,
    pub expires_at: Instant,
}
```

**Catatan:** rendering notifikasi dilakukan oleh `Statusline`, bukan sebagai overlay popup.

### ConfirmDialog

Confirmation dialog popup.

```rust
pub struct ConfirmDialog<'a> {
    title: &'a str,
    message: &'a str,
    confirm_label: &'a str,
    cancel_label: &'a str,
    selected: usize,  // 0 = cancel, 1 = confirm
}
```

**Rendering:**

```rust
impl Widget for ConfirmDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup = centered_rect(40, 20, area);
        Clear.render(popup, buf);
        
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.warning));
        
        block.render(popup, buf);
        
        // Message
        let message_area = Rect::new(
            popup.x + 2,
            popup.y + 2,
            popup.width - 4,
            popup.height - 5,
        );
        Paragraph::new(self.message).render(message_area, buf);
        
        // Buttons
        let button_y = popup.bottom() - 2;
        let cancel_style = if self.selected == 0 {
            Style::default().bg(theme.selection_bg)
        } else {
            Style::default()
        };
        let confirm_style = if self.selected == 1 {
            Style::default().bg(theme.error).fg(theme.bg)
        } else {
            Style::default().fg(theme.error)
        };
        
        buf.set_string(
            popup.x + 4,
            button_y,
            format!(" {} ", self.cancel_label),
            cancel_style,
        );
        buf.set_string(
            popup.right() - 4 - self.confirm_label.len() as u16,
            button_y,
            format!(" {} ", self.confirm_label),
            confirm_style,
        );
    }
}
```

### KindSelector

Widget untuk memilih tipe item.

```rust
pub struct KindSelector {
    kinds: Vec<ItemKind>,
    selected: usize,
}

impl KindSelector {
    pub fn new() -> Self {
        Self {
            kinds: vec![
                ItemKind::Generic,
                ItemKind::Password,
                ItemKind::CryptoSeed,
                ItemKind::SecureNote,
                ItemKind::ApiKey,
            ],
            selected: 0,
        }
    }
    
    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.kinds.len();
    }
    
    pub fn prev(&mut self) {
        self.selected = self.selected.checked_sub(1)
            .unwrap_or(self.kinds.len() - 1);
    }
    
    pub fn current(&self) -> ItemKind {
        self.kinds[self.selected]
    }
}
```

### ItemForm

Form untuk create/edit item.

```rust
pub struct ItemForm {
    pub kind: ItemKind,
    pub fields: Vec<FormField>,
    pub current_field: usize,
    pub mode: FormMode,
}

pub struct FormField {
    pub name: String,
    pub value: String,
    pub cursor: usize,
    pub masked: bool,
    pub required: bool,
}

pub enum FormMode {
    Create,
    Edit { item_id: Uuid },
}
```

## Theme System

### ThemePalette

```rust
#[derive(Debug, Clone)]
pub struct ThemePalette {
    // Base
    pub bg: Color,
    pub bg_alt: Color,
    pub fg: Color,
    pub fg_muted: Color,
    
    // Accents
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    
    // Semantic
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // UI
    pub border: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}
```

### Available Themes

```rust
pub enum ThemeChoice {
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,      // Default
    TokyoNightNight,
    TokyoNightStorm,
    TokyoNightDay,
}

impl ThemeChoice {
    pub fn palette(&self) -> ThemePalette {
        match self {
            Self::CatppuccinMocha => catppuccin::mocha(),
            Self::CatppuccinLatte => catppuccin::latte(),
            // ...
        }
    }
}
```

## Helper Functions

### Centered Rect

```rust
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

### Mask Content

```rust
fn mask_content(content: &str) -> String {
    "•".repeat(content.len().min(20))
}
```

### Highlight Matches

```rust
fn highlight_matches(
    text: &str,
    ranges: &[(usize, usize)],
    theme: &ThemePalette,
) -> Vec<Span> {
    // Split text into highlighted and non-highlighted spans
    let mut spans = Vec::new();
    let mut last_end = 0;
    
    for (start, end) in ranges {
        if *start > last_end {
            spans.push(Span::raw(&text[last_end..*start]));
        }
        spans.push(Span::styled(
            &text[*start..*end],
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ));
        last_end = *end;
    }
    
    if last_end < text.len() {
        spans.push(Span::raw(&text[last_end..]));
    }
    
    spans
}
```

## Icons

### ItemKind Icons

```rust
impl ItemKind {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Generic => "",
            Self::Password => "",
            Self::CryptoSeed => "󰌗",
            Self::SecureNote => "",
            Self::ApiKey => "",
        }
    }
}
```

### UI Icons

```rust
pub mod icons {
    pub const VAULT: &str = "";
    pub const LOCK: &str = "";
    pub const UNLOCK: &str = "";
    pub const SEARCH: &str = "";
    pub const STAR: &str = "";
    pub const STAR_EMPTY: &str = "";
    pub const COPY: &str = "";
    pub const SAVE: &str = "";
    pub const EDIT: &str = "";
    pub const DELETE: &str = "";
    pub const ADD: &str = "";
    pub const CHECK: &str = "";
    pub const ERROR: &str = "";
    pub const WARNING: &str = "";
    pub const INFO: &str = "";
}
```
