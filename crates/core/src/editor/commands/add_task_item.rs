use crate::editor::command::Command;
use crate::{Document, EditError, ListType, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to add a new item to a task list
pub struct AddTaskItemCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    position: usize,
    text: String,
    checked: bool,
    added_index: Option<usize>,
}

impl AddTaskItemCommand {
    /// Create a new command to add an item to a task list
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_index: usize,
        position: usize,
        text: impl Into<String>,
        checked: bool,
    ) -> Self {
        Self {
            document,
            node_index,
            position,
            text: text.into(),
            checked,
            added_index: None,
        }
    }
}

impl Command for AddTaskItemCommand {
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

                // Ensure position is valid (allowing appending at the end)
                if self.position > items.len() {
                    return Err(EditError::IndexOutOfBounds);
                }

                // Create a new list item with the text and checked state
                let item = crate::ListItem::task(&self.text, self.checked);

                // Insert the item at the specified position
                items.insert(self.position, item);

                // Store the added index for undo
                self.added_index = Some(self.position);

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

                // Remove the item if we know its position
                if let Some(position) = self.added_index {
                    if position < items.len() {
                        items.remove(position);
                        Ok(())
                    } else {
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
    fn test_add_task_item() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a task list with one item
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_task_list(vec![("Existing task", false)]);
        }

        // Add a new task item
        let mut cmd = AddTaskItemCommand::new(doc_rc.clone(), 0, 1, "New task", true);
        assert!(cmd.execute().is_ok());

        // Verify the task item was added
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { list_type, items } => {
                    assert_eq!(*list_type, ListType::Task);
                    assert_eq!(items.len(), 2);

                    // Check the existing item
                    assert_eq!(items[0].checked, Some(false));

                    // Check the new item
                    assert_eq!(items[1].checked, Some(true));

                    // Verify the text content
                    let paragraph = &items[1].children[0];
                    if let Node::Paragraph { children } = paragraph {
                        if let crate::InlineNode::Text(text_node) = &children[0] {
                            assert_eq!(text_node.text, "New task");
                        } else {
                            panic!("Expected text node");
                        }
                    } else {
                        panic!("Expected paragraph node");
                    }
                }
                _ => panic!("Expected list node"),
            }
        }

        // Test undo
        assert!(cmd.undo().is_ok());

        // Verify the item was removed
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 1);
                }
                _ => panic!("Expected list node"),
            }
        }
    }

    #[test]
    fn test_add_task_item_invalid_node() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a paragraph instead of a task list
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_paragraph_with_text("This is not a task list");
        }

        // Try to add a task item to a paragraph
        let mut cmd = AddTaskItemCommand::new(doc_rc.clone(), 0, 0, "New task", false);
        assert!(matches!(
            cmd.execute(),
            Err(EditError::UnsupportedOperation)
        ));
    }

    #[test]
    fn test_add_task_item_invalid_position() {
        let doc = Document::new();
        let doc_rc = Rc::new(RefCell::new(doc));

        // Add a task list with one item
        {
            let mut doc = doc_rc.borrow_mut();
            doc.add_task_list(vec![("Existing task", false)]);
        }

        // Try to add at an invalid position
        let mut cmd = AddTaskItemCommand::new(doc_rc.clone(), 0, 2, "New task", false);
        assert!(matches!(cmd.execute(), Err(EditError::IndexOutOfBounds)));
    }
}
