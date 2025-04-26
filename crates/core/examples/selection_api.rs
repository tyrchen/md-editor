use md_core::{Document, Markdown, Text};
use std::convert::TryInto;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a sample document
    let mut doc = Document::new();
    doc.add_heading(1, "Selection API Example");
    doc.add_paragraph_with_text("This example demonstrates the improved selection API.");
    doc.add_paragraph_with_text("You can easily select content in various ways.");
    doc.add_code_block("let selection = doc.select_all();", "rust");
    doc.add_paragraph_with_text("Use these methods to simplify handling selections!");

    println!("=== DOCUMENT STRUCTURE ===");
    println!("{}", doc.debug_structure());
    println!();

    // Example 1: Select a specific node
    println!("=== EXAMPLE 1: SELECT NODE ===");
    doc.select_node(1);
    print_selection_info(&doc, "Selected node 1 (first paragraph)")?;

    // Example 2: Select a text range within a node
    println!("\n=== EXAMPLE 2: SELECT TEXT RANGE ===");
    doc.select_text_range(1, 5, 13);
    print_selection_info(&doc, "Selected 'example' in the first paragraph")?;
    println!(
        "Selected text: {}",
        doc.get_selected_text().unwrap_or_default()
    );

    // Example 3: Select multiple nodes
    println!("\n=== EXAMPLE 3: SELECT NODE RANGE ===");
    doc.select_node_range(1, 3);
    print_selection_info(&doc, "Selected nodes 1-3 (paragraphs and code block)")?;

    // Example 4: Select all content
    println!("\n=== EXAMPLE 4: SELECT ALL ===");
    doc.select_all();
    print_selection_info(&doc, "Selected all content")?;

    // Example 5: Collapse selection
    println!("\n=== EXAMPLE 5: COLLAPSE SELECTION ===");
    doc.collapse_selection_to_start();
    print_selection_info(&doc, "Collapsed to start")?;

    // Example 6: Clear selection
    println!("\n=== EXAMPLE 6: CLEAR SELECTION ===");
    doc.clear_selection();
    println!("Has selection: {}", doc.has_selection());

    // Example 7: Select precise range across nodes
    println!("\n=== EXAMPLE 7: SELECT ACROSS NODES ===");
    doc.select_range(1, 10, 2, 15);
    print_selection_info(&doc, "Selected from node 1 to node 2")?;
    println!(
        "Is multi-node selection: {}",
        doc.has_multi_node_selection()
    );

    Ok(())
}

fn print_selection_info(doc: &Document, description: &str) -> Result<(), Box<dyn Error>> {
    println!("Operation: {}", description);

    if let Some(selection) = &doc.selection {
        println!(
            "Selection start: {:?}, offset: {}",
            selection.start.path, selection.start.offset
        );
        println!(
            "Selection end: {:?}, offset: {}",
            selection.end.path, selection.end.offset
        );
        println!("Is collapsed: {}", selection.is_collapsed);

        // Convert to markdown to see the document with selection
        let markdown: Text<Markdown> = doc.try_into()?;
        println!("\nDocument content:");
        println!("{}", markdown);
    } else {
        println!("No selection");
    }

    Ok(())
}
