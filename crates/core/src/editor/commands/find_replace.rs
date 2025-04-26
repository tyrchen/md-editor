use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::editor::EditError;
use crate::editor::command::Command;
use crate::{Document, InlineNode, Node};

/// Command for finding and replacing text throughout a document
pub struct FindReplaceCommand {
    /// Reference to the document
    document: Rc<RefCell<Document>>,
    /// String to find
    find: String,
    /// String to replace with
    replace: String,
    /// Whether the search should be case sensitive
    case_sensitive: bool,
    /// Original nodes state for undo
    original_nodes: Vec<(usize, Node)>,
    /// Count of replacements made
    replacements: usize,
}

impl FindReplaceCommand {
    /// Create a new FindReplaceCommand
    pub fn new(
        document: Rc<RefCell<Document>>,
        find: &str,
        replace: &str,
        case_sensitive: bool,
    ) -> Self {
        Self {
            document,
            find: find.to_string(),
            replace: replace.to_string(),
            case_sensitive,
            original_nodes: Vec::new(),
            replacements: 0,
        }
    }

    /// Get the number of replacements made
    pub fn replacements(&self) -> usize {
        self.replacements
    }
}

impl Command for FindReplaceCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        if self.find.is_empty() {
            return Ok(());
        }

        let mut modified_node_indices = Vec::new();
        let mut replacements = 0;

        // We need to clone the document to avoid borrowing issues
        let doc = self.document.borrow();
        let nodes_len = doc.nodes.len();

        // First pass: collect nodes that need modification
        for node_idx in 0..nodes_len {
            match &doc.nodes[node_idx] {
                Node::Paragraph { children } => {
                    let mut needs_update = false;
                    for child in children {
                        if let InlineNode::Text(text_node) = child {
                            let original_text = &text_node.text;
                            if self.case_sensitive {
                                if original_text.contains(&self.find) {
                                    needs_update = true;
                                    break;
                                }
                            } else if original_text
                                .to_lowercase()
                                .contains(&self.find.to_lowercase())
                            {
                                needs_update = true;
                                break;
                            }
                        }
                    }

                    if needs_update {
                        modified_node_indices.push(node_idx);
                        self.original_nodes
                            .push((node_idx, doc.nodes[node_idx].clone()));
                    }
                }
                Node::CodeBlock { code, .. } => {
                    let original_code = code;
                    let mut needs_update = false;

                    if self.case_sensitive {
                        if original_code.contains(&self.find) {
                            needs_update = true;
                        }
                    } else if original_code
                        .to_lowercase()
                        .contains(&self.find.to_lowercase())
                    {
                        needs_update = true;
                    }

                    if needs_update {
                        modified_node_indices.push(node_idx);
                        self.original_nodes
                            .push((node_idx, doc.nodes[node_idx].clone()));
                    }
                }
                _ => {} // Other node types don't need processing
            }
        }

        // Drop the borrow to modify the document
        drop(doc);

        // Second pass: update all identified nodes
        for node_idx in modified_node_indices {
            let mut document = self.document.borrow_mut();
            match &mut document.nodes[node_idx] {
                Node::Paragraph { children } => {
                    for child in children.iter_mut() {
                        if let InlineNode::Text(text_node) = child {
                            let original_text = text_node.text.clone();
                            let new_text = if self.case_sensitive {
                                let count = original_text.matches(&self.find).count();
                                replacements += count;
                                original_text.replace(&self.find, &self.replace)
                            } else {
                                // Case insensitive replacement
                                let mut result = original_text.to_string();
                                let mut last_end = 0;
                                let mut current_replacements = 0;

                                let text_lower = original_text.to_lowercase();
                                let find_lower = self.find.to_lowercase();

                                while let Some(start) = text_lower[last_end..].find(&find_lower) {
                                    let abs_start = last_end + start;
                                    let abs_end = abs_start + self.find.len();

                                    // Replace the original case text with the replacement
                                    let before = &result[..abs_start];
                                    let after = &result[abs_end..];
                                    result = format!("{}{}{}", before, self.replace, after);

                                    // Update the last end position accounting for any length difference
                                    let length_diff =
                                        self.replace.len() as isize - self.find.len() as isize;
                                    last_end = (abs_end as isize + length_diff) as usize;
                                    current_replacements += 1;
                                }

                                replacements += current_replacements;
                                result
                            };

                            text_node.text = new_text;
                        }
                    }
                }
                Node::CodeBlock { code, .. } => {
                    let original_code = code.clone();
                    let new_code = if self.case_sensitive {
                        let count = original_code.matches(&self.find).count();
                        replacements += count;
                        original_code.replace(&self.find, &self.replace)
                    } else {
                        // Case insensitive replacement for code blocks
                        let mut result = original_code.to_string();
                        let mut last_end = 0;
                        let mut current_replacements = 0;

                        let code_lower = original_code.to_lowercase();
                        let find_lower = self.find.to_lowercase();

                        while let Some(start) = code_lower[last_end..].find(&find_lower) {
                            let abs_start = last_end + start;
                            let abs_end = abs_start + self.find.len();

                            let before = &result[..abs_start];
                            let after = &result[abs_end..];
                            result = format!("{}{}{}", before, self.replace, after);

                            let length_diff =
                                self.replace.len() as isize - self.find.len() as isize;
                            last_end = (abs_end as isize + length_diff) as usize;
                            current_replacements += 1;
                        }

                        replacements += current_replacements;
                        result
                    };

                    *code = new_code;
                }
                _ => {}
            }
        }

        self.replacements = replacements;
        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        // Restore the original nodes
        for (node_idx, original_node) in self.original_nodes.drain(..) {
            let mut document = self.document.borrow_mut();
            if node_idx < document.nodes.len() {
                document.nodes[node_idx] = original_node;
            } else {
                return Err(EditError::IndexOutOfBounds);
            }
        }
        self.replacements = 0;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
