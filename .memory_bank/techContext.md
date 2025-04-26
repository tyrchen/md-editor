# Technical Context: md-core

## Tech Stack
- **Language**: Rust (2024 edition)
- **Build System**: Cargo
- **Serialization**: serde with serde_json
- **Error Handling**: Custom ParseError type with future thiserror integration
- **Testing**: Standard Rust test framework
- **Documentation**: rustdoc

## Project Structure
- `src/editor/` - Main module with core data structures
  - `document.rs` - Main Document struct and operations
  - `node.rs` - Block-level nodes (headings, paragraphs, etc.)
  - `inline.rs` - Inline content (text, links, etc.)
  - `selection.rs` - Cursor and selection tracking
  - `formatting.rs` - Text formatting options
  - `serialization.rs` - Conversion to/from JSON, markdown, HTML
  - `error.rs` - Error handling
  - `tests.rs` - Unit tests

## Key Design Patterns
- **Type-driven design**: Using Rust's strong type system to model the document structure
- **Enums for variants**: Using enums to represent different node types
- **Builder pattern**: Methods like `Node::heading()` for constructing nodes
- **Composition**: Building complex structures from simpler components
- **Result for error handling**: Functions that can fail return `Result<T, ParseError>`

## Environment
- **OS**: macOS (darwin 24.3.0)
- **Shell**: Nushell (/opt/homebrew/bin/nu)
- **Directory**: /Users/tchen/projects/mycode/rust/md-editor/crates/core
