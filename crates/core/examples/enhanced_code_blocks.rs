use md_core::{CodeBlockProperties, Document, Html, Markdown, Node, Text};

fn main() {
    // Create a new document
    let mut doc = Document::new();

    // Add a heading
    doc.add_heading(1, "Enhanced Code Block Examples");

    // Add a regular paragraph
    doc.add_paragraph_with_text("Below are examples of code blocks with various enhancements:");

    // Example 1: Basic code block with language
    let basic_rust_code = r#"
fn main() {
    println!("Hello, world!");
}
"#;

    doc.add_paragraph_with_text("1. Basic Rust code block:");
    doc.add_code_block(basic_rust_code, "rust");

    // Example 2: Code block with line numbers
    let numbered_code = r#"
use std::io;

fn main() {
    println!("Enter your name:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    println!("Hello, {}!", input.trim());
}
"#;

    doc.add_paragraph_with_text("2. Rust code with line numbers:");
    let line_number_props = CodeBlockProperties::default()
        .with_line_numbers(true)
        .with_start_line(1);

    // Add code block with custom properties
    doc.nodes.push(Node::CodeBlock {
        language: "rust".to_string(),
        code: numbered_code.to_string(),
        properties: line_number_props,
    });

    // Example 3: Code with highlighted lines
    let highlighted_code = r#"
fn calculate_factorial(n: u64) -> u64 {
    if n == 0 {
        return 1;
    }

    let mut result = 1;
    for i in 1..=n {
        result *= i;
    }

    result
}
"#;

    doc.add_paragraph_with_text("3. Code with highlighted lines (lines 2 and 7):");
    let highlight_props = CodeBlockProperties::default()
        .with_line_numbers(true)
        .with_highlight_lines(vec![2, 7]);

    doc.nodes.push(Node::CodeBlock {
        language: "rust".to_string(),
        code: highlighted_code.to_string(),
        properties: highlight_props,
    });

    // Example 4: Code with theme and max height
    let long_code = r#"
struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}

impl User {
    fn new(username: String, email: String) -> Self {
        User {
            username,
            email,
            sign_in_count: 1,
            active: true,
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn increment_sign_in(&mut self) {
        self.sign_in_count += 1;
    }
}

fn main() {
    let mut user = User::new(
        "johndoe".to_string(),
        "john@example.com".to_string()
    );

    println!("User {} is active: {}",
        user.username,
        user.is_active()
    );

    user.increment_sign_in();
    println!("Sign in count: {}", user.sign_in_count);
}
"#;

    doc.add_paragraph_with_text("4. Code with theme and max height (with scrolling):");
    let styled_props = CodeBlockProperties::default()
        .with_line_numbers(true)
        .with_theme("dracula")
        .with_max_height("200px")
        .with_css_class("styled-code");

    doc.nodes.push(Node::CodeBlock {
        language: "rust".to_string(),
        code: long_code.to_string(),
        properties: styled_props,
    });

    // Example 5: Code with custom styling
    let short_code = r#"// A simple TypeScript example
interface Person {
    name: string;
    age: number;
}

const greeting = (person: Person): string => {
    return `Hello, ${person.name}!`;
};
"#;

    doc.add_paragraph_with_text("5. Code with custom styling:");
    let custom_props = CodeBlockProperties::default()
        .with_style("border-radius: 8px; border: 1px solid #ddd;")
        .with_copy_button(true);

    doc.nodes.push(Node::CodeBlock {
        language: "typescript".to_string(),
        code: short_code.to_string(),
        properties: custom_props,
    });

    // Convert to HTML and print
    let html = Text::<Html>::try_from(&doc).unwrap().to_string();
    println!("HTML Output:\n{}", html);

    // Convert to Markdown and print
    let markdown = Text::<Markdown>::try_from(&doc).unwrap().to_string();
    println!("\nMarkdown Output:\n{}", markdown);
}
