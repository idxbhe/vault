//! Mouse event handling
//!
//! Processes mouse events for click and scroll interactions.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

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
        Self { x, y, width, height }
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

/// Named regions for UI layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRegion {
    List,
    Detail,
    StatusBar,
    SearchBox,
    FloatingWindow,
}

/// Layout regions for click detection
#[derive(Debug, Default)]
pub struct LayoutRegions {
    regions: Vec<(UiRegion, ClickRegion)>,
}

impl LayoutRegions {
    pub fn new() -> Self {
        Self::default()
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
}
