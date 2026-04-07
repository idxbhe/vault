# 📨 Messages

Referensi API untuk sistem pesan (Message enum).

## Overview

Message adalah satu-satunya cara untuk mengubah state aplikasi. Setiap interaksi user atau event sistem di-translate menjadi Message.

## Message Enum

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    Navigate(Screen),
    FocusPane(Pane),
    
    // Vault lifecycle
    CreateVault { name: String, password: String },
    OpenVault { path: PathBuf },
    UnlockVault { password: SecureString, keyfile: Option<Vec<u8>> },
    SaveVault,
    LockVault,
    CloseVault,
    ExportVault { format: ExportFormat, path: PathBuf },
    
    // Login flow
    StartCreateVault,
    EnterPasswordMode,
    CancelInput,
    
    // Item operations
    SelectItem(Uuid),
    SelectNextItem,
    SelectPrevItem,
    CreateItem { kind: ItemKind },
    UpdateItem { id: Uuid, updates: ItemUpdates },
    DeleteItem(Uuid),
    ConfirmDeleteItem(Uuid),
    ToggleFavorite(Uuid),
    
    // Tags
    CreateTag(Tag),
    DeleteTag(Uuid),
    AddTagToItem { item_id: Uuid, tag_id: Uuid },
    RemoveTagFromItem { item_id: Uuid, tag_id: Uuid },
    ToggleFavoritesFilter,
    
    // Input
    InputChar(char),
    InputBackspace,
    InputDelete,
    InputLeft,
    InputRight,
    InputHome,
    InputEnd,
    InputSubmit,
    InputCancel,
    
    // Search
    OpenSearch,
    UpdateSearchQuery(String),
    SearchNextResult,
    SearchPrevResult,
    SearchConfirm,
    
    // Kind selector
    KindSelectorNext,
    KindSelectorPrev,
    KindSelectorConfirm,
    
    // Form
    FormNextField,
    FormPrevField,
    FormSubmit,
    
    // Clipboard
    CopyToClipboard { content: String, is_sensitive: bool },
    CopyCurrentItem,
    ClearClipboard,
    
    // UI
    OpenFloatingWindow(FloatingWindow),
    CloseFloatingWindow,
    ToggleContentReveal,
    Scroll(ScrollDirection),
    ShowNotification { message: String, level: NotificationLevel },
    DismissNotification(usize),
    
    // History
    Undo,
    Redo,
    
    // System
    Tick,
    Quit,
    ForceQuit,
    Noop,
}
```

## Kategori Message

### Navigation Messages

#### `Navigate(Screen)`
Berpindah ke screen yang ditentukan.

```rust
// Example
Message::Navigate(Screen::Settings)
```

#### `FocusPane(Pane)`
Memfokuskan pane tertentu.

```rust
Message::FocusPane(Pane::Detail)
```

### Vault Lifecycle Messages

#### `CreateVault { name, password }`
Membuat vault baru dengan nama dan password.

```rust
Message::CreateVault {
    name: "My Vault".to_string(),
    password: "secure_password".to_string(),
}
```

#### `OpenVault { path }`
Membuka vault file dari path.

```rust
Message::OpenVault {
    path: PathBuf::from("/path/to/vault.vault"),
}
```

#### `UnlockVault { password, keyfile }`
Unlock vault dengan password dan optional keyfile.

```rust
Message::UnlockVault {
    password: SecureString::new("password".to_string()),
    keyfile: None,
}
```

#### `SaveVault`
Menyimpan perubahan ke file.

#### `LockVault`
Mengunci vault dan kembali ke login.

#### `CloseVault`
Menutup vault tanpa lock (untuk switch vault).

#### `ExportVault { format, path }`
Export vault ke file.

```rust
Message::ExportVault {
    format: ExportFormat::Json,
    path: PathBuf::from("export.json"),
}
```

### Login Flow Messages

#### `StartCreateVault`
Memulai flow pembuatan vault baru.

#### `EnterPasswordMode`
Masuk ke mode input password.

#### `CancelInput`
Membatalkan input dan kembali ke mode sebelumnya.

### Item Operation Messages

#### `SelectItem(Uuid)`
Memilih item berdasarkan ID.

```rust
Message::SelectItem(item_id)
```

#### `SelectNextItem` / `SelectPrevItem`
Navigasi ke item berikutnya/sebelumnya.

#### `CreateItem { kind }`
Membuat item baru dengan tipe tertentu.

```rust
Message::CreateItem { kind: ItemKind::Password }
```

#### `UpdateItem { id, updates }`
Mengupdate item dengan perubahan tertentu.

```rust
Message::UpdateItem {
    id: item_id,
    updates: ItemUpdates {
        title: Some("New Title".to_string()),
        ..Default::default()
    },
}
```

#### `DeleteItem(Uuid)` / `ConfirmDeleteItem(Uuid)`
Menghapus item (dengan konfirmasi).

#### `ToggleFavorite(Uuid)`
Toggle status favorite item.

### Tag Messages

#### `CreateTag(Tag)`
Membuat tag baru.

#### `DeleteTag(Uuid)`
Menghapus tag.

#### `AddTagToItem { item_id, tag_id }`
Menambahkan tag ke item.

#### `RemoveTagFromItem { item_id, tag_id }`
Menghapus tag dari item.

#### `ToggleFavoritesFilter`
Toggle filter untuk menampilkan favorites only.

### Input Messages

#### `InputChar(char)`
Menambahkan karakter ke input buffer.

#### `InputBackspace` / `InputDelete`
Menghapus karakter.

#### `InputLeft` / `InputRight`
Menggerakkan cursor.

#### `InputHome` / `InputEnd`
Jump ke awal/akhir input.

#### `InputSubmit`
Submit input (context-aware).

#### `InputCancel`
Membatalkan input.

### Search Messages

#### `OpenSearch`
Membuka dialog pencarian.

#### `UpdateSearchQuery(String)`
Mengupdate query pencarian.

#### `SearchNextResult` / `SearchPrevResult`
Navigasi hasil pencarian.

#### `SearchConfirm`
Memilih hasil pencarian.

### Kind Selector Messages

#### `KindSelectorNext` / `KindSelectorPrev`
Navigasi pilihan tipe item.

#### `KindSelectorConfirm`
Konfirmasi pilihan tipe.

### Form Messages

#### `FormNextField` / `FormPrevField`
Navigasi antar field.

#### `FormSubmit`
Submit form.

### Clipboard Messages

#### `CopyToClipboard { content, is_sensitive }`
Menyalin konten ke clipboard.

```rust
Message::CopyToClipboard {
    content: "secret".to_string(),
    is_sensitive: true,
}
```

#### `CopyCurrentItem`
Menyalin konten item yang dipilih.

#### `ClearClipboard`
Membersihkan clipboard.

### UI Messages

#### `OpenFloatingWindow(FloatingWindow)`
Membuka floating window.

```rust
Message::OpenFloatingWindow(FloatingWindow::Help)
```

#### `CloseFloatingWindow`
Menutup floating window.

#### `ToggleContentReveal`
Toggle reveal konten sensitif.

#### `Scroll(ScrollDirection)`
Scroll konten.

```rust
Message::Scroll(ScrollDirection::Down)
```

#### `ShowNotification { message, level }`
Menampilkan notifikasi.

```rust
Message::ShowNotification {
    message: "Success!".to_string(),
    level: NotificationLevel::Success,
}
```

#### `DismissNotification(usize)`
Menutup notifikasi berdasarkan ID.

### History Messages

#### `Undo` / `Redo`
Undo/redo aksi terakhir.

### System Messages

#### `Tick`
Timer tick untuk background tasks.

#### `Quit`
Keluar aplikasi (dengan save prompt jika dirty).

#### `ForceQuit`
Keluar tanpa konfirmasi.

#### `Noop`
No operation (untuk routing fallback).

## Supporting Types

### ItemUpdates

```rust
#[derive(Default, Clone)]
pub struct ItemUpdates {
    pub title: Option<String>,
    pub content: Option<ItemContent>,
    pub tags: Option<Vec<Uuid>>,
    pub favorite: Option<bool>,
    pub notes: Option<Option<String>>,
}
```

### ExportFormat

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,           // Unencrypted JSON
    EncryptedJson,  // Encrypted JSON
}
```

### ScrollDirection

```rust
pub enum ScrollDirection {
    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,
}
```

### NotificationLevel

```rust
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}
```

## Message Flow

```
User Input
    │
    ▼
crossterm::Event
    │
    ▼
route_event() ──▶ Message
    │
    ▼
update() ──▶ (State mutation, Effect)
    │
    ▼
runtime.execute() ──▶ EffectResult
    │
    ▼
handle_effect_result() ──▶ State update
```

## Testing Messages

```rust
#[test]
fn test_select_item() {
    let mut state = create_test_state();
    let item_id = state.vault_state.as_ref().unwrap()
        .vault.items[0].id;
    
    let effect = update(&mut state, Message::SelectItem(item_id));
    
    assert_eq!(
        state.vault_state.as_ref().unwrap().selected_item_id,
        Some(item_id)
    );
    assert!(effect.is_none());
}
```
