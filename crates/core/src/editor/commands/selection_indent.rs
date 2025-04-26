use crate::editor::command::Command;
use crate::{Document, EditError, ListType, Node, Selection};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Enum representing indentation direction
pub enum IndentDirection {
    /// Increase indentation level
    Increase,
    /// Decrease indentation level
    Decrease,
}

/// Command to indent or unindent selected content
pub struct SelectionIndentCommand {
    document: Rc<RefCell<Document>>,
    direction: IndentDirection,
    /// Store the original nodes for undo
    original_nodes: Vec<(usize, Node)>,
    /// Store the original selection
    original_selection: Option<Selection>,
}

impl SelectionIndentCommand {
    pub fn new(document: Rc<RefCell<Document>>, direction: IndentDirection) -> Self {
        Self {
            document,
            direction,
            original_nodes: Vec::new(),
            original_selection: None,
        }
    }

    // Helper function to check if two list types are compatible for merging
    fn list_types_compatible(type1: &ListType, type2: &ListType) -> bool {
        match (type1, type2) {
            (ListType::Ordered, ListType::Ordered) => true,
            (ListType::Unordered, ListType::Unordered) => true,
            (ListType::Task, ListType::Task) => true,
            _ => false,
        }
    }
}

impl Command for SelectionIndentCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Check if there's an active selection
        let selection = match document.selection.take() {
            Some(sel) => {
                // Store original selection for undo
                self.original_selection = Some(sel.clone());
                sel
            }
            None => return Ok(()),
        };

        // Get the affected range of nodes
        let start_node_idx = selection.start.path[0];
        let end_node_idx = selection.end.path[0];

        // Store affected nodes for undo
        for idx in start_node_idx..=end_node_idx {
            if idx < document.nodes.len() {
                self.original_nodes.push((idx, document.nodes[idx].clone()));
            }
        }

        // Modified implementation to avoid borrow issues
        match self.direction {
            IndentDirection::Increase => {
                // Handle increasing indentation
                handle_increase_indent(&mut document, start_node_idx, end_node_idx)?;
            }
            IndentDirection::Decrease => {
                // Handle decreasing indentation
                handle_decrease_indent(&mut document, start_node_idx, end_node_idx)?;
            }
        }

        // Restore selection
        document.selection = Some(selection);

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Restore original nodes
        for (idx, original_node) in self.original_nodes.drain(..) {
            if idx < document.nodes.len() {
                document.nodes[idx] = original_node;
            } else {
                // Handle edge case where node index may be out of bounds after modifications
                document.nodes.push(original_node);
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

// Helper method to handle increasing indentation - moved outside the impl to avoid borrow issues
fn handle_increase_indent(
    document: &mut Document,
    start_idx: usize,
    end_idx: usize,
) -> Result<(), EditError> {
    // Track nodes that need to be removed after processing
    let mut nodes_to_remove = Vec::new();

    // First pass: find lists that need to be merged and mark them
    for idx in start_idx..=end_idx {
        if idx >= document.nodes.len() || idx == 0 {
            continue;
        }

        let current_is_list = match &document.nodes[idx] {
            Node::List { .. } => true,
            _ => false,
        };

        let prev_is_list = match &document.nodes[idx - 1] {
            Node::List { .. } => true,
            _ => false,
        };

        if current_is_list && prev_is_list {
            if let (
                Node::List {
                    list_type: prev_type,
                    ..
                },
                Node::List {
                    list_type: curr_type,
                    ..
                },
            ) = (&document.nodes[idx - 1], &document.nodes[idx])
            {
                if SelectionIndentCommand::list_types_compatible(prev_type, curr_type) {
                    // Mark this node for merging
                    nodes_to_remove.push(idx);
                }
            }
        }
    }

    // Second pass: perform the merges
    for &idx in nodes_to_remove.iter() {
        if idx >= document.nodes.len() || idx == 0 {
            continue;
        }

        // Get the items from the current node
        let items_to_move = if let Node::List { items, .. } = &document.nodes[idx] {
            items.clone()
        } else {
            Vec::new()
        };

        // Add them to the previous node
        if let Node::List { items, .. } = &mut document.nodes[idx - 1] {
            items.extend(items_to_move);
        }

        // Mark the current node as empty for later removal
        document.nodes[idx] = Node::List {
            list_type: ListType::Unordered,
            items: Vec::new(),
        };
    }

    // Process other node types
    for idx in start_idx..=end_idx {
        if idx >= document.nodes.len() || nodes_to_remove.contains(&idx) {
            continue;
        }

        match &mut document.nodes[idx] {
            Node::BlockQuote { children } => {
                // Wrap current blockquote in another blockquote
                let inner_children = std::mem::take(children);
                children.push(Node::BlockQuote {
                    children: inner_children,
                });
            }
            Node::CodeBlock { code, .. } => {
                // Add 4 spaces or a tab to the beginning of each line
                *code = code
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
            }
            _ => {}
        }
    }

    // Clean up empty nodes
    let mut i = 0;
    while i < document.nodes.len() {
        match &document.nodes[i] {
            Node::List { items, .. } if items.is_empty() => {
                document.nodes.remove(i);
            }
            _ => i += 1,
        }
    }

    Ok(())
}

// Helper method to handle decreasing indentation - moved outside the impl to avoid borrow issues
fn handle_decrease_indent(
    document: &mut Document,
    start_idx: usize,
    end_idx: usize,
) -> Result<(), EditError> {
    for idx in start_idx..=end_idx {
        if idx >= document.nodes.len() {
            continue;
        }

        match &mut document.nodes[idx] {
            Node::BlockQuote { children } => {
                // If first child is a blockquote, unwrap it
                if !children.is_empty() {
                    if let Some(Node::BlockQuote {
                        children: inner_children,
                    }) = children.first()
                    {
                        // Clone to avoid borrow issues
                        let inner = inner_children.clone();
                        *children = inner;
                    }
                }
            }
            Node::CodeBlock { code, .. } => {
                // Remove up to 4 spaces or a tab from the beginning of each line
                *code = code
                    .lines()
                    .map(|line| {
                        if line.starts_with('\t') {
                            &line[1..]
                        } else if line.starts_with("    ") {
                            &line[4..]
                        } else if line.starts_with("   ") {
                            &line[3..]
                        } else if line.starts_with("  ") {
                            &line[2..]
                        } else if line.starts_with(' ') {
                            &line[1..]
                        } else {
                            line
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
            }
            _ => {}
        }
    }

    Ok(())
}
