# MD Editor

Build the core data structure for a markdown editor.

## 🎨🎨🎨 ENTERING CREATIVE PHASE: ARCHITECTURE DESIGN

## Component Description
The core data structure for a markdown editor, inspired by Slate.js's Value architecture, designed to represent and manipulate markdown documents in Rust. This structure will support serialization/deserialization between markdown text and HTML, enabling seamless conversion between formats.

## Requirements & Constraints
1. Represent hierarchical markdown structure (headings, paragraphs, lists, code blocks, etc.)
2. Support rich text formatting within nodes (bold, italic, links, code spans)
3. Maintain cursor state and selection ranges
4. Enable efficient editing operations (insert, delete, split, merge)
5. Support serialization/deserialization to/from markdown text
6. Support serialization/deserialization to/from HTML
7. Support serialization/deserialization to/from JSON
8. Maintain document history for undo/redo functionality
9. Must be memory efficient and performant
10. Design should align with Rust's ownership model
11. Support safe concurrent access where needed
12. Provide comprehensive test coverage

## Design Options

### Option 1: Direct Slate.js Port to Rust

A direct port of Slate.js's document model with Rust equivalents:

```rust
// Conceptual structure (not actual implementation)
struct Value {
    document: Document,
    selection: Option<Selection>,
    // Other document-level metadata
}

struct Document {
    nodes: Vec<Node>,
    // Document-level attributes
}

enum Node {
    Block(Block),
    Inline(Inline),
    Text(Text),
}

struct Block {
    nodes: Vec<Node>,
    data: HashMap<String, Value>,
    // Block-specific attributes (e.g., block type)
}

struct Inline {
    nodes: Vec<Node>,
    data: HashMap<String, Value>,
    // Inline-specific attributes
}

struct Text {
    text: String,
    marks: Vec<Mark>,
}

struct Mark {
    mark_type: String,
    data: HashMap<String, Value>,
}

struct Selection {
    anchor: Point,
    focus: Point,
    // Selection attributes
}

struct Point {
    node_id: NodeId,
    offset: usize,
    // Path information
}
```

**Pros:**
- Closely matches Slate.js's proven architecture
- Flexible and capable of representing complex documents
- Clear separation of block vs. inline elements
- Familiar for developers with Slate.js experience

**Cons:**
- Complex structure may have overhead in Rust
- Doesn't fully leverage Rust's type system
- Many dynamic containers (HashMap, Vec) could impact performance
- Potential ownership/borrowing challenges with deeply nested structures

### Option 2: Rust-Optimized Hierarchical Structure

A more Rust-idiomatic approach using enums and stronger typing:

```rust
// Conceptual structure (not actual implementation)
struct Document {
    nodes: Vec<Node>,
    selection: Option<Selection>,
    // Document-level metadata
}

enum Node {
    Heading { level: u8, children: Vec<InlineNode> },
    Paragraph { children: Vec<InlineNode> },
    List { list_type: ListType, items: Vec<ListItem> },
    CodeBlock { language: String, code: String },
    BlockQuote { children: Vec<Node> },
    // Other block elements
}

struct ListItem {
    children: Vec<Node>,
    // ListItem-specific attributes
}

enum ListType {
    Ordered,
    Unordered,
    Task,
}

enum InlineNode {
    Text(TextNode),
    Link { url: String, title: Option<String>, children: Vec<InlineNode> },
    Image { url: String, alt: String, title: Option<String> },
    CodeSpan(String),
    // Other inline elements
}

struct TextNode {
    text: String,
    formatting: TextFormatting,
}

struct TextFormatting {
    bold: bool,
    italic: bool,
    strikethrough: bool,
    // Other formatting attributes
}

struct Selection {
    start: Position,
    end: Position,
    is_collapsed: bool,
}

struct Position {
    path: Vec<usize>,
    offset: usize,
}
```

**Pros:**
- More idiomatic Rust with stronger typing
- Reduced need for dynamic type resolution at runtime
- Better compile-time guarantees
- More direct mapping to markdown semantics
- Potentially more memory efficient

**Cons:**
- Less flexible than Slate.js's generic structure
- Adding new node types requires modifying enums
- May be harder to extend with plugins
- Potentially complex serialization logic

### Option 3: Rust Entity-Component-System (ECS) Approach

Using an ECS approach to represent document elements:

```rust
// Conceptual structure (not actual implementation)
struct Document {
    registry: Registry,
    root_entity: EntityId,
    selection: Option<Selection>,
}

struct Registry {
    entities: HashMap<EntityId, Entity>,
    // Component storage
}

struct Entity {
    id: EntityId,
    components: Vec<ComponentId>,
}

enum Component {
    NodeType(NodeType),
    Children(Vec<EntityId>),
    TextContent(String),
    Formatting(Formatting),
    Metadata(HashMap<String, Value>),
    // Other components
}

enum NodeType {
    Heading(u8),
    Paragraph,
    ListItem,
    OrderedList,
    UnorderedList,
    CodeBlock(String), // language
    BlockQuote,
    Link(String),      // URL
    Image(String, String), // URL, alt
    // Other node types
}

struct Formatting {
    bold: bool,
    italic: bool,
    strikethrough: bool,
    // Other formatting attributes
}

struct Selection {
    // Similar to other options
}
```

**Pros:**
- Highly flexible and extensible
- Composable components allow for rich features
- Could efficiently support complex operations through component queries
- Good separation of concerns
- May be well-suited for reactive UI frameworks

**Cons:**
- Higher complexity than other options
- More indirection could impact performance
- Might be overkill for simpler markdown editing
- Entity/component management adds overhead
- Less straightforward serialization

### Option 4: Immutable Data Structure with Copy-on-Write

An immutable approach using persistent data structures:

```rust
// Conceptual structure (not actual implementation)
use std::rc::Rc;
use std::sync::Arc;
use im::{Vector, HashMap}; // Immutable collections

struct Document {
    nodes: Vector<Rc<Node>>,
    selection: Option<Selection>,
    // Document-level metadata
}

enum Node {
    Heading { level: u8, children: Vector<Rc<InlineNode>> },
    Paragraph { children: Vector<Rc<InlineNode>> },
    List { list_type: ListType, items: Vector<Rc<Node>> },
    CodeBlock { language: String, code: String },
    BlockQuote { children: Vector<Rc<Node>> },
    // Other block elements
}

enum InlineNode {
    Text { text: String, formatting: Rc<TextFormatting> },
    Link { url: String, title: Option<String>, children: Vector<Rc<InlineNode>> },
    // Other inline elements
}

struct TextFormatting {
    bold: bool,
    italic: bool,
    strikethrough: bool,
    // Other formatting attributes
}

struct Selection {
    // Similar to other options
}
```

**Pros:**
- Excellent for history/undo support (each edit creates a new version)
- Thread-safe with appropriate synchronization primitives
- Makes reasoning about document state simpler
- Efficient structural sharing reduces memory usage
- Well-suited for functional reactive patterns

**Cons:**
- Requires immutable data structure libraries
- May have performance overhead for large documents
- Less intuitive for developers used to mutable data structures
- Reference counting adds some overhead
- More complex to implement correctly

## Recommended Approach

**Option 2: Rust-Optimized Hierarchical Structure**, with elements of Option 4 (immutability) where appropriate.

This approach provides the best balance of:
1. Rust-idiomatic design with strong typing
2. Direct mapping to markdown semantics
3. Good performance characteristics
4. Reasonable complexity
5. Clear separation between block and inline elements

### Justification:
- Using Rust's enum types provides better type safety and more efficient memory representation than a generic node structure
- The stricter typing makes invalid states unrepresentable at compile time
- Direct mapping to markdown concepts simplifies serialization/deserialization
- Can incorporate immutable patterns for history tracking while maintaining performance
- Avoids the complexity of an ECS approach while still providing the necessary flexibility

## Implementation Guidelines

1. **Core Data Structures:**
   - Implement the `Document` as the root container
   - Use enums for `Node` and `InlineNode` to represent different element types
   - Implement specific structs for complex elements (e.g., `TextNode`, `ListItem`)
   - Use appropriate Rust types (e.g., `Option`, `Vec`) for optional and repeated fields
   - Add `#[derive(Serialize, Deserialize)]` to all structures to enable serde support

2. **Serialization/Deserialization with Serde:**
   - Implement the `serde::Serialize` and `serde::Deserialize` traits for all structures
   - Use `#[serde(tag = "type")]` for enum variants to create readable JSON
   - Add custom attributes where needed (`#[serde(rename)]`, `#[serde(skip)]`, etc.)
   - Implement `From<&str>` and `Display` traits for markdown and HTML conversions

   ### JSON Serialization:
   ```rust
   #[derive(Serialize, Deserialize, Debug, Clone)]
   #[serde(tag = "type")]
   pub enum Node {
       #[serde(rename = "heading")]
       Heading { level: u8, children: Vec<InlineNode> },

       #[serde(rename = "paragraph")]
       Paragraph { children: Vec<InlineNode> },

       // Other variants
   }

   // Example usage
   fn to_json(document: &Document) -> Result<String, serde_json::Error> {
       serde_json::to_string(document)
   }

   fn from_json(json: &str) -> Result<Document, serde_json::Error> {
       serde_json::from_str(json)
   }
   ```

   ### Markdown Serialization:
   ```rust
   impl Document {
       pub fn to_markdown(&self) -> String {
           let mut output = String::new();
           for node in &self.nodes {
               output.push_str(&node.to_markdown());
               output.push_str("\n\n");
           }
           output.trim_end().to_string()
       }

       pub fn from_markdown(markdown: &str) -> Result<Self, ParseError> {
           // Utilize pulldown-cmark for parsing
           let parser = pulldown_cmark::Parser::new(markdown);
           let mut document = Document::new();

           // Convert parser events to document structure
           // ...

           Ok(document)
       }
   }

   impl Node {
       fn to_markdown(&self) -> String {
           match self {
               Node::Heading { level, children } => {
                   let prefix = "#".repeat(*level as usize);
                   let content = children.iter()
                       .map(|n| n.to_markdown())
                       .collect::<Vec<_>>()
                       .join("");
                   format!("{} {}", prefix, content)
               },
               // Other variants
               // ...
           }
       }
   }
   ```

   ### HTML Serialization:
   ```rust
   impl Document {
       pub fn to_html(&self) -> String {
           let mut output = String::new();
           output.push_str("<article class=\"markdown-document\">\n");
           for node in &self.nodes {
               output.push_str(&node.to_html());
           }
           output.push_str("</article>");
           output
       }

       pub fn from_html(html: &str) -> Result<Self, ParseError> {
           // Use html5ever or similar for parsing
           // ...

           Ok(document)
       }
   }

   impl Node {
       fn to_html(&self) -> String {
           match self {
               Node::Heading { level, children } => {
                   let tag = format!("h{}", level);
                   let content = children.iter()
                       .map(|n| n.to_html())
                       .collect::<Vec<_>>()
                       .join("");
                   format!("<{}>{}</{}>", tag, content, tag)
               },
               // Other variants
               // ...
           }
       }
   }
   ```

3. **Custom Serde Implementations:**
   - Use `serde_with` crate for complex serialization scenarios
   - Implement custom serializer/deserializer for markdown and HTML formats:

   ```rust
   pub mod markdown {
       use serde::{Deserialize, Deserializer, Serialize, Serializer};
       use pulldown_cmark::{Parser, Event, Tag};

       pub fn serialize<S>(document: &Document, serializer: S) -> Result<S::Ok, S::Error>
       where
           S: Serializer,
       {
           let markdown = document.to_markdown();
           serializer.serialize_str(&markdown)
       }

       pub fn deserialize<'de, D>(deserializer: D) -> Result<Document, D::Error>
       where
           D: Deserializer<'de>,
       {
           let markdown = String::deserialize(deserializer)?;
           Document::from_markdown(&markdown).map_err(serde::de::Error::custom)
       }
   }

   pub mod html {
       // Similar implementation for HTML format
   }
   ```

   - Use these custom serializers with serde attributes:

   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct DocumentWrapper {
       #[serde(with = "json")]
       pub json_document: Document,

       #[serde(with = "markdown")]
       pub markdown_document: Document,

       #[serde(with = "html")]
       pub html_document: Document,
   }
   ```

4. **Editing Operations:**
   - Implement functions for common operations (insert, delete, split, merge)
   - Use a cursor/selection model that can traverse the document structure
   - Consider using the Command pattern for editing operations
   - Ensure operations maintain document validity

5. **History Management:**
   - Use an immutable approach for document history
   - Consider copy-on-write semantics for efficient memory usage
   - Implement transaction-based editing with atomic operations

6. **Performance Considerations:**
   - Use reference counting (`Rc` or `Arc`) where appropriate to avoid excessive cloning
   - Consider using arena allocation for nodes to improve memory locality
   - Implement incremental parsing/rendering for large documents
   - Design for efficient traversal of the document structure

7. **Potential Rust Crates to Use:**
   - `serde`, `serde_json`, `serde_derive` for serialization/deserialization
   - `serde_with` for custom serialization logic
   - `pulldown-cmark` for markdown parsing
   - `html5ever` or similar for HTML parsing
   - `im` for immutable data structures if needed
   - `thiserror` for error handling
   - `nom` or `pest` if custom parsers are needed

8. **Comprehensive Testing Strategy:**

   ### Unit Testing:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_document_creation() {
           let doc = Document::new();
           assert!(doc.nodes.is_empty());
       }

       #[test]
       fn test_paragraph_node() {
           let text = TextNode {
               text: "Hello world".to_string(),
               formatting: TextFormatting::default(),
           };
           let para = Node::Paragraph {
               children: vec![InlineNode::Text(text)],
           };

           // Test logic and structure
           if let Node::Paragraph { children } = para {
               assert_eq!(children.len(), 1);
               if let InlineNode::Text(text_node) = &children[0] {
                   assert_eq!(text_node.text, "Hello world");
               } else {
                   panic!("Expected TextNode");
               }
           } else {
               panic!("Expected Paragraph");
           }
       }

       // Test serialization to markdown
       #[test]
       fn test_markdown_serialization() {
           let doc = create_test_document();
           let markdown = doc.to_markdown();
           assert!(markdown.contains("# Heading 1"));
           assert!(markdown.contains("## Heading 2"));
           assert!(markdown.contains("* List item 1"));
           // More assertions for markdown formatting
       }

       // Test deserialization from markdown
       #[test]
       fn test_markdown_deserialization() {
           let markdown = "# Heading 1\n\nParagraph text.\n\n* List item 1\n* List item 2";
           let doc = Document::from_markdown(markdown).unwrap();

           assert_eq!(doc.nodes.len(), 3);

           if let Node::Heading { level, .. } = &doc.nodes[0] {
               assert_eq!(*level, 1);
           } else {
               panic!("Expected Heading");
           }

           if let Node::Paragraph { .. } = &doc.nodes[1] {
               // Assertions about paragraph content
           } else {
               panic!("Expected Paragraph");
           }

           if let Node::List { list_type, .. } = &doc.nodes[2] {
               assert_eq!(*list_type, ListType::Unordered);
           } else {
               panic!("Expected List");
           }
       }

       // Test JSON serialization roundtrip
       #[test]
       fn test_json_roundtrip() {
           let original = create_test_document();
           let json = serde_json::to_string(&original).unwrap();
           let deserialized: Document = serde_json::from_str(&json).unwrap();

           // Compare original and deserialized documents
           assert_eq!(original.nodes.len(), deserialized.nodes.len());
           // More detailed comparisons
       }

       // Test HTML serialization
       #[test]
       fn test_html_serialization() {
           let doc = create_test_document();
           let html = doc.to_html();

           assert!(html.contains("<h1>"));
           assert!(html.contains("<p>"));
           assert!(html.contains("<ul>"));
           // More assertions for HTML structure
       }

       // Test HTML deserialization
       #[test]
       fn test_html_deserialization() {
           let html = "<article><h1>Heading 1</h1><p>Paragraph text.</p></article>";
           let doc = Document::from_html(html).unwrap();

           // Assertions about parsed document structure
       }

       // Test editing operations
       #[test]
       fn test_insert_text() {
           let mut doc = Document::new();
           // Add a paragraph
           let para_id = doc.add_paragraph();

           // Insert text
           doc.insert_text(para_id, 0, "Hello world");

           // Verify text was inserted
           if let Node::Paragraph { children } = &doc.nodes[0] {
               if let InlineNode::Text(text_node) = &children[0] {
                   assert_eq!(text_node.text, "Hello world");
               }
           }
       }

       // Test node operations
       #[test]
       fn test_split_paragraph() {
           let mut doc = Document::new();
           // Add a paragraph with text
           let para_id = doc.add_paragraph();
           doc.insert_text(para_id, 0, "Hello world");

           // Split paragraph at position 5
           doc.split_node(para_id, 5);

           // Verify we now have two paragraphs
           assert_eq!(doc.nodes.len(), 2);

           // Verify text content in both paragraphs
           if let Node::Paragraph { children: children1 } = &doc.nodes[0] {
               if let InlineNode::Text(text_node) = &children1[0] {
                   assert_eq!(text_node.text, "Hello");
               }
           }

           if let Node::Paragraph { children: children2 } = &doc.nodes[1] {
               if let InlineNode::Text(text_node) = &children2[0] {
                   assert_eq!(text_node.text, " world");
               }
           }
       }

       // Helper function to create a test document
       fn create_test_document() -> Document {
           let mut doc = Document::new();

           // Add a heading
           doc.nodes.push(Node::Heading {
               level: 1,
               children: vec![InlineNode::Text(TextNode {
                   text: "Heading 1".to_string(),
                   formatting: TextFormatting::default(),
               })],
           });

           // Add a paragraph
           doc.nodes.push(Node::Paragraph {
               children: vec![InlineNode::Text(TextNode {
                   text: "Paragraph text.".to_string(),
                   formatting: TextFormatting::default(),
               })],
           });

           // Add a list
           doc.nodes.push(Node::List {
               list_type: ListType::Unordered,
               items: vec![
                   ListItem {
                       children: vec![Node::Paragraph {
                           children: vec![InlineNode::Text(TextNode {
                               text: "List item 1".to_string(),
                               formatting: TextFormatting::default(),
                           })],
                       }],
                   },
                   ListItem {
                       children: vec![Node::Paragraph {
                           children: vec![InlineNode::Text(TextNode {
                               text: "List item 2".to_string(),
                               formatting: TextFormatting::default(),
                           })],
                       }],
                   },
               ],
           });

           doc
       }
   }
   ```

   ### Integration Testing:
   - Test document transformations between formats (markdown → document → HTML)
   - Test editing operations across complex documents
   - Test performance with large documents

   ### Property-Based Testing:
   - Use the `proptest` crate to generate random documents
   - Test invariants like roundtrip serialization equality
   - Test that editing operations maintain document validity

   ```rust
   #[cfg(test)]
   mod property_tests {
       use super::*;
       use proptest::prelude::*;

       proptest! {
           #[test]
           fn test_json_roundtrip(doc in generate_document()) {
               let json = serde_json::to_string(&doc).unwrap();
               let deserialized: Document = serde_json::from_str(&json).unwrap();

               assert_eq!(doc, deserialized);
           }

           #[test]
           fn test_markdown_roundtrip(doc in generate_simple_document()) {
               let markdown = doc.to_markdown();
               let deserialized = Document::from_markdown(&markdown).unwrap();

               // Compare essential structure
               assert_eq!(doc.nodes.len(), deserialized.nodes.len());
           }
       }

       // Generate arbitrary documents for testing
       fn generate_document() -> impl Strategy<Value = Document> {
           // Implementation for document generation
       }

       fn generate_simple_document() -> impl Strategy<Value = Document> {
           // Implementation for simpler document generation
           // (markdown roundtrip will lose some formatting information)
       }
   }
   ```

## Verification
The recommended approach satisfies the requirements:
- Represents hierarchical markdown structure ✓
- Supports rich text formatting ✓
- Can maintain cursor state and selection ranges ✓
- Enables efficient editing operations ✓
- Supports serialization/deserialization to/from markdown, HTML, and JSON ✓
- Can be designed to maintain document history ✓
- Memory efficient and performant ✓
- Aligns with Rust's ownership model ✓
- Can support safe concurrent access with appropriate synchronization ✓
- Comprehensive test coverage ✓

## 🎨🎨🎨 EXITING CREATIVE PHASE
