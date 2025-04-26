use crate::editor::command::Command;
use crate::{Document, EditError, InlineNode, ListItem, Node, TextNode};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to create a table of contents from document headings
pub struct CreateTOCCommand {
    document: Rc<RefCell<Document>>,
    /// Position to insert the TOC
    position: usize,
    /// Maximum heading level to include (1 to 6)
    max_level: u8,
    /// Original document state for undo
    original_nodes: Option<Vec<Node>>,
}

impl CreateTOCCommand {
    /// Create a new TOC command
    pub fn new(document: Rc<RefCell<Document>>, position: usize, max_level: u8) -> Self {
        Self {
            document,
            position,
            max_level: max_level.clamp(1, 6), // Clamp between 1 and 6
            original_nodes: None,
        }
    }
}

impl Command for CreateTOCCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Store the original nodes for undo
        self.original_nodes = Some(document.nodes.clone());

        // Generate TOC from document headings
        let mut toc_entries = Vec::new();
        let mut toc_heading_found = false;

        // First pass: collect all headings
        for (index, node) in document.nodes.iter().enumerate() {
            if let Node::Heading { level, children } = node {
                // Only include headings up to the specified level
                if *level <= self.max_level {
                    // Extract heading text
                    let mut heading_text = String::new();
                    for child in children {
                        if let InlineNode::Text(text_node) = child {
                            heading_text.push_str(&text_node.text);
                        }
                    }

                    // Skip if this is a TOC heading itself
                    if heading_text.to_lowercase().contains("table of contents")
                        || heading_text.to_lowercase().contains("toc")
                    {
                        toc_heading_found = true;
                        continue;
                    }

                    // Create an anchor ID from the heading text
                    let anchor = heading_text
                        .to_lowercase()
                        .chars()
                        .map(|c| if c.is_alphanumeric() { c } else { '-' })
                        .collect::<String>();

                    toc_entries.push((*level, heading_text, anchor, index));
                }
            }
        }

        // Create TOC nodes
        let mut toc_nodes = Vec::new();

        // Add TOC heading if not already present
        if !toc_heading_found {
            toc_nodes.push(Node::Heading {
                level: 2,
                children: vec![InlineNode::Text(TextNode {
                    text: "Table of Contents".to_string(),
                    formatting: Default::default(),
                })],
            });
        }

        // Create list items for TOC entries
        let mut list_items = Vec::new();
        for (level, text, anchor, _) in toc_entries {
            // Create indentation based on heading level
            let indent = "  ".repeat((level - 1) as usize);

            // Create list item with link
            let item_text = InlineNode::Text(TextNode {
                text: format!("{}[{}](#{})", indent, text, anchor),
                formatting: Default::default(),
            });

            // Create paragraph node for list item
            let paragraph = Node::Paragraph {
                children: vec![item_text],
            };

            // Create list item
            list_items.push(ListItem {
                children: vec![paragraph],
                checked: None,
            });
        }

        // Add the TOC list if we have any entries
        if !list_items.is_empty() {
            toc_nodes.push(Node::List {
                list_type: crate::ListType::Unordered,
                items: list_items,
            });
        }

        // Insert TOC nodes at the specified position
        let position = self.position.min(document.nodes.len());
        let num_toc_nodes = toc_nodes.len();
        for (i, node) in toc_nodes.into_iter().enumerate() {
            document.nodes.insert(position + i, node);
        }

        // If TOC is inserted at the beginning, add a separator
        if position == 0 && !document.nodes.is_empty() {
            document.nodes.insert(num_toc_nodes, Node::ThematicBreak);
        }

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(original_nodes) = self.original_nodes.take() {
            let mut document = self.document.borrow_mut();
            document.nodes = original_nodes;
            Ok(())
        } else {
            Err(EditError::Other("No original state to restore".to_string()))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ListType;

    #[test]
    fn test_create_toc() {
        // Create a document with headings
        let mut doc = Document::new();

        // Add some headings and content
        doc.add_heading(1, "First Section");
        doc.add_paragraph_with_text("Some content here.");

        doc.add_heading(2, "Subsection 1");
        doc.add_paragraph_with_text("More content here.");

        doc.add_heading(2, "Subsection 2");
        doc.add_paragraph_with_text("Even more content.");

        doc.add_heading(1, "Second Section");
        doc.add_paragraph_with_text("Last paragraph.");

        // Create a TOC at the beginning
        let document_rc = Rc::new(RefCell::new(doc));
        let mut cmd = CreateTOCCommand::new(document_rc.clone(), 0, 2);

        // Execute the command
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check TOC was created
        let doc = document_rc.borrow();

        // First node should be a heading "Table of Contents"
        match &doc.nodes[0] {
            Node::Heading { level, children } => {
                assert_eq!(*level, 2);
                if let InlineNode::Text(text_node) = &children[0] {
                    assert_eq!(text_node.text, "Table of Contents");
                } else {
                    panic!("Expected Text node");
                }
            }
            _ => panic!("Expected Heading node"),
        }

        // Second node should be a list with TOC entries
        match &doc.nodes[1] {
            Node::List { list_type, items } => {
                assert_eq!(*list_type, ListType::Unordered);
                assert_eq!(items.len(), 4); // Should have 4 entries

                // Check first item (First Section)
                let first_item = &items[0];
                if let Node::Paragraph { children } = &first_item.children[0] {
                    if let InlineNode::Text(text_node) = &children[0] {
                        assert!(text_node.text.contains("First Section"));
                    } else {
                        panic!("Expected Text node");
                    }
                } else {
                    panic!("Expected Paragraph node");
                }

                // Check indentation of second item (Subsection 1)
                let second_item = &items[1];
                if let Node::Paragraph { children } = &second_item.children[0] {
                    if let InlineNode::Text(text_node) = &children[0] {
                        assert!(text_node.text.starts_with("  "));
                        assert!(text_node.text.contains("Subsection 1"));
                    } else {
                        panic!("Expected Text node");
                    }
                } else {
                    panic!("Expected Paragraph node");
                }
            }
            _ => panic!("Expected List node"),
        }

        // Test undo
        drop(doc); // Release borrow
        let result = cmd.undo();
        assert!(result.is_ok());

        // Verify document is back to original state
        let doc = document_rc.borrow();
        assert_eq!(doc.nodes.len(), 8); // Original 8 nodes

        // First node should be first heading again
        match &doc.nodes[0] {
            Node::Heading { level, children } => {
                assert_eq!(*level, 1);
                if let InlineNode::Text(text_node) = &children[0] {
                    assert_eq!(text_node.text, "First Section");
                } else {
                    panic!("Expected Text node");
                }
            }
            _ => panic!("Expected Heading node"),
        }
    }
}
