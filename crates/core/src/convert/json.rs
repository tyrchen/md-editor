use crate::{Document, ParseError};
use serde_json;

use super::Json;
use super::Text;

impl TryFrom<Text<Json>> for Document {
    type Error = ParseError;

    fn try_from(json: Text<Json>) -> Result<Self, Self::Error> {
        from_json(json.as_str())
    }
}

impl TryFrom<&Document> for Text<Json> {
    type Error = ParseError;

    fn try_from(document: &Document) -> Result<Self, Self::Error> {
        to_json(document).map(Text::new)
    }
}

fn from_json(json: &str) -> Result<Document, ParseError> {
    serde_json::from_str(json).map_err(|e| ParseError::Json(e.to_string()))
}

fn to_json(document: &Document) -> Result<String, ParseError> {
    // Use pretty printing for better readability
    serde_json::to_string_pretty(document).map_err(|e| ParseError::Json(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Document, FootnoteDefinition, InlineNode, ListItem, ListType, Node, TableAlignment,
        TableCell, TableProperties, TextFormatting, TextNode,
    };

    // Helper function to create a test document
    fn create_test_document() -> Document {
        let mut doc = Document::new();
        doc.add_heading(1, "JSON Test");
        doc.add_paragraph_with_text("Simple paragraph for JSON.");
        doc.add_code_block("{\"key\": \"value\"}", "json");
        doc
    }

    #[test]
    fn test_json_serialization() {
        let doc = create_test_document();
        let json_result = to_json(&doc);

        assert!(json_result.is_ok());
        let json = json_result.unwrap();

        // Basic validity checks
        assert!(json.contains("JSON Test"));
        assert!(json.contains("paragraph"));
        assert!(json.contains("code_block"));
        assert!(json.contains("{\\\"key\\\": \\\"value\\\"}")); // Check escaped JSON within code block
        assert!(json.contains("\"language\": \"json\""));
    }

    #[test]
    fn test_json_roundtrip() {
        let original = create_test_document();
        let json = to_json(&original).expect("Serialization failed");
        let deserialized_result = from_json(&json);

        assert!(deserialized_result.is_ok());
        let deserialized = deserialized_result.unwrap();

        // Compare structures - using PartialEq derived on Document/Node/etc.
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_invalid_json_deserialization() {
        let invalid_json = "{\"nodes\": [{\"malformed\": }]} ";
        let result = from_json(invalid_json);
        assert!(result.is_err());
        match result.err().unwrap() {
            ParseError::Json(_) => { /* Expected error */ }
            _ => panic!("Expected JSON parse error"),
        }
    }

    // New test cases

    #[test]
    fn test_json_serialization_empty_document() {
        let doc = Document::new();
        let json_result = to_json(&doc);

        assert!(json_result.is_ok());
        let json = json_result.unwrap();

        println!("Empty document JSON: {}", json);

        // Check that it has nodes array but it's empty
        assert!(json.contains("\"nodes\": []"));
    }

    #[test]
    fn test_json_serialization_complex_document() {
        let mut doc = Document::new();

        // Add various node types
        doc.add_heading(2, "Complex Document");
        doc.add_paragraph_with_text("This is a paragraph with complex content.");
        doc.nodes.push(Node::ThematicBreak);
        doc.nodes.push(Node::BlockQuote {
            children: vec![Node::paragraph("This is a blockquote.")],
        });

        // Add a table
        let header = vec![TableCell::text("Header 1"), TableCell::text("Header 2")];
        let rows = vec![
            vec![
                TableCell::text("Row 1, Col 1"),
                TableCell::text("Row 1, Col 2"),
            ],
            vec![
                TableCell::text("Row 2, Col 1"),
                TableCell::text("Row 2, Col 2"),
            ],
        ];
        let alignments = vec![TableAlignment::Left, TableAlignment::Center];

        doc.nodes.push(Node::Table {
            header,
            rows,
            alignments,
            properties: TableProperties::default(),
        });

        // Add nested lists
        let nested_list = Node::List {
            list_type: ListType::Unordered,
            items: vec![
                ListItem::paragraph("Nested Item 1"),
                ListItem::paragraph("Nested Item 2"),
            ],
        };

        let mut parent_item = ListItem::paragraph("Parent Item");
        parent_item.children.push(nested_list);

        doc.nodes.push(Node::List {
            list_type: ListType::Ordered,
            items: vec![ListItem::paragraph("List Item 1"), parent_item],
        });

        // Footnote
        doc.nodes.push(Node::FootnoteDefinition(FootnoteDefinition {
            label: "note".to_string(),
            content: vec![Node::paragraph("This is a footnote.")],
        }));

        // Add a math block
        doc.nodes.push(Node::MathBlock {
            math: "E = mc^2".to_string(),
        });

        // Serialize to JSON
        let json_result = to_json(&doc);
        assert!(json_result.is_ok());
        let json = json_result.unwrap();

        println!("Complex document JSON: {}", json);

        // Check that it contains representations of all the elements
        assert!(json.contains("\"type\": \"heading\""));
        assert!(json.contains("\"type\": \"paragraph\""));
        assert!(json.contains("\"type\": \"thematic_break\""));
        assert!(json.contains("\"type\": \"blockquote\""));
        assert!(json.contains("\"type\": \"table\""));
        assert!(json.contains("\"type\": \"list\""));
        assert!(json.contains("\"type\": \"footnote_definition\""));
        assert!(json.contains("\"type\": \"math_block\""));

        // Check specific values
        assert!(json.contains("\"level\": 2"));
        assert!(json.contains("\"list_type\": \"ordered\""));
        assert!(json.contains("\"list_type\": \"unordered\""));
        assert!(json.contains("\"label\": \"note\""));

        // Verify roundtrip
        let deserialized = from_json(&json).expect("Deserialization should succeed");
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn test_json_serialization_text_formatting() {
        let mut doc = Document::new();

        // Create paragraph with formatted text

        let para_children = vec![
            // Bold text
            InlineNode::Text(TextNode {
                text: "Bold text".to_string(),
                formatting: TextFormatting {
                    bold: true,
                    italic: false,
                    strikethrough: false,
                    code: false,
                },
            }),
            // Italic text
            InlineNode::Text(TextNode {
                text: " and italic text".to_string(),
                formatting: TextFormatting {
                    bold: false,
                    italic: true,
                    strikethrough: false,
                    code: false,
                },
            }),
            // Strikethrough text
            InlineNode::Text(TextNode {
                text: " and strikethrough".to_string(),
                formatting: TextFormatting {
                    bold: false,
                    italic: false,
                    strikethrough: true,
                    code: false,
                },
            }),
        ];

        // Add paragraph with the formatted text
        doc.nodes.push(Node::Paragraph {
            children: para_children,
        });

        // Serialize to JSON
        let json_result = to_json(&doc);
        assert!(json_result.is_ok());
        let json = json_result.unwrap();

        println!("Formatted text JSON: {}", json);

        // Check that formatting is correctly serialized
        assert!(json.contains("\"bold\": true"));
        assert!(json.contains("\"italic\": true"));
        assert!(json.contains("\"strikethrough\": true"));

        // Verify roundtrip
        let deserialized = from_json(&json).expect("Deserialization should succeed");
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn test_json_serialization_links_and_images() {
        let mut doc = Document::new();

        // Add links and images
        let para_children = vec![
            InlineNode::text("This has a "),
            InlineNode::Link {
                url: "https://example.com".to_string(),
                title: None,
                children: vec![InlineNode::text("link")],
            },
            InlineNode::text(" and an "),
            InlineNode::Image {
                url: "https://example.com/image.jpg".to_string(),
                alt: "example image".to_string(),
                title: Some("Image title".to_string()),
            },
        ];

        doc.nodes.push(Node::Paragraph {
            children: para_children,
        });

        // Serialize to JSON
        let json_result = to_json(&doc);
        assert!(json_result.is_ok());
        let json = json_result.unwrap();

        println!("Links and images JSON: {}", json);

        // Check that links and images are correctly serialized
        assert!(json.contains("\"url\": \"https://example.com\""));
        assert!(json.contains("\"url\": \"https://example.com/image.jpg\""));
        assert!(json.contains("\"alt\": \"example image\""));
        assert!(json.contains("\"title\": \"Image title\""));

        // Verify roundtrip
        let deserialized = from_json(&json).expect("Deserialization should succeed");
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn test_json_serialization_special_characters() {
        let mut doc = Document::new();

        // Add text with special characters that need escaping in JSON
        doc.add_paragraph_with_text("Special chars: \\ \" \n \t \r");
        doc.add_code_block(
            "function test() {\n  console.log(\"Hello\");\n}",
            "javascript",
        );

        // Serialize to JSON
        let json_result = to_json(&doc);
        assert!(json_result.is_ok());
        let json = json_result.unwrap();

        println!("Special characters JSON: {}", json);

        // Check that special characters are properly escaped
        assert!(json.contains("\\\\")); // Escaped backslash
        assert!(json.contains("\\\"")); // Escaped quote
        assert!(json.contains("\\n")); // Escaped newline
        assert!(json.contains("\\t")); // Escaped tab
        assert!(json.contains("\\r")); // Escaped carriage return

        // Verify roundtrip
        let deserialized = from_json(&json).expect("Deserialization should succeed");

        // Compare the original document with the deserialized one
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn test_json_serialization_large_document() {
        let mut doc = Document::new();

        // Create a reasonably large document
        doc.add_heading(1, "Large Document Test");

        // Add 50 paragraphs
        for i in 1..=50 {
            doc.add_paragraph_with_text(format!("This is paragraph number {}", i));
        }

        // Serialize to JSON
        let json_result = to_json(&doc);
        assert!(json_result.is_ok());
        let json = json_result.unwrap();

        // Verify the document is large
        assert!(json.len() > 2000, "JSON should be at least 2KB");

        // Verify roundtrip even for large document
        let deserialized =
            from_json(&json).expect("Deserialization of large document should succeed");
        assert_eq!(doc.nodes.len(), deserialized.nodes.len());
    }
}
