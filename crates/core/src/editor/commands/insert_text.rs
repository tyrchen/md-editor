use crate::editor::command::Command;
use crate::{Document, EditError, InlineNode, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to insert text at a specific position in a node
pub struct InsertTextCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    position: usize,
    text: String,
    // For undo: track if any formatting was split during insert
    affected_nodes: Option<Vec<InlineNode>>,
}

impl InsertTextCommand {
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_index: usize,
        position: usize,
        text: String,
    ) -> Self {
        Self {
            document,
            node_index,
            position,
            text,
            affected_nodes: None,
        }
    }
}

impl Command for InsertTextCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        // Early validation
        if self.text.is_empty() {
            // Nothing to insert, not an error but a no-op
            return Ok(());
        }

        let mut document = self.document.borrow_mut();

        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        match &mut document.nodes[self.node_index] {
            Node::Paragraph { children } | Node::Heading { children, .. } => {
                // Store original nodes for undo
                self.affected_nodes = Some(children.clone());

                // Track current offset to find the right position
                let mut current_offset = 0;
                let mut node_idx_to_modify = None;
                let mut position_in_node = 0;

                // Find which text node contains our insertion position
                for (idx, child) in children.iter().enumerate() {
                    match child {
                        InlineNode::Text(text_node) => {
                            let text_len = text_node.text.len();
                            let next_offset = current_offset + text_len;

                            // If position is within this text node
                            if self.position >= current_offset && self.position <= next_offset {
                                node_idx_to_modify = Some(idx);
                                position_in_node = self.position - current_offset;
                                break;
                            }

                            current_offset = next_offset;
                        }
                        _ => {
                            // For non-text nodes, count as length 1
                            current_offset += 1;

                            // If position is at this non-text node
                            if self.position == current_offset - 1 {
                                // We'll insert before this node
                                node_idx_to_modify = Some(idx);
                                position_in_node = 0;
                                break;
                            }
                        }
                    }
                }

                // If the position is at the end of all nodes
                if node_idx_to_modify.is_none() && self.position == current_offset {
                    // Insert a new text node at the end
                    children.push(InlineNode::text(&self.text));
                    return Ok(());
                }

                // If the position is beyond the end of all nodes
                if node_idx_to_modify.is_none() && self.position > current_offset {
                    return Err(EditError::InvalidRange);
                }

                // If we found a valid position within a node
                if let Some(idx) = node_idx_to_modify {
                    match &mut children[idx] {
                        InlineNode::Text(text_node) => {
                            // Sanity check for position bounds
                            if position_in_node > text_node.text.len() {
                                return Err(EditError::InvalidRange);
                            }

                            // Insert the text at the specified position
                            text_node.text.insert_str(position_in_node, &self.text);
                        }
                        _ => {
                            // For non-text nodes, insert a new text node before it
                            children.insert(idx, InlineNode::text(&self.text));
                        }
                    }
                    Ok(())
                } else {
                    // If no valid position found, the position is out of bounds
                    Err(EditError::InvalidRange)
                }
            }
            Node::CodeBlock { code, .. } => {
                // For code blocks, directly insert the text at the position
                if self.position <= code.len() {
                    // Store the original state
                    let original_code = code.clone();
                    self.affected_nodes = Some(vec![InlineNode::text(&original_code)]);

                    // Insert the text
                    code.insert_str(self.position, &self.text);
                    Ok(())
                } else {
                    Err(EditError::InvalidRange)
                }
            }
            // Be more specific about which node types are not supported
            Node::List { .. } => Err(EditError::UnsupportedOperation),
            Node::BlockQuote { .. } => Err(EditError::UnsupportedOperation),
            Node::ThematicBreak => Err(EditError::UnsupportedOperation),
            Node::Table { .. } => Err(EditError::UnsupportedOperation),
            Node::Group { .. } => Err(EditError::UnsupportedOperation),
            Node::FootnoteReference(_) => Err(EditError::UnsupportedOperation),
            Node::FootnoteDefinition(_) => Err(EditError::UnsupportedOperation),
            Node::DefinitionList { .. } => Err(EditError::UnsupportedOperation),
            Node::MathBlock { .. } => Err(EditError::UnsupportedOperation),
            // Handle temporary variants
            Node::TempListItem(_) => Err(EditError::UnsupportedOperation),
            Node::TempTableCell(_) => Err(EditError::UnsupportedOperation),
        }
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(original_nodes) = &self.affected_nodes {
            let mut document = self.document.borrow_mut();

            if self.node_index >= document.nodes.len() {
                return Err(EditError::IndexOutOfBounds);
            }

            match &mut document.nodes[self.node_index] {
                Node::Paragraph { children } | Node::Heading { children, .. } => {
                    // Restore the original children
                    *children = original_nodes.clone();
                    Ok(())
                }
                Node::CodeBlock { code, .. } => {
                    // Restore the original code from the first node (we stored it as a text node)
                    if let InlineNode::Text(text_node) = &original_nodes[0] {
                        *code = text_node.text.clone();
                        Ok(())
                    } else {
                        Err(EditError::OperationFailed)
                    }
                }
                // These node types should never have been modified
                Node::List { .. }
                | Node::BlockQuote { .. }
                | Node::ThematicBreak
                | Node::Table { .. }
                | Node::Group { .. }
                | Node::FootnoteReference(_)
                | Node::FootnoteDefinition(_)
                | Node::DefinitionList { .. }
                | Node::MathBlock { .. }
                | Node::TempListItem(_)
                | Node::TempTableCell(_) => Err(EditError::UnsupportedOperation),
            }
        } else {
            Err(EditError::OperationFailed)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
