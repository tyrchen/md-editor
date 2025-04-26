use crate::editor::command::Command;
use crate::{Document, EditError, Node, TableAlignment, TableCell, TableProperties};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Command to create a table
#[allow(dead_code)]
pub struct CreateTableCommand {
    document: Rc<RefCell<Document>>,
    position: usize,
    columns: usize,
    rows: usize,
    alignments: Option<Vec<TableAlignment>>,
    header_data: Option<Vec<String>>,
    row_data: Option<Vec<Vec<String>>>,
    properties: TableProperties,
    old_node: Option<Node>,
}

impl CreateTableCommand {
    /// Create a new table with default alignments
    pub fn new(
        document: Rc<RefCell<Document>>,
        position: usize,
        columns: usize,
        rows: usize,
    ) -> Self {
        Self {
            document,
            position,
            columns,
            rows,
            alignments: None,
            header_data: None,
            row_data: None,
            properties: TableProperties::default(),
            old_node: None,
        }
    }

    /// Create a new table with specific alignments
    pub fn with_alignments(
        document: Rc<RefCell<Document>>,
        position: usize,
        columns: usize,
        rows: usize,
        alignments: Vec<TableAlignment>,
    ) -> Self {
        Self {
            document,
            position,
            columns,
            rows,
            alignments: Some(alignments),
            header_data: None,
            row_data: None,
            properties: TableProperties::default(),
            old_node: None,
        }
    }

    /// Create a new table with specific data
    pub fn with_data(
        document: Rc<RefCell<Document>>,
        position: usize,
        header: Vec<String>,
        rows: Vec<Vec<String>>,
        alignments: Option<Vec<TableAlignment>>,
    ) -> Self {
        Self {
            document,
            position,
            columns: header.len(),
            rows: rows.len(),
            alignments,
            header_data: Some(header),
            row_data: Some(rows),
            properties: TableProperties::default(),
            old_node: None,
        }
    }

    /// Create a new table with properties
    pub fn with_properties(
        document: Rc<RefCell<Document>>,
        position: usize,
        columns: usize,
        rows: usize,
        properties: TableProperties,
    ) -> Self {
        Self {
            document,
            position,
            columns,
            rows,
            alignments: None,
            header_data: None,
            row_data: None,
            properties,
            old_node: None,
        }
    }

    /// Create a new table with data and properties
    pub fn with_data_and_properties(
        document: Rc<RefCell<Document>>,
        position: usize,
        header: Vec<String>,
        rows: Vec<Vec<String>>,
        alignments: Option<Vec<TableAlignment>>,
        properties: TableProperties,
    ) -> Self {
        Self {
            document,
            position,
            columns: header.len(),
            rows: rows.len(),
            alignments,
            header_data: Some(header),
            row_data: Some(rows),
            properties,
            old_node: None,
        }
    }
}

impl Command for CreateTableCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.position > document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Create a new table node
        let mut header = Vec::new();
        let mut rows = Vec::new();
        let mut alignments = Vec::new();

        // Generate header cells
        if let Some(header_data) = &self.header_data {
            header = header_data
                .iter()
                .map(|h| TableCell::header(h.clone()))
                .collect();

            // Use provided alignments if they exist, otherwise use defaults
            if let Some(align) = &self.alignments {
                alignments = align.clone();
            } else {
                alignments = vec![TableAlignment::default(); self.columns];
            }
        } else {
            for i in 0..self.columns {
                header.push(TableCell::header(format!("Header {}", i + 1)));
                alignments.push(TableAlignment::default());
            }
        }

        // Generate rows and cells
        if let Some(row_data) = &self.row_data {
            rows = row_data
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| TableCell::text(cell.clone()))
                        .collect()
                })
                .collect();
        } else {
            for i in 0..self.rows {
                let mut row = Vec::new();
                for j in 0..self.columns {
                    row.push(TableCell::text(format!("Row {}, Col {}", i + 1, j + 1)));
                }
                rows.push(row);
            }
        }

        // Create the table node
        let table_node = Node::Table {
            header,
            rows,
            alignments,
            properties: self.properties.clone(),
        };

        // Store the old node if there is one
        if self.position < document.nodes.len() {
            self.old_node = Some(document.nodes[self.position].clone());
        }

        // Insert the new table node
        if self.position < document.nodes.len() {
            document.nodes[self.position] = table_node;
        } else {
            document.nodes.push(table_node);
        }

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if let Some(old_node) = self.old_node.take() {
            // Restore the old node
            document.nodes[self.position] = old_node;
        } else {
            // If there was no old node, remove the inserted node
            if self.position < document.nodes.len() {
                document.nodes.remove(self.position);
            }
        }

        Ok(())
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
        let mut cmd = CreateTableCommand::new(document_rc.clone(), 1, 3, 2);

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
                properties,
            } => {
                // Check header
                assert_eq!(header.len(), 3);
                assert_eq!(header[0].content[0].as_text().unwrap(), "Header 1");

                // Check rows
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 3);

                // Check alignments
                assert_eq!(alignments.len(), 3);
                assert_eq!(alignments[0], TableAlignment::None);

                // Check properties
                assert_eq!(*properties, TableProperties::default());
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
            vec!["Name".to_string(), "Age".to_string(), "Role".to_string()],
            data,
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
                properties,
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

                // Check properties
                assert_eq!(*properties, TableProperties::default());
            }
            _ => panic!("Expected Table node"),
        }
    }
}
