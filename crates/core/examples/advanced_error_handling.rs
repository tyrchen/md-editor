use md_core::{Document, Html, Markdown, ParseError};
use md_core::{Json, Text};
use std::cell::RefCell;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use std::rc::Rc;

// Define a custom application error type using thiserror
// This would typically be in your application crate, not in the library
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Document parsing failed: {0}")]
    Parse(#[from] ParseError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid document: {0}")]
    Validation(String),
}

// A struct representing a document manager
struct DocumentManager {
    documents: Vec<Document>,
}

impl DocumentManager {
    fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    // Load a document from JSON file
    #[allow(dead_code)]
    fn load_from_json(&mut self, path: &Path) -> Result<usize, AppError> {
        let content = fs::read_to_string(path)?; // Could return Io error
        let text = Text::<Json>::new(content);
        let doc = text.try_into()?; // Could return Parse error
        self.documents.push(doc);
        Ok(self.documents.len() - 1)
    }

    // Save a document to disk in various formats
    fn save_document(&self, index: usize, path: &Path, format: &str) -> Result<(), AppError> {
        if index >= self.documents.len() {
            return Err(AppError::Validation(format!(
                "Invalid document index: {}",
                index
            )));
        }

        let doc = &self.documents[index];

        let content = match format.to_lowercase().as_str() {
            "json" => Text::<Json>::try_from(doc)?.into_inner(),
            "md" | "markdown" => Text::<Markdown>::try_from(doc)?.into_inner(),
            "html" => Text::<Html>::try_from(doc)?.into_inner(),
            _ => return Err(AppError::Config(format!("Unsupported format: {}", format))),
        };

        fs::write(path, content)?;
        Ok(())
    }

    // Validates a document meets certain criteria
    fn validate_document(&self, index: usize) -> Result<(), AppError> {
        if index >= self.documents.len() {
            return Err(AppError::Validation(format!(
                "Invalid document index: {}",
                index
            )));
        }

        let doc = &self.documents[index];

        // Example validation logic
        if doc.nodes.is_empty() {
            return Err(AppError::Validation("Document cannot be empty".into()));
        }

        // Check document has a title (first node is a heading)
        match &doc.nodes[0] {
            md_core::Node::Heading { .. } => {}
            _ => return Err(AppError::Validation("First node must be a heading".into())),
        }

        Ok(())
    }
}

fn main() {
    println!("=== Advanced Error Handling Example ===\n");

    // Use RefCell to allow interior mutability for our examples
    let manager = Rc::new(RefCell::new(DocumentManager::new()));

    // Create a sample document to work with
    let doc = Document::with_title("Sample Document");
    manager.borrow_mut().documents.push(doc);

    // Example of handling various error types
    let examples = [
        // Valid operation - should succeed
        example_validate_document(Rc::clone(&manager), 0),
        // Invalid index - should fail with Validation error
        example_validate_document(Rc::clone(&manager), 99),
        // Invalid document - doesn't have a heading
        example_validate_empty_document(Rc::clone(&manager)),
        // Configuration error - unsupported format
        example_save_invalid_format(Rc::clone(&manager)),
        // Error converted from ParseError
        example_parse_invalid_json(),
    ];

    // Display results
    for (i, result) in examples.iter().enumerate() {
        println!("\nExample {}:", i + 1);
        match result {
            Ok(_) => println!("  Success!"),
            Err(e) => {
                println!("  Error: {}", e);
                let mut source = e.source();
                let mut depth = 1;
                while let Some(err) = source {
                    println!("  Cause {}: {}", depth, err);
                    source = err.source();
                    depth += 1;
                }
            }
        }
    }
}

// Example functions that demonstrate different error scenarios

fn example_validate_document(
    manager: Rc<RefCell<DocumentManager>>,
    index: usize,
) -> Result<(), AppError> {
    println!("Validating document at index {}...", index);
    manager.borrow().validate_document(index)
}

fn example_validate_empty_document(manager: Rc<RefCell<DocumentManager>>) -> Result<(), AppError> {
    println!("Validating empty document...");
    let empty_doc = Document::new();

    // Add empty document to the manager
    let empty_index = {
        let mut mgr = manager.borrow_mut();
        mgr.documents.push(empty_doc);
        mgr.documents.len() - 1
    };

    // Validate the empty document
    manager.borrow().validate_document(empty_index)
}

fn example_save_invalid_format(manager: Rc<RefCell<DocumentManager>>) -> Result<(), AppError> {
    println!("Attempting to save with invalid format...");
    manager
        .borrow()
        .save_document(0, Path::new("/tmp/doc.xyz"), "xyz")
}

fn example_parse_invalid_json() -> Result<(), AppError> {
    println!("Attempting to parse invalid JSON...");
    let invalid_json = r#"{ "invalid": true, "#;
    let text = Text::<Json>::new(invalid_json);
    let _doc: Document = text.try_into()?;
    Ok(())
}
