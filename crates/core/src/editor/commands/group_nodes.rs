use crate::editor::command::Command;
use crate::{Document, EditError, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to group multiple nodes into a container node
pub struct GroupNodesCommand {
    document: Rc<RefCell<Document>>,
    /// The indices of nodes to group
    node_indices: Vec<usize>,
    /// The name/type of the group container
    group_name: String,
    /// Original document state for undo
    original_nodes: Option<Vec<Node>>,
}

impl GroupNodesCommand {
    /// Create a new group nodes command
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_indices: Vec<usize>,
        group_name: String,
    ) -> Self {
        Self {
            document,
            node_indices,
            group_name,
            original_nodes: None,
        }
    }
}

impl Command for GroupNodesCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Store original document state for undo
        self.original_nodes = Some(document.nodes.clone());

        // Validate node indices
        for &idx in &self.node_indices {
            if idx >= document.nodes.len() {
                return Err(EditError::IndexOutOfBounds);
            }
        }

        // Sort indices in descending order to remove from end first
        let mut sorted_indices = self.node_indices.clone();
        sorted_indices.sort_unstable_by(|a, b| b.cmp(a));

        // Collect nodes to group
        let mut nodes_to_group = Vec::new();
        for &idx in &sorted_indices {
            nodes_to_group.push(document.nodes.remove(idx));
        }

        // Reverse to restore original order
        nodes_to_group.reverse();

        // Find insertion point (minimum index)
        let insertion_point = *self.node_indices.iter().min().unwrap_or(&0);

        // Create a group container node
        let group_node = Node::Group {
            name: self.group_name.clone(),
            children: nodes_to_group,
        };

        // Insert group node at the insertion point
        document.nodes.insert(insertion_point, group_node);

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
    use crate::InlineNode;

    #[test]
    fn test_group_nodes() {
        let mut doc = Document::new();

        // Add three paragraphs
        doc.add_paragraph_with_text("First paragraph");
        doc.add_paragraph_with_text("Second paragraph");
        doc.add_paragraph_with_text("Third paragraph");

        // Verify initial state
        assert_eq!(doc.nodes.len(), 3);

        let document_rc = Rc::new(RefCell::new(doc));

        // Group first two paragraphs
        let mut cmd =
            GroupNodesCommand::new(document_rc.clone(), vec![0, 1], "Test Group".to_string());

        // Execute the command
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check document state
        let doc = document_rc.borrow();
        assert_eq!(doc.nodes.len(), 2); // Now should have: Group, Third paragraph

        // Verify group node structure
        match &doc.nodes[0] {
            Node::Group { name, children } => {
                assert_eq!(name, "Test Group");
                assert_eq!(children.len(), 2);

                // Check first child node
                match &children[0] {
                    Node::Paragraph {
                        children: paragraph_children,
                    } => match &paragraph_children[0] {
                        InlineNode::Text(text_node) => {
                            assert_eq!(text_node.text, "First paragraph");
                        }
                        _ => panic!("Expected Text node"),
                    },
                    _ => panic!("Expected Paragraph node"),
                }

                // Check second child node
                match &children[1] {
                    Node::Paragraph {
                        children: paragraph_children,
                    } => match &paragraph_children[0] {
                        InlineNode::Text(text_node) => {
                            assert_eq!(text_node.text, "Second paragraph");
                        }
                        _ => panic!("Expected Text node"),
                    },
                    _ => panic!("Expected Paragraph node"),
                }
            }
            _ => panic!("Expected Group node"),
        }

        // Test undo
        drop(doc);
        let result = cmd.undo();
        assert!(result.is_ok());

        // Verify original state is restored
        let doc = document_rc.borrow();
        assert_eq!(doc.nodes.len(), 3);

        // Verify first and second nodes
        for (i, expected_text) in [(0, "First paragraph"), (1, "Second paragraph")].iter() {
            match &doc.nodes[*i] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, *expected_text);
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
        }
    }

    #[test]
    fn test_group_non_contiguous_nodes() {
        let mut doc = Document::new();

        // Add four paragraphs
        doc.add_paragraph_with_text("First paragraph");
        doc.add_paragraph_with_text("Second paragraph");
        doc.add_paragraph_with_text("Third paragraph");
        doc.add_paragraph_with_text("Fourth paragraph");

        let document_rc = Rc::new(RefCell::new(doc));

        // Group first and third paragraphs (non-contiguous)
        let mut cmd = GroupNodesCommand::new(
            document_rc.clone(),
            vec![0, 2],
            "Non-Contiguous Group".to_string(),
        );

        // Execute the command
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check document state
        let doc = document_rc.borrow();
        assert_eq!(doc.nodes.len(), 3); // Group, Second, Fourth

        // Verify group node contains correct paragraphs
        match &doc.nodes[0] {
            Node::Group { name, children } => {
                assert_eq!(name, "Non-Contiguous Group");
                assert_eq!(children.len(), 2);

                // First child should be "First paragraph"
                match &children[0] {
                    Node::Paragraph {
                        children: paragraph_children,
                    } => match &paragraph_children[0] {
                        InlineNode::Text(text_node) => {
                            assert_eq!(text_node.text, "First paragraph");
                        }
                        _ => panic!("Expected Text node"),
                    },
                    _ => panic!("Expected Paragraph node"),
                }

                // Second child should be "Third paragraph"
                match &children[1] {
                    Node::Paragraph {
                        children: paragraph_children,
                    } => match &paragraph_children[0] {
                        InlineNode::Text(text_node) => {
                            assert_eq!(text_node.text, "Third paragraph");
                        }
                        _ => panic!("Expected Text node"),
                    },
                    _ => panic!("Expected Paragraph node"),
                }
            }
            _ => panic!("Expected Group node"),
        }

        // Second node should be "Second paragraph"
        match &doc.nodes[1] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(text_node) => {
                    assert_eq!(text_node.text, "Second paragraph");
                }
                _ => panic!("Expected Text node"),
            },
            _ => panic!("Expected Paragraph node"),
        }
    }
}
