//! History and undo/redo system for items

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::item::ItemContent;

/// A single history entry representing a past state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique identifier for this entry
    pub id: Uuid,
    /// When this change occurred
    pub timestamp: DateTime<Utc>,
    /// What action created this entry
    pub action: HistoryAction,
    /// Snapshot of the item at this point
    pub snapshot: ItemSnapshot,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(action: HistoryAction, snapshot: ItemSnapshot) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action,
            snapshot,
        }
    }

    /// Create an entry for item creation
    pub fn created(snapshot: ItemSnapshot) -> Self {
        Self::new(HistoryAction::Created, snapshot)
    }

    /// Create an entry for item modification
    pub fn modified(field: impl Into<String>, snapshot: ItemSnapshot) -> Self {
        Self::new(
            HistoryAction::Modified {
                field: field.into(),
            },
            snapshot,
        )
    }

    /// Create an entry for restoration from history
    pub fn restored(from_entry: Uuid, snapshot: ItemSnapshot) -> Self {
        Self::new(HistoryAction::Restored { from_entry }, snapshot)
    }
}

/// Types of actions that create history entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryAction {
    /// Item was created
    Created,
    /// Item was modified (with field name)
    Modified { field: String },
    /// Item was restored from a previous entry
    Restored { from_entry: Uuid },
}

impl HistoryAction {
    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self {
            HistoryAction::Created => "Created".to_string(),
            HistoryAction::Modified { field } => format!("Modified {}", field),
            HistoryAction::Restored { .. } => "Restored from history".to_string(),
        }
    }
}

/// Snapshot of an item's state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSnapshot {
    /// Title at this point
    pub title: String,
    /// Content at this point
    pub content: ItemContent,
    /// Notes at this point
    pub notes: Option<String>,
    /// Tags at this point
    pub tags: Vec<Uuid>,
}

impl ItemSnapshot {
    /// Create a snapshot from current item state
    pub fn new(
        title: impl Into<String>,
        content: ItemContent,
        notes: Option<String>,
        tags: Vec<Uuid>,
    ) -> Self {
        Self {
            title: title.into(),
            content,
            notes,
            tags,
        }
    }
}

/// Undo/redo stack for vault-level operations
#[derive(Debug, Default)]
pub struct UndoStack {
    /// Stack of undoable entries
    undo: Vec<UndoEntry>,
    /// Stack of redoable entries
    redo: Vec<UndoEntry>,
    /// Maximum stack size
    max_size: usize,
}

/// An entry in the undo/redo stack
#[derive(Debug, Clone)]
pub struct UndoEntry {
    /// Description of what can be undone
    pub description: String,
    /// The item that was changed
    pub item_id: Uuid,
    /// The previous state
    pub previous_state: ItemSnapshot,
}

impl UndoEntry {
    /// Create a new undo entry
    pub fn new(
        description: impl Into<String>,
        item_id: Uuid,
        previous_state: ItemSnapshot,
    ) -> Self {
        Self {
            description: description.into(),
            item_id,
            previous_state,
        }
    }
}

impl UndoStack {
    /// Create a new undo stack with default size limit
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_size: 50,
        }
    }

    /// Create with custom size limit
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_size,
        }
    }

    /// Push a new entry onto the undo stack
    pub fn push(&mut self, entry: UndoEntry) {
        // Clear redo stack when new action is performed
        self.redo.clear();

        self.undo.push(entry);

        // Trim if exceeds max size
        if self.undo.len() > self.max_size {
            self.undo.remove(0);
        }
    }

    /// Pop from undo stack (for undoing)
    pub fn pop_undo(&mut self) -> Option<UndoEntry> {
        self.undo.pop()
    }

    /// Push to redo stack (when undoing)
    pub fn push_redo(&mut self, entry: UndoEntry) {
        self.redo.push(entry);
    }

    /// Pop from redo stack (for redoing)
    pub fn pop_redo(&mut self) -> Option<UndoEntry> {
        self.redo.pop()
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Get the description of the next undo action
    pub fn undo_description(&self) -> Option<&str> {
        self.undo.last().map(|e| e.description.as_str())
    }

    /// Get the description of the next redo action
    pub fn redo_description(&self) -> Option<&str> {
        self.redo.last().map(|e| e.description.as_str())
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_entry_creation() {
        let snapshot = ItemSnapshot::new(
            "Test",
            ItemContent::Generic {
                value: "value".to_string(),
            },
            None,
            vec![],
        );
        let entry = HistoryEntry::created(snapshot);

        matches!(entry.action, HistoryAction::Created);
    }

    #[test]
    fn test_undo_stack() {
        let mut stack = UndoStack::new();

        assert!(!stack.can_undo());
        assert!(!stack.can_redo());

        let snapshot = ItemSnapshot::new(
            "Test",
            ItemContent::Generic {
                value: "old".to_string(),
            },
            None,
            vec![],
        );
        let entry = UndoEntry::new("Edit title", Uuid::new_v4(), snapshot);
        stack.push(entry);

        assert!(stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(stack.undo_description(), Some("Edit title"));

        let undone = stack.pop_undo().unwrap();
        stack.push_redo(undone);

        assert!(!stack.can_undo());
        assert!(stack.can_redo());
    }

    #[test]
    fn test_undo_stack_max_size() {
        let mut stack = UndoStack::with_max_size(3);

        for i in 0..5 {
            let snapshot = ItemSnapshot::new(
                format!("Item {}", i),
                ItemContent::Generic {
                    value: "v".to_string(),
                },
                None,
                vec![],
            );
            stack.push(UndoEntry::new(
                format!("Action {}", i),
                Uuid::new_v4(),
                snapshot,
            ));
        }

        // Should only have 3 entries
        let mut count = 0;
        while stack.pop_undo().is_some() {
            count += 1;
        }
        assert_eq!(count, 3);
    }
}
