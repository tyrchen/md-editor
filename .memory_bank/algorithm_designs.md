# 🎨🎨🎨 ENTERING CREATIVE PHASE: ALGORITHM DESIGN

## Core Markdown Editing Operations

This document explores algorithm designs for key operations in the md-editor core library.

## Component Description

The editing operations component provides algorithms for manipulating Markdown documents, including insertion, deletion, transformation, and navigation operations. These algorithms operate on the document model to provide efficient and predictable editing behaviors.

## Requirements & Constraints

- **Correctness**: Operations must maintain document validity
- **Performance**: Operations should scale well with document size
- **Composability**: Operations should be composable into more complex operations
- **Undoable**: All operations should be reversible
- **Structure-Aware**: Operations should respect document structure (e.g., list hierarchies)
- **Minimal Side Effects**: Operations should have predictable and limited effects on the document
- **Selection Handling**: Operations should handle and update selections appropriately

## Key Algorithm Designs

### Document Traversal

#### Option 1: Recursive Depth-First Traversal

```rust
fn traverse_recursive(node: &Node, visitor: &mut dyn Visitor) -> VisitResult {
    if visitor.visit_pre(node) == VisitResult::Stop {
        return VisitResult::Stop;
    }

    for child in node.children() {
        if traverse_recursive(child, visitor) == VisitResult::Stop {
            return VisitResult::Stop;
        }
    }

    visitor.visit_post(node)
}
```

**Pros:**
- Simple implementation
- Natural mapping to recursive document structure
- Intuitive visit ordering

**Cons:**
- Stack overflow risk with deeply nested documents
- Cannot easily pause and resume traversal
- No straightforward way to skip subtrees without custom logic

#### Option 2: Iterative Traversal with Stack

```rust
fn traverse_iterative(root: &Node, visitor: &mut dyn Visitor) -> VisitResult {
    let mut stack = Vec::new();
    let mut current = Some(root);
    let mut post_visit = false;

    while let Some(node) = current {
        if !post_visit {
            if visitor.visit_pre(node) == VisitResult::Stop {
                return VisitResult::Stop;
            }

            if !node.children().is_empty() {
                stack.push((node, 0));
                current = Some(&node.children()[0]);
                continue;
            }
        }

        if visitor.visit_post(node) == VisitResult::Stop {
            return VisitResult::Stop;
        }

        if let Some((parent, idx)) = stack.last_mut() {
            *idx += 1;
            if *idx < parent.children().len() {
                current = Some(&parent.children()[*idx]);
                post_visit = false;
            } else {
                current = Some(*parent);
                stack.pop();
                post_visit = true;
            }
        } else {
            break;
        }
    }

    VisitResult::Continue
}
```

**Pros:**
- No recursion, no stack overflow risk
- Can be paused and resumed
- Better handling of very large documents

**Cons:**
- More complex implementation
- Requires maintaining explicit stack
- Less intuitive code structure

#### Option 3: Visitor Pattern with Callbacks

```rust
trait Visitor {
    fn visit_pre(&mut self, node: &Node) -> VisitResult;
    fn visit_post(&mut self, node: &Node) -> VisitResult;
    fn visit_children(&mut self, node: &Node) -> VisitResult {
        for child in node.children() {
            match self.visit(child) {
                VisitResult::Stop => return VisitResult::Stop,
                _ => continue,
            }
        }
        VisitResult::Continue
    }

    fn visit(&mut self, node: &Node) -> VisitResult {
        match self.visit_pre(node) {
            VisitResult::Stop => return VisitResult::Stop,
            VisitResult::SkipChildren => {},
            VisitResult::Continue => {
                if self.visit_children(node) == VisitResult::Stop {
                    return VisitResult::Stop;
                }
            }
        }

        self.visit_post(node)
    }
}
```

**Pros:**
- Flexible control over traversal behavior
- Clear separation of concerns
- Can easily implement node-specific behaviors

**Cons:**
- Requires implementing visitor interface for each operation
- Slightly more overhead from trait dispatch
- Can lead to complex visitor implementations

### Document Path Resolution

#### Option 1: Direct Index Path Traversal

```rust
fn resolve_path(root: &Node, path: &[usize]) -> Option<&Node> {
    let mut current = root;

    for &index in path {
        if index >= current.children().len() {
            return None;
        }
        current = &current.children()[index];
    }

    Some(current)
}
```

**Pros:**
- Very simple implementation
- Fast for valid paths
- Minimal overhead

**Cons:**
- No error details when path is invalid
- Doesn't handle dynamic document changes well
- Can panic if not carefully checked

#### Option 2: Path Resolution with Validation

```rust
fn resolve_path(root: &Node, path: &[usize]) -> Result<&Node, PathError> {
    let mut current = root;

    for (depth, &index) in path.iter().enumerate() {
        if index >= current.children().len() {
            return Err(PathError::InvalidIndex {
                depth,
                index,
                max_valid: current.children().len().saturating_sub(1)
            });
        }
        current = &current.children()[index];
    }

    Ok(current)
}
```

**Pros:**
- Detailed error information
- Safe operation
- Better debugging support

**Cons:**
- Slightly more overhead
- More complex return type handling
- May be unnecessary for internal operations

#### Option 3: Cached Path Resolution

```rust
struct PathCache {
    cache: HashMap<Vec<usize>, Weak<Node>>,
}

impl PathCache {
    fn resolve_path(&mut self, root: &Rc<Node>, path: &[usize]) -> Result<Rc<Node>, PathError> {
        // Try to get from cache
        if let Some(weak_node) = self.cache.get(path) {
            if let Some(node) = weak_node.upgrade() {
                return Ok(node);
            }
        }

        // Normal resolution
        let mut current = Rc::clone(root);
        for (depth, &index) in path.iter().enumerate() {
            if index >= current.children().len() {
                return Err(PathError::InvalidIndex {
                    depth, index, max_valid: current.children().len().saturating_sub(1)
                });
            }
            current = Rc::clone(&current.children()[index]);
        }

        // Update cache
        self.cache.insert(path.to_vec(), Rc::downgrade(&current));

        Ok(current)
    }

    fn invalidate(&mut self, prefix: &[usize]) {
        self.cache.retain(|path, _| !path.starts_with(prefix));
    }
}
```

**Pros:**
- Much faster for repeated access to the same nodes
- Especially beneficial for UI that frequently accesses the same paths
- Can intelligently invalidate only affected paths

**Cons:**
- Significantly more complex
- Requires memory management
- Additional overhead for cache maintenance
- Needs invalidation when document changes

### Text Insertion

#### Option 1: Simple Recursive Split

```rust
fn insert_text(node: &mut TextNode, offset: usize, text: &str) -> Result<(), InsertError> {
    if offset > node.text().len() {
        return Err(InsertError::InvalidOffset);
    }

    let current_text = node.text();
    let new_text = format!(
        "{}{}{}",
        &current_text[..offset],
        text,
        &current_text[offset..]
    );

    node.set_text(new_text);
    Ok(())
}
```

**Pros:**
- Simple and straightforward implementation
- Works well for small documents
- Easy to understand

**Cons:**
- Creates new string allocations
- Potentially inefficient for large documents
- No structure awareness

#### Option 2: Rope-Based Text Handling

```rust
struct TextNode {
    content: Rope,
    // other fields
}

impl TextNode {
    fn insert_text(&mut self, offset: usize, text: &str) -> Result<(), InsertError> {
        if offset > self.content.len() {
            return Err(InsertError::InvalidOffset);
        }

        self.content.insert(offset, text);
        Ok(())
    }
}
```

**Pros:**
- Efficient for large documents
- Minimal copying
- Supports efficient slicing and concatenation

**Cons:**
- More complex data structure
- Additional dependency
- Slightly higher overhead for small texts

#### Option 3: Operation-Based Editing

```rust
enum Operation {
    InsertText { path: Vec<usize>, offset: usize, text: String },
    DeleteText { path: Vec<usize>, offset: usize, length: usize },
    // other operations
}

fn apply_operation(doc: &mut Document, op: Operation) -> Result<(), OperationError> {
    match op {
        Operation::InsertText { path, offset, text } => {
            let node = doc.resolve_path_mut(&path)?;
            match node {
                Node::Text(text_node) => text_node.insert_text(offset, &text),
                _ => Err(OperationError::InvalidNodeType),
            }
        },
        // handle other operations
    }
}
```

**Pros:**
- Supports operation history
- Enables undo/redo
- Operations can be transmitted for collaborative editing
- Path-based addressing works with document structure

**Cons:**
- More complex implementation
- Additional indirection
- Requires maintaining operation log

### Node Splitting

#### Option 1: Basic Node Split

```rust
fn split_node(doc: &mut Document, path: &[usize], offset: usize) -> Result<SplitResult, SplitError> {
    let node = doc.resolve_path_mut(path)?;

    match node {
        Node::Paragraph(para) => {
            let text = para.text();
            if offset > text.len() {
                return Err(SplitError::InvalidOffset);
            }

            // Create two new paragraph nodes
            let first_para = Node::paragraph(&text[..offset]);
            let second_para = Node::paragraph(&text[offset..]);

            // Find parent to replace this node with the two new ones
            let parent_path = &path[..path.len()-1];
            let index = *path.last().unwrap();
            let parent = doc.resolve_path_mut(parent_path)?;

            // Replace node with the two new ones
            parent.children_mut().remove(index);
            parent.children_mut().insert(index, first_para);
            parent.children_mut().insert(index + 1, second_para);

            Ok(SplitResult {
                first_path: path.to_vec(),
                second_path: [&parent_path[..], &[index + 1]].concat(),
            })
        },
        // handle other node types
        _ => Err(SplitError::UnsupportedNodeType),
    }
}
```

**Pros:**
- Relatively straightforward implementation
- Works for simple document structures
- Clear return paths

**Cons:**
- Doesn't handle complex node types well
- Limited flexibility
- May create invalid documents with certain node types

#### Option 2: Structure-Aware Split

```rust
fn split_node(doc: &mut Document, path: &[usize], offset: usize) -> Result<SplitResult, SplitError> {
    let node = doc.resolve_path(path)?;

    match node {
        Node::Paragraph(para) => {
            // Similar to Option 1, but simpler implementation
        },
        Node::ListItem(item) => {
            // Special handling for list items
            // 1. Split the content of the list item
            // 2. Create a new list item with the second part
            // 3. Handle nested lists appropriately
            // 4. Maintain list structure
        },
        Node::Heading(heading) => {
            // Convert second part to paragraph instead of heading
        },
        // Other specialized node handling
    }
}
```

**Pros:**
- Respects document structure
- Produces more natural editing behavior
- Better supports complex node types

**Cons:**
- Much more complex implementation
- Requires specialized handling for each node type
- More difficult to maintain

#### Option 3: Operation-Based Split with Command Pattern

```rust
struct SplitNodeCommand {
    path: Vec<usize>,
    offset: usize,
    node_type: NodeType,
}

impl Command for SplitNodeCommand {
    fn execute(&self, doc: &mut Document) -> Result<CommandResult, CommandError> {
        // Implementation similar to other options
    }

    fn undo(&self, doc: &mut Document, result: &CommandResult) -> Result<(), CommandError> {
        // Merge the previously split nodes back together
    }
}
```

**Pros:**
- Supports undo/redo naturally
- Clean separation of concerns
- Can be extended with specialized command types

**Cons:**
- More complex architecture
- Requires command result tracking
- Higher overhead

### Selection Operations

#### Option 1: Basic Start/End Position Selection

```rust
struct Selection {
    start: Position,
    end: Position,
}

impl Selection {
    fn is_collapsed(&self) -> bool {
        self.start == self.end
    }

    fn extend_to_character(&mut self, doc: &Document, direction: Direction) -> Result<(), SelectionError> {
        match direction {
            Direction::Forward => {
                // Move end position one character forward
                let Position { path, offset } = self.end;
                let node = doc.resolve_path(&path)?;
                if let Some(text_len) = node.text_length() {
                    if offset < text_len {
                        self.end.offset += 1;
                    } else {
                        // Move to next node
                        // ...
                    }
                }
            },
            Direction::Backward => {
                // Similar but opposite direction
            }
        }

        Ok(())
    }

    // Other selection operations
}
```

**Pros:**
- Simple data structure
- Easy to understand and visualize
- Works well for basic use cases

**Cons:**
- Limited expressiveness
- Difficult to handle complex selections (multi-cursor)
- Can become invalid if document changes

#### Option 2: Anchor/Focus Selection Model

```rust
struct Selection {
    anchor: Position,   // Where selection started
    focus: Position,    // Where selection currently is
}

impl Selection {
    fn normalize(&self) -> (Position, Position) {
        if self.anchor <= self.focus {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }

    fn is_collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    fn extend_to_character(&mut self, doc: &Document, direction: Direction) -> Result<(), SelectionError> {
        // Similar to Option 1, but only moves focus
    }

    // Other selection operations that maintain anchor
}
```

**Pros:**
- Better models user interaction
- Maintains selection direction
- Support for proper shift+arrow selection

**Cons:**
- Slightly more complex model
- Requires normalization for some operations
- Still limited for complex scenarios

#### Option 3: Range-Based Selection with Markers

```rust
struct SelectionMarker {
    id: MarkerID,
    position: Position,
    marker_type: MarkerType, // Anchor, Focus, etc.
}

struct Selection {
    anchor_id: MarkerID,
    focus_id: MarkerID,
}

struct SelectionManager {
    markers: HashMap<MarkerID, SelectionMarker>,
    selections: Vec<Selection>,

    // Methods to modify and manage selections
}
```

**Pros:**
- Supports multiple selections (multi-cursor)
- Can track selections through document changes
- More powerful selection model

**Cons:**
- Significantly more complex implementation
- Higher memory overhead
- Requires marker management

## Recommended Approach

For the core editing operations, I recommend combining:

1. **Visitor Pattern with Callbacks** for document traversal
2. **Path Resolution with Validation** for node addressing
3. **Operation-Based Editing** for text operations
4. **Structure-Aware Split** for node splitting
5. **Anchor/Focus Selection Model** for selections

This combination provides a good balance of:

- Flexibility for complex document structures
- Performance for most common operations
- Maintainability through clear abstraction boundaries
- Support for undo/redo through operation tracking
- Natural selection behavior matching user expectations

For larger documents or specialized use cases, consider incorporating:
- Rope-based text storage for large text nodes
- Cached path resolution for UI performance
- Range-based selection with markers for multi-cursor editing

## Implementation Guidelines

### Document Traversal

```rust
enum VisitResult {
    Continue,      // Continue normal traversal
    SkipChildren,  // Don't visit children of this node
    Stop,          // Stop traversal completely
}

trait Visitor {
    fn visit_pre(&mut self, node: &Node) -> VisitResult;
    fn visit_post(&mut self, node: &Node) -> VisitResult;

    // Default implementation that can be overridden
    fn visit_children(&mut self, node: &Node) -> VisitResult { /* ... */ }
    fn visit(&mut self, node: &Node) -> VisitResult { /* ... */ }
}

// Various visitor implementations for different operations
struct TextCollector { /* ... */ }
struct NodeCounter { /* ... */ }
struct NodeFinder { /* ... */ }
```

### Path Resolution

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    InvalidIndex { depth: usize, index: usize, max_valid: usize },
    EmptyPath,
    // Other error types as needed
}

fn resolve_path<'a>(root: &'a Node, path: &[usize]) -> Result<&'a Node, PathError> {
    // Implementation as described above
}

fn resolve_path_mut<'a>(root: &'a mut Node, path: &[usize]) -> Result<&'a mut Node, PathError> {
    // Similar to resolve_path but with mutable references
}
```

### Operation System

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    InsertText { path: Vec<usize>, offset: usize, text: String },
    DeleteText { path: Vec<usize>, offset: usize, length: usize },
    SplitNode { path: Vec<usize>, offset: usize },
    MergeNodes { first_path: Vec<usize>, second_path: Vec<usize> },
    // Other operations
}

#[derive(Debug)]
pub enum OperationError {
    PathError(PathError),
    InvalidOffset,
    InvalidNodeType,
    // Other error types
}

pub fn apply_operation(doc: &mut Document, op: &Operation) -> Result<(), OperationError> {
    // Implementation for each operation type
}

// For undo/redo support
pub fn invert_operation(doc: &Document, op: &Operation) -> Result<Operation, OperationError> {
    // Create inverse operations
}
```

### Node Splitting

```rust
#[derive(Debug)]
pub struct SplitResult {
    first_path: Vec<usize>,
    second_path: Vec<usize>,
}

#[derive(Debug)]
pub enum SplitError {
    PathError(PathError),
    InvalidOffset,
    UnsupportedNodeType,
    // Other error types
}

pub fn split_node(doc: &mut Document, path: &[usize], offset: usize) -> Result<SplitResult, SplitError> {
    // Implementation with structure-aware node splitting
}
```

### Selection Model

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    path: Vec<usize>,
    offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    anchor: Position,
    focus: Position,
}

impl Selection {
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    pub fn normalized(&self) -> (Position, Position) {
        if self.anchor <= self.focus {
            (self.anchor.clone(), self.focus.clone())
        } else {
            (self.focus.clone(), self.anchor.clone())
        }
    }

    // Various selection modification methods
    pub fn extend_character(&mut self, doc: &Document, direction: Direction) -> Result<(), SelectionError> {
        // Implementation
    }

    pub fn extend_word(&mut self, doc: &Document, direction: Direction) -> Result<(), SelectionError> {
        // Implementation
    }

    pub fn extend_line(&mut self, doc: &Document, direction: Direction) -> Result<(), SelectionError> {
        // Implementation
    }

    // Other selection operations
}
```

## Verification

The proposed approach meets the requirements in the following ways:

1. **Correctness**: Structure-aware operations maintain document validity.
2. **Performance**: Path validation and visitor pattern provide good performance with clear error cases.
3. **Composability**: Operation-based editing allows operations to be composed.
4. **Undoable**: Operations can be inverted for undo/redo.
5. **Structure-Aware**: Node-specific behavior handles document structure properly.
6. **Minimal Side Effects**: Operations have well-defined scopes.
7. **Selection Handling**: Anchor/focus model provides natural selection behavior.

## Implementation Priorities

1. First implement the core data structures and traversal algorithms
2. Next implement the path resolution system
3. Then build the basic text editing operations
4. Followed by node splitting and merging operations
5. Finally implement the selection model and operations

This ensures that the foundation is solid before building more complex functionality on top.

# 🎨🎨🎨 EXITING CREATIVE PHASE
