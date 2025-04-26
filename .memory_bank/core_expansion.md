# Core Functionality Expansion: md-core

## Document Manipulation and Navigation

The existing system focuses on document representation and serialization, but lacks comprehensive document manipulation capabilities. Adding the following core functionalities would create a more complete editing experience:

### 1. Advanced Cursor & Selection API
- Cursor movement operations (word, line, paragraph jumps)
- Multi-cursor support
- Selection expansion/contraction operations
- Selection based on document structure (select paragraph, heading, etc.)
- Text range representations and utilities

### 2. Document History & Transactions
- Transaction-based editing system
- Document history with undo/redo support
- Atomic operations for editing
- Command pattern for document operations
- Change recording and replaying

### 3. Document Structure Operations
- Moving blocks within document
- Merging and splitting nodes
- Document outline generation
- Table of contents generation
- Node insertion at specific locations

## Content Management and Analysis

The current implementation represents content well but lacks tools for analysis and manipulation:

### 1. Search and Replace
- Text search with options (case sensitive, whole word, regex)
- Structure-aware search (find in headings, code blocks, etc.)
- Advanced replacement options
- Search result navigation
- Match highlighting

### 2. Document Statistics and Analysis
- Word, character, and node counting
- Reading time estimation
- Document complexity analysis
- Heading structure validation
- Link validation

### 3. Content Validation
- Document schema validation
- Custom validation rules
- Error reporting and suggestions
- Linting for markdown best practices
- Accessibility checks

## Advanced Document Features

The library would benefit from features that enhance document capabilities:

### 1. References and Citations
- Footnote management
- Bibliography support
- Citation linking
- Cross-references between document sections
- Reference collection and formatting

### 2. Document Fragments
- Working with partial documents
- Merging document fragments
- Template support
- Document composition from fragments
- Inclusion of external content

### 3. Embedded Content
- Support for embedded media
- Interactive elements
- Custom block types
- Extension mechanism for custom content
- Rendering hints for embedded content

## Collaboration Foundations

Building blocks for collaborative editing would future-proof the library:

### 1. Operational Transformation Primitives
- Operation-based document changes
- Conflict resolution
- Document synchronization primitives
- Change tracking
- Operational transformation algorithms

### 2. Comments and Annotations
- Support for document comments
- Inline annotations
- Review/suggestion mode
- Threaded discussions
- Annotation rendering properties

## Proposed Architecture

A layered architecture with:

1. **Core Layer**
   - Document model
   - Basic operations
   - Serialization
   - Extension points

2. **Extension Modules**
   - Advanced editing
   - Navigation & selection
   - History & transactions
   - Search & analysis
   - Validation
   - References & embedded content
   - Collaboration

3. **Plugin System**
   - Custom node types
   - Custom operations
   - Custom serialization formats
   - Custom validation rules

## Implementation Strategy

- Phase 1: Define extension points in core
- Phase 2: Implement document history & transactions
- Phase 3: Implement advanced cursor & selection API
- Phase 4: Implement search & analysis
- Phase 5: Add document structure operations
- Phase 6: Create plugin system for third-party extensions

This approach allows for incremental development while maintaining backward compatibility and providing a path for users to adopt new features as needed.

## MD-Core Integration with Dioxus UI

### Integration Strategy

To effectively integrate md-core with our Dioxus UI, we will follow these strategies:

1. **Document-to-Component Mapping**:
   - Create a one-to-one mapping between md-core `Node` types and Dioxus components
   - Define clear boundaries between document model and view components
   - Implement a rendering system that traverses the document tree

2. **Event Handling**:
   - Capture DOM events in Dioxus components
   - Translate user interactions into md-core commands
   - Use the Command pattern to ensure all operations are undoable

3. **Change Propagation**:
   - Implement a reactive system to propagate changes from the document model to the UI
   - Use Dioxus signals to efficiently update only changed components
   - Handle selection changes separately from content changes

### Component Design

The Dioxus components will be structured as follows:

```rust
#[component]
fn DocumentView(document: Document) -> Element {
    // Render document nodes recursively
    rsx! {
        div { class: "document",
            // Map each node to its corresponding renderer
            document.nodes.iter().map(|node| match node {
                Node::Heading(h) => rsx!{ HeadingRenderer { heading: h } },
                Node::Paragraph(p) => rsx!{ ParagraphRenderer { paragraph: p } },
                // Other node types...
            })
        }
    }
}
```

### Content Editable Strategy

For the editable functionality, we'll implement a custom system:

1. **Controlled ContentEditable**:
   - Use DOM contentEditable with controlled state
   - Intercept browser editing operations
   - Translate DOM changes back to md-core model changes

2. **Selection Synchronization**:
   - Track DOM selection using a custom selection manager
   - Map DOM selection to md-core document positions
   - Ensure selection state persists across renders

3. **Input Handlers**:
   - Implement keyboard handlers for special keys (Enter, Tab, etc.)
   - Handle composition events for IME input
   - Implement clipboard event handlers (cut, copy, paste)

### Missing md-core Functionalities

Based on our analysis, we might need to expand md-core with these features:

1. **Selection API Improvements**:
   - Enhance the selection model to support collapsed cursors
   - Add methods for selection manipulation
   - Implement selection serialization for persistence

2. **Event System**:
   - Add an event system to notify about document changes
   - Implement document observers for specific parts of the document
   - Create custom events for selection changes

3. **Cursor Navigation**:
   - Add methods for cursor movement (by character, word, line)
   - Support for navigating between nodes
   - Implement smart cursor positioning

4. **Table Editing**:
   - Enhance table manipulation capabilities
   - Add commands for inserting/deleting rows and columns
   - Implement table cell selection

5. **Real-time Collaboration**:
   - Implement operational transformation or CRDT
   - Add conflict resolution strategies
   - Support for change tracking and annotations

### Integration Testing Strategy

To ensure the integration works correctly:

1. **Component Tests**:
   - Test each renderer component in isolation
   - Verify correct rendering of different node types
   - Test component interaction with mock events

2. **Integration Tests**:
   - Test the full editing pipeline from user input to document changes
   - Verify undo/redo functionality works across the UI
   - Test complex operations like selection-based formatting

3. **Performance Tests**:
   - Benchmark rendering performance for large documents
   - Test memory usage patterns during editing
   - Verify efficient re-rendering on document changes
