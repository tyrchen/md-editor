use crate::{Document, InlineNode, Node, Position, Selection, TextNode};

/// Extension methods for Document to help with selections
impl Document {
    /// Sets the selection to select all content in the document
    pub fn select_all(&mut self) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        // Start position at the beginning of the first node
        let start = Position::new(vec![0], 0);

        // End position at the end of the last node
        let last_idx = self.nodes.len() - 1;
        let end_offset = match &self.nodes[last_idx] {
            Node::Paragraph { children } => {
                // Get length of the paragraph
                children.iter().fold(0, |acc, child| {
                    acc + match child {
                        InlineNode::Text(TextNode { text, .. }) => text.len(),
                        // Other inline node types - estimate 1 character each
                        _ => 1,
                    }
                })
            }
            Node::CodeBlock { code, .. } => code.len(),
            // For other node types, use position 0 as we're selecting the whole node
            _ => 0,
        };

        let end = Position::new(vec![last_idx], end_offset);
        self.selection = Some(Selection::new(start, end));
        true
    }

    /// Selects a specific node by index
    pub fn select_node(&mut self, node_index: usize) -> bool {
        if node_index >= self.nodes.len() {
            return false;
        }

        // Calculate the end position based on node type
        let end_offset = match &self.nodes[node_index] {
            Node::Paragraph { children } => {
                // Get length of the paragraph
                children.iter().fold(0, |acc, child| {
                    acc + match child {
                        InlineNode::Text(TextNode { text, .. }) => text.len(),
                        // Other inline node types - estimate 1 character each
                        _ => 1,
                    }
                })
            }
            Node::CodeBlock { code, .. } => code.len(),
            // For other node types, use position 0 as we're selecting the whole node
            _ => 0,
        };

        let start = Position::new(vec![node_index], 0);
        let end = Position::new(vec![node_index], end_offset);

        self.selection = Some(Selection::new(start, end));
        true
    }

    /// Selects a range between two nodes (inclusive)
    pub fn select_node_range(&mut self, start_index: usize, end_index: usize) -> bool {
        if start_index >= self.nodes.len()
            || end_index >= self.nodes.len()
            || start_index > end_index
        {
            return false;
        }

        let start = Position::new(vec![start_index], 0);

        // Calculate the end position based on node type
        let end_offset = match &self.nodes[end_index] {
            Node::Paragraph { children } => {
                // Get length of the paragraph
                children.iter().fold(0, |acc, child| {
                    acc + match child {
                        InlineNode::Text(TextNode { text, .. }) => text.len(),
                        // Other inline node types - estimate 1 character each
                        _ => 1,
                    }
                })
            }
            Node::CodeBlock { code, .. } => code.len(),
            // For other node types, use position 0 as we're selecting the whole node
            _ => 0,
        };

        let end = Position::new(vec![end_index], end_offset);

        self.selection = Some(Selection::new(start, end));
        true
    }

    /// Selects a specific range within a single node
    pub fn select_text_range(
        &mut self,
        node_index: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> bool {
        if node_index >= self.nodes.len() || start_offset > end_offset {
            return false;
        }

        // Check valid offsets for this node type
        match &self.nodes[node_index] {
            Node::Paragraph { children } => {
                let total_length = children.iter().fold(0, |acc, child| {
                    acc + match child {
                        InlineNode::Text(TextNode { text, .. }) => text.len(),
                        _ => 1,
                    }
                });

                if end_offset > total_length {
                    return false;
                }
            }
            Node::CodeBlock { code, .. } => {
                if end_offset > code.len() {
                    return false;
                }
            }
            // Other node types would need specific implementation
            _ => {
                // For now, return false for unsupported node types
                return false;
            }
        }

        let start = Position::new(vec![node_index], start_offset);
        let end = Position::new(vec![node_index], end_offset);

        self.selection = Some(Selection::new(start, end));
        true
    }

    /// Select from one position to another across any nodes
    pub fn select_range(
        &mut self,
        start_node: usize,
        start_offset: usize,
        end_node: usize,
        end_offset: usize,
    ) -> bool {
        if start_node >= self.nodes.len() || end_node >= self.nodes.len() {
            return false;
        }

        let start = Position::new(vec![start_node], start_offset);
        let end = Position::new(vec![end_node], end_offset);

        self.selection = Some(Selection::new(start, end));
        true
    }

    /// Collapse selection to its start
    pub fn collapse_selection_to_start(&mut self) -> bool {
        if let Some(selection) = &self.selection {
            let start = selection.start.clone();
            self.selection = Some(Selection::collapsed(start));
            true
        } else {
            false
        }
    }

    /// Collapse selection to its end
    pub fn collapse_selection_to_end(&mut self) -> bool {
        if let Some(selection) = &self.selection {
            let end = selection.end.clone();
            self.selection = Some(Selection::collapsed(end));
            true
        } else {
            false
        }
    }

    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Returns true if there is an active selection
    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    /// Returns true if there is an active selection that spans multiple nodes
    pub fn has_multi_node_selection(&self) -> bool {
        if let Some(selection) = &self.selection {
            selection.start.path[0] != selection.end.path[0]
        } else {
            false
        }
    }

    /// Returns the selected text as a string, if possible
    pub fn get_selected_text(&self) -> Option<String> {
        let selection = self.selection.as_ref()?;

        // Handle single node selection
        if selection.start.path[0] == selection.end.path[0] {
            let node_idx = selection.start.path[0];
            let start_offset = selection.start.offset;
            let end_offset = selection.end.offset;

            match &self.nodes[node_idx] {
                Node::Paragraph { children } => {
                    let mut result = String::new();
                    let mut current_offset = 0;

                    for child in children {
                        match child {
                            InlineNode::Text(TextNode { text, .. }) => {
                                let next_offset = current_offset + text.len();

                                // If this text node overlaps with the selection
                                if start_offset < next_offset && end_offset > current_offset {
                                    let sel_start = start_offset.saturating_sub(current_offset);
                                    let sel_end =
                                        std::cmp::min(end_offset - current_offset, text.len());

                                    if sel_start < sel_end {
                                        result.push_str(&text[sel_start..sel_end]);
                                    }
                                }

                                current_offset = next_offset;
                            }
                            // Handle other inline node types as needed
                            _ => {
                                // For simplicity, skip non-text nodes for now
                                current_offset += 1;
                            }
                        }
                    }

                    Some(result)
                }
                Node::CodeBlock { code, .. } => {
                    if start_offset < code.len() && end_offset <= code.len() {
                        Some(code[start_offset..end_offset].to_string())
                    } else {
                        None
                    }
                }
                // Add cases for other node types as needed
                _ => None,
            }
        } else {
            // Multi-node selection - more complex to implement
            // This would require traversing multiple nodes and concatenating text
            // For now, just return None for multi-node selections
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_all() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("First paragraph");
        doc.add_paragraph_with_text("Second paragraph");

        // Select all
        assert!(doc.select_all());

        // Verify selection
        let selection = doc.selection.unwrap();
        assert_eq!(selection.start.path, vec![0]);
        assert_eq!(selection.start.offset, 0);
        assert_eq!(selection.end.path, vec![1]);
        assert!(!selection.is_collapsed);
    }

    #[test]
    fn test_select_node() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("First paragraph");
        doc.add_paragraph_with_text("Second paragraph");

        // Select node 1
        assert!(doc.select_node(1));

        // Verify selection
        let selection = doc.selection.unwrap();
        assert_eq!(selection.start.path, vec![1]);
        assert_eq!(selection.start.offset, 0);
        assert_eq!(selection.end.path, vec![1]);
        assert!(!selection.is_collapsed);
    }

    #[test]
    fn test_select_text_range() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("First paragraph");

        // Select part of the text
        assert!(doc.select_text_range(0, 6, 15));

        // Verify selection
        let selection = doc.selection.as_ref().unwrap();
        assert_eq!(selection.start.path, vec![0]);
        assert_eq!(selection.start.offset, 6);
        assert_eq!(selection.end.path, vec![0]);
        assert_eq!(selection.end.offset, 15);

        // Verify selected text
        assert_eq!(doc.get_selected_text(), Some("paragraph".to_string()));
    }

    #[test]
    fn test_collapse_selection() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("First paragraph");
        doc.add_paragraph_with_text("Second paragraph");

        // Select all
        doc.select_all();

        // Collapse to start
        assert!(doc.collapse_selection_to_start());

        // Verify selection
        let selection = doc.selection.unwrap();
        assert_eq!(selection.start.path, vec![0]);
        assert_eq!(selection.start.offset, 0);
        assert_eq!(selection.end.path, vec![0]);
        assert_eq!(selection.end.offset, 0);
        assert!(selection.is_collapsed);
    }

    #[test]
    fn test_clear_selection() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("Paragraph");

        // Select node
        doc.select_node(0);
        assert!(doc.has_selection());

        // Clear selection
        doc.clear_selection();
        assert!(!doc.has_selection());
    }

    #[test]
    fn test_has_multi_node_selection() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("First paragraph");
        doc.add_paragraph_with_text("Second paragraph");

        // Single node selection
        doc.select_node(0);
        assert!(!doc.has_multi_node_selection());

        // Multi-node selection
        doc.select_node_range(0, 1);
        assert!(doc.has_multi_node_selection());
    }
}
