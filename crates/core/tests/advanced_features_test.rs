use md_core::{Document, Html, InlineNode, Markdown, Text};
use std::convert::TryFrom;

#[test]
fn test_footnotes() {
    let mut doc = Document::new();

    // Add footnote references in a paragraph
    doc.add_paragraph_with_inlines(vec![
        InlineNode::text("This is a sentence with a footnote"),
        InlineNode::footnote_ref("1"),
        InlineNode::text(" and another footnote"),
        InlineNode::footnote_ref("2"),
        InlineNode::text("."),
    ]);

    // Add footnote definitions
    doc.add_footnote_definition("1", "First footnote.");
    doc.add_footnote_definition("2", "Second footnote.");

    // Convert to markdown and verify
    let markdown = Text::<Markdown>::try_from(&doc).unwrap().to_string();

    assert!(markdown.contains("[^1]"));
    assert!(markdown.contains("[^2]"));
    assert!(markdown.contains("[^1]: First footnote."));
    assert!(markdown.contains("[^2]: Second footnote."));

    // Convert to HTML and verify
    let html = Text::<Html>::try_from(&doc).unwrap().to_string();

    assert!(
        html.contains("<sup class=\"footnote-ref\"><a href=\"#fn-1\" id=\"fnref-1\">1</a></sup>")
    );
    assert!(html.contains("<div class=\"footnote\" id=\"fn-1\">"));
}

#[test]
fn test_math_notation() {
    let mut doc = Document::new();

    // Add inline math in a paragraph
    doc.add_paragraph_with_math("The Pythagorean theorem: ", "a^2 + b^2 = c^2");

    // Add a math block
    doc.add_math_block("E = mc^2");

    // Convert to markdown and verify
    let markdown = Text::<Markdown>::try_from(&doc).unwrap().to_string();

    assert!(markdown.contains("$a^2 + b^2 = c^2$"));
    assert!(markdown.contains("$$\nE = mc^2\n$$"));

    // Convert to HTML and verify
    let html = Text::<Html>::try_from(&doc).unwrap().to_string();

    assert!(html.contains("<span class=\"math-inline\">$a^2 + b^2 = c^2$</span>"));
    assert!(html.contains("<div class=\"math-block\">$E = mc^2$</div>"));
}

#[test]
fn test_emoji() {
    let mut doc = Document::new();

    // Add emoji in a paragraph
    doc.add_paragraph_with_emoji("I'm happy ", "smile");

    // Convert to markdown and verify
    let markdown = Text::<Markdown>::try_from(&doc).unwrap().to_string();

    assert!(markdown.contains(":smile:"));

    // Convert to HTML and verify
    let html = Text::<Html>::try_from(&doc).unwrap().to_string();

    assert!(html.contains("<span class=\"emoji emoji-smile\">smile</span>"));
}

#[test]
fn test_mentions_and_references() {
    let mut doc = Document::new();

    // Add user mention
    doc.add_paragraph_with_mention("Hello ", "johndoe");

    // Add issue reference
    doc.add_paragraph_with_issue("Fixed in issue ", "123");

    // Convert to markdown and verify
    let markdown = Text::<Markdown>::try_from(&doc).unwrap().to_string();

    assert!(markdown.contains("@johndoe"));
    assert!(markdown.contains("#123"));

    // Convert to HTML and verify
    let html = Text::<Html>::try_from(&doc).unwrap().to_string();

    assert!(html.contains("<span class=\"mention mention-user\">@johndoe</span>"));
    assert!(html.contains("<span class=\"mention mention-issue\">#123</span>"));
}

#[test]
fn test_definition_list() {
    let mut doc = Document::new();

    // Add a definition list
    doc.add_definition_list(vec![
        ("Term 1".to_string(), vec!["Definition 1".to_string()]),
        (
            "Term 2".to_string(),
            vec!["Definition 2A".to_string(), "Definition 2B".to_string()],
        ),
    ]);

    // Convert to markdown and verify
    let markdown = Text::<Markdown>::try_from(&doc).unwrap().to_string();

    assert!(markdown.contains("Term 1\n:   Definition 1"));
    assert!(markdown.contains("Term 2\n:   Definition 2A"));

    // Convert to HTML and verify
    let html = Text::<Html>::try_from(&doc).unwrap().to_string();

    assert!(html.contains("<dl>"));
    assert!(html.contains("<dt>Term 1</dt>"));
    assert!(html.contains("<dd>"));
}

#[test]
fn test_multiple_features_together() {
    let mut doc = Document::new();

    // Add a paragraph with multiple feature types
    doc.add_paragraph_with_inlines(vec![
        InlineNode::text("This paragraph has "),
        InlineNode::Emoji {
            shortcode: "heart".to_string(),
        },
        InlineNode::text(" emoji, "),
        InlineNode::Math {
            math: "x^2".to_string(),
        },
        InlineNode::text(" math, "),
        InlineNode::user_mention("user"),
        InlineNode::text(" mentions, and a "),
        InlineNode::footnote_ref("note"),
        InlineNode::text(" footnote."),
    ]);

    // Add footnote definition
    doc.add_footnote_definition("note", "This is a test footnote.");

    // Convert to markdown and verify
    let markdown = Text::<Markdown>::try_from(&doc).unwrap().to_string();

    assert!(markdown.contains(":heart:"));
    assert!(markdown.contains("$x^2$"));
    assert!(markdown.contains("@user"));
    assert!(markdown.contains("[^note]"));

    // Convert to HTML and verify
    let html = Text::<Html>::try_from(&doc).unwrap().to_string();

    assert!(html.contains("<span class=\"emoji"));
    assert!(html.contains("<span class=\"math-inline\">"));
    assert!(html.contains("<span class=\"mention mention-user\">"));
    assert!(html.contains("<sup class=\"footnote-ref\">"));
}
