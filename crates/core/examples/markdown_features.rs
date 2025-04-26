use md_core::{Document, Html, InlineNode, Markdown, Node, Text};
use std::convert::TryFrom;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a document showcasing the various features
    let mut doc = Document::with_title("Advanced Markdown Features");

    // Introduction paragraph
    doc.add_paragraph_with_text("This document demonstrates various advanced Markdown features.");

    // Add math section
    doc.add_heading(2, "Mathematical Notation");
    doc.add_paragraph_with_text(
        "Math notation is supported using TeX syntax. You can include inline formulas like:",
    );
    doc.add_paragraph_with_math("The famous equation: ", "E = mc^2");
    doc.add_paragraph_with_text("Or display formulas as blocks:");
    doc.add_math_block("\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}");

    // Add footnotes section
    doc.add_heading(2, "Footnotes");
    doc.add_paragraph_with_inlines(vec![
        InlineNode::text("Footnotes allow you to add references"),
        InlineNode::footnote_ref("note1"),
        InlineNode::text(" and explanations"),
        InlineNode::footnote_ref("note2"),
        InlineNode::text(" to your document."),
    ]);

    // Add footnote definitions
    doc.add_footnote_definition("note1", "This is a simple footnote.");

    // Add a complex footnote with multiple paragraphs
    let complex_footnote_content = vec![
        Node::paragraph("This is a complex footnote with multiple paragraphs."),
        Node::paragraph("It can contain additional formatting and structure."),
    ];
    doc.add_complex_footnote_definition("note2", complex_footnote_content);

    // Add emoji section
    doc.add_heading(2, "Emoji Support");
    doc.add_paragraph_with_text("Emoji shortcodes can be used to insert emoji:");
    doc.add_paragraph_with_emoji("Happy face: ", "smile");
    doc.add_paragraph_with_emoji("Thumbs up: ", "thumbsup");
    doc.add_paragraph_with_emoji("Rocket launch: ", "rocket");

    // Add mentions section
    doc.add_heading(2, "Mentions and References");
    doc.add_paragraph_with_text("Mentions can reference users or issues:");
    doc.add_paragraph_with_mention("User mention: ", "johndoe");
    doc.add_paragraph_with_issue("Issue reference: ", "42");

    // Add a definition list
    doc.add_heading(2, "Definition Lists");
    doc.add_definition_list(vec![
        (
            "Markdown".to_string(),
            vec![
                "A lightweight markup language with plain text formatting syntax.".to_string(),
                "Designed to be easy to read and write.".to_string(),
            ],
        ),
        (
            "HTML".to_string(),
            vec!["HyperText Markup Language used for creating web pages.".to_string()],
        ),
        (
            "CSS".to_string(),
            vec!["Cascading Style Sheets used for styling web pages.".to_string()],
        ),
    ]);

    // Convert document to Markdown
    let markdown_text: Text<Markdown> = Text::try_from(&doc)?;
    let markdown = markdown_text.as_str();
    println!("Generated Markdown:\n{}", markdown);

    // Convert document to HTML
    let html_text: Text<Html> = Text::try_from(&doc)?;
    let html = html_text.as_str();
    println!("\nGenerated HTML:\n{}", html);

    // Save to files
    fs::write("markdown_features.md", markdown)?;
    fs::write("markdown_features.html", html)?;

    Ok(())
}
