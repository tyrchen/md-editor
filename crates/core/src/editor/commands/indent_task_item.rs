use crate::editor::command::Command;
use crate::error::EditError;
use crate::models::{Document, ListItem, ListType, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// The direction in which to indent a task item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentDirection {
    /// Indent the task item (increase nesting level)
    Increase,
    /// Dedent the task item (decrease nesting level)
    Decrease,
}

/// Command for indenting/dedenting task items in a task list
#[derive(Debug)]
pub struct IndentTaskItemCommand {
    /// Document to modify
    document: Rc<RefCell<Document>>,
    /// Index of the task list node
    node_idx: usize,
    /// Index of the task item to indent/dedent
    item_idx: usize,
    /// The direction of indentation (increase or decrease)
    direction: IndentDirection,
    /// Stores the original items for undo
    original_items: Option<Vec<ListItem>>,
}

impl IndentTaskItemCommand {
    /// Create a new command to indent or dedent a task item
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_idx: usize,
        item_idx: usize,
        direction: IndentDirection,
    ) -> Self {
        Self {
            document,
            node_idx,
            item_idx,
            direction,
            original_items: None,
        }
    }

    /// Create a command to increase the indent of a task item
    pub fn increase_indent(
        document: Rc<RefCell<Document>>,
        node_idx: usize,
        item_idx: usize,
    ) -> Self {
        Self::new(document, node_idx, item_idx, IndentDirection::Increase)
    }

    /// Create a command to decrease the indent of a task item
    pub fn decrease_indent(
        document: Rc<RefCell<Document>>,
        node_idx: usize,
        item_idx: usize,
    ) -> Self {
        Self::new(document, node_idx, item_idx, IndentDirection::Decrease)
    }
}

impl Command for IndentTaskItemCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.node_idx >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Verify that we have a task list
        let node = &document.nodes[self.node_idx];
        let items = match node {
            Node::List {
                list_type, items, ..
            } => {
                if *list_type != ListType::Task {
                    return Err(EditError::Other("Node is not a task list".into()));
                }
                items
            }
            _ => return Err(EditError::Other("Node is not a list".into())),
        };

        // Check if item index is valid
        if self.item_idx >= items.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Store original items for undo
        self.original_items = Some(items.clone());

        // Clone the document to get a mutable reference we can modify
        let mut result = document.clone();

        // Get mutable reference to the items in the cloned document
        let items = match &mut result.nodes[self.node_idx] {
            Node::List { items, .. } => items,
            _ => unreachable!(), // We already verified it's a list
        };

        // Perform the operation based on direction
        match self.direction {
            IndentDirection::Increase => {
                if self.item_idx == 0 {
                    return Err(EditError::Other("Cannot indent the first item".into()));
                }

                // First, we need to clone the item we want to move
                let current_item = items[self.item_idx].clone();

                // Now we can safely remove it because we have a clone
                items.remove(self.item_idx);

                // Get a mutable reference to the previous item
                let previous_item = &mut items[self.item_idx - 1];

                // Check if previous item already has a task list
                let has_task_list = previous_item.children.iter().any(
                    |node| matches!(node, Node::List { list_type, .. } if *list_type == ListType::Task),
                );

                if !has_task_list {
                    // Create new nested task list in previous item
                    let nested_list = Node::List {
                        list_type: ListType::Task,
                        items: vec![current_item],
                    };
                    previous_item.children.push(nested_list);
                } else {
                    // Find existing task list and add item to it
                    for child in &mut previous_item.children {
                        if let Node::List {
                            list_type,
                            items: nested_items,
                            ..
                        } = child
                        {
                            if *list_type == ListType::Task {
                                nested_items.push(current_item);
                                break;
                            }
                        }
                    }
                }
            }
            IndentDirection::Decrease => {
                // Create a new list of items that we'll modify and then replace the original with
                let mut found = false;

                // First, search for a nested task list and the item to dedent
                let mut new_items = Vec::new();
                let mut item_to_dedent = None;
                let mut insertion_point = 0;

                for (idx, item) in items.iter().enumerate() {
                    // Clone the current item for our new list
                    new_items.push(item.clone());

                    // Remember this as our current position for insertion
                    insertion_point = idx + 1;

                    // Search for a nested task list in this item
                    for (child_idx, child) in item.children.iter().enumerate() {
                        if let Node::List {
                            list_type,
                            items: nested_items,
                        } = child
                        {
                            if *list_type == ListType::Task && self.item_idx < nested_items.len() {
                                // Found the nested item to dedent
                                found = true;

                                // Store the item to dedent
                                item_to_dedent = Some(nested_items[self.item_idx].clone());

                                // Update the parent item's children by rebuilding without the dedented item
                                let mut updated_parent = new_items.pop().unwrap();
                                let mut updated_children = Vec::new();

                                for (i, child_node) in item.children.iter().enumerate() {
                                    if i == child_idx {
                                        // Create an updated version of the nested list
                                        let mut remaining_items = nested_items.clone();
                                        remaining_items.remove(self.item_idx);

                                        // Only keep the list if it's not empty
                                        if !remaining_items.is_empty() {
                                            let updated_list = Node::List {
                                                list_type: ListType::Task,
                                                items: remaining_items,
                                            };
                                            updated_children.push(updated_list);
                                        }
                                    } else {
                                        // Keep any other children
                                        updated_children.push(child_node.clone());
                                    }
                                }

                                updated_parent.children = updated_children;
                                new_items.push(updated_parent);
                                break;
                            }
                        }
                    }

                    if found {
                        break;
                    }
                }

                if !found {
                    return Err(EditError::Other("Item not found in any nested list".into()));
                }

                // Insert the dedented item at the insertion point
                if let Some(item) = item_to_dedent {
                    new_items.insert(insertion_point, item);
                }

                // Add any remaining items from the original list
                new_items.extend(items.iter().skip(insertion_point).cloned());

                // Replace the original items with our new list
                *items = new_items;
            }
        }

        // Update the document
        *document = result;

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(original_items) = &self.original_items {
            let mut document = self.document.borrow_mut();

            if self.node_idx >= document.nodes.len() {
                return Err(EditError::IndexOutOfBounds);
            }

            match &mut document.nodes[self.node_idx] {
                Node::List { items, .. } => {
                    // Restore original items
                    *items = original_items.clone();
                    Ok(())
                }
                _ => Err(EditError::Other("Node is not a list".into())),
            }
        } else {
            Err(EditError::Other("No original items to restore".into()))
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
    fn test_increase_indent() {
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
        let mut cmd = IndentTaskItemCommand::increase_indent(doc_rc.clone(), 0, 2);

        // Execute the command to indent the third task
        assert!(cmd.execute().is_ok());

        // Verify the task was indented (became a child of Task 2)
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 2); // Now only 2 top-level items

                    // Check first item
                    let first_text = items[0].as_text().unwrap();
                    assert_eq!(first_text, "Task 1");

                    // Check second item
                    let second_text = items[1].as_text().unwrap();
                    assert_eq!(second_text, "Task 2");

                    // Check for nested list under Task 2
                    let task2 = &items[1];
                    let nested_list = task2.children.iter().find(|node| {
                        matches!(node, Node::List { list_type, .. } if *list_type == ListType::Task)
                    });

                    assert!(
                        nested_list.is_some(),
                        "Task 2 should have a nested task list"
                    );

                    if let Some(Node::List {
                        items: nested_items,
                        ..
                    }) = nested_list
                    {
                        assert_eq!(nested_items.len(), 1);
                        assert_eq!(nested_items[0].as_text().unwrap(), "Task 3");
                    } else {
                        panic!("Expected nested task list");
                    }
                }
                _ => panic!("Expected list node"),
            }
        }

        // Test undo
        assert!(cmd.undo().is_ok());

        // Verify the original structure is restored
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
    fn test_indent_first_item() {
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
        let mut cmd = IndentTaskItemCommand::increase_indent(doc_rc.clone(), 0, 0);

        // Try to indent the first task (should fail)
        let result = cmd.execute();
        assert!(result.is_err());

        if let Err(EditError::Other(msg)) = result {
            assert_eq!(msg, "Cannot indent the first item");
        } else {
            panic!("Expected EditError::Other");
        }
    }

    #[test]
    fn test_decrease_indent() {
        let mut doc = Document::new();

        // Add a task list with three items, where the second item has a nested task item
        let mut items = vec![
            ListItem::task("Task 1", false),
            ListItem::task("Task 2", true),
            ListItem::task("Task 3", false),
        ];

        // Add a nested task list under Task 2
        let nested_task = Node::List {
            list_type: ListType::Task,
            items: vec![ListItem::task("Nested Task", false)],
        };
        items[1].children.push(nested_task);

        doc.nodes.push(Node::List {
            list_type: ListType::Task,
            items,
        });

        let doc_rc = Rc::new(RefCell::new(doc));
        let mut cmd = IndentTaskItemCommand::decrease_indent(doc_rc.clone(), 0, 0);

        // Execute the command to dedent the nested task
        assert!(cmd.execute().is_ok());

        // Verify the nested task was moved to the top level
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 4); // Now 4 top-level items

                    // Check all items
                    assert_eq!(items[0].as_text().unwrap(), "Task 1");
                    assert_eq!(items[1].as_text().unwrap(), "Task 2");
                    assert_eq!(items[2].as_text().unwrap(), "Nested Task"); // Dedented item now at position 2
                    assert_eq!(items[3].as_text().unwrap(), "Task 3");

                    // Check that Task 2 no longer has a nested task list
                    let task2 = &items[1];
                    let has_nested_list = task2.children.iter().any(|node| {
                        matches!(node, Node::List { list_type, .. } if *list_type == ListType::Task)
                    });
                    assert!(
                        !has_nested_list,
                        "Task 2 should no longer have a nested task list"
                    );
                }
                _ => panic!("Expected list node"),
            }
        }

        // Test undo
        assert!(cmd.undo().is_ok());

        // Verify the original structure is restored
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    assert_eq!(items.len(), 3); // Back to 3 top-level items

                    // Check that Task 2 has a nested task list again
                    let task2 = &items[1];
                    let has_nested_list = task2.children.iter().any(|node| {
                        matches!(node, Node::List { list_type, .. } if *list_type == ListType::Task)
                    });
                    assert!(
                        has_nested_list,
                        "Task 2 should have a nested task list again"
                    );

                    // Check the nested task
                    let nested_list = task2.children.iter().find(|node| {
                        matches!(node, Node::List { list_type, .. } if *list_type == ListType::Task)
                    });

                    if let Some(Node::List {
                        items: nested_items,
                        ..
                    }) = nested_list
                    {
                        assert_eq!(nested_items.len(), 1);
                        assert_eq!(nested_items[0].as_text().unwrap(), "Nested Task");
                    } else {
                        panic!("Expected nested task list");
                    }
                }
                _ => panic!("Expected list node"),
            }
        }
    }

    #[test]
    fn test_dedent_errors() {
        let mut doc = Document::new();

        // Add a task list with three items (no nesting)
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
        let mut cmd = IndentTaskItemCommand::decrease_indent(doc_rc.clone(), 0, 0);

        // Try to dedent from a list with no nested items (should fail)
        let result = cmd.execute();
        assert!(result.is_err());

        if let Err(EditError::Other(msg)) = result {
            assert_eq!(msg, "Item not found in any nested list");
        } else {
            panic!("Expected EditError::Other");
        }
    }
}
