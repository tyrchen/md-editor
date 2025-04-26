use crate::editor::command::Command;
use crate::{Document, EditError, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to delete a node from a document
pub struct DeleteNodeCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    deleted_node: Option<Node>,
}

impl DeleteNodeCommand {
    pub fn new(document: Rc<RefCell<Document>>, node_index: usize) -> Self {
        Self {
            document,
            node_index,
            deleted_node: None,
        }
    }
}

impl Command for DeleteNodeCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Store the node for undo
        self.deleted_node = Some(document.nodes[self.node_index].clone());

        // Remove the node
        document.nodes.remove(self.node_index);

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        match &self.deleted_node {
            Some(node) => {
                if self.node_index > document.nodes.len() {
                    // If we're trying to restore at the end
                    document.nodes.push(node.clone());
                } else {
                    // Insert at the original position
                    document.nodes.insert(self.node_index, node.clone());
                }
                Ok(())
            }
            None => Err(EditError::OperationFailed),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
