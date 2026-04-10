//! Edit form widget - for creating and editing items
//!
//! A floating form dialog with multiple input fields.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::domain::ItemKind;
use crate::ui::theme::ThemePalette;
use crate::utils::icons;

/// Form field types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormField {
    Title,
    Username,
    Password,
    Url,
    SeedPhrase,
    DerivationPath,
    Network,
    ApiKey,
    Service,
    Content,
    Issuer,
    AccountName,
    TotpSecret,
    CustomFields,
    Notes,
}

impl FormField {
    /// Get display label for the field
    pub fn label(&self) -> &'static str {
        match self {
            FormField::Title => "Title",
            FormField::Username => "Username",
            FormField::Password => "Password",
            FormField::Url => "URL",
            FormField::SeedPhrase => "Seed Phrase",
            FormField::DerivationPath => "Derivation Path",
            FormField::Network => "Network",
            FormField::ApiKey => "API Key",
            FormField::Service => "Service",
            FormField::Content => "Content",
            FormField::Issuer => "Issuer",
            FormField::AccountName => "Account Name",
            FormField::TotpSecret => "TOTP Secret",
            FormField::CustomFields => "Fields (type:key=value; ...)",
            FormField::Notes => "Notes",
        }
    }

    /// Check if this field should be masked
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            FormField::Password | FormField::SeedPhrase | FormField::ApiKey | FormField::Content | FormField::TotpSecret
        )
    }
}

/// State for the edit form
#[derive(Debug, Clone)]
pub struct EditFormState {
    /// Item kind being edited
    pub kind: ItemKind,
    /// Fields for this form
    pub fields: Vec<FormField>,
    /// Current field values
    pub values: Vec<String>,
    /// Currently focused field index
    pub focused_field: usize,
    /// Cursor position in focused field
    pub cursor: usize,
    /// Whether this is a new item (vs editing existing)
    pub is_new: bool,
}

impl EditFormState {
    /// Create a new form for item kind
    pub fn new(kind: ItemKind, is_new: bool) -> Self {
        let fields = get_fields_for_kind(kind);
        let values = vec![String::new(); fields.len()];

        Self {
            kind,
            fields,
            values,
            focused_field: 0,
            cursor: 0,
            is_new,
        }
    }

    /// Set the title field value
    pub fn set_title(&mut self, title: &str) {
        if let Some(idx) = self.fields.iter().position(|f| *f == FormField::Title) {
            self.values[idx] = title.to_string();
        }
    }

    /// Get the current field being edited
    pub fn current_field(&self) -> Option<&FormField> {
        self.fields.get(self.focused_field)
    }

    /// Get the current field value
    pub fn current_value(&self) -> &str {
        self.values
            .get(self.focused_field)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Get the current field value mutably
    pub fn current_value_mut(&mut self) -> Option<&mut String> {
        self.values.get_mut(self.focused_field)
    }

    /// Move to next field
    pub fn next_field(&mut self) {
        if self.focused_field < self.fields.len() - 1 {
            self.focused_field += 1;
            self.cursor = self.values[self.focused_field].len();
        }
    }

    /// Move to previous field
    pub fn prev_field(&mut self) {
        if self.focused_field > 0 {
            self.focused_field -= 1;
            self.cursor = self.values[self.focused_field].len();
        }
    }

    /// Insert a character at cursor
    pub fn insert(&mut self, c: char) {
        let cursor = self.cursor;
        if let Some(value) = self.values.get_mut(self.focused_field) {
            value.insert(cursor, c);
            self.cursor += c.len_utf8();
        }
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let cursor = self.cursor;
            if let Some(value) = self.values.get_mut(self.focused_field) {
                let prev = value[..cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                value.remove(prev);
                self.cursor = prev;
            }
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            let value = self.current_value();
            self.cursor = value[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        let len = self.current_value().len();
        if self.cursor < len {
            let value = self.current_value();
            self.cursor = value[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(len);
        }
    }

    /// Get value for a specific field
    pub fn get_value(&self, field: &FormField) -> Option<&str> {
        self.fields
            .iter()
            .position(|f| f == field)
            .and_then(|idx| self.values.get(idx))
            .map(|s| s.as_str())
    }

    /// Validate the form
    pub fn validate(&self) -> Result<(), &'static str> {
        // Title is required
        let title = self.get_value(&FormField::Title).unwrap_or("");
        if title.trim().is_empty() {
            return Err("Title is required");
        }
        Ok(())
    }
}

/// Get the fields needed for an item kind
fn get_fields_for_kind(kind: ItemKind) -> Vec<FormField> {
    let mut fields = vec![FormField::Title];

    match kind {
        ItemKind::Generic => {
            fields.push(FormField::Content);
        }
        ItemKind::CryptoSeed => {
            fields.push(FormField::SeedPhrase);
            fields.push(FormField::DerivationPath);
            fields.push(FormField::Network);
        }
        ItemKind::Password => {
            fields.push(FormField::Username);
            fields.push(FormField::Password);
            fields.push(FormField::Url);
            fields.push(FormField::TotpSecret);
        }
        ItemKind::SecureNote => {
            fields.push(FormField::Content);
        }
        ItemKind::ApiKey => {
            fields.push(FormField::Service);
            fields.push(FormField::ApiKey);
        }
        ItemKind::Totp => {
            fields.push(FormField::Issuer);
            fields.push(FormField::AccountName);
            fields.push(FormField::TotpSecret);
        }
        ItemKind::Custom => {
            fields.push(FormField::CustomFields);
        }
    }

    fields.push(FormField::Notes);
    fields
}

#[derive(Debug, Clone)]
pub struct FormClickRegions {
    pub field_regions: Vec<(usize, crate::input::mouse::ClickRegion)>,
    pub button_regions: Vec<crate::ui::widgets::ButtonRegion>,
    pub form_area: crate::input::mouse::ClickRegion,
}

/// Render the edit form and return clickable regions
pub fn render(
    frame: &mut Frame,
    area: Rect,
    form_state: &EditFormState,
    theme: &ThemePalette,
) -> FormClickRegions {
    // Calculate form dimensions
    let form_width = area.width.min(70);
    let form_height = (form_state.fields.len() as u16 * 3 + 6).min(area.height - 4);
    let x = (area.width.saturating_sub(form_width)) / 2;
    let y = (area.height.saturating_sub(form_height)) / 2;

    let form_area = Rect::new(x, y, form_width, form_height);

    // Clear background
    frame.render_widget(Clear, form_area);

    // Build title
    let title = if form_state.is_new {
        format!(
            " {} New {} ",
            icons::ui::ADD,
            form_state.kind.display_name()
        )
    } else {
        format!(
            " {} Edit {} ",
            icons::ui::EDIT,
            form_state.kind.display_name()
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(form_area);
    frame.render_widget(block, form_area);

    // Layout for fields + buttons (hints now embedded in buttons)
    let mut constraints: Vec<Constraint> = form_state
        .fields
        .iter()
        .map(|_| Constraint::Length(3))
        .collect();
    constraints.push(Constraint::Min(1)); // Buttons with embedded hints

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Collect clickable field regions
    let mut field_regions = Vec::new();

    // Render each field
    for (i, field) in form_state.fields.iter().enumerate() {
        let is_focused = i == form_state.focused_field;
        let value = &form_state.values[i];

        render_field(
            frame,
            chunks[i],
            field,
            value,
            is_focused,
            if is_focused { form_state.cursor } else { 0 },
            theme,
        );

        // Register this field as clickable
        field_regions.push((
            i,
            crate::input::mouse::ClickRegion::new(
                chunks[i].x,
                chunks[i].y,
                chunks[i].width,
                chunks[i].height,
            ),
        ));
    }

    // Render action buttons at bottom (now includes keyboard hints in labels)
    let button_area = chunks[form_state.fields.len()];
    let button_regions = render_form_buttons(frame, button_area, theme);

    FormClickRegions {
        field_regions,
        button_regions,
        form_area: crate::input::mouse::ClickRegion::new(
            form_area.x,
            form_area.y,
            form_area.width,
            form_area.height,
        ),
    }
}

/// Render a single form field
fn render_field(
    frame: &mut Frame,
    area: Rect,
    field: &FormField,
    value: &str,
    focused: bool,
    cursor: usize,
    theme: &ThemePalette,
) {
    let border_color = if focused {
        theme.border_focused
    } else {
        theme.border
    };

    // Display value (mask if sensitive and not focused)
    let display_value = if field.is_sensitive() && !focused {
        "•".repeat(value.chars().count())
    } else {
        value.to_string()
    };

    // Add cursor indicator if focused
    let text = if focused {
        let before: String = display_value.chars().take(cursor).collect();
        let after: String = display_value.chars().skip(cursor).collect();
        format!("{}│{}", before, after)
    } else {
        display_value
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(theme.fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    format!(" {} ", field.label()),
                    Style::default().fg(if focused {
                        theme.accent
                    } else {
                        theme.fg_muted
                    }),
                )),
        );

    frame.render_widget(paragraph, area);
}

/// Render form action buttons with embedded keyboard hints
fn render_form_buttons(
    frame: &mut Frame,
    area: Rect,
    theme: &ThemePalette,
) -> Vec<crate::ui::widgets::ButtonRegion> {
    use crate::ui::widgets::{ButtonStyle, render_button_row};

    let buttons = vec![
        (
            "form-save".to_string(),
            "Save",
            Some("Enter"),
            ButtonStyle::Primary,
        ),
        (
            "form-cancel".to_string(),
            "Cancel",
            Some("Esc"),
            ButtonStyle::Secondary,
        ),
    ];

    render_button_row(frame, area, &buttons, theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_fields_password() {
        let form = EditFormState::new(ItemKind::Password, true);
        assert!(form.fields.contains(&FormField::Title));
        assert!(form.fields.contains(&FormField::Username));
        assert!(form.fields.contains(&FormField::Password));
        assert!(form.fields.contains(&FormField::Notes));
    }

    #[test]
    fn test_form_fields_crypto() {
        let form = EditFormState::new(ItemKind::CryptoSeed, true);
        assert!(form.fields.contains(&FormField::SeedPhrase));
        assert!(form.fields.contains(&FormField::DerivationPath));
    }

    #[test]
    fn test_form_fields_custom() {
        let form = EditFormState::new(ItemKind::Custom, true);
        assert!(form.fields.contains(&FormField::Title));
        assert!(form.fields.contains(&FormField::CustomFields));
        assert!(form.fields.contains(&FormField::Notes));
    }

    #[test]
    fn test_form_navigation() {
        let mut form = EditFormState::new(ItemKind::Password, true);
        assert_eq!(form.focused_field, 0);

        form.next_field();
        assert_eq!(form.focused_field, 1);

        form.prev_field();
        assert_eq!(form.focused_field, 0);

        form.prev_field(); // Should stay at 0
        assert_eq!(form.focused_field, 0);
    }

    #[test]
    fn test_form_input() {
        let mut form = EditFormState::new(ItemKind::Generic, true);

        form.insert('H');
        form.insert('e');
        form.insert('l');
        form.insert('l');
        form.insert('o');

        assert_eq!(form.current_value(), "Hello");

        form.backspace();
        assert_eq!(form.current_value(), "Hell");
    }

    #[test]
    fn test_form_validation() {
        let form = EditFormState::new(ItemKind::Generic, true);
        assert!(form.validate().is_err());

        let mut form = EditFormState::new(ItemKind::Generic, true);
        form.set_title("Test Item");
        assert!(form.validate().is_ok());
    }
}
