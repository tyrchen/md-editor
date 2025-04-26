/*! # MD Editor Core

This crate provides a core data structure for a markdown editor, inspired by Slate.js's architecture.
It allows representing, manipulating, and serializing hierarchical markdown documents with rich formatting.

## Features

- Hierarchical document representation with blocks and inline elements
- Rich text formatting (bold, italic, code, strikethrough)
- Support for lists (ordered, unordered, tasks), code blocks, tables, and more
- Cursor state and selection tracking
- Document metadata handling
- Serialization and deserialization to/from JSON
- Conversion to/from markdown and HTML formats
- Editing operations (insert, split, etc.)

## Basic Example

```rust
use md_core::{Document, Text, Markdown, Html, Json};
use std::convert::TryInto;

// Create a new document with a title
let mut doc = Document::with_title("My Document");

// Add content
doc.add_paragraph_with_text("This is a paragraph in the document.");
doc.add_code_block("println!(\"Hello, world!\");", "rust");

// Convert to different formats
let markdown: Text<Markdown> = doc.as_ref().try_into().unwrap();
let html: Text<Html> = doc.as_ref().try_into().unwrap();
let json: Text<Json> = doc.as_ref().try_into().unwrap();

println!("{}", markdown);
```

## Data Structure

The document is organized as a tree structure:
- A `Document` contains a list of block-level `Node`s
- Block nodes (headings, paragraphs, etc.) can contain inline elements or other blocks
- Inline elements (text, links, etc.) represent formatted content
- Selection and cursor state can be tracked within the document

See the module documentation for more details on individual components.
*/

mod convert;
mod editor;
mod error;
mod models;

pub use convert::{Html, Json, Markdown, Text};
pub use editor::*;
pub use error::{EditError, ParseError};
pub use models::*;
