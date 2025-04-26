use md_core::{Document, DocumentBuilder, Html, Json, Markdown, Text};
use std::convert::TryInto;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a new document using the builder pattern
    let doc = DocumentBuilder::new()
        .title("DocumentBuilder Example")
        .author("Markdown Editor Team")
        .date("2023-05-01")
        // Add metadata
        .metadata("category", "tutorial")
        .metadata("language", "en-US")
        // Add content
        .heading(2, "Introduction")
        .paragraph("This example demonstrates the use of DocumentBuilder for creating markdown documents with a fluent API.")
        .code_block(r#"let doc = DocumentBuilder::new()
    .title("Example")
    .paragraph("Hello world")
    .build();"#, "rust")
        .heading(2, "Features")
        .paragraph("The builder pattern offers several advantages:")
        .unordered_list(vec![
            "Method chaining for concise document creation",
            "Fluent API for better readability",
            "Logical document structure flows naturally in code",
            "Self-documenting code with clear intent",
        ])
        .heading(2, "Example with Groups")
        .paragraph("Groups can be used to organize content:")
        .group("Note", |builder| {
            builder
                .paragraph("This is content inside a group.")
                .unordered_list(vec![
                    "Groups can contain any document elements",
                    "They help organize related content",
                ])
        })
        .heading(2, "Tables")
        .paragraph("You can easily create tables:")
        .table(
            vec!["Feature", "Description"],
            vec![
                vec!["Builder Pattern", "Chain method calls for fluent API"],
                vec!["Groups", "Organize related content together"],
                vec!["Tables", "Create structured tabular data"],
            ],
        )
        .build();

    // Convert to different formats
    let markdown: Text<Markdown> = (&doc).try_into()?;
    let html: Text<Html> = (&doc).try_into()?;
    let json: Text<Json> = (&doc).try_into()?;

    // Print the document in different formats
    println!("=== MARKDOWN ===\n{}\n", markdown);
    println!("=== HTML ===\n{}\n", html);
    println!("=== JSON ===\n{}\n", json);

    // Compare with traditional creation
    println!("=== TRADITIONAL API ===");
    let mut traditional_doc = Document::with_title("Traditional API Example");
    traditional_doc.add_paragraph_with_text("This demonstrates the traditional API.");
    traditional_doc.add_code_block("// Some code", "javascript");

    let traditional_markdown: Text<Markdown> = (&traditional_doc).try_into()?;
    println!("{}", traditional_markdown);

    Ok(())
}
