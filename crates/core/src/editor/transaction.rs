use std::cell::RefCell;
use std::rc::Rc;

use crate::TextFormatting;
use crate::editor::NodeConversionType;
use crate::editor::command::Command as EditorCommand;
use crate::editor::command::{DeleteTextCommand, MergeNodesCommand};
use crate::editor::commands::*;
use crate::error::EditError;
use crate::{Document, Node, Position, Selection};

/// A transaction that groups multiple document operations together as a single atomic change.
///
/// The transaction follows a builder pattern, allowing multiple operations to be chained
/// together before being committed as a single unit. This is useful for ensuring that
/// complex edits are applied atomically and can be undone as a single operation.
///
/// # Example
/// ```
/// let mut transaction = editor.begin_transaction();
/// transaction
///     .insert_heading(0, 1, "Document Title")
///     .insert_paragraph(1, "First paragraph");
///
/// // Commit the transaction to get the commands
/// let commands = transaction.commit()?;
///
/// // Execute the commands on the editor
/// editor.execute_transaction_commands(commands)?;
/// ```
///
/// Transactions are automatically rolled back if they are dropped without being committed,
/// ensuring that no changes are made to the document if a transaction is abandoned.
pub struct Transaction {
    document: Rc<RefCell<Document>>,
    commands: Vec<Box<dyn EditorCommand>>,
    committed: bool,
    selection: Option<Selection>,
}

impl Transaction {
    /// Creates a new transaction.
    ///
    /// This is typically called by the Editor's begin_transaction method.
    pub(crate) fn new(document: Rc<RefCell<Document>>) -> Self {
        Self {
            document,
            commands: Vec::new(),
            committed: false,
            selection: None,
        }
    }

    /// Add a command to the transaction.
    fn add_command<C: EditorCommand + 'static>(&mut self, command: C) {
        self.commands.push(Box::new(command));
    }

    /// Insert text at a specific position in a node.
    pub fn insert_text(&mut self, node_index: usize, position: usize, text: &str) -> &mut Self {
        let command = InsertTextCommand::new(
            self.document.clone(),
            node_index,
            position,
            text.to_string(),
        );
        self.add_command(command);
        self
    }

    /// Delete text from a specific node.
    pub fn delete_text(&mut self, node_index: usize, start: usize, end: usize) -> &mut Self {
        let command = DeleteTextCommand::new(self.document.clone(), node_index, start, end);
        self.add_command(command);
        self
    }

    /// Format text within a paragraph.
    pub fn format_text(
        &mut self,
        node_index: usize,
        start: usize,
        end: usize,
        formatting: TextFormatting,
    ) -> &mut Self {
        let command =
            FormatTextCommand::new(self.document.clone(), node_index, start, end, formatting);
        self.add_command(command);
        self
    }

    /// Insert a new node at a specific position.
    pub fn insert_node(&mut self, position: usize, node: Node) -> &mut Self {
        let command = InsertNodeCommand::new(self.document.clone(), position, node);
        self.add_command(command);
        self
    }

    /// Insert a paragraph with text.
    pub fn insert_paragraph(&mut self, position: usize, text: &str) -> &mut Self {
        let command = InsertNodeCommand::new_paragraph(self.document.clone(), position, text);
        self.add_command(command);
        self
    }

    /// Insert a heading with text.
    pub fn insert_heading(&mut self, position: usize, level: u8, text: &str) -> &mut Self {
        let command = InsertNodeCommand::new_heading(self.document.clone(), position, level, text);
        self.add_command(command);
        self
    }

    /// Insert a code block.
    pub fn insert_code_block(&mut self, position: usize, code: &str, language: &str) -> &mut Self {
        let command =
            InsertNodeCommand::new_code_block(self.document.clone(), position, code, language);
        self.add_command(command);
        self
    }

    /// Delete a node.
    pub fn delete_node(&mut self, node_index: usize) -> &mut Self {
        let command = DeleteNodeCommand::new(self.document.clone(), node_index);
        self.add_command(command);
        self
    }

    /// Move a node from one position to another.
    pub fn move_node(&mut self, from_index: usize, to_index: usize) -> &mut Self {
        let command = MoveNodeCommand::new(self.document.clone(), from_index, to_index);
        self.add_command(command);
        self
    }

    /// Convert a node from one type to another.
    pub fn convert_node_type(
        &mut self,
        node_index: usize,
        target_type: NodeConversionType,
    ) -> &mut Self {
        let command = ConvertNodeTypeCommand::new(self.document.clone(), node_index, target_type);
        self.add_command(command);
        self
    }

    /// Merge two adjacent nodes.
    pub fn merge_nodes(&mut self, first_index: usize, second_index: usize) -> &mut Self {
        let command = MergeNodesCommand::new(self.document.clone(), first_index, second_index);
        self.add_command(command);
        self
    }

    /// Duplicate a node.
    pub fn duplicate_node(&mut self, node_index: usize) -> &mut Self {
        let command = DuplicateNodeCommand::new(self.document.clone(), node_index);
        self.add_command(command);
        self
    }

    /// Apply formatting to selection.
    pub fn format_selection(&mut self, formatting: TextFormatting) -> &mut Self {
        let command = SelectionFormatCommand::new(self.document.clone(), formatting);
        self.add_command(command);
        self
    }

    /// Indent selection.
    pub fn indent_selection(&mut self) -> &mut Self {
        let command = SelectionIndentCommand::new(self.document.clone(), IndentDirection::Increase);
        self.add_command(command);
        self
    }

    /// Unindent selection.
    pub fn unindent_selection(&mut self) -> &mut Self {
        let command = SelectionIndentCommand::new(self.document.clone(), IndentDirection::Decrease);
        self.add_command(command);
        self
    }

    /// Create a table.
    pub fn create_table(
        &mut self,
        position: usize,
        columns: usize,
        rows: usize,
        with_header: bool,
    ) -> &mut Self {
        let command =
            CreateTableCommand::new(self.document.clone(), position, columns, rows, with_header);
        self.add_command(command);
        self
    }

    /// Add a table operation (add/remove row/column, set cell, etc.).
    pub fn table_operation(&mut self, node_index: usize, operation: TableOperation) -> &mut Self {
        let command = TableOperationsCommand::new(self.document.clone(), node_index, operation);
        self.add_command(command);
        self
    }

    /// Selects an entire node by index.
    pub fn select_node(&mut self, node_index: usize) -> &mut Self {
        {
            let document = self.document.borrow();
            if node_index < document.nodes.len() {
                let start_position = Position::new(vec![node_index], 0);
                // For the end position, ideally we'd get the content length
                // but for simplicity, use a large value that exceeds any reasonable content
                let end_position = Position::new(vec![node_index], usize::MAX);
                self.selection = Some(Selection::new(start_position, end_position));
            }
        }
        self
    }

    /// Selects a range of text in a node.
    pub fn select_text_range(&mut self, node_index: usize, start: usize, end: usize) -> &mut Self {
        {
            let document = self.document.borrow();
            if node_index < document.nodes.len() {
                let start_position = Position::new(vec![node_index], start);
                let end_position = Position::new(vec![node_index], end);
                self.selection = Some(Selection::new(start_position, end_position));
            }
        }
        self
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) -> &mut Self {
        self.selection = None;
        self
    }

    /// Commits all changes to the document as a single operation.
    ///
    /// Returns a Vec of all the commands for adding to the editor's undo stack.
    /// Note that this consumes the transaction, so it can only be committed once.
    pub fn commit(mut self) -> Result<Vec<Box<dyn EditorCommand>>, EditError> {
        if self.committed {
            return Err(EditError::Other(
                "Transaction already committed".to_string(),
            ));
        }

        // No commands to execute
        if self.commands.is_empty() {
            return Ok(Vec::new());
        }

        // Execute all commands in order
        for cmd in &mut self.commands {
            // If any command fails, roll back all previous commands
            if let Err(err) = cmd.execute() {
                // Roll back all commands that were already executed
                self.rollback();
                return Err(err);
            }
        }

        self.committed = true;
        let mut commands = Vec::new();
        std::mem::swap(&mut commands, &mut self.commands);
        Ok(commands)
    }

    /// Rolls back any executed commands in reverse order.
    fn rollback(&mut self) {
        // Undo each command in reverse order
        for cmd in self.commands.iter_mut().rev() {
            // Try to undo, but ignore any errors since we're rolling back
            let _ = cmd.undo();
        }

        // Clear the commands as they've all been undone
        self.commands.clear();
    }

    /// Discards the transaction without applying any changes.
    pub fn discard(mut self) {
        self.rollback();
        self.committed = true; // Mark as committed to prevent future use
    }
}

/// Implementing Drop to automatically discard uncommitted transactions.
impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed {
            // Only need to roll back if not committed
            self.rollback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Document, Editor, InlineNode};

    #[test]
    fn test_transaction_api() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("Initial text");

        let mut editor = Editor::new(doc);

        // Create a transaction to modify text
        let mut transaction = Transaction::new(editor.document().clone());
        transaction.insert_text(0, 7, " modified").format_text(
            0,
            8,
            16,
            TextFormatting {
                bold: true,
                ..Default::default()
            },
        );

        // Commit the transaction
        let commands = transaction.commit().expect("Transaction should commit");

        // Execute the commands
        editor
            .execute_transaction_commands(commands)
            .expect("Should execute commands");

        // Verify changes
        {
            let doc = editor.document().borrow();
            match &doc.nodes[0] {
                Node::Paragraph { children } => {
                    assert_eq!(children.len(), 3);

                    // Second part should be bold
                    match &children[1] {
                        InlineNode::Text(text_node) => {
                            assert_eq!(text_node.text, "modified");
                            assert!(text_node.formatting.bold);
                        }
                        _ => panic!("Expected Text node"),
                    }
                }
                _ => panic!("Expected Paragraph node"),
            }
        }

        // Test undo
        editor.undo().expect("Undo should succeed");

        // Verify undone changes
        {
            let doc = editor.document().borrow();
            match &doc.nodes[0] {
                Node::Paragraph { children } => {
                    assert_eq!(children.len(), 1);
                    match &children[0] {
                        InlineNode::Text(text_node) => {
                            assert_eq!(text_node.text, "Initial text");
                        }
                        _ => panic!("Expected Text node"),
                    }
                }
                _ => panic!("Expected Paragraph node"),
            }
        }
    }

    #[test]
    fn test_transaction_rollback() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("Initial text");

        let editor = Editor::new(doc);
        let original_text = {
            let doc = editor.document().borrow();
            match &doc.nodes[0] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => text_node.text.clone(),
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
        };

        // Create a transaction but don't commit it
        {
            let mut transaction = Transaction::new(editor.document().clone());
            transaction.insert_text(0, 7, " modified");
            // Transaction will be dropped here without committing
        }

        // Verify no changes were made (transaction should be rolled back on drop)
        {
            let doc = editor.document().borrow();
            match &doc.nodes[0] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, original_text);
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
        }
    }

    #[test]
    fn test_transaction_with_error() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("Initial text");

        let editor = Editor::new(doc);

        // Create a transaction with an operation that will fail
        let mut transaction = Transaction::new(editor.document().clone());
        transaction
            .insert_text(0, 7, " good") // This should succeed
            .delete_node(99); // This should fail (invalid index)

        // Commit should fail and rollback
        let result = transaction.commit();
        assert!(result.is_err());

        // Verify all changes were rolled back
        {
            let doc = editor.document().borrow();
            match &doc.nodes[0] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "Initial text");
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
        }
    }

    #[test]
    fn test_complex_transaction() {
        let doc = Document::new();
        let mut editor = Editor::new(doc);

        // Create a transaction for building a document structure
        let mut transaction = Transaction::new(editor.document().clone());
        transaction
            .insert_heading(0, 1, "Document Title")
            .insert_paragraph(1, "First paragraph")
            .insert_code_block(2, "let x = 42;", "rust")
            .insert_paragraph(3, "Another paragraph");

        // Commit the transaction
        let commands = transaction.commit().expect("Transaction should commit");

        // Execute the commands
        editor
            .execute_transaction_commands(commands)
            .expect("Should execute commands");

        // Verify all changes were made
        {
            let doc = editor.document().borrow();
            assert_eq!(doc.nodes.len(), 4);
        }

        // Test undo
        editor.undo().expect("Undo should succeed");

        // Document should be empty again
        {
            let doc = editor.document().borrow();
            assert_eq!(doc.nodes.len(), 0);
        }
    }
}
