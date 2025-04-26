use md_core::{Document, Json, ParseError, Text};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of handling JSON parsing errors
    println!("=== Testing error handling ===\n");

    // Attempt to parse invalid JSON
    let invalid_json = r#"{ "invalid": "json, missing closing brace"#;
    let text = Text::<Json>::new(invalid_json);
    match Document::try_from(text) {
        Ok(_) => println!("Parsing succeeded (unexpected)"),
        Err(err) => {
            println!("Caught error: {}", err);
            match err {
                ParseError::Json(msg) => println!("JSON error details: {}", msg),
                _ => println!("Unexpected error type"),
            }
        }
    }

    // Example of error propagation with ?
    println!("\n=== Testing error propagation ===\n");
    match process_document() {
        Ok(doc) => println!(
            "Document processed successfully:\n{}",
            doc.debug_structure()
        ),
        Err(err) => println!("Error during processing: {}", err),
    }

    // Example of creating custom errors
    let custom_error = create_custom_error("Custom error reason");
    println!("\n=== Custom error ===\n");
    println!("Custom error: {}", custom_error);

    Ok(())
}

// Function that demonstrates using the ? operator with ParseError
fn process_document() -> Result<Document, ParseError> {
    // This could be from a file, network request, etc.
    let json_data = r#"{
        "nodes": [
            {
                "type": "heading",
                "level": 1,
                "children": [
                    {
                        "type": "text",
                        "text": "Example Document",
                        "formatting": {
                            "bold": false,
                            "italic": false,
                            "strikethrough": false,
                            "code": false
                        }
                    }
                ]
            }
        ]
    }"#;

    // Parse the JSON string to create a Document
    let mut document: Document = Text::<Json>::new(json_data).try_into()?;

    // Try to add a paragraph
    document.add_paragraph_with_text("This paragraph was added after parsing");

    // Convert back to JSON (could also fail with ?-propagation)
    let _serialized = Text::<Json>::try_from(&document)?.into_inner();

    Ok(document)
}

// Function to create a custom ParseError
fn create_custom_error(reason: &str) -> ParseError {
    ParseError::Generic(format!("Failed to process document: {}", reason))
}
