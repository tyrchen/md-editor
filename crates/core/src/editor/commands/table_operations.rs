use crate::editor::command::Command;
use crate::{Document, EditError, Node, TableAlignment, TableCell, TableProperties};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Types of table operations that can be performed
pub enum TableOperation {
    /// Add a row at the specified index (0 is first row after header)
    AddRow(usize),
    /// Remove the row at the specified index
    RemoveRow(usize),
    /// Add a column at the specified index (0 is first column)
    AddColumn(usize),
    /// Remove the column at the specified index
    RemoveColumn(usize),
    /// Change cell content at specified row and column
    SetCell {
        row: usize,
        column: usize,
        content: String,
        is_header: bool,
    },
    /// Set column alignment
    SetAlignment {
        column: usize,
        alignment: TableAlignment,
    },
    /// Set cell background color
    SetCellBackground {
        row: usize,
        column: usize,
        color: String,
        is_header: bool,
    },
    /// Set cell style
    SetCellStyle {
        row: usize,
        column: usize,
        style: String,
        is_header: bool,
    },
    /// Set cell span
    SetCellSpan {
        row: usize,
        column: usize,
        colspan: u32,
        rowspan: u32,
        is_header: bool,
    },
    /// Set table properties
    SetTableProperties(TableProperties),
}

/// Command to perform operations on an existing table
pub struct TableOperationsCommand {
    document: Rc<RefCell<Document>>,
    /// The index of the table node in the document
    node_index: usize,
    /// The operation to perform
    operation: TableOperation,
    /// Original node for undo
    original_node: Option<Node>,
}

impl TableOperationsCommand {
    /// Create a new table operations command
    pub fn new(
        document: Rc<RefCell<Document>>,
        node_index: usize,
        operation: TableOperation,
    ) -> Self {
        Self {
            document,
            node_index,
            operation,
            original_node: None,
        }
    }
}

impl Command for TableOperationsCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Check if node_index is valid
        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Check if the node is a table
        if !matches!(document.nodes[self.node_index], Node::Table { .. }) {
            return Err(EditError::Other("Node is not a table".to_string()));
        }

        // Store original node for undo
        self.original_node = Some(document.nodes[self.node_index].clone());

        // Apply the operation
        match &mut document.nodes[self.node_index] {
            Node::Table {
                header,
                rows,
                alignments,
                properties,
            } => {
                match &self.operation {
                    TableOperation::AddRow(index) => {
                        let rows_len = rows.len();
                        let row_index = index.min(&rows_len);
                        let num_columns = alignments.len();
                        let empty_row = (0..num_columns)
                            .map(|i| {
                                TableCell::text(format!("Row {}, Col {}", rows.len() + 1, i + 1))
                            })
                            .collect();
                        rows.insert(*row_index, empty_row);
                    }
                    TableOperation::RemoveRow(index) => {
                        if *index < rows.len() {
                            rows.remove(*index);
                        } else {
                            return Err(EditError::IndexOutOfBounds);
                        }
                    }
                    TableOperation::AddColumn(index) => {
                        let alignments_len = alignments.len();
                        let col_index = index.min(&alignments_len);

                        // Add column to header if present
                        if !header.is_empty() {
                            header.insert(
                                *col_index,
                                TableCell::text(format!("Column {}", alignments.len() + 1)),
                            );
                        }

                        // Add column to each row
                        for (i, row) in rows.iter_mut().enumerate() {
                            row.insert(
                                *col_index,
                                TableCell::text(format!(
                                    "Row {}, Col {}",
                                    i + 1,
                                    alignments.len() + 1
                                )),
                            );
                        }

                        // Add alignment for the new column
                        alignments.insert(*col_index, TableAlignment::default());
                    }
                    TableOperation::RemoveColumn(index) => {
                        if *index < alignments.len() {
                            // Remove from header if present
                            if !header.is_empty() {
                                header.remove(*index);
                            }

                            // Remove from each row
                            for row in rows.iter_mut() {
                                if *index < row.len() {
                                    row.remove(*index);
                                }
                            }

                            // Remove alignment
                            alignments.remove(*index);
                        } else {
                            return Err(EditError::IndexOutOfBounds);
                        }
                    }
                    TableOperation::SetCell {
                        row,
                        column,
                        content,
                        is_header,
                    } => {
                        if *is_header {
                            // Modify header cell
                            if !header.is_empty() && *column < header.len() {
                                header[*column] = TableCell::text(content);
                            } else {
                                return Err(EditError::IndexOutOfBounds);
                            }
                        } else {
                            // Modify body cell
                            if *row < rows.len() && *column < rows[*row].len() {
                                rows[*row][*column] = TableCell::text(content);
                            } else {
                                return Err(EditError::IndexOutOfBounds);
                            }
                        }
                    }
                    TableOperation::SetAlignment { column, alignment } => {
                        if *column < alignments.len() {
                            alignments[*column] = alignment.clone();
                        } else {
                            return Err(EditError::IndexOutOfBounds);
                        }
                    }
                    TableOperation::SetCellBackground {
                        row,
                        column,
                        color,
                        is_header,
                    } => {
                        if *is_header {
                            // Modify header cell
                            if !header.is_empty() && *column < header.len() {
                                header[*column].background_color = Some(color.clone());
                            } else {
                                return Err(EditError::IndexOutOfBounds);
                            }
                        } else {
                            // Modify body cell
                            if *row < rows.len() && *column < rows[*row].len() {
                                rows[*row][*column].background_color = Some(color.clone());
                            } else {
                                return Err(EditError::IndexOutOfBounds);
                            }
                        }
                    }
                    TableOperation::SetCellStyle {
                        row,
                        column,
                        style,
                        is_header,
                    } => {
                        if *is_header {
                            // Modify header cell
                            if !header.is_empty() && *column < header.len() {
                                header[*column].style = Some(style.clone());
                            } else {
                                return Err(EditError::IndexOutOfBounds);
                            }
                        } else {
                            // Modify body cell
                            if *row < rows.len() && *column < rows[*row].len() {
                                rows[*row][*column].style = Some(style.clone());
                            } else {
                                return Err(EditError::IndexOutOfBounds);
                            }
                        }
                    }
                    TableOperation::SetCellSpan {
                        row,
                        column,
                        colspan,
                        rowspan,
                        is_header,
                    } => {
                        if *is_header {
                            // Modify header cell
                            if !header.is_empty() && *column < header.len() {
                                header[*column].colspan = *colspan;
                                header[*column].rowspan = *rowspan;
                            } else {
                                return Err(EditError::IndexOutOfBounds);
                            }
                        } else {
                            // Modify body cell
                            if *row < rows.len() && *column < rows[*row].len() {
                                rows[*row][*column].colspan = *colspan;
                                rows[*row][*column].rowspan = *rowspan;
                            } else {
                                return Err(EditError::IndexOutOfBounds);
                            }
                        }
                    }
                    TableOperation::SetTableProperties(new_properties) => {
                        *properties = new_properties.clone();
                    }
                }
            }
            _ => unreachable!(), // We already checked this is a table
        }

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(original_node) = self.original_node.take() {
            let mut document = self.document.borrow_mut();
            if self.node_index < document.nodes.len() {
                document.nodes[self.node_index] = original_node;
                Ok(())
            } else {
                Err(EditError::IndexOutOfBounds)
            }
        } else {
            Err(EditError::Other("No original state to restore".to_string()))
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
    fn test_add_row() {
        let mut doc = Document::new();

        // Create a simple 2x2 table with header
        let header = vec![TableCell::text("H1"), TableCell::text("H2")];
        let rows = vec![vec![TableCell::text("R1C1"), TableCell::text("R1C2")]];
        let alignments = vec![TableAlignment::default(), TableAlignment::default()];

        let table_node = Node::Table {
            header,
            rows,
            alignments,
            properties: TableProperties::default(),
        };

        doc.nodes.push(table_node);

        let document_rc = Rc::new(RefCell::new(doc));

        // Add a row at the end
        let mut cmd =
            TableOperationsCommand::new(document_rc.clone(), 0, TableOperation::AddRow(1));

        // Execute the command
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check the table
        let doc = document_rc.borrow();
        match &doc.nodes[0] {
            Node::Table {
                header: _,
                rows,
                alignments: _,
                properties: _,
            } => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[1].len(), 2);
            }
            _ => panic!("Expected Table node"),
        }

        // Test undo
        drop(doc);
        let result = cmd.undo();
        assert!(result.is_ok());

        // Verify original state is restored
        let doc = document_rc.borrow();
        match &doc.nodes[0] {
            Node::Table {
                header: _,
                rows,
                alignments: _,
                properties: _,
            } => {
                assert_eq!(rows.len(), 1);
            }
            _ => panic!("Expected Table node"),
        }
    }

    #[test]
    fn test_add_column() {
        let mut doc = Document::new();

        // Create a simple 2x2 table with header
        let header = vec![TableCell::text("H1"), TableCell::text("H2")];
        let rows = vec![vec![TableCell::text("R1C1"), TableCell::text("R1C2")]];
        let alignments = vec![TableAlignment::default(), TableAlignment::default()];

        let table_node = Node::Table {
            header,
            rows,
            alignments,
            properties: TableProperties::default(),
        };

        doc.nodes.push(table_node);

        let document_rc = Rc::new(RefCell::new(doc));

        // Add a column in the middle
        let mut cmd =
            TableOperationsCommand::new(document_rc.clone(), 0, TableOperation::AddColumn(1));

        // Execute the command
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check the table
        let doc = document_rc.borrow();
        match &doc.nodes[0] {
            Node::Table {
                header,
                rows,
                alignments,
                properties: _,
            } => {
                assert_eq!(header.len(), 3);
                assert_eq!(rows[0].len(), 3);
                assert_eq!(alignments.len(), 3);
                assert_eq!(header[1].content[0].as_text().unwrap(), "Column 3");
            }
            _ => panic!("Expected Table node"),
        }
    }

    #[test]
    fn test_set_cell_content() {
        let mut doc = Document::new();

        // Create a simple 2x2 table with header
        let header = vec![TableCell::text("H1"), TableCell::text("H2")];
        let rows = vec![vec![TableCell::text("R1C1"), TableCell::text("R1C2")]];
        let alignments = vec![TableAlignment::default(), TableAlignment::default()];

        let table_node = Node::Table {
            header,
            rows,
            alignments,
            properties: TableProperties::default(),
        };

        doc.nodes.push(table_node);

        let document_rc = Rc::new(RefCell::new(doc));

        // Set a cell's content
        let mut cmd = TableOperationsCommand::new(
            document_rc.clone(),
            0,
            TableOperation::SetCell {
                row: 0,
                column: 1,
                content: "Updated".to_string(),
                is_header: false,
            },
        );

        // Execute the command
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check the table
        let doc = document_rc.borrow();
        match &doc.nodes[0] {
            Node::Table {
                header: _,
                rows,
                alignments: _,
                properties: _,
            } => {
                assert_eq!(rows[0][1].content[0].as_text().unwrap(), "Updated");
            }
            _ => panic!("Expected Table node"),
        }
    }

    #[test]
    fn test_set_alignment() {
        let mut doc = Document::new();

        // Create a simple 2x2 table with header
        let header = vec![TableCell::text("H1"), TableCell::text("H2")];
        let rows = vec![vec![TableCell::text("R1C1"), TableCell::text("R1C2")]];
        let alignments = vec![TableAlignment::default(), TableAlignment::default()];

        let table_node = Node::Table {
            header,
            rows,
            alignments,
            properties: TableProperties::default(),
        };

        doc.nodes.push(table_node);

        let document_rc = Rc::new(RefCell::new(doc));

        // Set alignment
        let mut cmd = TableOperationsCommand::new(
            document_rc.clone(),
            0,
            TableOperation::SetAlignment {
                column: 1,
                alignment: TableAlignment::Center,
            },
        );

        // Execute the command
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check the table
        let doc = document_rc.borrow();
        match &doc.nodes[0] {
            Node::Table {
                header: _,
                rows: _,
                alignments,
                properties: _,
            } => {
                assert_eq!(alignments[1], TableAlignment::Center);
            }
            _ => panic!("Expected Table node"),
        }
    }
}
