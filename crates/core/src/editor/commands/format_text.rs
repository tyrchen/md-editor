use crate::editor::command::Command;
use crate::{Document, EditError, InlineNode, Node, TextFormatting, TextNode};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to format text within a paragraph node
pub struct FormatTextCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    start: usize,
    end: usize,
    formatting: TextFormatting,
    original_nodes: Option<Vec<InlineNode>>,
}

impl FormatTextCommand {
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_index: usize,
        start: usize,
        end: usize,
        formatting: TextFormatting,
    ) -> Self {
        Self {
            document,
            node_index,
            start,
            end,
            formatting,
            original_nodes: None,
        }
    }
}

impl Command for FormatTextCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        if self.start >= self.end {
            return Err(EditError::InvalidRange);
        }

        match &mut document.nodes[self.node_index] {
            Node::Paragraph { children } | Node::Heading { children, .. } => {
                // Store original nodes for undo
                self.original_nodes = Some(children.clone());

                // Locate the range to format, splitting nodes if needed
                let mut new_children = Vec::new();
                let mut current_offset = 0;

                for child in children.iter() {
                    match child {
                        InlineNode::Text(TextNode { text, formatting }) => {
                            let text_len = text.len();
                            let next_offset = current_offset + text_len;

                            // This node contains part of the range to format
                            if self.start < next_offset && self.end > current_offset {
                                // Calculate the range within this node
                                let node_start = self.start.saturating_sub(current_offset);
                                let node_end = std::cmp::min(self.end - current_offset, text_len);

                                // If starting after the beginning of this node, add text before the formatted part
                                if node_start > 0 {
                                    new_children.push(InlineNode::Text(TextNode {
                                        text: text[..node_start].to_string(),
                                        formatting: formatting.clone(),
                                    }));
                                }

                                // Add the formatted part
                                let mut new_formatting = formatting.clone();

                                // Apply the new formatting
                                if self.formatting.bold {
                                    new_formatting.bold = true;
                                }
                                if self.formatting.italic {
                                    new_formatting.italic = true;
                                }
                                if self.formatting.code {
                                    new_formatting.code = true;
                                }
                                if self.formatting.strikethrough {
                                    new_formatting.strikethrough = true;
                                }

                                new_children.push(InlineNode::Text(TextNode {
                                    text: text[node_start..node_end].to_string(),
                                    formatting: new_formatting,
                                }));

                                // If ending before the end of this node, add text after the formatted part
                                if node_end < text_len {
                                    new_children.push(InlineNode::Text(TextNode {
                                        text: text[node_end..].to_string(),
                                        formatting: formatting.clone(),
                                    }));
                                }
                            } else {
                                // This node is outside the format range, keep it as is
                                new_children.push(child.clone());
                            }

                            current_offset = next_offset;
                        }
                        _ => {
                            // For non-text nodes, just add them unchanged for now
                            // A more complete implementation would handle other node types
                            new_children.push(child.clone());

                            // Increment by 1 for non-text nodes
                            current_offset += 1;
                        }
                    }
                }

                // Replace the children with our new collection
                *children = new_children;

                Ok(())
            }
            _ => Err(EditError::UnsupportedOperation),
        }
    }

    fn undo(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        match &mut document.nodes[self.node_index] {
            Node::Paragraph { children } | Node::Heading { children, .. } => {
                if let Some(original) = &self.original_nodes {
                    // Restore the original nodes
                    *children = original.clone();
                    Ok(())
                } else {
                    Err(EditError::OperationFailed)
                }
            }
            _ => Err(EditError::UnsupportedOperation),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
