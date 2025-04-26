use std::fmt;
use thiserror::Error;
/// Represents errors that can occur during parsing or serialization
#[derive(Debug, Error)]
pub enum ParseError {
    /// Error parsing markdown content
    Markdown(String),
    /// Error parsing HTML content
    Html(String),
    /// Error parsing JSON content
    Json(String),
    /// Generic parsing error
    Generic(String),
}

/// Represents errors that can occur during document editing operations
#[derive(Debug, Error)]
pub enum EditError {
    /// The index is out of bounds
    IndexOutOfBounds,
    /// The operation is not supported for the given node type
    UnsupportedOperation,
    /// The range is invalid (e.g., end before start)
    InvalidRange,
    /// The operation was attempted on an invalid node
    InvalidNode,
    /// The operation could not be completed successfully
    OperationFailed,
    /// Other error with a message
    Other(String),
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            EditError::UnsupportedOperation => {
                write!(f, "Operation not supported for this node type")
            }
            EditError::InvalidRange => write!(f, "Invalid range provided"),
            EditError::InvalidNode => write!(f, "Operation attempted on invalid node"),
            EditError::OperationFailed => write!(f, "Operation failed to complete"),
            EditError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Markdown(msg) => write!(f, "Markdown parse error: {}", msg),
            ParseError::Html(msg) => write!(f, "HTML parse error: {}", msg),
            ParseError::Json(msg) => write!(f, "JSON parse error: {}", msg),
            ParseError::Generic(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::Json(err.to_string())
    }
}
