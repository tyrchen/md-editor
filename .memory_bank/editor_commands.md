# Editor Commands Implementation

## Overview
This document tracks the implementation of command patterns in the Markdown editor. The editor follows a command pattern for all editing operations, enabling undo/redo functionality.

## Existing Commands (Pre-Implementation)
- `DeleteTextCommand` - Removes text within nodes
- `MergeNodesCommand` - Combines adjacent nodes
- `FormatTextCommand` - Applies text formatting (bold, italic, etc.)
- `MoveNodeCommand` - Repositions nodes in the document
- `ConvertNodeTypeCommand` - Changes node types (paragraph to heading, etc.)
- `DeleteNodeCommand` - Removes entire nodes
- `FindReplaceCommand` - Searches and replaces text

## Newly Implemented Commands (Phase 1)

### InsertTextCommand
- **Purpose**: Insert text at a specific position within a node
- **Implementation**:
  - Locates the correct text node and position based on offset
  - Handles multiple inline nodes
  - Supports code blocks
  - Preserves the original state for undo
- **Edge Cases**:
  - Handles positions at node boundaries
  - Handles insertion at the end of all nodes
  - Handles positions in non-text nodes

### InsertNodeCommand
- **Purpose**: Insert a new node at a specific position in the document
- **Implementation**:
  - Inserts the node at the specified position
  - Includes helper methods for common node types:
    - `new_paragraph()` - Creates a paragraph node with text
    - `new_heading()` - Creates a heading node with text
    - `new_code_block()` - Creates a code block node
- **Edge Cases**:
  - Handles insertion at document boundaries
  - Validates position bounds

### DuplicateNodeCommand
- **Purpose**: Create an exact copy of a node and place it after the original
- **Implementation**:
  - Clones the source node
  - Inserts the duplicate right after the original
  - Tracks new node index for proper undo
- **Edge Cases**:
  - Handles node index validation

## Future Planned Commands

### Phase 2: Selection-Based Operations
- `CutSelectionCommand` - Cut selected content to clipboard
- `CopySelectionCommand` - Copy selected content
- `SelectionFormatCommand` - Format selected content spanning multiple nodes
- `SelectionIndentCommand` - Increase/decrease indentation of selected blocks

### Phase 3: Advanced Document Structure
- `TableOperationsCommand` - Add/remove rows/columns
- `GroupNodesCommand` - Group nodes under a container
- `CreateTableCommand` - Insert and structure table content
- `CreateTOCCommand` - Generate table of contents from headings

### Phase 4: Collaboration Features
- `AddCommentCommand` - Add inline comment to content
- `ResolveCommentCommand` - Mark comment as resolved
- `TrackChangesCommand` - Enable/disable change tracking
- `AcceptRejectChangesCommand` - Process tracked changes

## Command Pattern Implementation

All commands follow the same pattern:
```rust
pub trait Command {
    fn execute(&mut self) -> Result<(), EditError>;
    fn undo(&mut self) -> Result<(), EditError>;
    fn as_any(&self) -> &dyn Any;
}
```

This pattern enables:
- Consistent execution of operations
- Undo/redo functionality
- Command history tracking
- Type-safe downcasting when needed

## Testing
All commands include comprehensive tests to verify both their functionality and their undo operations.
