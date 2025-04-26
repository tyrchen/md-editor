use crate::editor::command::Command;
use crate::{Document, EditError, InlineNode, Node, Selection};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to cut the currently selected content
pub struct CutSelectionCommand {
    document: Rc<RefCell<Document>>,
    /// Store the original selection for undo
    original_selection: Option<Selection>,
    /// Store the original nodes that were modified or deleted
    original_nodes: Vec<(usize, Node)>,
    /// Store cut content for clipboard or undo
    cut_content: Vec<Node>,
}

impl CutSelectionCommand {
    pub fn new(document: Rc<RefCell<Document>>) -> Self {
        Self {
            document,
            original_selection: None,
            original_nodes: Vec::new(),
            cut_content: Vec::new(),
        }
    }

    /// Get the content that was cut
    pub fn cut_content(&self) -> &[Node] {
        &self.cut_content
    }
}

impl Command for CutSelectionCommand {
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
                // Put the selection back and return - nothing to cut
                document.selection = Some(sel);
                return Ok(());
            }
            None => return Ok(()),
        };

        // Simple case: if the selection spans node boundaries, cut the entire nodes
        if selection.start.path[0] != selection.end.path[0] {
            // Get the range of nodes to cut
            let start_node_idx = selection.start.path[0];
            let end_node_idx = selection.end.path[0];

            // Store original nodes for undo
            for idx in start_node_idx..=end_node_idx {
                if idx < document.nodes.len() {
                    self.original_nodes.push((idx, document.nodes[idx].clone()));
                }
            }

            // Cut the nodes
            let mut nodes_to_cut = Vec::new();
            for idx in (start_node_idx..=end_node_idx).rev() {
                if idx < document.nodes.len() {
                    nodes_to_cut.push(document.nodes.remove(idx));
                }
            }

            // Store in reverse order (natural reading order)
            nodes_to_cut.reverse();
            self.cut_content = nodes_to_cut;

            // Create a new collapsed selection at the start of the cut
            if start_node_idx < document.nodes.len() {
                document.selection = Some(Selection::collapsed(selection.start.clone()));
            } else if !document.nodes.is_empty() {
                // If we cut to the end, position selection at the last node
                let last_idx = document.nodes.len() - 1;
                document.selection = Some(Selection::collapsed(crate::Position::new(
                    vec![last_idx],
                    0,
                )));
            }

            return Ok(());
        }

        // Single node selection - text cut operation
        let node_idx = selection.start.path[0];
        if node_idx >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Clone the node for inspection before modification
        let node_clone = document.nodes[node_idx].clone();

        // Store the original node for undo
        self.original_nodes.push((node_idx, node_clone.clone()));

        // Based on node type, handle selection cut
        match (&mut document.nodes[node_idx], &node_clone) {
            (
                Node::Paragraph { children },
                Node::Paragraph {
                    children: orig_children,
                },
            ) => {
                // Get start and end offsets within the node
                let start_offset = selection.start.offset;
                let end_offset = selection.end.offset;

                // Create a copy of the content for the cut clipboard
                let mut cut_paragraph = Node::Paragraph {
                    children: Vec::new(),
                };

                // Extract selected text and add to cut_paragraph
                if let Node::Paragraph {
                    children: cut_children,
                } = &mut cut_paragraph
                {
                    let mut current_offset = 0;

                    for child in orig_children.iter() {
                        if let InlineNode::Text(text_node) = child {
                            let text_len = text_node.text.len();
                            let next_offset = current_offset + text_len;

                            // If this text node is within the selection range
                            if current_offset < end_offset && next_offset > start_offset {
                                let sel_start = start_offset.saturating_sub(current_offset);
                                let sel_end = std::cmp::min(end_offset - current_offset, text_len);

                                if sel_start < sel_end {
                                    let selected_text =
                                        text_node.text[sel_start..sel_end].to_string();
                                    cut_children.push(InlineNode::Text(crate::TextNode {
                                        text: selected_text,
                                        formatting: text_node.formatting.clone(),
                                    }));
                                }
                            }

                            current_offset = next_offset;
                        } else {
                            // For non-text nodes, consider if they're within the selection
                            let next_offset = current_offset + 1;
                            if current_offset >= start_offset && next_offset <= end_offset {
                                cut_children.push(child.clone());
                            }
                            current_offset = next_offset;
                        }
                    }
                }

                // Store cut content
                self.cut_content = vec![cut_paragraph];

                // Now remove the selected text from the document
                let mut modified_children = Vec::new();
                let mut current_offset = 0;

                for child in orig_children.iter() {
                    match child {
                        InlineNode::Text(text_node) => {
                            let text_len = text_node.text.len();
                            let next_offset = current_offset + text_len;

                            // If this node is entirely outside the selection, keep it
                            if next_offset <= start_offset || current_offset >= end_offset {
                                modified_children.push(child.clone());
                            }
                            // If it partially overlaps the selection
                            else {
                                // Add text before the selection
                                if current_offset < start_offset {
                                    let sel_start = start_offset - current_offset;
                                    let before_text = text_node.text[0..sel_start].to_string();
                                    modified_children.push(InlineNode::Text(crate::TextNode {
                                        text: before_text,
                                        formatting: text_node.formatting.clone(),
                                    }));
                                }

                                // Add text after the selection
                                if next_offset > end_offset {
                                    let sel_end = end_offset - current_offset;
                                    let after_text = text_node.text[sel_end..].to_string();
                                    modified_children.push(InlineNode::Text(crate::TextNode {
                                        text: after_text,
                                        formatting: text_node.formatting.clone(),
                                    }));
                                }
                            }

                            current_offset = next_offset;
                        }
                        _ => {
                            // For non-text nodes, keep if outside selection
                            let next_offset = current_offset + 1;
                            if next_offset <= start_offset || current_offset >= end_offset {
                                modified_children.push(child.clone());
                            }
                            current_offset = next_offset;
                        }
                    }
                }

                // Replace the children with modified list
                *children = modified_children;

                // Set collapsed selection at the start of the cut
                document.selection = Some(Selection::collapsed(selection.start));
            }
            (
                Node::Heading {
                    children,
                    level: _level,
                },
                Node::Heading {
                    children: orig_children,
                    level: orig_level,
                },
            ) => {
                // Get start and end offsets within the node
                let start_offset = selection.start.offset;
                let end_offset = selection.end.offset;

                // Create a copy of the content for the cut clipboard
                let mut cut_heading = Node::Heading {
                    level: *orig_level,
                    children: Vec::new(),
                };

                // Extract selected text and add to cut_heading
                if let Node::Heading {
                    children: cut_children,
                    ..
                } = &mut cut_heading
                {
                    let mut current_offset = 0;

                    for child in orig_children.iter() {
                        if let InlineNode::Text(text_node) = child {
                            let text_len = text_node.text.len();
                            let next_offset = current_offset + text_len;

                            // If this text node is within the selection range
                            if current_offset < end_offset && next_offset > start_offset {
                                let sel_start = start_offset.saturating_sub(current_offset);
                                let sel_end = std::cmp::min(end_offset - current_offset, text_len);

                                if sel_start < sel_end {
                                    let selected_text =
                                        text_node.text[sel_start..sel_end].to_string();
                                    cut_children.push(InlineNode::Text(crate::TextNode {
                                        text: selected_text,
                                        formatting: text_node.formatting.clone(),
                                    }));
                                }
                            }

                            current_offset = next_offset;
                        } else {
                            // For non-text nodes, consider if they're within the selection
                            let next_offset = current_offset + 1;
                            if current_offset >= start_offset && next_offset <= end_offset {
                                cut_children.push(child.clone());
                            }
                            current_offset = next_offset;
                        }
                    }
                }

                // Store cut content
                self.cut_content = vec![cut_heading];

                // Now remove the selected text from the document
                let mut modified_children = Vec::new();
                let mut current_offset = 0;

                for child in orig_children.iter() {
                    match child {
                        InlineNode::Text(text_node) => {
                            let text_len = text_node.text.len();
                            let next_offset = current_offset + text_len;

                            // If this node is entirely outside the selection, keep it
                            if next_offset <= start_offset || current_offset >= end_offset {
                                modified_children.push(child.clone());
                            }
                            // If it partially overlaps the selection
                            else {
                                // Add text before the selection
                                if current_offset < start_offset {
                                    let sel_start = start_offset - current_offset;
                                    let before_text = text_node.text[0..sel_start].to_string();
                                    modified_children.push(InlineNode::Text(crate::TextNode {
                                        text: before_text,
                                        formatting: text_node.formatting.clone(),
                                    }));
                                }

                                // Add text after the selection
                                if next_offset > end_offset {
                                    let sel_end = end_offset - current_offset;
                                    let after_text = text_node.text[sel_end..].to_string();
                                    modified_children.push(InlineNode::Text(crate::TextNode {
                                        text: after_text,
                                        formatting: text_node.formatting.clone(),
                                    }));
                                }
                            }

                            current_offset = next_offset;
                        }
                        _ => {
                            // For non-text nodes, keep if outside selection
                            let next_offset = current_offset + 1;
                            if next_offset <= start_offset || current_offset >= end_offset {
                                modified_children.push(child.clone());
                            }
                            current_offset = next_offset;
                        }
                    }
                }

                // Replace the children with modified list
                *children = modified_children;

                // Set collapsed selection at the start of the cut
                document.selection = Some(Selection::collapsed(selection.start));
            }
            (
                Node::CodeBlock {
                    code,
                    language: _language,
                },
                Node::CodeBlock {
                    code: orig_code,
                    language: orig_language,
                },
            ) => {
                // For code blocks, just cut the selected text
                let start_offset = selection.start.offset;
                let end_offset = selection.end.offset;

                if end_offset > orig_code.len()
                    || start_offset > orig_code.len()
                    || start_offset >= end_offset
                {
                    return Err(EditError::InvalidRange);
                }

                // Store the cut content
                let cut_text = orig_code[start_offset..end_offset].to_string();
                let cut_node = Node::CodeBlock {
                    code: cut_text,
                    language: orig_language.clone(),
                };
                self.cut_content = vec![cut_node];

                // Remove the selected text
                code.replace_range(start_offset..end_offset, "");

                // Set collapsed selection at the start of the cut
                document.selection = Some(Selection::collapsed(selection.start));
            }
            _ => {
                // For unsupported node types, just restore the selection
                document.selection = Some(self.original_selection.take().unwrap());
                return Err(EditError::UnsupportedOperation);
            }
        }

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        // Restore original nodes
        let mut document = self.document.borrow_mut();

        for (idx, original_node) in self.original_nodes.drain(..) {
            if idx < document.nodes.len() {
                document.nodes[idx] = original_node;
            } else {
                // Handle edge case - may need to append
                document.nodes.push(original_node);
            }
        }

        // Restore original selection
        if let Some(sel) = self.original_selection.take() {
            document.selection = Some(sel);
        }

        // Clear cut content
        self.cut_content.clear();

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
