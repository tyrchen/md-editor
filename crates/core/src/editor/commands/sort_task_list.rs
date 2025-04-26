use crate::{
    editor::command::Command,
    error::EditError,
    models::{Document, ListItem, ListType, Node},
};
use std::any::Any;

/// Criteria for sorting task list items
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortCriteria {
    /// Sort alphabetically by task content
    Alphabetical,
    /// Sort with unchecked items first, then alphabetically
    UncheckedFirst,
    /// Sort with checked items first, then alphabetically
    CheckedFirst,
}

/// Command to sort task list items based on different criteria
#[derive(Debug)]
pub struct SortTaskListCommand {
    /// The document to operate on
    document: Document,
    /// Index of the task list node to sort
    node_idx: usize,
    /// Criteria for sorting
    criteria: SortCriteria,
    /// Original items for undo operation
    original_items: Option<Vec<ListItem>>,
}

impl SortTaskListCommand {
    /// Create a new sort task list command
    pub fn new(document: Document, node_idx: usize, criteria: SortCriteria) -> Self {
        Self {
            document,
            node_idx,
            criteria,
            original_items: None,
        }
    }
}

impl Command for SortTaskListCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        // Check if node_idx is valid
        if self.node_idx >= self.document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Check if the node is a task list
        let node = &self.document.nodes[self.node_idx];
        let items = match node {
            Node::List {
                list_type, items, ..
            } => {
                if *list_type != ListType::Task {
                    return Err(EditError::Other("Node is not a task list".to_string()));
                }
                items
            }
            _ => return Err(EditError::Other("Node is not a list".to_string())),
        };

        // Store the original items for undo
        self.original_items = Some(items.clone());

        // Clone the document for modification
        let mut result = self.document.clone();

        // Get mutable reference to the list node in the result document
        if let Node::List { items, .. } = &mut result.nodes[self.node_idx] {
            // Sort items based on criteria
            match self.criteria {
                SortCriteria::Alphabetical => {
                    items.sort_by(|a, b| {
                        let a_text = a.as_text().unwrap_or("").to_lowercase();
                        let b_text = b.as_text().unwrap_or("").to_lowercase();
                        a_text.cmp(&b_text)
                    });
                }
                SortCriteria::UncheckedFirst => {
                    items.sort_by(|a, b| {
                        match (a.checked, b.checked) {
                            // If both have same checked status, sort alphabetically
                            (Some(true), Some(true))
                            | (Some(false), Some(false))
                            | (None, None) => {
                                let a_text = a.as_text().unwrap_or("").to_lowercase();
                                let b_text = b.as_text().unwrap_or("").to_lowercase();
                                a_text.cmp(&b_text)
                            }
                            // Unchecked comes before checked
                            (Some(false), Some(true)) | (None, Some(true)) => {
                                std::cmp::Ordering::Less
                            }
                            (Some(true), Some(false)) | (Some(true), None) => {
                                std::cmp::Ordering::Greater
                            }
                            // Handle remaining cases
                            (None, Some(false)) => std::cmp::Ordering::Less,
                            (Some(false), None) => std::cmp::Ordering::Greater,
                        }
                    });
                }
                SortCriteria::CheckedFirst => {
                    items.sort_by(|a, b| {
                        match (a.checked, b.checked) {
                            // If both have same checked status, sort alphabetically
                            (Some(true), Some(true))
                            | (Some(false), Some(false))
                            | (None, None) => {
                                let a_text = a.as_text().unwrap_or("").to_lowercase();
                                let b_text = b.as_text().unwrap_or("").to_lowercase();
                                a_text.cmp(&b_text)
                            }
                            // Checked comes before unchecked
                            (Some(true), Some(false)) | (Some(true), None) => {
                                std::cmp::Ordering::Less
                            }
                            (Some(false), Some(true)) | (None, Some(true)) => {
                                std::cmp::Ordering::Greater
                            }
                            // Handle remaining cases
                            (Some(false), None) => std::cmp::Ordering::Less,
                            (None, Some(false)) => std::cmp::Ordering::Greater,
                        }
                    });
                }
            }
        }

        // Update the document
        self.document = result;

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        // Check if node_idx is valid
        if self.node_idx >= self.document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Check if we have the original items
        if self.original_items.is_none() {
            return Err(EditError::Other("No original items to restore".to_string()));
        }

        // Clone the document for modification
        let mut result = self.document.clone();

        // Get mutable reference to the list node in the result document
        if let Node::List { items, .. } = &mut result.nodes[self.node_idx] {
            // Restore original items
            *items = self.original_items.clone().unwrap();
        } else {
            return Err(EditError::Other("Node is not a list".to_string()));
        }

        // Update the document
        self.document = result;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_task_list_alphabetically() {
        let document = create_test_document();
        let mut command = SortTaskListCommand::new(document, 0, SortCriteria::Alphabetical);

        assert!(command.execute().is_ok());

        // Verify items are sorted alphabetically
        if let Node::List { items, .. } = &command.document.nodes[0] {
            assert_eq!(items[0].as_text().unwrap(), "Attend meeting");
            assert_eq!(items[1].as_text().unwrap(), "Book flights");
            assert_eq!(items[2].as_text().unwrap(), "Buy groceries");
            assert_eq!(items[3].as_text().unwrap(), "Call plumber");
            assert_eq!(items[4].as_text().unwrap(), "Finish report");
        } else {
            panic!("Expected a List node");
        }

        // Test undo
        assert!(command.undo().is_ok());

        // Verify original order is restored
        if let Node::List { items, .. } = &command.document.nodes[0] {
            assert_eq!(items[0].as_text().unwrap(), "Buy groceries");
            assert_eq!(items[1].as_text().unwrap(), "Call plumber");
            assert_eq!(items[2].as_text().unwrap(), "Attend meeting");
            assert_eq!(items[3].as_text().unwrap(), "Finish report");
            assert_eq!(items[4].as_text().unwrap(), "Book flights");
        } else {
            panic!("Expected a List node");
        }
    }

    #[test]
    fn test_sort_task_list_unchecked_first() {
        let document = create_test_document();
        let mut command = SortTaskListCommand::new(document, 0, SortCriteria::UncheckedFirst);

        assert!(command.execute().is_ok());

        // Verify unchecked items come first, then sorted alphabetically
        if let Node::List { items, .. } = &command.document.nodes[0] {
            // Unchecked items
            assert_eq!(items[0].as_text().unwrap(), "Attend meeting");
            assert_eq!(items[0].checked, Some(false));
            assert_eq!(items[1].as_text().unwrap(), "Book flights");
            assert_eq!(items[1].checked, Some(false));
            assert_eq!(items[2].as_text().unwrap(), "Call plumber");
            assert_eq!(items[2].checked, Some(false));

            // Checked items
            assert_eq!(items[3].as_text().unwrap(), "Buy groceries");
            assert_eq!(items[3].checked, Some(true));
            assert_eq!(items[4].as_text().unwrap(), "Finish report");
            assert_eq!(items[4].checked, Some(true));
        } else {
            panic!("Expected a List node");
        }
    }

    #[test]
    fn test_sort_task_list_checked_first() {
        let document = create_test_document();
        let mut command = SortTaskListCommand::new(document, 0, SortCriteria::CheckedFirst);

        assert!(command.execute().is_ok());

        // Verify checked items come first, then sorted alphabetically
        if let Node::List { items, .. } = &command.document.nodes[0] {
            // Checked items
            assert_eq!(items[0].as_text().unwrap(), "Buy groceries");
            assert_eq!(items[0].checked, Some(true));
            assert_eq!(items[1].as_text().unwrap(), "Finish report");
            assert_eq!(items[1].checked, Some(true));

            // Unchecked items
            assert_eq!(items[2].as_text().unwrap(), "Attend meeting");
            assert_eq!(items[2].checked, Some(false));
            assert_eq!(items[3].as_text().unwrap(), "Book flights");
            assert_eq!(items[3].checked, Some(false));
            assert_eq!(items[4].as_text().unwrap(), "Call plumber");
            assert_eq!(items[4].checked, Some(false));
        } else {
            panic!("Expected a List node");
        }
    }

    #[test]
    fn test_sort_task_list_invalid_node_idx() {
        let document = create_test_document();
        let mut command = SortTaskListCommand::new(document, 1, SortCriteria::Alphabetical);

        let result = command.execute();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EditError::IndexOutOfBounds));
    }

    #[test]
    fn test_sort_task_list_non_list_node() {
        let mut document = Document::new();
        document.nodes.push(Node::paragraph("Not a list"));

        let mut command = SortTaskListCommand::new(document, 0, SortCriteria::Alphabetical);

        let result = command.execute();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EditError::Other(msg) if msg == "Node is not a list"
        ));
    }

    #[test]
    fn test_sort_task_list_non_task_list() {
        let mut document = Document::new();
        document
            .nodes
            .push(Node::unordered_list(vec!["Item 1", "Item 2"]));

        let mut command = SortTaskListCommand::new(document, 0, SortCriteria::Alphabetical);

        let result = command.execute();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EditError::Other(msg) if msg == "Node is not a task list"
        ));
    }

    /// Helper function to create a document with a task list for testing
    fn create_test_document() -> Document {
        let mut document = Document::new();
        let items = vec![
            ListItem::task("Buy groceries", true),
            ListItem::task("Call plumber", false),
            ListItem::task("Attend meeting", false),
            ListItem::task("Finish report", true),
            ListItem::task("Book flights", false),
        ];
        document.nodes.push(Node::List {
            list_type: ListType::Task,
            items,
        });
        document
    }
}
