use std::{convert::Infallible, fmt, ops::Deref, str::FromStr};

pub mod html;
pub mod json;
pub mod markdown;

pub struct Html;
pub struct Json;
pub struct Markdown;

pub struct Text<T> {
    text: String,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Text<T> {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn into_inner(self) -> String {
        self.text
    }
}

impl<T> From<String> for Text<T> {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl<T> FromStr for Text<T> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl<T> fmt::Display for Text<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl<T> Deref for Text<T> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

/// Escape HTML special characters
pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use std::convert::TryInto;

    fn create_test_document() -> Document {
        let mut doc = Document::new();
        doc.add_heading(1, "Test Document");
        doc.add_paragraph_with_text("This is a test paragraph.");
        doc
    }

    #[test]
    fn test_html_conversion() {
        let doc = create_test_document();

        // Document to HTML
        let html: Result<Text<Html>, _> = doc.as_ref().try_into();
        assert!(html.is_ok());
        let html = html.unwrap();
        assert!(html.as_str().contains("<h1>Test Document</h1>"));

        // HTML to Document
        let doc2: Result<Document, _> = html.try_into();
        assert!(doc2.is_ok());
        let doc2 = doc2.unwrap();
        assert_eq!(doc2.nodes.len(), doc.nodes.len());
    }

    #[test]
    fn test_json_conversion() {
        let doc = create_test_document();

        // Document to JSON
        let json: Result<Text<Json>, _> = doc.as_ref().try_into();
        assert!(json.is_ok());
        let json = json.unwrap();
        assert!(json.as_str().contains("Test Document"));

        // JSON to Document
        let doc2: Result<Document, _> = json.try_into();
        assert!(doc2.is_ok());
        let doc2 = doc2.unwrap();
        assert_eq!(doc2.nodes.len(), doc.nodes.len());
    }

    #[test]
    fn test_markdown_conversion() {
        let doc = create_test_document();

        // Document to Markdown
        let md: Result<Text<Markdown>, _> = doc.as_ref().try_into();
        assert!(md.is_ok());
        let md = md.unwrap();
        assert!(md.as_str().contains("# Test Document"));

        // Markdown to Document
        let doc2: Result<Document, _> = md.try_into();
        assert!(doc2.is_ok());
        let doc2 = doc2.unwrap();
        assert_eq!(doc2.nodes.len(), doc.nodes.len());
    }
}
