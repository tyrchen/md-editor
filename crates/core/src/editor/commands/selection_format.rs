use crate::editor::command::Command;
use crate::{Document, EditError, InlineNode, Node, Selection, TextFormatting, TextNode};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to apply formatting to the currently selected text
pub struct SelectionFormatCommand {
    document: Rc<RefCell<Document>>,
    formatting: TextFormatting,
    /// Store the original nodes for undo
    original_nodes: Vec<(usize, Node)>,
    /// Store the original selection
    original_selection: Option<Selection>,
}

impl SelectionFormatCommand {
    pub fn new(document: Rc<RefCell<Document>>, formatting: TextFormatting) -> Self {
        Self {
            document,
            formatting,
            original_nodes: Vec::new(),
            original_selection: None,
        }
    }
}

impl Command for SelectionFormatCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Check if there's an active selection
        let selection = match document.selection.take() {
            Some(sel) if !sel.is_collapsed => {
                // Store original selection for undo
                self.original_selection = Some(sel.clone());
                sel
            }
            Some(sel) => {
                // Put the selection back and return - nothing to format
                document.selection = Some(sel);
                return Ok(());
            }
            None => return Ok(()),
        };

        // Handle multi-node selection
        if selection.start.path[0] != selection.end.path[0] {
            // Currently only support formatting within a single node
            document.selection = Some(selection);
            return Err(EditError::UnsupportedOperation);
        }

        // Get node index from selection
        let node_idx = selection.start.path[0];
        if node_idx >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Store the original node for undo
        self.original_nodes
            .push((node_idx, document.nodes[node_idx].clone()));

        // Based on node type, handle formatting
        match &mut document.nodes[node_idx] {
            Node::Paragraph { children } | Node::Heading { children, .. } => {
                // Get start and end offsets within the node
                let start_offset = selection.start.offset;
                let end_offset = selection.end.offset;

                // Track the current text position
                let mut current_offset = 0;
                let mut new_children = Vec::new();

                // Process each inline element
                for child in children.iter() {
                    match child {
                        InlineNode::Text(text_node) => {
                            let text_len = text_node.text.len();
                            let next_offset = current_offset + text_len;

                            // Case 1: Text node is completely before the selection
                            // Case 2: Text node is completely after the selection
                            if next_offset <= start_offset || current_offset >= end_offset {
                                new_children.push(child.clone());
                            }
                            // Case 3: Text node overlaps with the selection
                            else {
                                // Add text before selection if any
                                if current_offset < start_offset {
                                    let before_len = start_offset - current_offset;
                                    let before_text = text_node.text[..before_len].to_string();
                                    new_children.push(InlineNode::Text(TextNode {
                                        text: before_text,
                                        formatting: text_node.formatting.clone(),
                                    }));
                                }

                                // Add selected text with new formatting
                                let sel_start = start_offset.saturating_sub(current_offset);
                                let sel_end = std::cmp::min(end_offset - current_offset, text_len);

                                if sel_start < sel_end {
                                    let selected_text =
                                        text_node.text[sel_start..sel_end].to_string();

                                    // Create new formatting by merging existing and new formats
                                    let mut merged_formatting = text_node.formatting.clone();

                                    // Apply requested formatting changes
                                    if self.formatting.bold {
                                        merged_formatting.bold = true;
                                    }
                                    if self.formatting.italic {
                                        merged_formatting.italic = true;
                                    }
                                    if self.formatting.code {
                                        merged_formatting.code = true;
                                    }
                                    if self.formatting.strikethrough {
                                        merged_formatting.strikethrough = true;
                                    }

                                    new_children.push(InlineNode::Text(TextNode {
                                        text: selected_text,
                                        formatting: merged_formatting,
                                    }));
                                }

                                // Add text after selection if any
                                if next_offset > end_offset {
                                    let after_start = sel_end;
                                    let after_text = text_node.text[after_start..].to_string();
                                    new_children.push(InlineNode::Text(TextNode {
                                        text: after_text,
                                        formatting: text_node.formatting.clone(),
                                    }));
                                }
                            }

                            current_offset = next_offset;
                        }
                        _ => {
                            // Keep other inline node types as is if they're not in the selection
                            let node_offset = current_offset;
                            let node_end = current_offset + 1; // Non-text nodes count as 1 position

                            // If outside selection range, keep as is
                            if node_end <= start_offset || node_offset >= end_offset {
                                new_children.push(child.clone());
                            }

                            current_offset = node_end;
                        }
                    }
                }

                // Replace with the new children
                *children = new_children;

                // Restore the selection
                document.selection = Some(selection);
            }
            // Code blocks don't support rich text formatting
            _ => {
                document.selection = Some(selection);
                return Err(EditError::UnsupportedOperation);
            }
        }

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Restore original nodes
        for (idx, original_node) in self.original_nodes.drain(..) {
            if idx < document.nodes.len() {
                document.nodes[idx] = original_node;
            }
        }

        // Restore original selection
        if let Some(selection) = self.original_selection.take() {
            document.selection = Some(selection);
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
