use crate::editor::command::Command;
use crate::error::EditError;
use crate::models::{Document, ListItem, ListType, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command for moving a task list item up or down in a task list.
#[derive(Debug)]
pub struct MoveTaskItemCommand {
    /// Document to modify
    document: Rc<RefCell<Document>>,
    /// Index of the task list node
    node_idx: usize,
    /// Index of the task item to move
    item_idx: usize,
    /// Direction to move: true = down, false = up
    move_down: bool,
    /// Previous items state (for undo)
    previous_items: Option<Vec<ListItem>>,
}

/// Command for moving a task list item to a specific position
#[derive(Debug)]
pub struct MoveTaskPositionCommand {
    /// Document to modify
    document: Rc<RefCell<Document>>,
    /// Index of the task list node
    node_idx: usize,
    /// Original index of the task item
    from_idx: usize,
    /// Target index for the task item
    to_idx: usize,
    /// Previous items state (for undo)
    previous_items: Option<Vec<ListItem>>,
}

impl MoveTaskItemCommand {
    /// Creates a new command to move a task list item up or down.
    ///
    /// # Arguments
    ///
    /// * `document` - Reference to the document to modify
    /// * `node_idx` - Index of the task list node in the document
    /// * `item_idx` - Index of the task item within the list
    /// * `move_down` - Direction to move: true = down, false = up
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_idx: usize,
        item_idx: usize,
        move_down: bool,
    ) -> Self {
        Self {
            document,
            node_idx,
            item_idx,
            move_down,
            previous_items: None,
        }
    }

    /// Creates a command to move a task item down
    pub fn move_down(document: Rc<RefCell<Document>>, node_idx: usize, item_idx: usize) -> Self {
        Self::new(document, node_idx, item_idx, true)
    }

    /// Creates a command to move a task item up
    pub fn move_up(document: Rc<RefCell<Document>>, node_idx: usize, item_idx: usize) -> Self {
        Self::new(document, node_idx, item_idx, false)
    }
}

impl MoveTaskPositionCommand {
    /// Creates a new command to move a task list item to a specific position.
    ///
    /// # Arguments
    ///
    /// * `document` - Reference to the document to modify
    /// * `node_idx` - Index of the task list node in the document
    /// * `from_idx` - Current index of the task item
    /// * `to_idx` - Target index for the item
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_idx: usize,
        from_idx: usize,
        to_idx: usize,
    ) -> Self {
        Self {
            document,
            node_idx,
            from_idx,
            to_idx,
            previous_items: None,
        }
    }
}

impl Command for MoveTaskItemCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.node_idx >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        match &mut document.nodes[self.node_idx] {
            Node::List { list_type, items } => {
                // Verify that it's a task list
                if *list_type != ListType::Task {
                    return Err(EditError::Other("Node is not a task list".into()));
                }

                // Check if the item index is valid
                if self.item_idx >= items.len() {
                    return Err(EditError::IndexOutOfBounds);
                }

                // Store previous items for undo
                self.previous_items = Some(items.clone());

                // Calculate new position
                let new_idx = if self.move_down {
                    // Moving down
                    if self.item_idx + 1 >= items.len() {
                        return Err(EditError::Other("Already at the bottom".into()));
                    }
                    self.item_idx + 1
                } else {
                    // Moving up
                    if self.item_idx == 0 {
                        return Err(EditError::Other("Already at the top".into()));
                    }
                    self.item_idx - 1
                };

                // Swap the items
                items.swap(self.item_idx, new_idx);

                Ok(())
            }
            _ => Err(EditError::Other("Node is not a list".into())),
        }
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(previous_items) = &self.previous_items {
            let mut document = self.document.borrow_mut();

            if self.node_idx >= document.nodes.len() {
                return Err(EditError::IndexOutOfBounds);
            }

            match &mut document.nodes[self.node_idx] {
                Node::List { items, .. } => {
                    // Restore previous items
                    *items = previous_items.clone();
                    Ok(())
                }
                _ => Err(EditError::Other("Node is not a list".into())),
            }
        } else {
            Err(EditError::Other(
                "Previous state not available for undo".into(),
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Command for MoveTaskPositionCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.node_idx >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        match &mut document.nodes[self.node_idx] {
            Node::List { list_type, items } => {
                // Verify that it's a task list
                if *list_type != ListType::Task {
                    return Err(EditError::Other("Node is not a task list".into()));
                }

                // Check if indices are valid
                if self.from_idx >= items.len() || self.to_idx >= items.len() {
                    return Err(EditError::IndexOutOfBounds);
                }

                // No-op if from and to are the same
                if self.from_idx == self.to_idx {
                    return Ok(());
                }

                // Store previous items for undo
                self.previous_items = Some(items.clone());

                // Get the item to move
                let item = items.remove(self.from_idx);

                // Insert it at the target position
                items.insert(self.to_idx, item);

                Ok(())
            }
            _ => Err(EditError::Other("Node is not a list".into())),
        }
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(previous_items) = &self.previous_items {
            let mut document = self.document.borrow_mut();

            if self.node_idx >= document.nodes.len() {
                return Err(EditError::IndexOutOfBounds);
            }

            match &mut document.nodes[self.node_idx] {
                Node::List { items, .. } => {
                    // Restore previous items
                    *items = previous_items.clone();
                    Ok(())
                }
                _ => Err(EditError::Other("Node is not a list".into())),
            }
        } else {
            Err(EditError::Other(
                "Previous state not available for undo".into(),
            ))
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
    fn test_move_task_item_down() {
        let mut doc = Document::new();

        // Add a task list with three items
        let items = vec![
            ListItem::task("Task 1", false),
            ListItem::task("Task 2", true),
            ListItem::task("Task 3", false),
        ];

        doc.nodes.push(Node::List {
            list_type: ListType::Task,
            items: items.clone(),
        });

        let doc_rc = Rc::new(RefCell::new(doc));
        let mut cmd = MoveTaskItemCommand::move_down(doc_rc.clone(), 0, 0);

        // Execute the command to move the first item down
        assert!(cmd.execute().is_ok());

        // Verify the item moved down
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[0].as_text().unwrap(), "Task 2");
                    assert_eq!(items[1].as_text().unwrap(), "Task 1");
                    assert_eq!(items[2].as_text().unwrap(), "Task 3");
                }
                _ => panic!("Expected list node"),
            }
        }

        // Test undo
        assert!(cmd.undo().is_ok());

        // Verify the original order is restored
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[0].as_text().unwrap(), "Task 1");
                    assert_eq!(items[1].as_text().unwrap(), "Task 2");
                    assert_eq!(items[2].as_text().unwrap(), "Task 3");
                }
                _ => panic!("Expected list node"),
            }
        }
    }

    #[test]
    fn test_move_task_position() {
        let mut doc = Document::new();

        // Add a task list with three items
        let items = vec![
            ListItem::task("Task 1", false),
            ListItem::task("Task 2", true),
            ListItem::task("Task 3", false),
        ];

        doc.nodes.push(Node::List {
            list_type: ListType::Task,
            items: items.clone(),
        });

        let doc_rc = Rc::new(RefCell::new(doc));
        let mut cmd = MoveTaskPositionCommand::new(doc_rc.clone(), 0, 0, 2);

        // Execute the command to move the first item to the end
        assert!(cmd.execute().is_ok());

        // Verify the item moved to the end
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[0].as_text().unwrap(), "Task 2");
                    assert_eq!(items[1].as_text().unwrap(), "Task 3");
                    assert_eq!(items[2].as_text().unwrap(), "Task 1");
                }
                _ => panic!("Expected list node"),
            }
        }

        // Test undo
        assert!(cmd.undo().is_ok());

        // Verify the original order is restored
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[0].as_text().unwrap(), "Task 1");
                    assert_eq!(items[1].as_text().unwrap(), "Task 2");
                    assert_eq!(items[2].as_text().unwrap(), "Task 3");
                }
                _ => panic!("Expected list node"),
            }
        }
    }

    #[test]
    fn test_move_task_item_up() {
        let mut doc = Document::new();

        // Add a task list with three items
        let items = vec![
            ListItem::task("Task 1", false),
            ListItem::task("Task 2", true),
            ListItem::task("Task 3", false),
        ];

        doc.nodes.push(Node::List {
            list_type: ListType::Task,
            items: items.clone(),
        });

        let doc_rc = Rc::new(RefCell::new(doc));
        let mut cmd = MoveTaskItemCommand::move_up(doc_rc.clone(), 0, 2);

        // Execute the command to move the last item up
        assert!(cmd.execute().is_ok());

        // Verify the item moved up
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[0].as_text().unwrap(), "Task 1");
                    assert_eq!(items[1].as_text().unwrap(), "Task 3");
                    assert_eq!(items[2].as_text().unwrap(), "Task 2");
                }
                _ => panic!("Expected list node"),
            }
        }

        // Test undo
        assert!(cmd.undo().is_ok());

        // Verify the original order is restored
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[0].as_text().unwrap(), "Task 1");
                    assert_eq!(items[1].as_text().unwrap(), "Task 2");
                    assert_eq!(items[2].as_text().unwrap(), "Task 3");
                }
                _ => panic!("Expected list node"),
            }
        }
    }

    #[test]
    fn test_move_task_item_boundaries() {
        let mut doc = Document::new();

        // Add a task list with two items
        let items = vec![
            ListItem::task("Task 1", false),
            ListItem::task("Task 2", true),
        ];

        doc.nodes.push(Node::List {
            list_type: ListType::Task,
            items: items.clone(),
        });

        let doc_rc = Rc::new(RefCell::new(doc));

        // Try to move first item up (should fail)
        let mut cmd_up = MoveTaskItemCommand::move_up(doc_rc.clone(), 0, 0);
        let result = cmd_up.execute();
        assert!(result.is_err());
        if let Err(EditError::Other(msg)) = result {
            assert_eq!(msg, "Already at the top");
        } else {
            panic!("Expected EditError::Other");
        }

        // Try to move last item down (should fail)
        let mut cmd_down = MoveTaskItemCommand::move_down(doc_rc.clone(), 0, 1);
        let result = cmd_down.execute();
        assert!(result.is_err());
        if let Err(EditError::Other(msg)) = result {
            assert_eq!(msg, "Already at the bottom");
        } else {
            panic!("Expected EditError::Other");
        }
    }
}
