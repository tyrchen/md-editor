use crate::editor::command::Command;
use crate::{Document, EditError};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to move a node from one position to another in a document
pub struct MoveNodeCommand {
    document: Rc<RefCell<Document>>,
    from_index: usize,
    to_index: usize,
}

impl MoveNodeCommand {
    pub fn new(document: Rc<RefCell<Document>>, from_index: usize, to_index: usize) -> Self {
        Self {
            document,
            from_index,
            to_index,
        }
    }
}

impl Command for MoveNodeCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.from_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        if self.to_index > document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // If source and destination are the same, no need to move
        if self.from_index == self.to_index {
            return Ok(());
        }

        // Remove the node from its original position
        let node = document.nodes.remove(self.from_index);

        // Adjust the insertion index if it was after the removal
        let adjusted_to_index = if self.to_index > self.from_index {
            self.to_index - 1
        } else {
            self.to_index
        };

        // Insert the node at the new position
        document.nodes.insert(adjusted_to_index, node);

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // For undo, we need to move the node back from to_index to from_index
        // Calculate the current position of the node that was moved
        let current_pos = if self.to_index > self.from_index {
            self.to_index - 1
        } else {
            self.to_index
        };

        if current_pos >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Remove the node from its current position
        let node = document.nodes.remove(current_pos);

        // Insert it back at the original position
        document.nodes.insert(self.from_index, node);

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
