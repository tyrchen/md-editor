mod command;
mod commands;

use std::cell::RefCell;
use std::rc::Rc;

// Only import specific items from the command module
use command::{DeleteTextCommand, MergeNodesCommand};

// Import specific commands by name
use commands::ConvertNodeTypeCommand;
use commands::CopySelectionCommand;
use commands::CutSelectionCommand;
use commands::DeleteNodeCommand;
use commands::DuplicateNodeCommand;
use commands::FindReplaceCommand;
use commands::FormatTextCommand;
use commands::IndentDirection;
use commands::InsertNodeCommand;
use commands::InsertTextCommand;
use commands::MoveNodeCommand;
use commands::SelectionFormatCommand;
use commands::SelectionIndentCommand;

use crate::error::EditError;
use crate::{Document, ListType, Node, TextFormatting};

// Define an alias for the Command trait to avoid conflicts
use command::Command as EditorCommand;

/// Editor implementation that provides operations for modifying a document
pub struct Editor {
    document: Rc<RefCell<Document>>,
    undo_stack: Vec<Box<dyn EditorCommand>>,
    redo_stack: Vec<Box<dyn EditorCommand>>,
    max_history: usize,
}

/// Enum representing node conversion types
pub enum NodeConversionType {
    /// Convert to paragraph
    Paragraph,
    /// Convert to heading with level
    Heading(u8),
    /// Convert to list with type
    List(ListType),
    /// Convert to code block with language
    CodeBlock(String),
    /// Convert to blockquote
    BlockQuote,
}

impl Editor {
    /// Create a new editor for an existing document
    pub fn new(document: Document) -> Self {
        Self {
            document: Rc::new(RefCell::new(document)),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100, // Default history limit
        }
    }

    /// Get a reference to the current document
    pub fn document(&self) -> Rc<RefCell<Document>> {
        self.document.clone()
    }

    /// Set the maximum number of operations to keep in history
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        if self.undo_stack.len() > max {
            self.undo_stack.drain(0..(self.undo_stack.len() - max));
        }
        if self.redo_stack.len() > max {
            self.redo_stack.drain(0..(self.redo_stack.len() - max));
        }
    }

    /// Delete text from a specific node
    pub fn delete_text(
        &mut self,
        node_index: usize,
        start: usize,
        end: usize,
    ) -> Result<(), EditError> {
        let command = Box::new(DeleteTextCommand::new(
            self.document.clone(),
            node_index,
            start,
            end,
        ));
        self.execute_command(command)
    }

    /// Merge two adjacent nodes of the same type
    pub fn merge_nodes(
        &mut self,
        first_index: usize,
        second_index: usize,
    ) -> Result<(), EditError> {
        let command = Box::new(MergeNodesCommand::new(
            self.document.clone(),
            first_index,
            second_index,
        ));
        self.execute_command(command)
    }

    /// Format text within a paragraph
    pub fn format_text(
        &mut self,
        node_index: usize,
        start: usize,
        end: usize,
        formatting: TextFormatting,
    ) -> Result<(), EditError> {
        let command = Box::new(FormatTextCommand::new(
            self.document.clone(),
            node_index,
            start,
            end,
            formatting,
        ));
        self.execute_command(command)
    }

    /// Move a node from one position to another
    pub fn move_node(&mut self, from_index: usize, to_index: usize) -> Result<(), EditError> {
        let command = Box::new(MoveNodeCommand::new(
            self.document.clone(),
            from_index,
            to_index,
        ));
        self.execute_command(command)
    }

    /// Convert a node from one type to another
    pub fn convert_node_type(
        &mut self,
        node_index: usize,
        target_type: NodeConversionType,
    ) -> Result<(), EditError> {
        let command = Box::new(ConvertNodeTypeCommand::new(
            self.document.clone(),
            node_index,
            target_type,
        ));
        self.execute_command(command)
    }

    /// Delete a node entirely
    pub fn delete_node(&mut self, node_index: usize) -> Result<(), EditError> {
        let command = Box::new(DeleteNodeCommand::new(self.document.clone(), node_index));
        self.execute_command(command)
    }

    /// Find and replace text across the document
    /// Returns the number of replacements made
    pub fn find_replace(&mut self, find: &str, replace: &str, case_sensitive: bool) -> usize {
        let mut fr_command =
            FindReplaceCommand::new(self.document.clone(), find, replace, case_sensitive);

        // Execute the command
        match fr_command.execute() {
            Ok(_) => {
                let replacements = fr_command.replacements();

                // Add to undo stack
                self.undo_stack.push(Box::new(fr_command));
                self.redo_stack.clear();

                // Trim history if needed
                if self.undo_stack.len() > self.max_history {
                    self.undo_stack.remove(0);
                }

                replacements
            }
            Err(_) => 0,
        }
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> Result<(), EditError> {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo()?;
            self.redo_stack.push(command);
            Ok(())
        } else {
            Err(EditError::Other("Nothing to undo".to_string()))
        }
    }

    /// Redo the last undone operation
    pub fn redo(&mut self) -> Result<(), EditError> {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute()?;
            self.undo_stack.push(command);
            Ok(())
        } else {
            Err(EditError::Other("Nothing to redo".to_string()))
        }
    }

    /// Execute a command and add it to the undo stack
    fn execute_command(&mut self, mut command: Box<dyn EditorCommand>) -> Result<(), EditError> {
        command.execute()?;

        self.undo_stack.push(command);
        self.redo_stack.clear();

        // Trim history if needed
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }

        Ok(())
    }

    /// Insert text at a specific position in a node
    pub fn insert_text(
        &mut self,
        node_index: usize,
        position: usize,
        text: &str,
    ) -> Result<(), EditError> {
        let command = Box::new(InsertTextCommand::new(
            self.document.clone(),
            node_index,
            position,
            text.to_string(),
        ));
        self.execute_command(command)
    }

    /// Insert a new node at a specific position in the document
    pub fn insert_node(&mut self, position: usize, node: Node) -> Result<(), EditError> {
        let command = Box::new(InsertNodeCommand::new(
            self.document.clone(),
            position,
            node,
        ));
        self.execute_command(command)
    }

    /// Insert a new paragraph with text at a specific position
    pub fn insert_paragraph(&mut self, position: usize, text: &str) -> Result<(), EditError> {
        let command = Box::new(InsertNodeCommand::new_paragraph(
            self.document.clone(),
            position,
            text,
        ));
        self.execute_command(command)
    }

    /// Insert a new heading with text at a specific position
    pub fn insert_heading(
        &mut self,
        position: usize,
        level: u8,
        text: &str,
    ) -> Result<(), EditError> {
        let command = Box::new(InsertNodeCommand::new_heading(
            self.document.clone(),
            position,
            level,
            text,
        ));
        self.execute_command(command)
    }

    /// Insert a new code block at a specific position
    pub fn insert_code_block(
        &mut self,
        position: usize,
        code: &str,
        language: &str,
    ) -> Result<(), EditError> {
        let command = Box::new(InsertNodeCommand::new_code_block(
            self.document.clone(),
            position,
            code,
            language,
        ));
        self.execute_command(command)
    }

    /// Duplicate a node at a specific index
    pub fn duplicate_node(&mut self, node_index: usize) -> Result<(), EditError> {
        let command = Box::new(DuplicateNodeCommand::new(self.document.clone(), node_index));
        self.execute_command(command)
    }

    /// Cut the currently selected content
    /// Returns a vector of nodes that were cut
    pub fn cut_selection(&mut self) -> Vec<Node> {
        let mut cut_cmd = CutSelectionCommand::new(self.document.clone());

        match cut_cmd.execute() {
            Ok(_) => {
                let cut_content = cut_cmd.cut_content().to_vec();

                // Add to undo stack
                self.undo_stack.push(Box::new(cut_cmd));
                self.redo_stack.clear();

                // Trim history if needed
                if self.undo_stack.len() > self.max_history {
                    self.undo_stack.remove(0);
                }

                cut_content
            }
            Err(_) => Vec::new(),
        }
    }

    /// Copy the currently selected content without modifying the document
    /// Returns a vector of nodes that were copied
    pub fn copy_selection(&mut self) -> Vec<Node> {
        let mut copy_cmd = CopySelectionCommand::new(self.document.clone());

        match copy_cmd.execute() {
            Ok(_) => {
                // Since copy doesn't modify the document, we don't add it to the undo stack
                copy_cmd.get_copied_nodes().to_vec()
            }
            Err(_) => Vec::new(),
        }
    }

    /// Apply formatting to the selected text
    pub fn format_selection(&mut self, formatting: TextFormatting) -> Result<(), EditError> {
        let command = Box::new(SelectionFormatCommand::new(
            self.document.clone(),
            formatting,
        ));
        self.execute_command(command)
    }

    /// Increase the indentation of the selected content
    pub fn indent_selection(&mut self) -> Result<(), EditError> {
        let command = Box::new(SelectionIndentCommand::new(
            self.document.clone(),
            IndentDirection::Increase,
        ));
        self.execute_command(command)
    }

    /// Decrease the indentation of the selected content
    pub fn unindent_selection(&mut self) -> Result<(), EditError> {
        let command = Box::new(SelectionIndentCommand::new(
            self.document.clone(),
            IndentDirection::Decrease,
        ));
        self.execute_command(command)
    }
}

#[cfg(test)]
mod command_tests {
    use crate::{Document, Editor, InlineNode, Node, NodeConversionType, TextFormatting};

    #[test]
    fn test_delete_text() {
        let mut doc = Document::new();
        let index = doc.add_paragraph_with_text("Hello, world!");

        let mut editor = Editor::new(doc);

        // Delete "world"
        let result = editor.delete_text(index, 7, 12);
        assert!(result.is_ok());

        // Clone the document reference to avoid borrowing conflicts
        let document_ref = editor.document();
        {
            // Scope the borrow to ensure it's dropped before we call undo
            let borrowed_doc = document_ref.borrow();

            match &borrowed_doc.nodes[index] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "Hello, !");
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
        }

        // Test undo
        let result = editor.undo();
        assert!(result.is_ok());

        // Borrow again after the undo operation
        let borrowed_doc = document_ref.borrow();
        match &borrowed_doc.nodes[index] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(text_node) => {
                    assert_eq!(text_node.text, "Hello, world!");
                }
                _ => panic!("Expected Text node"),
            },
            _ => panic!("Expected Paragraph node"),
        }
    }

    #[test]
    fn test_merge_nodes() {
        let mut doc = Document::new();
        let p1 = doc.add_paragraph_with_text("First paragraph.");
        let p2 = doc.add_paragraph_with_text("Second paragraph.");

        let mut editor = Editor::new(doc);

        // Merge paragraphs
        let result = editor.merge_nodes(p1, p2);
        assert!(result.is_ok());

        // Clone the document reference to avoid borrowing conflicts
        let document_ref = editor.document();
        {
            // Scope the borrow to ensure it's dropped before we call undo
            let borrowed_doc = document_ref.borrow();

            // Should now have only one paragraph
            assert_eq!(borrowed_doc.nodes.len(), 1);

            match &borrowed_doc.nodes[0] {
                Node::Paragraph { children } => {
                    // Should have two text nodes now
                    assert_eq!(children.len(), 2);
                }
                _ => panic!("Expected Paragraph node"),
            }
        }

        // Test undo
        let result = editor.undo();
        assert!(result.is_ok());

        // Borrow again after the undo operation
        let borrowed_doc = document_ref.borrow();

        // Should be back to two paragraphs
        assert_eq!(borrowed_doc.nodes.len(), 2);
    }

    #[test]
    fn test_format_text() {
        let mut doc = Document::new();
        let index = doc.add_paragraph_with_text("Hello, world!");

        let mut editor = Editor::new(doc);

        // Make "world" bold
        let formatting = TextFormatting {
            bold: true,
            ..Default::default()
        };

        let result = editor.format_text(index, 7, 12, formatting);
        assert!(result.is_ok());

        // Clone the document reference to avoid borrowing conflicts
        let document_ref = editor.document();
        {
            // Scope the borrow to ensure it's dropped before we call undo
            let borrowed_doc = document_ref.borrow();

            // Should now have three text nodes: before, formatted, after
            match &borrowed_doc.nodes[index] {
                Node::Paragraph { children } => {
                    assert_eq!(children.len(), 3);

                    // Check the middle node is bold
                    match &children[1] {
                        InlineNode::Text(text_node) => {
                            assert_eq!(text_node.text, "world");
                            assert!(text_node.formatting.bold);
                        }
                        _ => panic!("Expected Text node"),
                    }
                }
                _ => panic!("Expected Paragraph node"),
            }
        }

        // Test undo
        let result = editor.undo();
        assert!(result.is_ok());

        // Borrow again after the undo operation
        let borrowed_doc = document_ref.borrow();

        // Should be back to one text node
        match &borrowed_doc.nodes[index] {
            Node::Paragraph { children } => {
                assert_eq!(children.len(), 1);

                match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "Hello, world!");
                        assert!(!text_node.formatting.bold);
                    }
                    _ => panic!("Expected Text node"),
                }
            }
            _ => panic!("Expected Paragraph node"),
        }
    }

    #[test]
    fn test_delete_node() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("First paragraph.");
        doc.add_paragraph_with_text("Second paragraph.");

        let mut editor = Editor::new(doc);

        // Delete first paragraph
        let result = editor.delete_node(0);
        assert!(result.is_ok());

        // Clone the document reference to avoid borrowing conflicts
        let document_ref = editor.document();
        {
            // Scope the borrow to ensure it's dropped before we call undo
            let borrowed_doc = document_ref.borrow();

            // Should now have only one paragraph
            assert_eq!(borrowed_doc.nodes.len(), 1);

            match &borrowed_doc.nodes[0] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "Second paragraph.");
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
        }

        // Test undo
        let result = editor.undo();
        assert!(result.is_ok());

        // Borrow again after the undo operation
        let borrowed_doc = document_ref.borrow();

        // Should be back to two paragraphs
        assert_eq!(borrowed_doc.nodes.len(), 2);
    }

    #[test]
    fn test_move_node() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("First paragraph.");
        doc.add_paragraph_with_text("Second paragraph.");
        doc.add_paragraph_with_text("Third paragraph.");

        let mut editor = Editor::new(doc);

        // Move first paragraph to the end
        let result = editor.move_node(0, 3);
        assert!(result.is_ok());

        // Clone the document reference to avoid borrowing conflicts
        let document_ref = editor.document();
        {
            // Scope the borrow to ensure it's dropped before we call undo
            let borrowed_doc = document_ref.borrow();

            // Order should now be: Second, Third, First
            match &borrowed_doc.nodes[2] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "First paragraph.");
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
        }

        // Test undo
        let result = editor.undo();
        assert!(result.is_ok());

        // Borrow again after the undo operation
        let borrowed_doc = document_ref.borrow();

        // Should be back to original order
        match &borrowed_doc.nodes[0] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(text_node) => {
                    assert_eq!(text_node.text, "First paragraph.");
                }
                _ => panic!("Expected Text node"),
            },
            _ => panic!("Expected Paragraph node"),
        }
    }

    #[test]
    fn test_convert_node_type() {
        let mut doc = Document::new();
        let index = doc.add_paragraph_with_text("This is a paragraph");

        let mut editor = Editor::new(doc);

        // Convert paragraph to heading level 2
        let result = editor.convert_node_type(index, NodeConversionType::Heading(2));
        assert!(result.is_ok());

        // Clone the document reference to avoid borrowing conflicts
        let document_ref = editor.document();
        {
            // Scope the borrow to ensure it's dropped before we call undo
            let borrowed_doc = document_ref.borrow();

            // Should now be a heading
            match &borrowed_doc.nodes[index] {
                Node::Heading { level, children } => {
                    assert_eq!(*level, 2);
                    match &children[0] {
                        InlineNode::Text(text_node) => {
                            assert_eq!(text_node.text, "This is a paragraph");
                        }
                        _ => panic!("Expected Text node"),
                    }
                }
                _ => panic!("Expected Heading node"),
            }
        }

        // Test undo
        let result = editor.undo();
        assert!(result.is_ok());

        // Borrow again after the undo operation
        let borrowed_doc = document_ref.borrow();

        // Should be back to a paragraph
        match &borrowed_doc.nodes[index] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(text_node) => {
                    assert_eq!(text_node.text, "This is a paragraph");
                }
                _ => panic!("Expected Text node"),
            },
            _ => panic!("Expected Paragraph node"),
        }
    }

    #[test]
    fn test_insert_text() {
        let mut doc = Document::new();
        let index = doc.add_paragraph_with_text("Hello world!");

        let mut editor = Editor::new(doc);

        // Insert text in the middle
        let result = editor.insert_text(index, 5, ", beautiful");
        assert!(result.is_ok());

        // Clone the document reference to avoid borrowing conflicts
        let document_ref = editor.document();
        {
            // Scope the borrow to ensure it's dropped before we call undo
            let borrowed_doc = document_ref.borrow();

            match &borrowed_doc.nodes[index] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "Hello, beautiful world!");
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
        }

        // Test undo
        let result = editor.undo();
        assert!(result.is_ok());

        // Borrow again after the undo operation
        let borrowed_doc = document_ref.borrow();
        match &borrowed_doc.nodes[index] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(text_node) => {
                    assert_eq!(text_node.text, "Hello world!");
                }
                _ => panic!("Expected Text node"),
            },
            _ => panic!("Expected Paragraph node"),
        }
    }

    #[test]
    fn test_insert_node() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("First paragraph.");

        let mut editor = Editor::new(doc);

        // Insert a new heading after the paragraph
        let result = editor.insert_heading(1, 2, "New Heading");
        assert!(result.is_ok());

        // Clone the document reference to avoid borrowing conflicts
        let document_ref = editor.document();
        {
            // Scope the borrow to ensure it's dropped before we call undo
            let borrowed_doc = document_ref.borrow();

            // Should now have two nodes
            assert_eq!(borrowed_doc.nodes.len(), 2);

            // Check if the new node is a heading with the right content
            match &borrowed_doc.nodes[1] {
                Node::Heading { level, children } => {
                    assert_eq!(*level, 2);
                    match &children[0] {
                        InlineNode::Text(text_node) => {
                            assert_eq!(text_node.text, "New Heading");
                        }
                        _ => panic!("Expected Text node"),
                    }
                }
                _ => panic!("Expected Heading node"),
            }
        }

        // Test undo
        let result = editor.undo();
        assert!(result.is_ok());

        // Borrow again after the undo operation
        let borrowed_doc = document_ref.borrow();

        // Should be back to one paragraph
        assert_eq!(borrowed_doc.nodes.len(), 1);
    }

    #[test]
    fn test_duplicate_node() {
        let mut doc = Document::new();
        let p1 = doc.add_paragraph_with_text("This is a paragraph to duplicate.");

        let mut editor = Editor::new(doc);

        // Duplicate the paragraph
        let result = editor.duplicate_node(p1);
        assert!(result.is_ok());

        // Clone the document reference to avoid borrowing conflicts
        let document_ref = editor.document();
        {
            // Scope the borrow to ensure it's dropped before we call undo
            let borrowed_doc = document_ref.borrow();

            // Should now have two identical paragraphs
            assert_eq!(borrowed_doc.nodes.len(), 2);

            // Check that both paragraphs have the same text
            match (&borrowed_doc.nodes[0], &borrowed_doc.nodes[1]) {
                (
                    Node::Paragraph {
                        children: children1,
                    },
                    Node::Paragraph {
                        children: children2,
                    },
                ) => match (&children1[0], &children2[0]) {
                    (InlineNode::Text(text_node1), InlineNode::Text(text_node2)) => {
                        assert_eq!(text_node1.text, "This is a paragraph to duplicate.");
                        assert_eq!(text_node2.text, "This is a paragraph to duplicate.");
                    }
                    _ => panic!("Expected Text nodes"),
                },
                _ => panic!("Expected two Paragraph nodes"),
            }
        }

        // Test undo
        let result = editor.undo();
        assert!(result.is_ok());

        // Borrow again after the undo operation
        let borrowed_doc = document_ref.borrow();

        // Should be back to one paragraph
        assert_eq!(borrowed_doc.nodes.len(), 1);
    }
}
