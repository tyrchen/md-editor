use crate::editor::command::Command;
use crate::{Document, EditError, Node, TableAlignment, TableCell};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to create and insert a table in the document
pub struct CreateTableCommand {
    document: Rc<RefCell<Document>>,
    /// Position to insert the table
    position: usize,
    /// Number of columns
    columns: usize,
    /// Number of rows (not including header row)
    rows: usize,
    /// Whether to include a header row
    with_header: bool,
    /// Column alignments
    alignments: Vec<TableAlignment>,
    /// Optional data to populate the table with
    data: Option<Vec<Vec<String>>>,
    /// Original nodes for undo
    original_nodes: Option<Vec<Node>>,
}

impl CreateTableCommand {
    /// Create a new empty table command with default alignments
    pub fn new(
        document: Rc<RefCell<Document>>,
        position: usize,
        columns: usize,
        rows: usize,
        with_header: bool,
    ) -> Self {
        let alignments = vec![TableAlignment::default(); columns];
        Self {
            document,
            position,
            columns,
            rows,
            with_header,
            alignments,
            data: None,
            original_nodes: None,
        }
    }

    /// Create a new table command with specified alignments
    pub fn with_alignments(
        document: Rc<RefCell<Document>>,
        position: usize,
        columns: usize,
        rows: usize,
        with_header: bool,
        alignments: Vec<TableAlignment>,
    ) -> Self {
        let alignments = if alignments.len() != columns {
            vec![TableAlignment::default(); columns]
        } else {
            alignments
        };

        Self {
            document,
            position,
            columns,
            rows,
            with_header,
            alignments,
            data: None,
            original_nodes: None,
        }
    }

    /// Create a new table command with specified data
    pub fn with_data(
        document: Rc<RefCell<Document>>,
        position: usize,
        data: Vec<Vec<String>>,
        with_header: bool,
        alignments: Option<Vec<TableAlignment>>,
    ) -> Self {
        if data.is_empty() {
            return Self::new(document, position, 1, 1, with_header);
        }

        let rows = if with_header {
            data.len() - 1
        } else {
            data.len()
        };
        let columns = data[0].len();

        let alignments = match alignments {
            Some(a) if a.len() == columns => a,
            _ => vec![TableAlignment::default(); columns],
        };

        Self {
            document,
            position,
            columns,
            rows,
            with_header,
            alignments,
            data: Some(data),
            original_nodes: None,
        }
    }
}

impl Command for CreateTableCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        // Store original document state for undo
        self.original_nodes = Some(document.nodes.clone());

        // Ensure position is valid
        let position = self.position.min(document.nodes.len());

        // Create table header cells
        let header = if self.with_header {
            (0..self.columns)
                .map(|i| {
                    if let Some(data) = &self.data {
                        if !data.is_empty() && i < data[0].len() {
                            TableCell::text(&data[0][i])
                        } else {
                            TableCell::text(format!("Header {}", i + 1))
                        }
                    } else {
                        TableCell::text(format!("Header {}", i + 1))
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        // Create table rows
        let rows: Vec<Vec<TableCell>> = if let Some(data) = &self.data {
            // Use provided data to populate the table
            let start_idx = if self.with_header { 1 } else { 0 };
            data[start_idx..]
                .iter()
                .map(|row| row.iter().map(TableCell::text).collect::<Vec<TableCell>>())
                .collect()
        } else {
            // Create empty rows
            (0..self.rows)
                .map(|_| {
                    (0..self.columns)
                        .map(|i| TableCell::text(format!("Cell {}", i + 1)))
                        .collect::<Vec<TableCell>>()
                })
                .collect()
        };

        // Create table node
        let table_node = Node::Table {
            header,
            rows,
            alignments: self.alignments.clone(),
        };

        // Insert table into document
        document.nodes.insert(position, table_node);

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        if let Some(original_nodes) = self.original_nodes.take() {
            let mut document = self.document.borrow_mut();
            document.nodes = original_nodes;
            Ok(())
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
    use crate::InlineNode;

    #[test]
    fn test_create_empty_table() {
        let mut doc = Document::new();
        doc.add_paragraph_with_text("Test paragraph");

        let document_rc = Rc::new(RefCell::new(doc));
        let mut cmd = CreateTableCommand::new(document_rc.clone(), 1, 3, 2, true);

        // Execute the command
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check the table was created
        let doc = document_rc.borrow();
        assert_eq!(doc.nodes.len(), 2);

        // Check if the second node is a table
        match &doc.nodes[1] {
            Node::Table {
                header,
                rows,
                alignments,
            } => {
                // Check header
                assert_eq!(header.len(), 3);
                assert_eq!(header[0].content[0].as_text().unwrap(), "Header 1");

                // Check rows
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 3);
                assert_eq!(rows[0][0].content[0].as_text().unwrap(), "Cell 1");

                // Check alignments
                assert_eq!(alignments.len(), 3);
                assert_eq!(alignments[0], TableAlignment::None);
            }
            _ => panic!("Expected Table node"),
        }

        // Test undo
        drop(doc);
        let result = cmd.undo();
        assert!(result.is_ok());

        // Verify original state is restored
        let doc = document_rc.borrow();
        assert_eq!(doc.nodes.len(), 1);
        match &doc.nodes[0] {
            Node::Paragraph { children } => {
                if let InlineNode::Text(text_node) = &children[0] {
                    assert_eq!(text_node.text, "Test paragraph");
                } else {
                    panic!("Expected Text node");
                }
            }
            _ => panic!("Expected Paragraph node"),
        }
    }

    #[test]
    fn test_create_table_with_data() {
        let doc = Document::new();

        let document_rc = Rc::new(RefCell::new(doc));

        // Sample data with header row and two data rows
        let data = vec![
            vec!["Name".to_string(), "Age".to_string(), "Role".to_string()],
            vec![
                "Alice".to_string(),
                "30".to_string(),
                "Developer".to_string(),
            ],
            vec!["Bob".to_string(), "28".to_string(), "Designer".to_string()],
        ];

        let mut cmd = CreateTableCommand::with_data(
            document_rc.clone(),
            0,
            data,
            true,
            Some(vec![
                TableAlignment::Left,
                TableAlignment::Center,
                TableAlignment::Right,
            ]),
        );

        // Execute the command
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check the table was created
        let doc = document_rc.borrow();
        assert_eq!(doc.nodes.len(), 1);

        // Validate table contents
        match &doc.nodes[0] {
            Node::Table {
                header,
                rows,
                alignments,
            } => {
                // Check header
                assert_eq!(header.len(), 3);
                assert_eq!(header[0].content[0].as_text().unwrap(), "Name");
                assert_eq!(header[1].content[0].as_text().unwrap(), "Age");
                assert_eq!(header[2].content[0].as_text().unwrap(), "Role");

                // Check rows
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0][0].content[0].as_text().unwrap(), "Alice");
                assert_eq!(rows[0][1].content[0].as_text().unwrap(), "30");
                assert_eq!(rows[1][0].content[0].as_text().unwrap(), "Bob");

                // Check alignments
                assert_eq!(alignments.len(), 3);
                assert_eq!(alignments[0], TableAlignment::Left);
                assert_eq!(alignments[1], TableAlignment::Center);
                assert_eq!(alignments[2], TableAlignment::Right);
            }
            _ => panic!("Expected Table node"),
        }
    }
}
