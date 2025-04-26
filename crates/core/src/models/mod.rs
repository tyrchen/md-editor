mod document;
mod formatting;
mod inline;
mod node;
mod selection;

pub use document::*;
pub use formatting::TextFormatting;
pub use inline::{InlineNode, TextNode};
pub use node::{
    DefinitionItem, FootnoteDefinition, FootnoteReference, ListItem, ListType, Node,
    TableAlignment, TableCell,
};
pub use selection::{Position, Selection};

// Public serialization functions are now in crate::convert
// pub use serialization::{from_html, from_json, from_markdown, to_html, to_json, to_markdown}; // Removed old export

#[cfg(test)]
mod test {
    use crate::{
        Document, Html, Json, ListType, Markdown, Node, Position, Selection, Text, TextFormatting,
    };
    use crate::{InlineNode, TextNode};

    #[test]
    fn test_document_creation() {
        let doc = Document::new();
        assert!(doc.nodes.is_empty());
        assert!(doc.selection.is_none());
        assert!(doc.metadata.is_none());
    }

    #[test]
    fn test_document_with_title() {
        let doc = Document::with_title("Test Document");

        assert_eq!(doc.nodes.len(), 1);

        match &doc.nodes[0] {
            Node::Heading { level, children } => {
                assert_eq!(*level, 1);
                assert_eq!(children.len(), 1);

                match &children[0] {
                    InlineNode::Text(TextNode { text, .. }) => {
                        assert_eq!(text, "Test Document");
                    }
                    _ => panic!("Expected Text node"),
                }
            }
            _ => panic!("Expected Heading node"),
        }

        assert!(doc.metadata.is_some());
        let metadata = doc.metadata.unwrap();
        assert_eq!(metadata.title, Some("Test Document".to_string()));
    }

    #[test]
    fn test_paragraph_operations() {
        let mut doc = Document::new();

        // Add empty paragraph
        let idx = doc.add_paragraph();
        assert_eq!(idx, 0);
        assert_eq!(doc.nodes.len(), 1);

        match &doc.nodes[0] {
            Node::Paragraph { children } => {
                assert!(children.is_empty());
            }
            _ => panic!("Expected Paragraph node"),
        }

        // Insert text into paragraph
        let result = doc.insert_text(idx, 0, "Hello, world!");
        assert!(result);

        match &doc.nodes[0] {
            Node::Paragraph { children } => {
                assert_eq!(children.len(), 1);

                match &children[0] {
                    InlineNode::Text(TextNode { text, .. }) => {
                        assert_eq!(text, "Hello, world!");
                    }
                    _ => panic!("Expected Text node"),
                }
            }
            _ => panic!("Expected Paragraph node"),
        }

        // Split paragraph
        let result = doc.split_node(idx, 7);
        assert!(result);
        assert_eq!(doc.nodes.len(), 2);

        match &doc.nodes[0] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(TextNode { text, .. }) => {
                    assert_eq!(text, "Hello, ");
                }
                _ => panic!("Expected Text node"),
            },
            _ => panic!("Expected Paragraph node"),
        }

        match &doc.nodes[1] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(TextNode { text, .. }) => {
                    assert_eq!(text, "world!");
                }
                _ => panic!("Expected Text node"),
            },
            _ => panic!("Expected Paragraph node"),
        }
    }

    #[test]
    fn test_list_operations() {
        let mut doc = Document::new();

        // Add unordered list
        let idx = doc.add_unordered_list(vec!["Item 1", "Item 2", "Item 3"]);
        assert_eq!(idx, 0);

        match &doc.nodes[0] {
            Node::List { list_type, items } => {
                assert_eq!(*list_type, ListType::Unordered);
                assert_eq!(items.len(), 3);

                for (i, item) in items.iter().enumerate() {
                    match &item.children[0] {
                        Node::Paragraph { children } => match &children[0] {
                            InlineNode::Text(TextNode { text, .. }) => {
                                assert_eq!(text, &format!("Item {}", i + 1));
                            }
                            _ => panic!("Expected Text node"),
                        },
                        _ => panic!("Expected Paragraph node"),
                    }
                }
            }
            _ => panic!("Expected List node"),
        }

        // Add task list
        let idx = doc.add_task_list(vec![("Task 1", true), ("Task 2", false)]);
        assert_eq!(idx, 1);

        match &doc.nodes[1] {
            Node::List { list_type, items } => {
                assert_eq!(*list_type, ListType::Task);
                assert_eq!(items.len(), 2);

                assert_eq!(items[0].checked, Some(true));
                assert_eq!(items[1].checked, Some(false));
            }
            _ => panic!("Expected List node"),
        }
    }

    #[test]
    fn test_inline_node_creation() {
        // Test text node
        let text = InlineNode::text("Plain text");
        match text {
            InlineNode::Text(TextNode { text, formatting }) => {
                assert_eq!(text, "Plain text");
                assert!(!formatting.bold);
                assert!(!formatting.italic);
                assert!(!formatting.code);
                assert!(!formatting.strikethrough);
            }
            _ => panic!("Expected Text node"),
        }

        // Test bold text
        let bold = InlineNode::bold_text("Bold text");
        match bold {
            InlineNode::Text(TextNode { text, formatting }) => {
                assert_eq!(text, "Bold text");
                assert!(formatting.bold);
                assert!(!formatting.italic);
            }
            _ => panic!("Expected Text node"),
        }

        // Test link
        let link = InlineNode::link("https://example.com", "Example Link");
        match link {
            InlineNode::Link {
                url,
                title,
                children,
            } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(title, None);
                assert_eq!(children.len(), 1);

                match &children[0] {
                    InlineNode::Text(TextNode { text, .. }) => {
                        assert_eq!(text, "Example Link");
                    }
                    _ => panic!("Expected Text node"),
                }
            }
            _ => panic!("Expected Link node"),
        }
    }

    #[test]
    fn test_selection() {
        let start = Position::new(vec![0, 1], 5);
        let end = Position::new(vec![0, 2], 10);

        let selection = Selection::new(start.clone(), end.clone());
        assert_eq!(selection.start, start);
        assert_eq!(selection.end, end);
        assert!(!selection.is_collapsed);

        let collapsed = Selection::collapsed(start.clone());
        assert_eq!(collapsed.start, start);
        assert_eq!(collapsed.end, start);
        assert!(collapsed.is_collapsed);
    }

    #[test]
    fn test_json_serialization() {
        let mut doc = Document::new();
        doc.add_heading(1, "Test Document");
        doc.add_paragraph_with_text("This is a paragraph");

        let json = Text::<Json>::try_from(&doc).unwrap();
        let doc2 = Document::try_from(json).unwrap();

        assert_eq!(doc.nodes.len(), doc2.nodes.len());

        match &doc2.nodes[0] {
            Node::Heading { level, children } => {
                assert_eq!(*level, 1);

                match &children[0] {
                    InlineNode::Text(TextNode { text, .. }) => {
                        assert_eq!(text, "Test Document");
                    }
                    _ => panic!("Expected Text node"),
                }
            }
            _ => panic!("Expected Heading node"),
        }
    }

    #[test]
    fn test_markdown_serialization() {
        let mut doc = Document::new();
        doc.add_heading(1, "Test Document");
        doc.add_paragraph_with_text("This is a paragraph");
        doc.add_code_block("fn main() {}", "rust");

        let markdown = Text::<Markdown>::try_from(&doc).unwrap();

        assert!(markdown.contains("# Test Document"));
        assert!(markdown.contains("This is a paragraph"));
        assert!(markdown.contains("```rust"));
        assert!(markdown.contains("fn main() {}"));
    }

    #[test]
    fn test_html_serialization() {
        let mut doc = Document::new();
        doc.add_heading(1, "Test Document");
        doc.add_paragraph_with_text("This is a paragraph");

        let html = Text::<Html>::try_from(&doc).unwrap();

        assert!(html.contains("<h1>Test Document</h1>"));
        assert!(html.contains("<p>This is a paragraph</p>"));
        // The document wrapper is not included in the current implementation
    }

    #[test]
    fn test_document_methods() {
        let mut doc = Document::new();
        doc.add_heading(1, "Test Document");
        doc.add_paragraph_with_text("This is a paragraph");

        // Basic operations using nodes directly
        assert_eq!(doc.nodes.len(), 2);
        assert!(matches!(doc.nodes[0], Node::Heading { .. }));
        assert!(matches!(doc.nodes[1], Node::Paragraph { .. }));

        // Iterate through nodes
        let mut count = 0;
        for node in &doc.nodes {
            match node {
                Node::Heading { .. } | Node::Paragraph { .. } => count += 1,
                _ => {}
            }
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_all_heading_levels() {
        let mut doc = Document::new();

        // Add all heading levels
        for level in 1..=6 {
            let heading_text = format!("Heading Level {}", level);
            doc.add_heading(level, &heading_text);
        }

        // Verify all headings were added correctly
        assert_eq!(doc.nodes.len(), 6);

        // Check each heading level
        for i in 0..6 {
            let level = i + 1;
            match &doc.nodes[i] {
                Node::Heading {
                    level: node_level,
                    children,
                } => {
                    // Check level
                    assert_eq!(*node_level, level as u8);

                    // Check text content
                    match &children[0] {
                        InlineNode::Text(TextNode { text, .. }) => {
                            assert_eq!(text, &format!("Heading Level {}", level));
                        }
                        _ => panic!("Expected text node in heading"),
                    }
                }
                _ => panic!("Expected heading node"),
            }
        }

        // Test serialization to Markdown
        let markdown = Text::<Markdown>::try_from(&doc).unwrap();
        for level in 1..=6 {
            // Check correct heading syntax
            let heading_marker = "#".repeat(level);
            assert!(markdown.contains(&format!("{} Heading Level {}", heading_marker, level)));
        }

        // Test serialization to HTML
        let html = Text::<Html>::try_from(&doc).unwrap();
        for level in 1..=6 {
            // Check correct HTML tags
            assert!(html.contains(&format!("<h{}>Heading Level {}</h{}>", level, level, level)));
        }

        // Test JSON roundtrip
        let json = Text::<Json>::try_from(&doc).unwrap();
        let deserialized = Document::try_from(json).unwrap();

        assert_eq!(doc.nodes.len(), deserialized.nodes.len());

        // Verify headings were preserved in JSON
        for i in 0..6 {
            let level = i + 1;
            match &deserialized.nodes[i] {
                Node::Heading {
                    level: node_level,
                    children,
                } => {
                    assert_eq!(*node_level, level as u8);

                    match &children[0] {
                        InlineNode::Text(TextNode { text, .. }) => {
                            assert_eq!(text, &format!("Heading Level {}", level));
                        }
                        _ => panic!("Expected text node in heading after JSON roundtrip"),
                    }
                }
                _ => panic!("Expected heading node after JSON roundtrip"),
            }
        }
    }

    #[test]
    fn test_text_formatting() {
        let mut doc = Document::new();

        // Create paragraph with formatted text
        let mut para = Node::paragraph("hello");
        if let Node::Paragraph { children } = &mut para {
            // Bold text
            children.push(InlineNode::bold_text("Bold text"));

            // Italic text
            children.push(InlineNode::text(" and "));
            children.push(InlineNode::italic_text("italic text"));

            // Bold and italic
            children.push(InlineNode::text(" and "));
            let mut text = TextNode {
                text: "bold italic text".to_string(),
                formatting: TextFormatting::default(),
            };
            text.formatting.bold = true;
            text.formatting.italic = true;
            children.push(InlineNode::Text(text));

            // Strikethrough
            children.push(InlineNode::text(" and "));
            let mut strikethrough = TextNode {
                text: "strikethrough text".to_string(),
                formatting: TextFormatting::default(),
            };
            strikethrough.formatting.strikethrough = true;
            children.push(InlineNode::Text(strikethrough));

            // Code span
            children.push(InlineNode::text(" and "));
            let mut code_text = TextNode {
                text: "code text".to_string(),
                formatting: TextFormatting::default(),
            };
            code_text.formatting.code = true;
            children.push(InlineNode::Text(code_text));

            // Inline code span
            children.push(InlineNode::text(" and "));
            children.push(InlineNode::CodeSpan {
                code: "inline code".to_string(),
            });
        }

        doc.nodes.push(para);

        // Verify structure
        assert_eq!(doc.nodes.len(), 1);

        // Check paragraph content
        match &doc.nodes[0] {
            Node::Paragraph { children } => {
                assert_eq!(children.len(), 12);

                // Check hello text
                assert_eq!(children[0].as_text(), Some("hello"));

                // Check bold text
                match &children[1] {
                    InlineNode::Text(TextNode { text, formatting }) => {
                        assert_eq!(text, "Bold text");
                        assert!(formatting.bold);
                        assert!(!formatting.italic);
                    }
                    _ => panic!("Expected bold text node"),
                }

                // Check italic text
                match &children[3] {
                    InlineNode::Text(TextNode { text, formatting }) => {
                        assert_eq!(text, "italic text");
                        assert!(!formatting.bold);
                        assert!(formatting.italic);
                    }
                    _ => panic!("Expected italic text node"),
                }

                // Check bold italic text
                match &children[5] {
                    InlineNode::Text(TextNode { text, formatting }) => {
                        assert_eq!(text, "bold italic text");
                        assert!(formatting.bold);
                        assert!(formatting.italic);
                    }
                    _ => panic!("Expected bold italic text node"),
                }

                // Check strikethrough text
                match &children[7] {
                    InlineNode::Text(TextNode { text, formatting }) => {
                        assert_eq!(text, "strikethrough text");
                        assert!(formatting.strikethrough);
                    }
                    _ => panic!("Expected strikethrough text node"),
                }

                // Check code text
                match &children[9] {
                    InlineNode::Text(TextNode { text, formatting }) => {
                        assert_eq!(text, "code text");
                        assert!(formatting.code);
                    }
                    _ => panic!("Expected code text node"),
                }

                // Check inline code
                match &children[11] {
                    InlineNode::CodeSpan { code } => {
                        assert_eq!(code, "inline code");
                    }
                    _ => panic!("Expected code span node"),
                }
            }
            _ => panic!("Expected paragraph node"),
        }

        // Test markdown serialization
        let markdown = Text::<Markdown>::try_from(&doc).unwrap();
        assert!(markdown.contains("**Bold text**"));
        assert!(markdown.contains("*italic text*"));
        assert!(
            markdown.contains("**_bold italic text_**")
                || markdown.contains("_**bold italic text**_")
                || markdown.contains("***bold italic text***")
        );
        assert!(markdown.contains("~~strikethrough text~~"));
        assert!(markdown.contains("`code text`"));
        assert!(markdown.contains("`inline code`"));

        // Test HTML serialization
        let html = Text::<Html>::try_from(&doc).unwrap();
        assert!(html.contains("<strong>Bold text</strong>"));
        assert!(html.contains("<em>italic text</em>"));
        assert!(
            html.contains("<strong><em>bold italic text</em></strong>")
                || html.contains("<em><strong>bold italic text</strong></em>")
        );
        assert!(html.contains("<del>strikethrough text</del>"));
        assert!(html.contains("<code>code text</code>"));
        assert!(html.contains("<code>inline code</code>"));

        // Test JSON roundtrip
        let json = Text::<Json>::try_from(&doc).unwrap();
        let deserialized = Document::try_from(json).unwrap();

        // Verify structure preserved
        match &deserialized.nodes[0] {
            Node::Paragraph { children } => {
                assert_eq!(children.len(), 12);

                // Check bold text persisted
                match &children[1] {
                    InlineNode::Text(TextNode { text, formatting }) => {
                        assert_eq!(text, "Bold text");
                        assert!(formatting.bold);
                    }
                    _ => panic!("Expected bold text node after JSON roundtrip"),
                }

                // Check italic text persisted
                match &children[3] {
                    InlineNode::Text(TextNode { text, formatting }) => {
                        assert_eq!(text, "italic text");
                        assert!(formatting.italic);
                    }
                    _ => panic!("Expected italic text node after JSON roundtrip"),
                }
            }
            _ => panic!("Expected paragraph node after JSON roundtrip"),
        }
    }

    #[test]
    fn test_list_types_and_nesting() {
        let text = r#"
- Item 1
    1. Subitem 1
    2. Subitem 2
- Item 2
- Item 3
    1. Subitem 3
    2. Subitem 4
"#;

        let doc = Document::try_from(Text::<Markdown>::new(text)).unwrap();
        assert_eq!(doc.nodes.len(), 1);

        let (list_type, items) = doc.nodes[0].as_list().unwrap();
        assert_eq!(*list_type, ListType::Unordered);
        assert_eq!(items.len(), 3);

        let item1 = items[0].clone();
        assert_eq!(item1.as_text(), Some("Item 1"));

        let item2 = items[1].clone();
        assert_eq!(item2.as_text(), Some("Item 2"));

        let item3 = items[2].clone();
        assert_eq!(item3.as_text(), Some("Item 3"));

        // Check the nested lists
        let item1_children = &item1.children;
        assert_eq!(item1_children.len(), 2); // The paragraph and the nested list

        if let Node::List { list_type, items } = &item1_children[1] {
            assert_eq!(*list_type, ListType::Ordered);
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].as_text(), Some("Subitem 1"));
            assert_eq!(items[1].as_text(), Some("Subitem 2"));
        } else {
            panic!("Expected nested list in item 1");
        }

        let item3_children = &item3.children;
        assert_eq!(item3_children.len(), 2); // The paragraph and the nested list

        if let Node::List { list_type, items } = &item3_children[1] {
            assert_eq!(*list_type, ListType::Ordered);
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].as_text(), Some("Subitem 3"));
            assert_eq!(items[1].as_text(), Some("Subitem 4"));
        } else {
            panic!("Expected nested list in item 3");
        }

        // Test serialization to Markdown
        let markdown = Text::<Markdown>::try_from(&doc).unwrap();
        let doc2 = Document::try_from(markdown).unwrap();
        assert_eq!(doc, doc2);

        // Test serialization to HTML and roundtrip
        let html = Text::<Html>::try_from(&doc).unwrap();

        let doc3 = Document::try_from(html).unwrap();
        assert_eq!(doc, doc3);

        // Test JSON roundtrip
        let json = Text::<Json>::try_from(&doc).unwrap();
        let doc4 = Document::try_from(json).unwrap();
        assert_eq!(doc, doc4);
    }
}
