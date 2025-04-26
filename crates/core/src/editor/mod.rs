mod command;
mod commands;
mod transaction;

use std::cell::RefCell;
use std::rc::Rc;

// Only import specific items from the command module
use command::{DeleteTextCommand, MergeNodesCommand};

// Import specific commands by name
use commands::AddTaskItemCommand;
use commands::ConvertNodeTypeCommand;
use commands::CopySelectionCommand;
use commands::CreateTOCCommand;
use commands::CreateTableCommand;
use commands::CutSelectionCommand;
use commands::DeleteNodeCommand;
use commands::DuplicateNodeCommand;
use commands::EditTaskItemCommand;
use commands::FindReplaceCommand;
use commands::FormatTextCommand;
use commands::GroupNodesCommand;
use commands::IndentDirection;
use commands::InsertNodeCommand;
use commands::InsertTextCommand;
use commands::MoveNodeCommand;
use commands::MoveTaskItemCommand;
use commands::RemoveTaskItemCommand;
use commands::SelectionFormatCommand;
use commands::SelectionIndentCommand;
use commands::TableOperation;
use commands::TableOperationsCommand;
use commands::ToggleTaskCommand;

// Export the Transaction type
pub use transaction::Transaction;

use crate::error::EditError;
use crate::{Document, ListType, Node, TableAlignment, TableProperties, TextFormatting};

// Define an alias for the Command trait to avoid conflicts
use command::Command as EditorCommand;

/// Editor manages a document and provides operations to modify it
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
    /// Creates a new editor instance with the given document
    pub fn new(document: Document) -> Self {
        Self {
            document: Rc::new(RefCell::new(document)),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100, // Default history limit
        }
    }

    /// Creates a new editor instance with a default empty document
    pub fn new_empty() -> Self {
        Self::new(Document::new())
    }

    /// Get a reference to the current document
    pub fn document(&self) -> &Rc<RefCell<Document>> {
        &self.document
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

    /// Create a table of contents from document headings
    ///
    /// - `position`: The position in the document where the TOC should be inserted
    /// - `max_level`: The maximum heading level to include (1-6)
    pub fn create_table_of_contents(
        &mut self,
        position: usize,
        max_level: u8,
    ) -> Result<(), EditError> {
        let command = Box::new(CreateTOCCommand::new(
            self.document.clone(),
            position,
            max_level,
        ));
        self.execute_command(command)
    }

    /// Create an empty table with default alignments
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `columns`: The number of columns in the table
    /// - `rows`: The number of rows in the table (not including header)
    pub fn create_table(
        &mut self,
        position: usize,
        columns: usize,
        rows: usize,
    ) -> Result<(), EditError> {
        let command = Box::new(CreateTableCommand::new(
            self.document.clone(),
            position,
            columns,
            rows,
        ));
        self.execute_command(command)
    }

    /// Create a table with custom column alignments
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `columns`: The number of columns in the table
    /// - `rows`: The number of rows in the table (not including header)
    /// - `alignments`: Column alignments (one per column)
    pub fn create_table_with_alignments(
        &mut self,
        position: usize,
        columns: usize,
        rows: usize,
        alignments: Vec<TableAlignment>,
    ) -> Result<(), EditError> {
        let command = Box::new(CreateTableCommand::with_alignments(
            self.document.clone(),
            position,
            columns,
            rows,
            alignments,
        ));
        self.execute_command(command)
    }

    /// Create a table with predefined data
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `header`: Header row cells
    /// - `rows`: Table data (rows of cells)
    /// - `alignments`: Optional column alignments
    pub fn create_table_with_data(
        &mut self,
        position: usize,
        header: Vec<String>,
        rows: Vec<Vec<String>>,
        alignments: Option<Vec<TableAlignment>>,
    ) -> Result<(), EditError> {
        let command = Box::new(CreateTableCommand::with_data(
            self.document.clone(),
            position,
            header,
            rows,
            alignments,
        ));
        self.execute_command(command)
    }

    /// Create a table with custom properties
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `columns`: The number of columns in the table
    /// - `rows`: The number of rows in the table (not including header)
    /// - `properties`: Table styling and behavior properties
    pub fn create_table_with_properties(
        &mut self,
        position: usize,
        columns: usize,
        rows: usize,
        properties: TableProperties,
    ) -> Result<(), EditError> {
        let command = Box::new(CreateTableCommand::with_properties(
            self.document.clone(),
            position,
            columns,
            rows,
            properties,
        ));
        self.execute_command(command)
    }

    /// Create a table with predefined data and custom properties
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `header`: Header row cells
    /// - `rows`: Table data (rows of cells)
    /// - `alignments`: Optional column alignments
    /// - `properties`: Table styling and behavior properties
    pub fn create_table_with_data_and_properties(
        &mut self,
        position: usize,
        header: Vec<String>,
        rows: Vec<Vec<String>>,
        alignments: Option<Vec<TableAlignment>>,
        properties: TableProperties,
    ) -> Result<(), EditError> {
        let command = Box::new(CreateTableCommand::with_data_and_properties(
            self.document.clone(),
            position,
            header,
            rows,
            alignments,
            properties,
        ));
        self.execute_command(command)
    }

    /// Add a row to an existing table
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row_index`: The index where the new row should be inserted (0 is first row after header)
    pub fn add_table_row(&mut self, node_index: usize, row_index: usize) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::AddRow(row_index),
        ));
        self.execute_command(command)
    }

    /// Remove a row from an existing table
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row_index`: The index of the row to remove
    pub fn remove_table_row(
        &mut self,
        node_index: usize,
        row_index: usize,
    ) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::RemoveRow(row_index),
        ));
        self.execute_command(command)
    }

    /// Add a column to an existing table
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `column_index`: The index where the new column should be inserted
    pub fn add_table_column(
        &mut self,
        node_index: usize,
        column_index: usize,
    ) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::AddColumn(column_index),
        ));
        self.execute_command(command)
    }

    /// Remove a column from an existing table
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `column_index`: The index of the column to remove
    pub fn remove_table_column(
        &mut self,
        node_index: usize,
        column_index: usize,
    ) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::RemoveColumn(column_index),
        ));
        self.execute_command(command)
    }

    /// Set the content of a table cell
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row`: The row index of the cell (ignored if is_header is true)
    /// - `column`: The column index of the cell
    /// - `content`: The new content for the cell
    /// - `is_header`: Whether the cell is in the header row
    pub fn set_table_cell(
        &mut self,
        node_index: usize,
        row: usize,
        column: usize,
        content: &str,
        is_header: bool,
    ) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::SetCell {
                row,
                column,
                content: content.to_string(),
                is_header,
            },
        ));
        self.execute_command(command)
    }

    /// Set the alignment of a table column
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `column`: The column index
    /// - `alignment`: The alignment to set
    pub fn set_table_column_alignment(
        &mut self,
        node_index: usize,
        column: usize,
        alignment: TableAlignment,
    ) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::SetAlignment { column, alignment },
        ));
        self.execute_command(command)
    }

    /// Group multiple nodes together
    ///
    /// - `node_indices`: Indices of nodes to group
    /// - `group_name`: Name or type of the group
    pub fn group_nodes(
        &mut self,
        node_indices: Vec<usize>,
        group_name: &str,
    ) -> Result<(), EditError> {
        let command = Box::new(GroupNodesCommand::new(
            self.document.clone(),
            node_indices,
            group_name.to_string(),
        ));
        self.execute_command(command)
    }

    /// Selects all content in the document
    pub fn select_all(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();
        if !document.select_all() {
            return Err(EditError::OperationFailed);
        }
        Ok(())
    }

    /// Selects a specific node by index
    pub fn select_node(&mut self, node_index: usize) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();
        if !document.select_node(node_index) {
            return Err(EditError::IndexOutOfBounds);
        }
        Ok(())
    }

    /// Selects a range of nodes
    pub fn select_node_range(
        &mut self,
        start_index: usize,
        end_index: usize,
    ) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();
        if !document.select_node_range(start_index, end_index) {
            return Err(EditError::InvalidRange);
        }
        Ok(())
    }

    /// Selects a specific range of text within a node
    pub fn select_text_range(
        &mut self,
        node_index: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();
        if !document.select_text_range(node_index, start_offset, end_offset) {
            return Err(EditError::InvalidRange);
        }
        Ok(())
    }

    /// Selects from one position to another across any nodes
    pub fn select_range(
        &mut self,
        start_node: usize,
        start_offset: usize,
        end_node: usize,
        end_offset: usize,
    ) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();
        if !document.select_range(start_node, start_offset, end_node, end_offset) {
            return Err(EditError::InvalidRange);
        }
        Ok(())
    }

    /// Collapses the current selection to its start position
    pub fn collapse_selection_to_start(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();
        if !document.collapse_selection_to_start() {
            return Err(EditError::Other("No selection to collapse".to_string()));
        }
        Ok(())
    }

    /// Collapses the current selection to its end position
    pub fn collapse_selection_to_end(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();
        if !document.collapse_selection_to_end() {
            return Err(EditError::Other("No selection to collapse".to_string()));
        }
        Ok(())
    }

    /// Clears the current selection
    pub fn clear_selection(&mut self) {
        let mut document = self.document.borrow_mut();
        document.clear_selection();
    }

    /// Returns whether there is currently a selection
    pub fn has_selection(&self) -> bool {
        let document = self.document.borrow();
        document.has_selection()
    }

    /// Returns whether the current selection spans multiple nodes
    pub fn has_multi_node_selection(&self) -> bool {
        let document = self.document.borrow();
        document.has_multi_node_selection()
    }

    /// Gets the currently selected text, if any
    pub fn get_selected_text(&self) -> Option<String> {
        let document = self.document.borrow();
        document.get_selected_text()
    }

    /// Begin a transaction to group multiple operations into a single atomic change.
    ///
    /// Returns a Transaction object that can be used to build up a series of operations.
    /// The transaction is not applied to the document until it is committed and executed.
    ///
    /// # Example
    /// ```
    /// # use md_core::{Document, Editor, EditError};
    /// # fn example() -> Result<(), EditError> {
    /// # let doc = Document::new();
    /// # let mut editor = Editor::new(doc);
    /// let mut transaction = editor.begin_transaction();
    /// transaction.insert_text(0, 0, "Hello");
    /// editor.execute_transaction(transaction)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_transaction(&self) -> Transaction {
        Transaction::new(self.document.clone())
    }

    /// Execute a transaction with a provided closure that builds the transaction.
    ///
    /// # Example
    /// ```
    /// # use md_core::{Document, Editor, TextFormatting, EditError};
    /// # fn example() -> Result<(), EditError> {
    /// # let doc = Document::new();
    /// # let mut editor = Editor::new(doc);
    /// editor.with_transaction(|mut transaction| {
    ///     transaction
    ///         .insert_text(0, 0, "Hello")
    ///         .format_text(0, 0, 5, TextFormatting {
    ///             bold: true,
    ///             ..Default::default()
    ///         });
    ///     transaction
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_transaction<F>(&mut self, transaction_builder: F) -> Result<(), EditError>
    where
        F: FnOnce(Transaction) -> Transaction,
    {
        // Create a new transaction
        let transaction = Transaction::new(self.document.clone());

        // Let the closure build the transaction
        let transaction = transaction_builder(transaction);

        // Execute the transaction
        self.execute_transaction(transaction)
    }

    /// Execute a transaction, updating the undo stack as a single composite command.
    ///
    /// This method commits the transaction and applies the changes to the document.
    pub fn execute_transaction(&mut self, transaction: Transaction) -> Result<(), EditError> {
        // Commit the transaction
        let commands = transaction.commit()?;

        // Execute the committed commands
        self.execute_transaction_commands(commands)
    }

    /// Execute a list of commands from a transaction and add to undo stack.
    ///
    /// This is a lower-level method that's used by execute_transaction.
    pub fn execute_transaction_commands(
        &mut self,
        commands: Vec<Box<dyn EditorCommand>>,
    ) -> Result<(), EditError> {
        // If there are no commands, nothing to do
        if commands.is_empty() {
            return Ok(());
        }

        // Create a composite command that represents all commands as one operation
        let composite = CompositeCommand::new(commands);

        // Add to undo stack
        self.undo_stack.push(Box::new(composite));

        // Clear redo stack since we executed a new command
        self.redo_stack.clear();

        // Trim history if needed
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }

        Ok(())
    }

    /// Set the background color of a table cell
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row`: The row index of the cell (0 is the first row after the header)
    /// - `column`: The column index of the cell (0 is the first column)
    /// - `color`: The background color in hex format (e.g., "#f5f5f5")
    /// - `is_header`: Whether to modify a header cell or a body cell
    pub fn set_table_cell_background(
        &mut self,
        node_index: usize,
        row: usize,
        column: usize,
        color: impl Into<String>,
        is_header: bool,
    ) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::SetCellBackground {
                row,
                column,
                color: color.into(),
                is_header,
            },
        ));
        self.execute_command(command)
    }

    /// Set custom CSS style for a table cell
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row`: The row index of the cell (0 is the first row after the header)
    /// - `column`: The column index of the cell (0 is the first column)
    /// - `style`: CSS style string (e.g., "font-weight: bold; color: red;")
    /// - `is_header`: Whether to modify a header cell or a body cell
    pub fn set_table_cell_style(
        &mut self,
        node_index: usize,
        row: usize,
        column: usize,
        style: impl Into<String>,
        is_header: bool,
    ) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::SetCellStyle {
                row,
                column,
                style: style.into(),
                is_header,
            },
        ));
        self.execute_command(command)
    }

    /// Set the spanning of a table cell
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row`: The row index of the cell (0 is the first row after the header)
    /// - `column`: The column index of the cell (0 is the first column)
    /// - `colspan`: Number of columns this cell should span
    /// - `rowspan`: Number of rows this cell should span
    /// - `is_header`: Whether to modify a header cell or a body cell
    pub fn set_table_cell_span(
        &mut self,
        node_index: usize,
        row: usize,
        column: usize,
        colspan: u32,
        rowspan: u32,
        is_header: bool,
    ) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::SetCellSpan {
                row,
                column,
                colspan,
                rowspan,
                is_header,
            },
        ));
        self.execute_command(command)
    }

    /// Set table properties
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `properties`: The table properties to set
    pub fn set_table_properties(
        &mut self,
        node_index: usize,
        properties: TableProperties,
    ) -> Result<(), EditError> {
        let command = Box::new(TableOperationsCommand::new(
            self.document.clone(),
            node_index,
            TableOperation::SetTableProperties(properties),
        ));
        self.execute_command(command)
    }

    /// Toggle the checked status of a task list item
    pub fn toggle_task(&mut self, node_index: usize, item_index: usize) -> Result<(), EditError> {
        let command = Box::new(ToggleTaskCommand::new(
            self.document.clone(),
            node_index,
            item_index,
        ));
        self.execute_command(command)
    }

    /// Add a new item to a task list
    pub fn add_task_item(
        &mut self,
        node_index: usize,
        position: usize,
        text: impl Into<String>,
        checked: bool,
    ) -> Result<(), EditError> {
        let command = Box::new(AddTaskItemCommand::new(
            self.document.clone(),
            node_index,
            position,
            text,
            checked,
        ));
        self.execute_command(command)
    }

    /// Remove an item from a task list
    pub fn remove_task_item(
        &mut self,
        node_index: usize,
        item_index: usize,
    ) -> Result<(), EditError> {
        let command = Box::new(RemoveTaskItemCommand::new(
            self.document.clone(),
            node_index,
            item_index,
        ));
        self.execute_command(command)
    }

    /// Edit the text content of a task list item
    pub fn edit_task_item(
        &mut self,
        node_index: usize,
        item_index: usize,
        text: impl Into<String>,
    ) -> Result<(), EditError> {
        let command = Box::new(EditTaskItemCommand::new(
            self.document.clone(),
            node_index,
            item_index,
            text,
        ));
        self.execute_command(command)
    }

    /// Move a task item from one position to another within the same task list
    ///
    /// This method allows reordering items within a task list by moving a task item
    /// from its current position to a new position.
    ///
    /// # Arguments
    ///
    /// * `node_index` - The index of the task list node in the document
    /// * `from_index` - The current index of the task item to move
    /// * `to_index` - The destination index for the task item
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful
    /// * `Err` with appropriate error if the operation failed (e.g., indices out of bounds)
    ///
    /// # Example
    ///
    /// ```
    /// use md_core::{Document, Editor, ListItem, ListType, Node};
    ///
    /// // Create a document with a task list
    /// let mut document = Document::new();
    /// let items = vec![
    ///     ListItem::task("Task 1", false),
    ///     ListItem::task("Task 2", true),
    ///     ListItem::task("Task 3", false),
    /// ];
    /// document.nodes.push(Node::List {
    ///     list_type: ListType::Task,
    ///     items,
    /// });
    ///
    /// let mut editor = Editor::new(document);
    ///
    /// // Move the first task item to the end of the list
    /// editor.move_task_item(0, 0, 2).unwrap();
    /// ```
    pub fn move_task_item(
        &mut self,
        node_index: usize,
        from_index: usize,
        to_index: usize,
    ) -> Result<(), EditError> {
        let command = Box::new(MoveTaskItemCommand::new(
            self.document.clone(),
            node_index,
            from_index,
            to_index,
        ));
        self.execute_command(command)
    }
}

/// A command that groups multiple commands together as a single undo/redo unit
struct CompositeCommand {
    commands: Vec<Box<dyn EditorCommand>>,
}

impl CompositeCommand {
    fn new(commands: Vec<Box<dyn EditorCommand>>) -> Self {
        Self { commands }
    }
}

impl EditorCommand for CompositeCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        // All commands should already be executed by the transaction
        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        // Undo in reverse order
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo()?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
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

        // Verify changes after deletion
        {
            let doc = editor.document().borrow();
            match &doc.nodes[index] {
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

        // Verify content after undo
        {
            let doc = editor.document().borrow();
            match &doc.nodes[index] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "Hello, world!");
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
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

        // Verify after merge
        {
            let doc = editor.document().borrow();
            // Should now have only one paragraph
            assert_eq!(doc.nodes.len(), 1);

            match &doc.nodes[0] {
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

        // Verify after undo
        {
            let doc = editor.document().borrow();
            // Should be back to two paragraphs
            assert_eq!(doc.nodes.len(), 2);
        }
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

        // Verify changes after formatting
        {
            let doc = editor.document().borrow();
            // Should now have three text nodes: before, formatted, after
            match &doc.nodes[index] {
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

        // Verify after undo
        {
            let doc = editor.document().borrow();
            // Should be back to one text node
            match &doc.nodes[index] {
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

        // Verify changes after deletion
        {
            let doc = editor.document().borrow();
            // Should now have only one paragraph
            assert_eq!(doc.nodes.len(), 1);

            match &doc.nodes[0] {
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

        // Verify after undo
        {
            let doc = editor.document().borrow();
            // Should be back to two paragraphs
            assert_eq!(doc.nodes.len(), 2);
        }
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

        // Verify changes after move
        {
            let doc = editor.document().borrow();
            // Order should now be: Second, Third, First
            match &doc.nodes[2] {
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

        // Verify after undo
        {
            let doc = editor.document().borrow();
            // Should be back to original order
            match &doc.nodes[0] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "First paragraph.");
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
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

        // Verify changes after conversion
        {
            let doc = editor.document().borrow();
            // Should now be a heading
            match &doc.nodes[index] {
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

        // Verify after undo
        {
            let doc = editor.document().borrow();
            // Should be back to a paragraph
            match &doc.nodes[index] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "This is a paragraph");
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
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

        // Verify changes after insertion
        {
            let doc = editor.document().borrow();
            match &doc.nodes[index] {
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

        // Verify after undo
        {
            let doc = editor.document().borrow();
            match &doc.nodes[index] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "Hello world!");
                    }
                    _ => panic!("Expected Text node"),
                },
                _ => panic!("Expected Paragraph node"),
            }
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

        // Verify changes after insertion
        {
            let doc = editor.document().borrow();
            // Should now have two nodes
            assert_eq!(doc.nodes.len(), 2);

            // Check if the new node is a heading with the right content
            match &doc.nodes[1] {
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

        // Verify after undo
        {
            let doc = editor.document().borrow();
            // Should be back to one paragraph
            assert_eq!(doc.nodes.len(), 1);
        }
    }

    #[test]
    fn test_duplicate_node() {
        let mut doc = Document::new();
        let p1 = doc.add_paragraph_with_text("This is a paragraph to duplicate.");

        let mut editor = Editor::new(doc);

        // Duplicate the paragraph
        let result = editor.duplicate_node(p1);
        assert!(result.is_ok());

        // Verify changes after duplication
        {
            let doc = editor.document().borrow();
            // Should now have two identical paragraphs
            assert_eq!(doc.nodes.len(), 2);

            // Check that both paragraphs have the same text
            match (&doc.nodes[0], &doc.nodes[1]) {
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

        // Verify after undo
        {
            let doc = editor.document().borrow();
            // Should be back to one paragraph
            assert_eq!(doc.nodes.len(), 1);
        }
    }
}
