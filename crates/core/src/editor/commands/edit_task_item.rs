use crate::editor::command::Command;
use crate::models::{Document, ListType, Node};
use crate::{EditError, InlineNode};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command for editing the text content of a task list item.
///
/// # Examples
///
/// ```
/// use md_core::{Editor, Document};
///
/// let mut document = Document::new();
/// let mut editor = Editor::new(document);
///
/// // Assuming document has a task list at index 0
/// let result = editor.edit_task_item(0, 1, "Updated task description");
/// ```
#[derive(Debug)]
pub struct EditTaskItemCommand {
    /// Document to modify
    document: Rc<RefCell<Document>>,
    /// Index of the task list node
    node_idx: usize,
    /// Index of the task item to edit
    item_idx: usize,
    /// New text content for the task item
    text: String,
    /// Previous text content (used for undo)
    previous_text: Option<String>,
}

impl EditTaskItemCommand {
    /// Creates a new command to edit a task list item.
    ///
    /// # Arguments
    ///
    /// * `document` - Reference to the document to modify
    /// * `node_idx` - Index of the task list node in the document
    /// * `item_idx` - Index of the task item within the list
    /// * `text` - New text content for the task item
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_idx: usize,
        item_idx: usize,
        text: impl Into<String>,
    ) -> Self {
        Self {
            document,
            node_idx,
            item_idx,
            text: text.into(),
            previous_text: None,
        }
    }
}

impl Command for EditTaskItemCommand {
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

                // Get the task item
                if self.item_idx >= items.len() {
                    return Err(EditError::IndexOutOfBounds);
                }

                let item = &mut items[self.item_idx];

                // Find or create the paragraph node inside the list item
                let para_idx = item
                    .children
                    .iter()
                    .position(|child| matches!(child, Node::Paragraph { .. }));

                match para_idx {
                    Some(idx) => {
                        // Found existing paragraph
                        if let Node::Paragraph { children } = &mut item.children[idx] {
                            // Store previous text for undo
                            self.previous_text = Some(
                                children
                                    .first()
                                    .and_then(|node| {
                                        if let InlineNode::Text(text_node) = node {
                                            Some(text_node.text.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or_default(),
                            );

                            // Replace content with new text
                            children.clear();
                            children.push(InlineNode::text(self.text.clone()));
                        }
                    }
                    None => {
                        // No paragraph exists, store empty string for undo
                        self.previous_text = Some(String::new());

                        // Create new paragraph with text
                        let para = Node::Paragraph {
                            children: vec![InlineNode::text(self.text.clone())],
                        };

                        item.children.push(para);
                    }
                }

                Ok(())
            }
            _ => Err(EditError::Other("Node is not a list".into())),
        }
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(previous_text) = &self.previous_text {
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

                    // Get the task item
                    if self.item_idx >= items.len() {
                        return Err(EditError::IndexOutOfBounds);
                    }

                    let item = &mut items[self.item_idx];

                    // Find the paragraph node
                    let para_idx = item
                        .children
                        .iter()
                        .position(|child| matches!(child, Node::Paragraph { .. }));

                    if let Some(idx) = para_idx {
                        if let Node::Paragraph { children } = &mut item.children[idx] {
                            // Restore previous text
                            children.clear();
                            children.push(InlineNode::text(previous_text.clone()));
                            return Ok(());
                        }
                    }

                    return Err(EditError::Other(
                        "Paragraph node not found in task item".into(),
                    ));
                }
                _ => return Err(EditError::Other("Node is not a list".into())),
            }
        }

        Err(EditError::Other(
            "Previous text state not available for undo".into(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_task_item() {
        let mut doc = Document::new();

        // Add a task list with one item
        let items = vec![crate::models::ListItem::task("Original text", false)];

        let task_list = Node::List {
            list_type: ListType::Task,
            items,
        };

        doc.nodes.push(task_list);

        let doc_rc = Rc::new(RefCell::new(doc));
        let mut cmd = EditTaskItemCommand::new(doc_rc.clone(), 0, 0, "Updated text");

        // Execute the command
        assert!(cmd.execute().is_ok());

        // Verify the text was updated
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    if let Node::Paragraph { children } = &items[0].children[0] {
                        if let InlineNode::Text(text_node) = &children[0] {
                            assert_eq!(text_node.text, "Updated text");
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

        // Verify the text was restored
        {
            let doc = doc_rc.borrow();
            match &doc.nodes[0] {
                Node::List { items, .. } => {
                    if let Node::Paragraph { children } = &items[0].children[0] {
                        if let InlineNode::Text(text_node) = &children[0] {
                            assert_eq!(text_node.text, "Original text");
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
    }

    #[test]
    fn test_edit_task_item_invalid_node() {
        let mut doc = Document::new();

        // Add a paragraph instead of a task list
        doc.nodes.push(Node::Paragraph {
            children: vec![InlineNode::text("This is not a task list")],
        });

        let doc_rc = Rc::new(RefCell::new(doc));
        let mut cmd = EditTaskItemCommand::new(doc_rc, 0, 0, "Updated text");

        // Try to edit a task in a paragraph
        let result = cmd.execute();
        assert!(result.is_err());
        if let Err(EditError::Other(msg)) = result {
            assert_eq!(msg, "Node is not a list");
        } else {
            panic!("Expected EditError::Other");
        }
    }

    #[test]
    fn test_edit_task_item_invalid_index() {
        let mut doc = Document::new();

        // Add a task list with one item
        let items = vec![crate::models::ListItem::task("Task 1", false)];

        let task_list = Node::List {
            list_type: ListType::Task,
            items,
        };

        doc.nodes.push(task_list);

        let doc_rc = Rc::new(RefCell::new(doc));

        // Try to edit with an invalid item index
        let mut cmd = EditTaskItemCommand::new(doc_rc.clone(), 0, 1, "Updated text");
        let result = cmd.execute();
        assert!(result.is_err());
        assert!(matches!(result, Err(EditError::IndexOutOfBounds)));

        // Try to edit with an invalid node index
        let mut cmd = EditTaskItemCommand::new(doc_rc, 1, 0, "Updated text");
        let result = cmd.execute();
        assert!(result.is_err());
        assert!(matches!(result, Err(EditError::IndexOutOfBounds)));
    }
}
