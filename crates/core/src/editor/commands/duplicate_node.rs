use crate::editor::command::Command;
use crate::{Document, EditError, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to duplicate a node in the document
pub struct DuplicateNodeCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    // Store the new node index for undo
    new_node_index: Option<usize>,
    // Store node type for validation
    node_type: Option<String>,
}

impl DuplicateNodeCommand {
    pub fn new(document: Rc<RefCell<Document>>, node_index: usize) -> Self {
        Self {
            document,
            node_index,
            new_node_index: None,
            node_type: None,
        }
    }

    // Helper method to get a descriptive name for a node type
    fn get_node_type(node: &Node) -> String {
        match node {
            Node::Paragraph { .. } => "Paragraph".to_string(),
            Node::Heading { level, .. } => format!("Heading {}", level),
            Node::List { list_type, .. } => format!("List ({:?})", list_type),
            Node::CodeBlock { language, .. } => format!("CodeBlock ({})", language),
            Node::BlockQuote { .. } => "BlockQuote".to_string(),
            Node::ThematicBreak => "ThematicBreak".to_string(),
            Node::Table { .. } => "Table".to_string(),
            Node::FootnoteReference(_) => "FootnoteReference".to_string(),
            Node::FootnoteDefinition(_) => "FootnoteDefinition".to_string(),
            Node::DefinitionList { .. } => "DefinitionList".to_string(),
            Node::MathBlock { .. } => "MathBlock".to_string(),
            Node::TempListItem(_) => "TemporaryListItem".to_string(),
            Node::TempTableCell(_) => "TemporaryTableCell".to_string(),
        }
    }
}

impl Command for DuplicateNodeCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Empty document check
        if document.nodes.is_empty() {
            return Err(EditError::IndexOutOfBounds);
        }

        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Store the node type for debugging/tracing
        self.node_type = Some(Self::get_node_type(&document.nodes[self.node_index]));

        // Check if trying to duplicate a temporary node
        match &document.nodes[self.node_index] {
            Node::TempListItem(_) | Node::TempTableCell(_) => {
                return Err(EditError::UnsupportedOperation);
            }
            _ => {}
        }

        // Clone the source node
        let node_to_duplicate = document.nodes[self.node_index].clone();

        // Insert the duplicate right after the original
        let insertion_index = self.node_index + 1;
        document.nodes.insert(insertion_index, node_to_duplicate);

        // Store the new index for undo
        self.new_node_index = Some(insertion_index);

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(index) = self.new_node_index {
            let mut document = self.document.borrow_mut();

            if index >= document.nodes.len() {
                return Err(EditError::IndexOutOfBounds);
            }

            // Remove the duplicated node
            document.nodes.remove(index);

            // Clear the stored index
            self.new_node_index = None;

            Ok(())
        } else {
            Err(EditError::OperationFailed)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
