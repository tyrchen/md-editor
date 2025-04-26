use super::Document;

mod basic;
mod selection;

/// A builder for creating documents with a fluent API
pub struct DocumentBuilder {
    /// The document being built
    document: Document,
}
