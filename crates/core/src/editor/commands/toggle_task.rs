use crate::editor::command::Command;
use crate::{Document, EditError, ListType, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to toggle the checked status of a task list item
pub struct ToggleTaskCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    item_index: usize,
    previous_state: Option<bool>,
}

impl ToggleTaskCommand {
    /// Create a new command to toggle a task list item
    pub fn new(document: Rc<RefCell<Document>>, node_index: usize, item_index: usize) -> Self {
        Self {
            document,
            node_index,
            item_index,
            previous_state: None,
        }
    }
}

impl Command for ToggleTaskCommand {
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

                // Get the item and toggle its checked status
                let item = &mut items[self.item_index];

                // Store the previous state for undo
                self.previous_state = item.checked;

                // Toggle the checked status
                item.checked = match item.checked {
                    Some(checked) => Some(!checked),
                    None => Some(true), // If not set, default to checked
                };

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

                // Check if item_index is valid
                if self.item_index >= items.len() {
                    return Err(EditError::IndexOutOfBounds);
                }

                // Restore the previous state
                if let Some(previous_state) = self.previous_state {
                    items[self.item_index].checked = Some(previous_state);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Node;

    #[test]
    fn test_toggle_task() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a task list with two items
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_task_list(vec![("Task 1", false), ("Task 2", true)]);
        }

        // Toggle the first item (unchecked -> checked)
        let mut cmd1 = ToggleTaskCommand::new(doc_rc.clone(), 0, 0);
        assert!(cmd1.execute().is_ok());

        // Verify first item is now checked
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { list_type, items } => {
                    assert_eq!(*list_type, ListType::Task);
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0].checked, Some(true));
                    assert_eq!(items[1].checked, Some(true));
                }
                _ => panic!("Expected a list node"),
            }
        }

        // Toggle the second item (checked -> unchecked)
        let mut cmd2 = ToggleTaskCommand::new(doc_rc.clone(), 0, 1);
        assert!(cmd2.execute().is_ok());

        // Verify second item is now unchecked
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { list_type, items } => {
                    assert_eq!(*list_type, ListType::Task);
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0].checked, Some(true));
                    assert_eq!(items[1].checked, Some(false));
                }
                _ => panic!("Expected a list node"),
            }
        }

        // Undo the second command
        assert!(cmd2.undo().is_ok());

        // Verify second item is checked again
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { list_type, items } => {
                    assert_eq!(*list_type, ListType::Task);
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0].checked, Some(true));
                    assert_eq!(items[1].checked, Some(true));
                }
                _ => panic!("Expected a list node"),
            }
        }

        // Undo the first command
        assert!(cmd1.undo().is_ok());

        // Verify first item is unchecked again
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { list_type, items } => {
                    assert_eq!(*list_type, ListType::Task);
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0].checked, Some(false));
                    assert_eq!(items[1].checked, Some(true));
                }
                _ => panic!("Expected a list node"),
            }
        }
    }

    #[test]
    fn test_toggle_task_invalid_node() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a paragraph
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_paragraph_with_text("This is not a task list");
        }

        // Try to toggle a "task" in a paragraph
        let mut cmd = ToggleTaskCommand::new(doc_rc.clone(), 0, 0);
        assert!(matches!(
            cmd.execute(),
            Err(EditError::UnsupportedOperation)
        ));
    }

    #[test]
    fn test_toggle_task_invalid_index() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a task list with one item
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_task_list(vec![("Task 1", false)]);
        }

        // Try to toggle an out-of-bounds item
        let mut cmd = ToggleTaskCommand::new(doc_rc.clone(), 0, 1);
        assert!(matches!(cmd.execute(), Err(EditError::IndexOutOfBounds)));

        // Try to toggle an item in an out-of-bounds node
        let mut cmd = ToggleTaskCommand::new(doc_rc.clone(), 1, 0);
        assert!(matches!(cmd.execute(), Err(EditError::IndexOutOfBounds)));
    }
}
