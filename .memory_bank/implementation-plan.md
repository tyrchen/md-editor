# Markdown Editor Implementation Plan

## Phase 1: Core Content Insertion Commands ✅ COMPLETED

We've successfully implemented the core content insertion commands to complement the existing deletion and editing commands:

- `InsertTextCommand` - Insert text at specific positions within nodes
- `InsertNodeCommand` - Insert new nodes at specified positions
- `DuplicateNodeCommand` - Create a copy of an existing node

These commands provide essential editing capabilities and follow the Command pattern with proper undo/redo functionality. The implementation handles edge cases such as insertion at boundaries, multi-node text formatting, and ensures that the document structure remains valid.

All commands have been thoroughly tested with unit tests that verify both their execution and undo capabilities.

## Phase 2: Selection-Based Operations (PLANNED)

The next phase will focus on implementing a selection model and commands that operate on selections:

- [ ] Implement a Selection data structure
- [ ] Add SelectionState to the Document model
- [ ] Implement selection commands:
  - [ ] `CutSelectionCommand` - Cut selected content to clipboard
  - [ ] `CopySelectionCommand` - Copy selected content
  - [ ] `SelectionFormatCommand` - Format selected content spanning multiple nodes
  - [ ] `SelectionIndentCommand` - Increase/decrease indentation of selected blocks

## Phase 3: Advanced Document Structure (PLANNED)

Once selection operations are in place, we'll implement more complex document structure commands:

- [ ] `TableOperationsCommand` - Add/remove rows/columns
- [ ] `GroupNodesCommand` - Group nodes under a container
- [ ] `CreateTableCommand` - Insert and structure table content
- [ ] `CreateTOCCommand` - Generate table of contents from headings

## Phase 4: Collaboration Features (PLANNED)

The final phase will introduce collaboration features:

- [ ] `AddCommentCommand` - Add inline comment to content
- [ ] `ResolveCommentCommand` - Mark comment as resolved
- [ ] `TrackChangesCommand` - Enable/disable change tracking
- [ ] `AcceptRejectChangesCommand` - Process tracked changes

## Command Pattern

All commands follow the same Command pattern:

```rust
pub trait Command {
    fn execute(&mut self) -> Result<(), EditError>;
    fn undo(&mut self) -> Result<(), EditError>;
    fn as_any(&self) -> &dyn Any;
}
```

This ensures consistent operation, proper undo/redo functionality, and seamless integration with the Editor's command history.
