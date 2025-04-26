use md_core::{Document, Editor, Markdown, Text, TextFormatting};
use std::convert::TryInto;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a new empty document
    let doc = Document::new();
    let mut editor = Editor::new(doc);

    println!("=== TRANSACTION API EXAMPLE ===\n");

    // Example 1: Simple transaction
    println!("=== EXAMPLE 1: SIMPLE TRANSACTION ===");

    // Begin a transaction
    let mut transaction = editor.begin_transaction();

    transaction
        .insert_heading(0, 1, "Transaction API Example")
        .insert_paragraph(
            1,
            "This example demonstrates using transactions for grouped operations.",
        );

    // Commit the transaction (all operations succeed or fail together)
    editor.execute_transaction(transaction)?;

    // Print the document
    print_document(&editor, "After simple transaction")?;

    // Example 2: Complex formatting transaction
    println!("\n=== EXAMPLE 2: FORMATTING TRANSACTION ===");

    // Create a more complex document with multiple edits in one atomic operation
    let mut transaction = editor.begin_transaction();

    transaction
        .insert_paragraph(
            2,
            "Normal text, followed by bold, italic, and code formatting.",
        )
        .select_text_range(2, 20, 24) // Select "bold"
        .format_selection(TextFormatting {
            bold: true,
            ..Default::default()
        })
        .select_text_range(2, 26, 32) // Select "italic"
        .format_selection(TextFormatting {
            italic: true,
            ..Default::default()
        })
        .select_text_range(2, 38, 42) // Select "code"
        .format_selection(TextFormatting {
            code: true,
            ..Default::default()
        })
        .clear_selection();

    // Commit the transaction
    editor.execute_transaction(transaction)?;

    // Print the document
    print_document(&editor, "After formatting transaction")?;

    // Example 3: Transaction with undo
    println!("\n=== EXAMPLE 3: TRANSACTION WITH UNDO ===");

    // Create another transaction with multiple operations
    let mut transaction = editor.begin_transaction();

    transaction
        .insert_code_block(3, "let x = 42;\nconsole.log(x);", "javascript")
        .insert_paragraph(4, "This paragraph will be added and then undone.");

    // Commit the transaction
    editor.execute_transaction(transaction)?;

    // Print the document
    print_document(&editor, "After third transaction")?;

    // Undo the entire transaction as one operation
    editor.undo()?;

    // Print the document
    print_document(&editor, "After undo")?;

    // Example 4: Error handling and rollback
    println!("\n=== EXAMPLE 4: ERROR HANDLING AND ROLLBACK ===");
    println!("Attempting a transaction with an invalid operation...");

    // Start a transaction with an operation that will fail
    let mut transaction = editor.begin_transaction();

    transaction
        .insert_paragraph(3, "This will succeed...")
        .delete_node(99); // This will fail - no such node

    // Try to commit - should fail and roll back automatically
    match editor.execute_transaction(transaction) {
        Ok(_) => println!("Transaction succeeded (unexpected)"),
        Err(e) => println!("Transaction failed as expected: {:?}", e),
    }

    // Verify the document wasn't changed
    print_document(&editor, "After failed transaction")?;

    // Example 5: Automatic transaction discard
    println!("\n=== EXAMPLE 5: TRANSACTION DISCARD ===");

    {
        // Start a transaction in a new scope
        let mut transaction = editor.begin_transaction();
        transaction.insert_paragraph(
            3,
            "This text won't be added because the transaction is dropped.",
        );

        // Transaction goes out of scope without being committed
        println!("Transaction created but not committed...");
    }

    // Verify the document wasn't changed
    print_document(&editor, "After discarded transaction")?;

    // Example 6: Explicit transaction discard
    println!("\n=== EXAMPLE 6: EXPLICIT DISCARD ===");

    // Start a transaction
    let mut transaction = editor.begin_transaction();

    transaction.insert_paragraph(3, "This paragraph won't be added.");

    // Explicitly discard the transaction
    transaction.discard();
    println!("Transaction explicitly discarded");

    // Verify the document wasn't changed
    print_document(&editor, "After explicitly discarded transaction")?;

    Ok(())
}

fn print_document(editor: &Editor, description: &str) -> Result<(), Box<dyn Error>> {
    let doc = editor.document().borrow();
    let markdown: Text<Markdown> = (&*doc).try_into()?;

    println!("\n{}", description);
    println!("Document structure:");
    println!("{}", doc.debug_structure());
    println!("\nMarkdown content:");
    println!("{}", markdown);

    Ok(())
}
