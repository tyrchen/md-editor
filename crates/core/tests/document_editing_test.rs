use std::{fs, path::Path};

use md_core::{DocumentBuilder, Editor, Markdown, Text, TextFormatting};
use std::convert::TryInto;

#[test]
fn test_document_editing_workflow() {
    // Step 1: Load the test.md file
    let input_path = Path::new("fixtures/test.md");
    let markdown_text = fs::read_to_string(input_path).expect("Failed to read input file");

    // Step 2: Parse it as a document
    let document = DocumentBuilder::from_markdown(&markdown_text)
        .expect("Failed to parse markdown")
        .build();
    let mut editor = Editor::new(document);

    // Step 3: Make multiple edits to the document

    // Add a heading
    let mut transaction = editor.begin_transaction();
    transaction.insert_heading(0, 1, "Integration Test for Markdown Editor");
    editor
        .execute_transaction(transaction)
        .expect("Failed to execute commands");

    // Add a paragraph
    let mut transaction = editor.begin_transaction();
    transaction.insert_paragraph(1, "This is a test paragraph added by the integration test.");
    editor
        .execute_transaction(transaction)
        .expect("Failed to execute commands");

    // Format some text
    let mut transaction = editor.begin_transaction();
    transaction.format_text(
        1,
        0,
        10,
        TextFormatting {
            bold: true,
            ..Default::default()
        },
    );
    editor
        .execute_transaction(transaction)
        .expect("Failed to execute commands");

    // Insert a code block
    let mut transaction = editor.begin_transaction();
    transaction.insert_code_block(3, "println!(\"Hello, World!\");", "rust");
    editor
        .execute_transaction(transaction)
        .expect("Failed to execute commands");

    // Save the intermediate state
    let intermediate_doc = {
        let doc = editor.document().borrow();
        let markdown: Text<Markdown> = (&*doc).try_into().expect("Failed to convert to markdown");
        markdown.to_string()
    };
    println!("INTERMEDIATE DOC: {}", intermediate_doc);

    // Step 4: Test undo functionality
    editor.undo().expect("Failed to undo");
    editor.undo().expect("Failed to undo");

    // Save the state after undos
    let after_undos = {
        let doc = editor.document().borrow();
        let markdown: Text<Markdown> = (&*doc).try_into().expect("Failed to convert to markdown");
        markdown.to_string()
    };

    // Step 5: Make additional changes
    let mut transaction = editor.begin_transaction();
    transaction.insert_paragraph(2, "This is another paragraph added after undos.");
    editor
        .execute_transaction(transaction)
        .expect("Failed to execute commands");

    // Step 6: Test undo again
    editor.undo().expect("Failed to undo");

    // Step 7: Test redo functionality
    editor.redo().expect("Failed to redo"); // Redo the undo operation
    // Only call redo as many times as we have operations on the redo stack
    if editor.redo().is_ok() {
        println!("Successfully redid second operation");
    }

    // Save the final state
    let final_doc = {
        let doc = editor.document().borrow();
        let markdown: Text<Markdown> = (&*doc).try_into().expect("Failed to convert to markdown");
        markdown.to_string()
    };

    // Step 8: Save the result to another file
    let output_path = Path::new("fixtures/test_edited.md");
    fs::write(output_path, &final_doc).expect("Failed to write output file");

    // Cleanup: Remove the output file
    // Uncomment this to clean up after the test if needed
    // fs::remove_file(output_path).expect("Failed to remove output file");

    // Assertions to verify test correctness
    println!("Checking if intermediate_doc contains expected text:");
    println!("Intermediate doc: {}", intermediate_doc);
    assert!(intermediate_doc.contains("Integration Test for Markdown Editor"));
    assert!(intermediate_doc.contains("test paragraph added by the integration test"));
    assert!(!after_undos.contains("println!(\"Hello, World!\")"));
    assert!(final_doc.contains("Integration Test for Markdown Editor"));
}
