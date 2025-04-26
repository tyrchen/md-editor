use serde::{Deserialize, Serialize};

/// Represents text formatting options for text nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TextFormatting {
    /// Whether the text is bold
    pub bold: bool,
    /// Whether the text is italic
    pub italic: bool,
    /// Whether the text has strikethrough
    pub strikethrough: bool,
    /// Whether the text is code (monospace)
    pub code: bool,
}

impl TextFormatting {
    /// Creates a new default formatting (no styling)
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates bold formatting
    pub fn bold() -> Self {
        Self {
            bold: true,
            ..Default::default()
        }
    }

    /// Creates italic formatting
    pub fn italic() -> Self {
        Self {
            italic: true,
            ..Default::default()
        }
    }

    /// Creates code formatting
    pub fn code() -> Self {
        Self {
            code: true,
            ..Default::default()
        }
    }

    /// Creates strikethrough formatting
    pub fn strikethrough() -> Self {
        Self {
            strikethrough: true,
            ..Default::default()
        }
    }

    /// Adds bold to existing formatting
    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Adds italic to existing formatting
    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Adds code to existing formatting
    pub fn with_code(mut self) -> Self {
        self.code = true;
        self
    }

    /// Adds strikethrough to existing formatting
    pub fn with_strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }
}
