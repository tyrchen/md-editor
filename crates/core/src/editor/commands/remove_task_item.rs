use crate::editor::command::Command;
use crate::{Document, EditError, ListItem, ListType, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to remove an item from a task list
pub struct RemoveTaskItemCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    item_index: usize,
    removed_item: Option<ListItem>,
}

impl RemoveTaskItemCommand {
    /// Create a new command to remove an item from a task list
    pub fn new(document: Rc<RefCell<Document>>, node_index: usize, item_index: usize) -> Self {
        Self {
            document,
            node_index,
            item_index,
            removed_item: None,
        }
    }
}

impl Command for RemoveTaskItemCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Check if node_index is valid
        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Get the node
        match &mut document.nodes[self.node_index] {
            Node::List { list_type, items } => {
                // Check if it's a task list
                if *list_type != ListType::Task {
                    return Err(EditError::UnsupportedOperation);
                }

                // Check if item_index is valid
                if self.item_index >= items.len() {
                    return Err(EditError::IndexOutOfBounds);
                }

                // Prevent removing the last item (would leave an empty list)
                if items.len() == 1 {
                    return Err(EditError::UnsupportedOperation);
                }

                // Remove the item and store it for undo
                self.removed_item = Some(items.remove(self.item_index));

                Ok(())
            }
            _ => Err(EditError::UnsupportedOperation),
        }
    }

    fn undo(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Check if node_index is valid
        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Get the node
        match &mut document.nodes[self.node_index] {
            Node::List { list_type, items } => {
                // Check if it's a task list
                if *list_type != ListType::Task {
                    return Err(EditError::UnsupportedOperation);
                }

                // Ensure we have a stored item to put back
                if let Some(item) = self.removed_item.take() {
                    // Make sure the index is valid (or at the end)
                    if self.item_index <= items.len() {
                        items.insert(self.item_index, item);
                        Ok(())
                    } else {
                        self.removed_item = Some(item); // Put it back for future undo attempts
                        Err(EditError::IndexOutOfBounds)
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_task_item() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a task list with two items
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_task_list(vec![("Task 1", false), ("Task 2", true), ("Task 3", false)]);
        }

        // Remove the middle task
        let mut cmd = RemoveTaskItemCommand::new(doc_rc.clone(), 0, 1);
        assert!(cmd.execute().is_ok());

        // Verify the task item was removed
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0].checked, Some(false)); // Task 1
                    assert_eq!(items[1].checked, Some(false)); // Task 3
                }
                _ => panic!("Expected list node"),
            }
        }

        // Test undo
        assert!(cmd.undo().is_ok());

        // Verify the item was restored
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[0].checked, Some(false)); // Task 1
                    assert_eq!(items[1].checked, Some(true)); // Task 2
                    assert_eq!(items[2].checked, Some(false)); // Task 3
                }
                _ => panic!("Expected list node"),
            }
        }
    }

    #[test]
    fn test_remove_task_item_last_remaining() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a task list with one item
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_task_list(vec![("Only task", false)]);
        }

        // Try to remove the only item
        let mut cmd = RemoveTaskItemCommand::new(doc_rc.clone(), 0, 0);
        assert!(matches!(
            cmd.execute(),
            Err(EditError::UnsupportedOperation)
        ));
    }

    #[test]
    fn test_remove_task_item_invalid_node() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a paragraph instead of a task list
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_paragraph_with_text("This is not a task list");
        }

        // Try to remove from a paragraph
        let mut cmd = RemoveTaskItemCommand::new(doc_rc.clone(), 0, 0);
        assert!(matches!(
            cmd.execute(),
            Err(EditError::UnsupportedOperation)
        ));
    }

    #[test]
    fn test_remove_task_item_invalid_index() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a task list with two items
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_task_list(vec![("Task 1", false), ("Task 2", true)]);
        }

        // Try to remove with an invalid index
        let mut cmd = RemoveTaskItemCommand::new(doc_rc.clone(), 0, 2);
        assert!(matches!(cmd.execute(), Err(EditError::IndexOutOfBounds)));
    }
}
