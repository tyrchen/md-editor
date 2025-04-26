# Project Brief: Markdown Editor Core (md-core)

## Overview
The `md-core` crate provides a core data structure for a markdown editor, inspired by Slate.js's architecture. It allows representing, manipulating, and serializing hierarchical markdown documents with rich formatting.

## Features
- Hierarchical document representation with blocks and inline elements
- Rich text formatting (bold, italic, code, strikethrough)
- Support for lists (ordered, unordered, tasks), code blocks, tables, and more
- Cursor state and selection tracking
- Document metadata handling
- Serialization and deserialization to/from JSON
- Conversion to/from markdown and HTML formats
- Editing operations (insert, split, etc.)
- Planned GitHub Flavored Markdown (GFM) support and enhanced table functionality

## Architecture
The document is organized as a tree structure:
- A `Document` contains a list of block-level `Node`s
- Block nodes (headings, paragraphs, etc.) can contain inline elements or other blocks
- Inline elements (text, links, etc.) represent formatted content
- Selection and cursor state can be tracked within the document

## Current Implementation Status
- Core document structure is well-implemented
- Serialization to JSON is fully implemented and tested
- Serialization to markdown and HTML is implemented for output
- Parsing from markdown and HTML is currently stubbed (not implemented)
- Error handling through a ParseError type
- Basic table support implemented, with plans for enhanced functionality
- Planning in progress for comprehensive GitHub Flavored Markdown support

## Dependencies
- serde and serde_json for serialization
- Future implementation plans include:
  - pulldown-cmark for markdown parsing
  - html5ever for HTML parsing

## Future Development
- Full GitHub Flavored Markdown (GFM) support
- Enhanced tables with alignment, spanning, and formatting
- Extended Markdown features (footnotes, definition lists, etc.)
- Comprehensive parsing capabilities

## Notes
This crate is the foundation for a markdown editor application, providing data structures and serialization capabilities but not the UI components.
