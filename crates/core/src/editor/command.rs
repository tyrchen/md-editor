use crate::{Document, EditError, InlineNode, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Trait representing a document editing command that can be executed and undone
pub trait Command {
    /// Execute the command
    fn execute(&mut self) -> Result<(), EditError>;
    /// Undo the command
    fn undo(&mut self) -> Result<(), EditError>;
    /// Get this command as Any to allow downcasting to specific types
    #[allow(dead_code)]
    fn as_any(&self) -> &dyn Any;
}

/// Command to delete text from a node
pub struct DeleteTextCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    start: usize,
    end: usize,
    deleted_text: Option<String>,
}

impl DeleteTextCommand {
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_index: usize,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            document,
            node_index,
            start,
            end,
            deleted_text: None,
        }
    }
}

impl Command for DeleteTextCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        if self.start >= self.end {
            return Err(EditError::InvalidRange);
        }

        match &mut document.nodes[self.node_index] {
            Node::Paragraph { children } => {
                // Find the right text node(s) to delete from
                let mut current_offset = 0;
                let mut text_to_delete = String::new();
                let mut offsets_to_modify = Vec::new();

                for (idx, child) in children.iter().enumerate() {
                    if let InlineNode::Text(text_node) = child {
                        let next_offset = current_offset + text_node.text.len();

                        // This node contains part of the range to delete
                        if self.start < next_offset && self.end > current_offset {
                            let start_in_node = self.start.saturating_sub(current_offset);
                            let end_in_node =
                                std::cmp::min(self.end - current_offset, text_node.text.len());

                            if start_in_node < end_in_node {
                                text_to_delete
                                    .push_str(&text_node.text[start_in_node..end_in_node]);
                                offsets_to_modify.push((idx, start_in_node, end_in_node));
                            }
                        }

                        current_offset = next_offset;
                    } else {
                        // For simplicity, treat other inline nodes as atomic with length 1
                        if current_offset == self.start && current_offset + 1 > self.end {
                            // Delete entire non-text node
                            offsets_to_modify.push((idx, 0, 1));
                        }
                        current_offset += 1;
                    }
                }

                // Store deleted text for undo
                self.deleted_text = Some(text_to_delete);

                // Apply deletions in reverse order to avoid index shifting
                for (idx, start_in_node, end_in_node) in offsets_to_modify.into_iter().rev() {
                    if let InlineNode::Text(text_node) = &mut children[idx] {
                        text_node.text.replace_range(start_in_node..end_in_node, "");

                        // Remove empty text nodes
                        if text_node.text.is_empty() {
                            children.remove(idx);
                        }
                    } else if start_in_node == 0 && end_in_node == 1 {
                        // Remove entire non-text node
                        children.remove(idx);
                    }
                }

                Ok(())
            }
            Node::CodeBlock { code, .. } => {
                if self.start < code.len() && self.end <= code.len() {
                    self.deleted_text = Some(code[self.start..self.end].to_string());
                    code.replace_range(self.start..self.end, "");
                    Ok(())
                } else {
                    Err(EditError::InvalidRange)
                }
            }
            // Add more node types as needed
            _ => Err(EditError::UnsupportedOperation),
        }
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(deleted_text) = &self.deleted_text {
            let mut document = self.document.borrow_mut();

            if self.node_index >= document.nodes.len() {
                return Err(EditError::IndexOutOfBounds);
            }

            match &mut document.nodes[self.node_index] {
                Node::Paragraph { children } => {
                    // Direct manipulation of the children instead of calling insert_text
                    // to avoid borrow issues
                    if children.is_empty() {
                        // If there are no children, recreate the text node
                        children.push(InlineNode::text(deleted_text));
                    } else if children.len() == 1 {
                        // If there's just one child, modify it directly to insert the text
                        if let InlineNode::Text(text_node) = &mut children[0] {
                            // Make sure we don't insert outside the string bounds
                            let pos = std::cmp::min(self.start, text_node.text.len());
                            text_node.text.insert_str(pos, deleted_text);
                        } else {
                            // If it's not a text node, create a new one and insert before
                            children.insert(0, InlineNode::text(deleted_text));
                        }
                    } else {
                        // For more complex cases, try to find the right position
                        // For now, insert at the position that seems most appropriate
                        let text_to_insert = deleted_text.clone();

                        // Try to find a node near our start position
                        let mut current_offset = 0;
                        let mut insert_idx = 0;

                        for (idx, child) in children.iter().enumerate() {
                            if let InlineNode::Text(text_node) = child {
                                let next_offset = current_offset + text_node.text.len();
                                if self.start >= current_offset && self.start <= next_offset {
                                    // Found the right node
                                    if self.start == current_offset {
                                        // Insert before this node
                                        insert_idx = idx;
                                    } else {
                                        // Insert after this node
                                        insert_idx = idx + 1;
                                    }
                                    break;
                                }
                                current_offset = next_offset;
                            }
                            insert_idx = idx + 1;
                        }

                        // Insert at the found position
                        children.insert(insert_idx, InlineNode::text(text_to_insert));
                    }
                    Ok(())
                }
                Node::CodeBlock { code, .. } => {
                    if self.start <= code.len() {
                        code.insert_str(self.start, deleted_text);
                        Ok(())
                    } else {
                        Err(EditError::InvalidRange)
                    }
                }
                // Add more node types as needed
                _ => Err(EditError::UnsupportedOperation),
            }
        } else {
            Err(EditError::OperationFailed)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to merge two adjacent nodes
pub struct MergeNodesCommand {
    document: Rc<RefCell<Document>>,
    first_index: usize,
    second_index: usize,
    original_second_node: Option<Node>,
}

impl MergeNodesCommand {
    pub fn new(document: Rc<RefCell<Document>>, first_index: usize, second_index: usize) -> Self {
        Self {
            document,
            first_index,
            second_index,
            original_second_node: None,
        }
    }
}

impl Command for MergeNodesCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.first_index >= document.nodes.len() || self.second_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        if self.second_index != self.first_index + 1 {
            return Err(EditError::InvalidRange);
        }

        // Store original second node for undo
        self.original_second_node = Some(document.nodes[self.second_index].clone());

        // Clone the second node to avoid borrowing conflicts
        let second_node = document.nodes[self.second_index].clone();

        // Need to be same node type
        match (&mut document.nodes[self.first_index], &second_node) {
            (
                Node::Paragraph {
                    children: first_children,
                },
                Node::Paragraph {
                    children: second_children,
                },
            ) => {
                // Merge children of second paragraph into first
                first_children.extend_from_slice(second_children);

                // Remove second node
                document.nodes.remove(self.second_index);

                Ok(())
            }
            (
                Node::CodeBlock {
                    code: first_code,
                    language: first_lang,
                    ..
                },
                Node::CodeBlock {
                    code: second_code,
                    language: second_lang,
                    ..
                },
            ) => {
                // Only merge if languages match
                if first_lang != second_lang {
                    return Err(EditError::UnsupportedOperation);
                }

                // Merge code
                first_code.push('\n');
                first_code.push_str(second_code);

                // Remove second node
                document.nodes.remove(self.second_index);

                Ok(())
            }
            // Add more mergeable node type pairs as needed
            _ => Err(EditError::UnsupportedOperation),
        }
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(node) = &self.original_second_node {
            let mut document = self.document.borrow_mut();

            if self.first_index >= document.nodes.len() {
                return Err(EditError::IndexOutOfBounds);
            }

            document.nodes.insert(self.second_index, node.clone());

            // Restore first node to its original state by removing the merged content
            match (&mut document.nodes[self.first_index], node) {
                (
                    Node::Paragraph { children },
                    Node::Paragraph {
                        children: second_children,
                    },
                ) => {
                    // Remove the number of elements that were added from the second node
                    children.truncate(children.len() - second_children.len());
                }
                (
                    Node::CodeBlock {
                        code, language: _, ..
                    },
                    Node::CodeBlock {
                        code: second_code, ..
                    },
                ) => {
                    // Remove the newline and the second code content
                    let first_len = code.len() - second_code.len() - 1;
                    code.truncate(first_len);
                }
                _ => return Err(EditError::UnsupportedOperation),
            }

            Ok(())
        } else {
            Err(EditError::OperationFailed)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
