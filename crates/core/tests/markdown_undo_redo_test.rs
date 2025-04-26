use md_core::{Document, Editor, Markdown, Text};
use std::{convert::TryFrom, convert::TryInto, fs, path::Path};

#[test]
fn test_markdown_undo_redo() {
    // Step 1: Load the test.md file
    let input_path = Path::new("fixtures/test.md");
    let markdown_text = fs::read_to_string(input_path).expect("Failed to read input file");

    // Step 2: Parse it as a document
    let markdown = Text::<Markdown>::new(markdown_text);
    let document = Document::try_from(markdown).expect("Failed to parse markdown");
    let mut editor = Editor::new(document);

    // Step 3: Make first edit - add a heading
    let mut transaction = editor.begin_transaction();
    transaction.insert_heading(0, 1, "Undo/Redo Test Heading");
    let commands = transaction.commit().expect("Failed to commit transaction");
    editor
        .execute_transaction_commands(commands)
        .expect("Failed to execute commands");

    // Save state after heading
    let after_heading = {
        let doc = editor.document().borrow();
        let markdown: Text<Markdown> = (&*doc).try_into().expect("Failed to convert to markdown");
        markdown.to_string()
    };
    println!("AFTER HEADING: {}", after_heading);

    // Step 4: Make second edit - add a paragraph
    let mut transaction = editor.begin_transaction();
    transaction.insert_paragraph(1, "This is a test paragraph for undo/redo functionality.");
    let commands = transaction.commit().expect("Failed to commit transaction");
    editor
        .execute_transaction_commands(commands)
        .expect("Failed to execute commands");

    // Save state after paragraph
    let after_paragraph = {
        let doc = editor.document().borrow();
        let markdown: Text<Markdown> = (&*doc).try_into().expect("Failed to convert to markdown");
        markdown.to_string()
    };
    println!("AFTER PARAGRAPH: {}", after_paragraph);

    // Step 5: Undo the paragraph addition
    editor.undo().expect("Failed to undo");
    let after_undo = {
        let doc = editor.document().borrow();
        let markdown: Text<Markdown> = (&*doc).try_into().expect("Failed to convert to markdown");
        markdown.to_string()
    };
    println!("AFTER UNDO: {}", after_undo);

    // Step 6: Redo the paragraph addition
    editor.redo().expect("Failed to redo");
    let after_redo = {
        let doc = editor.document().borrow();
        let markdown: Text<Markdown> = (&*doc).try_into().expect("Failed to convert to markdown");
        markdown.to_string()
    };
    println!("AFTER REDO: {}", after_redo);

    // Step 7: Make an additional edit - add a code block
    let mut transaction = editor.begin_transaction();
    transaction.insert_code_block(2, "println!(\"Testing undo/redo!\");", "rust");
    let commands = transaction.commit().expect("Failed to commit transaction");
    editor
        .execute_transaction_commands(commands)
        .expect("Failed to execute commands");

    // Get final state
    let final_state = {
        let doc = editor.document().borrow();
        let markdown: Text<Markdown> = (&*doc).try_into().expect("Failed to convert to markdown");
        markdown.to_string()
    };
    println!("FINAL STATE: {}", final_state);

    // Step 8: Save the result to another file
    let output_path = Path::new("fixtures/test_undo_redo.md");
    fs::write(output_path, &final_state).expect("Failed to write output file");

    // Verify results with assertions
    assert!(after_heading.contains("Undo/Redo Test Heading"));
    assert!(after_paragraph.contains("test paragraph for undo/redo functionality"));
    assert_eq!(after_undo, after_heading); // After undo, should be back to heading only
    assert!(final_state.contains("println!(\"Testing undo/redo!\")"));
}
