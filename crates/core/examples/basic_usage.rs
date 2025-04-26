use md_core::{Document, Html, InlineNode, Json, Markdown, Node, Text, TextFormatting, TextNode};

fn main() {
    // Create a new document
    let mut doc = Document::with_title("My Markdown Document");

    // Add some content
    doc.add_paragraph_with_text("Welcome to the Markdown Editor Core! This is a simple example of how to use the core data structure.");

    // Add a code block
    doc.add_code_block(
        r#"
fn main() {
    println!("Hello from Rust!");
}
"#,
        "rust",
    );

    // Add a list
    doc.add_unordered_list(vec![
        "You can create lists",
        "With multiple items",
        "That will be properly formatted",
    ]);

    // Add a task list
    doc.add_task_list(vec![
        ("Completed task", true),
        ("Task in progress", false),
        ("Future task", false),
    ]);

    // Add a paragraph with mixed formatting
    let para_idx = doc.add_paragraph();
    let para = &mut doc.nodes[para_idx];

    if let Node::Paragraph { children } = para {
        // Add different types of inline nodes
        children.push(InlineNode::Text(TextNode {
            text: "This paragraph has ".to_string(),
            formatting: TextFormatting::default(),
        }));

        children.push(InlineNode::Text(TextNode {
            text: "bold".to_string(),
            formatting: TextFormatting::bold(),
        }));

        children.push(InlineNode::Text(TextNode {
            text: " and ".to_string(),
            formatting: TextFormatting::default(),
        }));

        children.push(InlineNode::Text(TextNode {
            text: "italic".to_string(),
            formatting: TextFormatting::italic(),
        }));

        children.push(InlineNode::Text(TextNode {
            text: " text, as well as ".to_string(),
            formatting: TextFormatting::default(),
        }));

        children.push(InlineNode::code_span("inline code"));

        children.push(InlineNode::Text(TextNode {
            text: " and a ".to_string(),
            formatting: TextFormatting::default(),
        }));

        children.push(InlineNode::link(
            "https://www.rust-lang.org",
            "link to Rust",
        ));
    }

    // Convert to different formats
    let markdown = Text::<Markdown>::try_from(&doc).unwrap();
    let html = Text::<Html>::try_from(&doc).unwrap();
    let json = Text::<Json>::try_from(&doc).unwrap();

    // Print outputs
    println!("===== DOCUMENT STRUCTURE =====");
    println!("{}", doc.debug_structure());
    println!();

    println!("===== MARKDOWN =====");
    println!("{}", markdown);
    println!();

    println!("===== HTML =====");
    println!("{}", html);
    println!();

    println!("===== JSON =====");
    println!("{}", json);
    println!();

    // Demonstrate roundtrip JSON serialization
    let doc2 = Text::<Json>::try_from(&doc).unwrap();
    let doc2 = Document::try_from(doc2).unwrap();
    assert_eq!(doc.nodes.len(), doc2.nodes.len());
    println!("JSON roundtrip successful! Document structure preserved.");
}
