use crate::editor::command::Command;
use crate::{Document, EditError, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to copy selected content
pub struct CopySelectionCommand {
    document: Rc<RefCell<Document>>,
    /// The nodes that were copied
    copied_nodes: Vec<Node>,
}

impl CopySelectionCommand {
    pub fn new(document: Rc<RefCell<Document>>) -> Self {
        Self {
            document,
            copied_nodes: Vec::new(),
        }
    }

    /// Get the nodes that were copied
    pub fn get_copied_nodes(&self) -> &[Node] {
        &self.copied_nodes
    }
}

impl Command for CopySelectionCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let document = self.document.borrow();

        // Check if there's an active selection
        let selection = match document.selection.as_ref() {
            Some(sel) => sel,
            None => return Ok(()),
        };

        // Get the affected range of nodes
        let start_node_idx = selection.start.path[0];
        let end_node_idx = selection.end.path[0];

        // Copy nodes in the selection
        self.copied_nodes.clear();

        // If selection spans multiple nodes
        if start_node_idx != end_node_idx {
            // Copy full nodes
            for idx in start_node_idx..=end_node_idx {
                if idx < document.nodes.len() {
                    self.copied_nodes.push(document.nodes[idx].clone());
                }
            }
        } else {
            // Selection is within a single node
            let node_idx = start_node_idx;
            if node_idx >= document.nodes.len() {
                return Ok(());
            }

            let node = &document.nodes[node_idx];
            match node {
                Node::Paragraph { children } => {
                    // Extract the selected portion of text
                    if selection.start.path.len() > 1 && selection.end.path.len() > 1 {
                        let start_pos = selection.start.path[1];
                        let end_pos = selection.end.path[1];

                        // Create a copy with just the selected portion
                        let selected_children =
                            extract_selected_content(children, start_pos, end_pos);
                        self.copied_nodes.push(Node::Paragraph {
                            children: selected_children,
                        });
                    } else {
                        // Copy the entire paragraph
                        self.copied_nodes.push(node.clone());
                    }
                }
                Node::Heading { level, children } => {
                    // Extract the selected portion of text
                    if selection.start.path.len() > 1 && selection.end.path.len() > 1 {
                        let start_pos = selection.start.path[1];
                        let end_pos = selection.end.path[1];

                        // Create a copy with just the selected portion
                        let selected_children =
                            extract_selected_content(children, start_pos, end_pos);
                        self.copied_nodes.push(Node::Heading {
                            level: *level,
                            children: selected_children,
                        });
                    } else {
                        // Copy the entire heading
                        self.copied_nodes.push(node.clone());
                    }
                }
                Node::CodeBlock {
                    language,
                    code,
                    properties,
                } => {
                    // Deep-copy code block
                    self.copied_nodes.push(Node::CodeBlock {
                        language: language.clone(),
                        code: code.clone(),
                        properties: properties.clone(),
                    });
                }
                // Handle other node types by copying them entirely
                _ => {
                    self.copied_nodes.push(node.clone());
                }
            }
        }

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        // Copy operation doesn't modify the document, so nothing to undo
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Helper function to extract selected content from node children
fn extract_selected_content(
    children: &[crate::InlineNode],
    start_pos: usize,
    end_pos: usize,
) -> Vec<crate::InlineNode> {
    let mut selected_children = Vec::new();
    let mut current_pos = 0;

    for child in children {
        let child_length = match child {
            crate::InlineNode::Text(text_node) => text_node.text.len(),
            _ => 1, // For simplicity, other inline nodes count as 1 character
        };

        let next_pos = current_pos + child_length;

        // If this child is within the selection range
        if current_pos < end_pos && next_pos > start_pos {
            match child {
                crate::InlineNode::Text(text_node) => {
                    // Calculate the portion of this text node to include
                    let sel_start = start_pos.saturating_sub(current_pos);
                    let sel_end = std::cmp::min(end_pos - current_pos, child_length);

                    if sel_start < sel_end {
                        let selected_text = text_node.text[sel_start..sel_end].to_string();
                        selected_children.push(crate::InlineNode::Text(crate::TextNode {
                            text: selected_text,
                            formatting: text_node.formatting.clone(),
                        }));
                    }
                }
                _ => {
                    // For other inline nodes, include them completely if they're within the selection
                    if current_pos >= start_pos && next_pos <= end_pos {
                        selected_children.push(child.clone());
                    }
                }
            }
        }

        current_pos = next_pos;
    }

    selected_children
}
