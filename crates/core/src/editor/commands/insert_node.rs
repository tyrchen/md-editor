use crate::editor::command::Command;
use crate::error::EditError;
use crate::models::{CodeBlockProperties, Document, InlineNode, Node};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to insert a new node at a specific position in the document
pub struct InsertNodeCommand {
    document: Rc<RefCell<Document>>,
    position: usize,
    node: Node,
    // Store the inserted index for undo
    inserted_index: Option<usize>,
}

impl InsertNodeCommand {
    /// Create a new command to insert a node at the specified position
    pub fn new(document: Rc<RefCell<Document>>, position: usize, node: Node) -> Self {
        Self {
            document,
            position,
            node,
            inserted_index: None,
        }
    }

    /// Helper method to create a new paragraph node with text
    pub fn new_paragraph(document: Rc<RefCell<Document>>, position: usize, text: &str) -> Self {
        Self::new(
            document,
            position,
            Node::Paragraph {
                children: vec![InlineNode::text(text)],
            },
        )
    }

    /// Helper method to create a new heading node
    pub fn new_heading(
        document: Rc<RefCell<Document>>,
        position: usize,
        level: u8,
        text: &str,
    ) -> Self {
        // Validate heading level (1-6)
        let safe_level = level.clamp(1, 6);

        Self::new(
            document,
            position,
            Node::Heading {
                level: safe_level,
                children: vec![InlineNode::text(text)],
            },
        )
    }

    /// Helper method to create a new code block
    pub fn new_code_block(
        document: Rc<RefCell<Document>>,
        position: usize,
        code: &str,
        language: &str,
    ) -> Self {
        Self::new(
            document,
            position,
            Node::CodeBlock {
                language: language.to_string(),
                code: code.to_string(),
                properties: CodeBlockProperties::default(),
            },
        )
    }

    /// Creates a command to insert a thematic break (horizontal rule)
    ///
    /// # Arguments
    /// * `document` - Reference to the document
    /// * `position` - Position to insert the thematic break
    ///
    /// # Returns
    /// A new InsertNodeCommand
    #[allow(dead_code)]
    pub fn new_thematic_break(document: Rc<RefCell<Document>>, position: usize) -> Self {
        Self::new(document, position, Node::ThematicBreak)
    }
}

impl Command for InsertNodeCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        // Validate the document
        if self.document.borrow().nodes.is_empty() && self.position > 0 {
            return Err(EditError::IndexOutOfBounds);
        }

        let mut document = self.document.borrow_mut();

        // Check if position is valid (can be equal to length to append at the end)
        if self.position > document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Check for temporary node variants that shouldn't be inserted directly
        match &self.node {
            Node::TempListItem(_) | Node::TempTableCell(_) => {
                return Err(EditError::UnsupportedOperation);
            }
            _ => {}
        }

        // Insert the node at the specified position
        document.nodes.insert(self.position, self.node.clone());

        // Store the inserted index for undo
        self.inserted_index = Some(self.position);

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(index) = self.inserted_index {
            let mut document = self.document.borrow_mut();

            if index >= document.nodes.len() {
                return Err(EditError::IndexOutOfBounds);
            }

            // Remove the node we inserted
            document.nodes.remove(index);

            // Clear the inserted index
            self.inserted_index = None;

            Ok(())
        } else {
            Err(EditError::OperationFailed)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
