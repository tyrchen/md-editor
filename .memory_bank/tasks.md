# Markdown Editor Tasks

## Completed Tasks

### Phase 1: Content Insertion Commands
- [x] Implement `InsertTextCommand` for inserting text at specific positions
- [x] Implement `InsertNodeCommand` for inserting new nodes
- [x] Add helper methods for creating common node types
- [x] Implement `DuplicateNodeCommand` for cloning existing nodes
- [x] Add corresponding methods to the Editor struct
- [x] Write tests for all new commands
- [x] Create documentation in memory bank

## Current Tasks

### Phase 2: Selection-Based Operations
- [ ] Design Selection data structure
- [ ] Add SelectionState to Document model
- [ ] Implement `CutSelectionCommand`
- [ ] Implement `CopySelectionCommand`
- [ ] Implement `SelectionFormatCommand`
- [ ] Implement `SelectionIndentCommand`

## Upcoming Tasks

### Phase 3: Advanced Document Structure
- [ ] Design table data structure
- [ ] Implement `TableOperationsCommand`
- [ ] Implement `CreateTableCommand`
- [ ] Implement `GroupNodesCommand`
- [ ] Implement `CreateTOCCommand`

### Phase 4: Collaboration Features
- [ ] Design comment/annotation data structure
- [ ] Implement `AddCommentCommand`
- [ ] Implement `ResolveCommentCommand`
- [ ] Design change tracking system
- [ ] Implement `TrackChangesCommand`
- [ ] Implement `AcceptRejectChangesCommand`

## Notes
- All commands should follow the Command pattern
- All commands should be properly tested for both execute and undo operations
- Document all commands in the memory bank
