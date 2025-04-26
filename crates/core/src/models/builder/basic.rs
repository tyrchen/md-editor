use super::DocumentBuilder;
use crate::{Document, DocumentMetadata, Markdown, Node, ParseError, TableAlignment, Text};

impl DocumentBuilder {
    /// Creates a new document builder
    pub fn new() -> Self {
        Self {
            document: Document::new(),
        }
    }

    /// Creates a new document builder from a markdown string
    pub fn from_markdown(markdown: impl Into<String>) -> Result<Self, ParseError> {
        let text = Text::<Markdown>::new(markdown);
        let document = Document::try_from(text)?;
        Ok(Self { document })
    }

    /// Sets a title for the document, creating a title heading
    pub fn title(mut self, title: impl Into<String>) -> Self {
        let title_str = title.into();
        self.document
            .nodes
            .push(Node::heading(1, title_str.clone()));
        self.document.metadata = Some(DocumentMetadata {
            title: Some(title_str),
            ..Default::default()
        });
        self
    }

    /// Sets the author metadata for the document
    pub fn author(mut self, author: impl Into<String>) -> Self {
        let author_str = author.into();
        if let Some(metadata) = &mut self.document.metadata {
            metadata.author = Some(author_str);
        } else {
            self.document.metadata = Some(DocumentMetadata {
                author: Some(author_str),
                ..Default::default()
            });
        }
        self
    }

    /// Sets the date metadata for the document
    pub fn date(mut self, date: impl Into<String>) -> Self {
        let date_str = date.into();
        if let Some(metadata) = &mut self.document.metadata {
            metadata.date = Some(date_str);
        } else {
            self.document.metadata = Some(DocumentMetadata {
                date: Some(date_str),
                ..Default::default()
            });
        }
        self
    }

    /// Adds custom metadata to the document
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key_str = key.into();
        let value_str = value.into();

        if let Some(metadata) = &mut self.document.metadata {
            metadata.custom.push((key_str, value_str));
        } else {
            let mut metadata = DocumentMetadata::default();
            metadata.custom.push((key_str, value_str));
            self.document.metadata = Some(metadata);
        }
        self
    }

    /// Adds a heading to the document
    pub fn heading(mut self, level: u8, text: impl Into<String>) -> Self {
        self.document.nodes.push(Node::heading(level, text));
        self
    }

    /// Adds an empty paragraph to the document
    pub fn empty_paragraph(mut self) -> Self {
        self.document.nodes.push(Node::Paragraph {
            children: Vec::new(),
        });
        self
    }

    /// Adds a paragraph with text to the document
    pub fn paragraph(mut self, text: impl Into<String>) -> Self {
        self.document.nodes.push(Node::paragraph(text));
        self
    }

    /// Adds a code block to the document
    pub fn code_block(mut self, code: impl Into<String>, language: impl Into<String>) -> Self {
        self.document.nodes.push(Node::code_block(code, language));
        self
    }

    /// Adds a blockquote to the document
    pub fn blockquote(mut self, text: impl Into<String>) -> Self {
        self.document.nodes.push(Node::blockquote(text));
        self
    }

    /// Adds an unordered list to the document
    pub fn unordered_list(mut self, items: Vec<impl Into<String>>) -> Self {
        self.document.nodes.push(Node::unordered_list(items));
        self
    }

    /// Adds an ordered list to the document
    pub fn ordered_list(mut self, items: Vec<impl Into<String>>) -> Self {
        self.document.nodes.push(Node::ordered_list(items));
        self
    }

    /// Adds a task list to the document
    pub fn task_list(mut self, items: Vec<(impl Into<String>, bool)>) -> Self {
        self.document.nodes.push(Node::task_list(items));
        self
    }

    /// Adds a horizontal rule to the document
    pub fn horizontal_rule(mut self) -> Self {
        self.document.nodes.push(Node::horizontal_rule());
        self
    }

    /// Adds a simple table to the document
    pub fn table(
        mut self,
        headers: Vec<impl Into<String>>,
        rows: Vec<Vec<impl Into<String>>>,
    ) -> Self {
        self.document.nodes.push(Node::simple_table(headers, rows));
        self
    }

    /// Adds a table with column alignments to the document
    pub fn table_with_alignments(
        mut self,
        headers: Vec<impl Into<String>>,
        rows: Vec<Vec<impl Into<String>>>,
        alignments: Vec<TableAlignment>,
    ) -> Self {
        self.document
            .nodes
            .push(Node::table_with_alignments(headers, rows, alignments));
        self
    }

    /// Adds a math block to the document
    pub fn math_block(mut self, math: impl Into<String>) -> Self {
        self.document.nodes.push(Node::math_block(math));
        self
    }

    /// Adds a footnote reference to the document
    pub fn footnote_reference(mut self, label: impl Into<String>) -> Self {
        self.document.nodes.push(Node::footnote_reference(label));
        self
    }

    /// Adds a footnote definition to the document
    pub fn footnote_definition(
        mut self,
        label: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        self.document
            .nodes
            .push(Node::footnote_definition(label, text));
        self
    }

    /// Adds a group of nodes with a name to the document
    pub fn group(
        mut self,
        name: impl Into<String>,
        builder: impl FnOnce(DocumentBuilder) -> DocumentBuilder,
    ) -> Self {
        // Create a new builder for the group content
        let group_builder = builder(DocumentBuilder::new());
        // Extract the nodes from the built document
        let children = group_builder.document.nodes;
        // Add the group to the current document
        self.document.nodes.push(Node::group(name, children));
        self
    }

    /// Builds the document
    pub fn build(self) -> Document {
        self.document
    }
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InlineNode, ListType};

    #[test]
    fn test_document_builder_basic() {
        let doc = DocumentBuilder::new()
            .title("Test Document")
            .author("Test Author")
            .date("2023-05-01")
            .paragraph("This is a paragraph")
            .build();

        assert_eq!(doc.nodes.len(), 2);

        // Check title
        match &doc.nodes[0] {
            Node::Heading { level, children } => {
                assert_eq!(*level, 1);
                assert_eq!(children.len(), 1);
                match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "Test Document");
                    }
                    _ => panic!("Expected Text node"),
                }
            }
            _ => panic!("Expected Heading node"),
        }

        // Check metadata
        assert!(doc.metadata.is_some());
        let metadata = doc.metadata.unwrap();
        assert_eq!(metadata.title, Some("Test Document".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.date, Some("2023-05-01".to_string()));
    }

    #[test]
    fn test_document_builder_complex() {
        let doc = DocumentBuilder::new()
            .title("Complex Document")
            .heading(2, "Section 1")
            .paragraph("Section 1 content")
            .code_block("fn main() {\n    println!(\"Hello, world!\");\n}", "rust")
            .heading(2, "Section 2")
            .paragraph("Section 2 content")
            .unordered_list(vec!["Item 1", "Item 2", "Item 3"])
            .ordered_list(vec!["First", "Second", "Third"])
            .task_list(vec![("Task 1", true), ("Task 2", false)])
            .horizontal_rule()
            .table(
                vec!["Header 1", "Header 2"],
                vec![
                    vec!["Row 1, Cell 1", "Row 1, Cell 2"],
                    vec!["Row 2, Cell 1", "Row 2, Cell 2"],
                ],
            )
            .build();

        // Basic structure validation
        assert_eq!(doc.nodes.len(), 11);

        // Check title
        match &doc.nodes[0] {
            Node::Heading { level, .. } => {
                assert_eq!(*level, 1);
            }
            _ => panic!("Expected Heading node"),
        }

        // Check code block
        match &doc.nodes[3] {
            Node::CodeBlock { language, code } => {
                assert_eq!(language, "rust");
                assert!(code.contains("println!"));
            }
            _ => panic!("Expected CodeBlock node"),
        }

        // Check lists
        match &doc.nodes[6] {
            Node::List { list_type, items } => {
                assert_eq!(*list_type, ListType::Unordered);
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected List node"),
        }

        // Check horizontal rule
        match &doc.nodes[9] {
            Node::ThematicBreak => {}
            _ => panic!("Expected ThematicBreak node"),
        }

        // Check table
        match &doc.nodes[10] {
            Node::Table { header, rows, .. } => {
                assert_eq!(header.len(), 2);
                assert_eq!(rows.len(), 2);
            }
            _ => panic!("Expected Table node"),
        }
    }

    #[test]
    fn test_document_builder_with_groups() {
        let doc = DocumentBuilder::new()
            .title("Document with Groups")
            .paragraph("Top level paragraph")
            .group("Section", |builder| {
                builder
                    .heading(2, "Section Heading")
                    .paragraph("Section content")
                    .code_block("let x = 42;", "rust")
            })
            .paragraph("Another top level paragraph")
            .build();

        assert_eq!(doc.nodes.len(), 4);

        // Check group
        match &doc.nodes[2] {
            Node::Group { name, children } => {
                assert_eq!(name, "Section");
                assert_eq!(children.len(), 3);

                // Check group content
                match &children[0] {
                    Node::Heading { level, .. } => {
                        assert_eq!(*level, 2);
                    }
                    _ => panic!("Expected Heading node in group"),
                }
            }
            _ => panic!("Expected Group node"),
        }
    }
}
