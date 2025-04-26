use serde::{Deserialize, Serialize};

/// Represents a position within the document
/// Path is a series of indices to traverse the document tree
/// Offset is the character offset within the final node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Path to the node containing the position
    pub path: Vec<usize>,
    /// Character offset within the node
    pub offset: usize,
}

impl Position {
    /// Creates a new position
    pub fn new(path: Vec<usize>, offset: usize) -> Self {
        Self { path, offset }
    }

    /// Creates a position at the start of the document
    pub fn start() -> Self {
        Self {
            path: vec![0],
            offset: 0,
        }
    }
}

/// Represents a selection range within the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Start position of the selection
    pub start: Position,
    /// End position of the selection
    pub end: Position,
    /// Whether the selection is collapsed to a single point
    pub is_collapsed: bool,
}

impl Selection {
    /// Creates a new selection
    pub fn new(start: Position, end: Position) -> Self {
        let is_collapsed = start == end;
        Self {
            start,
            end,
            is_collapsed,
        }
    }

    /// Creates a collapsed selection at the specified position
    pub fn collapsed(position: Position) -> Self {
        Self {
            start: position.clone(),
            end: position,
            is_collapsed: true,
        }
    }

    /// Creates a selection at the start of the document
    pub fn at_start() -> Self {
        let position = Position::start();
        Self::collapsed(position)
    }
}
