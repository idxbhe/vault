//! Mouse event handling
//!
//! Processes mouse events for click and scroll interactions.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use std::cell::Cell;
use std::time::Instant;
use uuid::Uuid;

/// Region of the screen for click detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickRegion {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl ClickRegion {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if a point is within this region
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// Type of mouse action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Click { x: u16, y: u16, button: MouseButton },
    DoubleClick { x: u16, y: u16 },
    ScrollUp { x: u16, y: u16 },
    ScrollDown { x: u16, y: u16 },
    Drag { x: u16, y: u16 },
}

impl MouseAction {
    /// Get the position of the mouse action
    pub fn position(&self) -> (u16, u16) {
        match self {
            MouseAction::Click { x, y, .. } => (*x, *y),
            MouseAction::DoubleClick { x, y } => (*x, *y),
            MouseAction::ScrollUp { x, y } => (*x, *y),
            MouseAction::ScrollDown { x, y } => (*x, *y),
            MouseAction::Drag { x, y } => (*x, *y),
        }
    }

    /// Check if this action is within a region
    pub fn is_in_region(&self, region: &ClickRegion) -> bool {
        let (x, y) = self.position();
        region.contains(x, y)
    }
}

/// Parse a crossterm mouse event into our MouseAction
pub fn parse_mouse_event(event: MouseEvent) -> Option<MouseAction> {
    match event.kind {
        MouseEventKind::Down(button) => Some(MouseAction::Click {
            x: event.column,
            y: event.row,
            button,
        }),
        MouseEventKind::ScrollUp => Some(MouseAction::ScrollUp {
            x: event.column,
            y: event.row,
        }),
        MouseEventKind::ScrollDown => Some(MouseAction::ScrollDown {
            x: event.column,
            y: event.row,
        }),
        MouseEventKind::Drag(_) => Some(MouseAction::Drag {
            x: event.column,
            y: event.row,
        }),
        _ => None,
    }
}

/// Named regions for UI layout (general areas)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRegion {
    List,
    Detail,
    StatusBar,
    SearchBox,
    FloatingWindow,
}

/// Clickable element types with associated data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickableElement {
    /// A vault entry in login screen (index)
    VaultEntry(usize),
    /// An item in the main list (uuid)
    ListItem(Uuid),
    /// A form field (field index)
    FormField(usize),
    /// A kind option in selector (index)
    KindOption(usize),
    /// A search result (index)
    SearchResult(usize),
    /// A category option in the UI (ItemKind or None for All)
    CategoryOption(Option<crate::domain::ItemKind>),
    /// Left scroll button for categories
    CategoryScrollLeft,
    /// Right scroll button for categories
    CategoryScrollRight,
    /// A button with action name
    Button(String),
    /// Close button / click outside to close
    CloseArea,
}

/// A clickable region with associated element data
#[derive(Debug, Clone)]
pub struct ClickableRegion {
    pub region: ClickRegion,
    pub element: ClickableElement,
}

impl ClickableRegion {
    pub fn new(region: ClickRegion, element: ClickableElement) -> Self {
        Self { region, element }
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        self.region.contains(x, y)
    }
}

/// Layout regions for click detection
#[derive(Debug)]
pub struct LayoutRegions {
    regions: Vec<(UiRegion, ClickRegion)>,
    /// Clickable elements with their regions
    clickable_elements: Vec<ClickableRegion>,
    /// Last click time and position for double-click detection
    last_click: Cell<Option<(Instant, u16, u16)>>,
}

/// Double-click threshold in milliseconds
const DOUBLE_CLICK_MS: u128 = 400;
/// Double-click position tolerance in pixels
const DOUBLE_CLICK_TOLERANCE: u16 = 2;

impl LayoutRegions {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            clickable_elements: Vec::new(),
            last_click: Cell::new(None),
        }
    }

    /// Set a region
    pub fn set(&mut self, name: UiRegion, region: ClickRegion) {
        // Remove existing if present
        self.regions.retain(|(n, _)| *n != name);
        self.regions.push((name, region));
    }

    /// Find which region contains a point
    pub fn find_region(&self, x: u16, y: u16) -> Option<UiRegion> {
        // Search in reverse order so floating windows (added last) take priority
        for (name, region) in self.regions.iter().rev() {
            if region.contains(x, y) {
                return Some(*name);
            }
        }
        None
    }

    /// Clear all regions
    pub fn clear(&mut self) {
        self.regions.clear();
        self.clickable_elements.clear();
    }

    /// Register a clickable element
    pub fn register_clickable(&mut self, region: ClickRegion, element: ClickableElement) {
        self.clickable_elements
            .push(ClickableRegion::new(region, element));
    }

    /// Find clickable element at position
    pub fn find_clickable(&self, x: u16, y: u16) -> Option<&ClickableElement> {
        // Search in reverse order (last registered = highest priority)
        for clickable in self.clickable_elements.iter().rev() {
            if clickable.contains(x, y) {
                return Some(&clickable.element);
            }
        }
        None
    }

    /// Register a click and check for double-click
    /// Returns true if this is a double-click
    pub fn register_click(&self, x: u16, y: u16) -> bool {
        let now = Instant::now();

        if let Some((last_time, last_x, last_y)) = self.last_click.get() {
            let elapsed = now.duration_since(last_time).as_millis();
            let dx = (x as i32 - last_x as i32).unsigned_abs() as u16;
            let dy = (y as i32 - last_y as i32).unsigned_abs() as u16;

            if elapsed < DOUBLE_CLICK_MS
                && dx <= DOUBLE_CLICK_TOLERANCE
                && dy <= DOUBLE_CLICK_TOLERANCE
            {
                // Double-click detected, reset last_click
                self.last_click.set(None);
                return true;
            }
        }

        // Record this click for potential double-click
        self.last_click.set(Some((now, x, y)));
        false
    }
}

impl Default for LayoutRegions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_region() {
        let region = ClickRegion::new(10, 10, 20, 10);

        assert!(region.contains(10, 10)); // Top-left corner
        assert!(region.contains(15, 15)); // Inside
        assert!(region.contains(29, 19)); // Bottom-right (exclusive)
        assert!(!region.contains(30, 10)); // Right edge (exclusive)
        assert!(!region.contains(10, 20)); // Bottom edge (exclusive)
        assert!(!region.contains(5, 10)); // Left of region
    }

    #[test]
    fn test_layout_regions() {
        let mut layout = LayoutRegions::new();

        layout.set(UiRegion::List, ClickRegion::new(0, 0, 30, 20));
        layout.set(UiRegion::Detail, ClickRegion::new(30, 0, 50, 20));
        layout.set(UiRegion::StatusBar, ClickRegion::new(0, 20, 80, 1));

        assert_eq!(layout.find_region(10, 10), Some(UiRegion::List));
        assert_eq!(layout.find_region(40, 10), Some(UiRegion::Detail));
        assert_eq!(layout.find_region(40, 20), Some(UiRegion::StatusBar));
    }

    #[test]
    fn test_floating_priority() {
        let mut layout = LayoutRegions::new();

        layout.set(UiRegion::List, ClickRegion::new(0, 0, 80, 24));
        layout.set(UiRegion::FloatingWindow, ClickRegion::new(20, 5, 40, 10));

        // Floating window should take priority
        assert_eq!(layout.find_region(30, 10), Some(UiRegion::FloatingWindow));
        // Outside floating but inside list
        assert_eq!(layout.find_region(5, 5), Some(UiRegion::List));
    }

    #[test]
    fn test_clickable_elements() {
        let mut layout = LayoutRegions::new();

        layout.register_clickable(
            ClickRegion::new(0, 0, 30, 1),
            ClickableElement::VaultEntry(0),
        );
        layout.register_clickable(
            ClickRegion::new(0, 1, 30, 1),
            ClickableElement::VaultEntry(1),
        );

        assert_eq!(
            layout.find_clickable(10, 0),
            Some(&ClickableElement::VaultEntry(0))
        );
        assert_eq!(
            layout.find_clickable(10, 1),
            Some(&ClickableElement::VaultEntry(1))
        );
        assert_eq!(layout.find_clickable(10, 5), None);
    }

    #[test]
    fn test_double_click_detection() {
        let layout = LayoutRegions::new();

        // First click - not a double-click
        assert!(!layout.register_click(10, 10));

        // Second click immediately at same position - double-click
        assert!(layout.register_click(10, 10));

        // Third click - not a double-click (previous was consumed)
        assert!(!layout.register_click(10, 10));
    }
}
