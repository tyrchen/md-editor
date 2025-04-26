# Code Bank

## convert/html.rs
```rust
impl TryFrom<Text<Html>> for Document {
    fn try_from(html: Text<Html>) -> Result<Self, Self::Error> { ... }
}

impl TryFrom<&Document> for Text<Html> {
    fn try_from(document: &Document) -> Result<Self, Self::Error> { ... }
}

```

## convert/json.rs
```rust
impl TryFrom<Text<Json>> for Document {
    fn try_from(json: Text<Json>) -> Result<Self, Self::Error> { ... }
}

impl TryFrom<&Document> for Text<Json> {
    fn try_from(document: &Document) -> Result<Self, Self::Error> { ... }
}

```

## convert/markdown/mod.rs
```rust
impl TryFrom<Text<Markdown>> for Document {
    fn try_from(markdown: Text<Markdown>) -> Result<Self, Self::Error> { ... }
}

impl TryFrom<&Document> for Text<Markdown> {
    fn try_from(document: &Document) -> Result<Self, Self::Error> { ... }
}

```

## convert/markdown/parser.rs
```rust
impl ParserStack {
    fn new() -> Self { ... }
    /// Get a mutable reference to the nodes of the current context.
    fn current_nodes(&mut self) -> &mut Vec<Node> { ... }
    /// Get the current context.
    fn current_context(&self) -> &Context { ... }
    /// Start a new context (e.g., entering a list).
    fn push_context(&mut self, context: Context) { ... }
    /// End the current context, returning the constructed Node.
    fn pop_context(&mut self) -> Option<Node> { ... }
    /// Add an inline node to the current accumulator.
    fn push_inline(&mut self, inline: InlineNode) { ... }
    /// Flush accumulated inline nodes into a Paragraph node if applicable.
    fn flush_inline_accumulator(&mut self) { ... }
    /// Handle Text events.
    fn handle_text(&mut self, text: String) { ... }
}



```

## convert/mod.rs
```rust
pub mod html {
}


pub mod json {
}


pub mod markdown {
}


pub struct Html;

pub struct Json;

pub struct Markdown;

pub struct Text<T> {
    text: String,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Text<T> {
    pub fn new(text: impl Into<String>) -> Self{ ... }
    pub fn as_str(&self) -> &str{ ... }
    pub fn into_inner(self) -> String{ ... }
}









```

## editor/command.rs
```rust
/// Command to delete text from a node
pub struct DeleteTextCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    start: usize,
    end: usize,
    deleted_text: Option<String>,
}

/// Command to merge two adjacent nodes
pub struct MergeNodesCommand {
    document: Rc<RefCell<Document>>,
    first_index: usize,
    second_index: usize,
    original_second_node: Option<Node>,
}

/// Trait representing a document editing command that can be executed and undone
pub trait Command {
}


impl DeleteTextCommand {
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_index: usize,
            start: usize,
            end: usize,
        ) -> Self{ ... }
}

impl Command for DeleteTextCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

impl MergeNodesCommand {
    pub fn new(document: Rc<RefCell<Document>>, first_index: usize, second_index: usize) -> Self{ ... }
}

impl Command for MergeNodesCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/add_task_item.rs
```rust
/// Command to add a new item to a task list
pub struct AddTaskItemCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    position: usize,
    text: String,
    checked: bool,
    added_index: Option<usize>,
}

impl AddTaskItemCommand {
    /// Create a new command to add an item to a task list
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_index: usize,
            position: usize,
            text: impl Into<String>,
            checked: bool,
        ) -> Self{ ... }
}

impl Command for AddTaskItemCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/copy_selection.rs
```rust
/// Command to copy selected content
pub struct CopySelectionCommand {
    document: Rc<RefCell<Document>>,
    /// The nodes that were copied
    copied_nodes: Vec<Node>,
}

impl CopySelectionCommand {
    pub fn new(document: Rc<RefCell<Document>>) -> Self{ ... }
    /// Get the nodes that were copied
    pub fn get_copied_nodes(&self) -> &[Node]{ ... }
}

impl Command for CopySelectionCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/create_table.rs
```rust
/// Command to create a table
#[allow(dead_code)]
pub struct CreateTableCommand {
    document: Rc<RefCell<Document>>,
    position: usize,
    columns: usize,
    rows: usize,
    alignments: Option<Vec<TableAlignment>>,
    header_data: Option<Vec<String>>,
    row_data: Option<Vec<Vec<String>>>,
    properties: TableProperties,
    old_node: Option<Node>,
}

impl CreateTableCommand {
    /// Create a new table with default alignments
    pub fn new(
            document: Rc<RefCell<Document>>,
            position: usize,
            columns: usize,
            rows: usize,
        ) -> Self{ ... }
    /// Create a new table with specific alignments
    pub fn with_alignments(
            document: Rc<RefCell<Document>>,
            position: usize,
            columns: usize,
            rows: usize,
            alignments: Vec<TableAlignment>,
        ) -> Self{ ... }
    /// Create a new table with specific data
    pub fn with_data(
            document: Rc<RefCell<Document>>,
            position: usize,
            header: Vec<String>,
            rows: Vec<Vec<String>>,
            alignments: Option<Vec<TableAlignment>>,
        ) -> Self{ ... }
    /// Create a new table with properties
    pub fn with_properties(
            document: Rc<RefCell<Document>>,
            position: usize,
            columns: usize,
            rows: usize,
            properties: TableProperties,
        ) -> Self{ ... }
    /// Create a new table with data and properties
    pub fn with_data_and_properties(
            document: Rc<RefCell<Document>>,
            position: usize,
            header: Vec<String>,
            rows: Vec<Vec<String>>,
            alignments: Option<Vec<TableAlignment>>,
            properties: TableProperties,
        ) -> Self{ ... }
}

impl Command for CreateTableCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/create_toc.rs
```rust
/// Command to create a table of contents from document headings
pub struct CreateTOCCommand {
    document: Rc<RefCell<Document>>,
    /// Position to insert the TOC
    position: usize,
    /// Maximum heading level to include (1 to 6)
    max_level: u8,
    /// Original document state for undo
    original_nodes: Option<Vec<Node>>,
}

impl CreateTOCCommand {
    /// Create a new TOC command
    pub fn new(document: Rc<RefCell<Document>>, position: usize, max_level: u8) -> Self{ ... }
}

impl Command for CreateTOCCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/cut_selection.rs
```rust
/// Command to cut the currently selected content
pub struct CutSelectionCommand {
    document: Rc<RefCell<Document>>,
    /// Store the original selection for undo
    original_selection: Option<Selection>,
    /// Store the original nodes that were modified or deleted
    original_nodes: Vec<(usize, Node)>,
    /// Store cut content for clipboard or undo
    cut_content: Vec<Node>,
}

impl CutSelectionCommand {
    pub fn new(document: Rc<RefCell<Document>>) -> Self{ ... }
    /// Get the content that was cut
    pub fn cut_content(&self) -> &[Node]{ ... }
}

impl Command for CutSelectionCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/delete_node.rs
```rust
/// Command to delete a node from a document
pub struct DeleteNodeCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    deleted_node: Option<Node>,
}

impl DeleteNodeCommand {
    pub fn new(document: Rc<RefCell<Document>>, node_index: usize) -> Self{ ... }
}

impl Command for DeleteNodeCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/duplicate_node.rs
```rust
/// Command to duplicate a node in the document
pub struct DuplicateNodeCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    // Store the new node index for undo
    new_node_index: Option<usize>,
    // Store node type for validation
    node_type: Option<String>,
}

impl DuplicateNodeCommand {
    pub fn new(document: Rc<RefCell<Document>>, node_index: usize) -> Self{ ... }
    fn get_node_type(node: &Node) -> String { ... }
}

impl Command for DuplicateNodeCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/edit_task_item.rs
```rust
/// Command for editing the text content of a task list item.
///
/// # Examples
///
/// ```
/// use md_core::{Editor, Document};
///
/// let mut document = Document::new();
/// let mut editor = Editor::new(document);
///
/// // Assuming document has a task list at index 0
/// let result = editor.edit_task_item(0, 1, "Updated task description");
/// ```
#[derive(Debug)]
pub struct EditTaskItemCommand {
    /// Document to modify
    document: Rc<RefCell<Document>>,
    /// Index of the task list node
    node_idx: usize,
    /// Index of the task item to edit
    item_idx: usize,
    /// New text content for the task item
    text: String,
    /// Previous text content (used for undo)
    previous_text: Option<String>,
}

impl EditTaskItemCommand {
    /// Creates a new command to edit a task list item.
    ///
    /// # Arguments
    ///
    /// * `document` - Reference to the document to modify
    /// * `node_idx` - Index of the task list node in the document
    /// * `item_idx` - Index of the task item within the list
    /// * `text` - New text content for the task item
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_idx: usize,
            item_idx: usize,
            text: impl Into<String>,
        ) -> Self{ ... }
}

impl Command for EditTaskItemCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/find_replace.rs
```rust
/// Command for finding and replacing text throughout a document
pub struct FindReplaceCommand {
    /// Reference to the document
    document: Rc<RefCell<Document>>,
    /// String to find
    find: String,
    /// String to replace with
    replace: String,
    /// Whether the search should be case sensitive
    case_sensitive: bool,
    /// Original nodes state for undo
    original_nodes: Vec<(usize, Node)>,
    /// Count of replacements made
    replacements: usize,
}

impl FindReplaceCommand {
    /// Create a new FindReplaceCommand
    pub fn new(
            document: Rc<RefCell<Document>>,
            find: &str,
            replace: &str,
            case_sensitive: bool,
        ) -> Self{ ... }
    /// Get the number of replacements made
    pub fn replacements(&self) -> usize{ ... }
}

impl Command for FindReplaceCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/format_text.rs
```rust
/// Command to format text within a paragraph node
pub struct FormatTextCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    start: usize,
    end: usize,
    formatting: TextFormatting,
    original_nodes: Option<Vec<InlineNode>>,
}

impl FormatTextCommand {
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_index: usize,
            start: usize,
            end: usize,
            formatting: TextFormatting,
        ) -> Self{ ... }
}

impl Command for FormatTextCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/group_nodes.rs
```rust
/// Command to group multiple nodes into a container node
pub struct GroupNodesCommand {
    document: Rc<RefCell<Document>>,
    /// The indices of nodes to group
    node_indices: Vec<usize>,
    /// The name/type of the group container
    group_name: String,
    /// Original document state for undo
    original_nodes: Option<Vec<Node>>,
}

impl GroupNodesCommand {
    /// Create a new group nodes command
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_indices: Vec<usize>,
            group_name: String,
        ) -> Self{ ... }
}

impl Command for GroupNodesCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/indent_task_item.rs
```rust
/// The direction in which to indent a task item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentDirection {
    /// Indent the task item (increase nesting level)
    Increase,
    /// Dedent the task item (decrease nesting level)
    Decrease,
}

/// Command for indenting/dedenting task items in a task list
#[derive(Debug)]
pub struct IndentTaskItemCommand {
    /// Document to modify
    document: Rc<RefCell<Document>>,
    /// Index of the task list node
    node_idx: usize,
    /// Index of the task item to indent/dedent
    item_idx: usize,
    /// The direction of indentation (increase or decrease)
    direction: IndentDirection,
    /// Stores the original items for undo
    original_items: Option<Vec<ListItem>>,
}

impl IndentTaskItemCommand {
    /// Create a new command to indent or dedent a task item
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_idx: usize,
            item_idx: usize,
            direction: IndentDirection,
        ) -> Self{ ... }
    /// Create a command to increase the indent of a task item
    pub fn increase_indent(
            document: Rc<RefCell<Document>>,
            node_idx: usize,
            item_idx: usize,
        ) -> Self{ ... }
    /// Create a command to decrease the indent of a task item
    pub fn decrease_indent(
            document: Rc<RefCell<Document>>,
            node_idx: usize,
            item_idx: usize,
        ) -> Self{ ... }
}

impl Command for IndentTaskItemCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/insert_node.rs
```rust
/// Command to insert a new node at a specific position in the document
pub struct InsertNodeCommand {
    document: Rc<RefCell<Document>>,
    position: usize,
    node: Node,
    // Store the inserted index for undo
    inserted_index: Option<usize>,
}

impl InsertNodeCommand {
    /// Create a new command to insert a node at the specified position
    pub fn new(document: Rc<RefCell<Document>>, position: usize, node: Node) -> Self{ ... }
    /// Helper method to create a new paragraph node with text
    pub fn new_paragraph(document: Rc<RefCell<Document>>, position: usize, text: &str) -> Self{ ... }
    /// Helper method to create a new heading node
    pub fn new_heading(
            document: Rc<RefCell<Document>>,
            position: usize,
            level: u8,
            text: &str,
        ) -> Self{ ... }
    /// Helper method to create a new code block
    pub fn new_code_block(
            document: Rc<RefCell<Document>>,
            position: usize,
            code: &str,
            language: &str,
        ) -> Self{ ... }
    /// Creates a command to insert a thematic break (horizontal rule)
    ///
    /// # Arguments
    /// * `document` - Reference to the document
    /// * `position` - Position to insert the thematic break
    ///
    /// # Returns
    /// A new InsertNodeCommand
    #[allow(dead_code)]
    pub fn new_thematic_break(document: Rc<RefCell<Document>>, position: usize) -> Self{ ... }
}

impl Command for InsertNodeCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/insert_text.rs
```rust
/// Command to insert text at a specific position in a node
pub struct InsertTextCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    position: usize,
    text: String,
    // For undo: track if any formatting was split during insert
    affected_nodes: Option<Vec<InlineNode>>,
}

impl InsertTextCommand {
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_index: usize,
            position: usize,
            text: String,
        ) -> Self{ ... }
}

impl Command for InsertTextCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/mod.rs
```rust
pub mod add_task_item {
}


pub mod copy_selection {
}


pub mod create_table {
}


pub mod create_toc {
}


pub mod cut_selection {
}


pub mod delete_node {
}


pub mod duplicate_node {
}


pub mod edit_task_item {
}


pub mod find_replace {
}


pub mod format_text {
}


pub mod group_nodes {
}


pub mod indent_task_item {
}


pub mod insert_node {
}


pub mod insert_text {
}


pub mod move_node {
}


pub mod move_task_item {
}


pub mod node_conversion {
}


pub mod remove_task_item {
}


pub mod selection_format {
}


pub mod selection_indent {
}


pub mod sort_task_list {
}


pub mod table_operations {
}


pub mod toggle_task {
}


```

## editor/commands/move_node.rs
```rust
/// Command to move a node from one position to another in a document
pub struct MoveNodeCommand {
    document: Rc<RefCell<Document>>,
    from_index: usize,
    to_index: usize,
}

impl MoveNodeCommand {
    pub fn new(document: Rc<RefCell<Document>>, from_index: usize, to_index: usize) -> Self{ ... }
}

impl Command for MoveNodeCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/move_task_item.rs
```rust
/// Command for moving a task list item up or down in a task list.
#[derive(Debug)]
pub struct MoveTaskItemCommand {
    /// Document to modify
    document: Rc<RefCell<Document>>,
    /// Index of the task list node
    node_idx: usize,
    /// Index of the task item to move
    item_idx: usize,
    /// Direction to move: true = down, false = up
    move_down: bool,
    /// Previous items state (for undo)
    previous_items: Option<Vec<ListItem>>,
}

/// Command for moving a task list item to a specific position
#[derive(Debug)]
pub struct MoveTaskPositionCommand {
    /// Document to modify
    document: Rc<RefCell<Document>>,
    /// Index of the task list node
    node_idx: usize,
    /// Original index of the task item
    from_idx: usize,
    /// Target index for the task item
    to_idx: usize,
    /// Previous items state (for undo)
    previous_items: Option<Vec<ListItem>>,
}

impl MoveTaskItemCommand {
    /// Creates a new command to move a task list item up or down.
    ///
    /// # Arguments
    ///
    /// * `document` - Reference to the document to modify
    /// * `node_idx` - Index of the task list node in the document
    /// * `item_idx` - Index of the task item within the list
    /// * `move_down` - Direction to move: true = down, false = up
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_idx: usize,
            item_idx: usize,
            move_down: bool,
        ) -> Self{ ... }
    /// Creates a command to move a task item down
    pub fn move_down(document: Rc<RefCell<Document>>, node_idx: usize, item_idx: usize) -> Self{ ... }
    /// Creates a command to move a task item up
    pub fn move_up(document: Rc<RefCell<Document>>, node_idx: usize, item_idx: usize) -> Self{ ... }
}

impl MoveTaskPositionCommand {
    /// Creates a new command to move a task list item to a specific position.
    ///
    /// # Arguments
    ///
    /// * `document` - Reference to the document to modify
    /// * `node_idx` - Index of the task list node in the document
    /// * `from_idx` - Current index of the task item
    /// * `to_idx` - Target index for the item
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_idx: usize,
            from_idx: usize,
            to_idx: usize,
        ) -> Self{ ... }
}

impl Command for MoveTaskItemCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

impl Command for MoveTaskPositionCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/node_conversion.rs
```rust
/// Command to convert a node from one type to another
pub struct ConvertNodeTypeCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    target_type: NodeConversionType,
    original_node: Option<Node>,
}

impl ConvertNodeTypeCommand {
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_index: usize,
            target_type: NodeConversionType,
        ) -> Self{ ... }
}

impl Command for ConvertNodeTypeCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/remove_task_item.rs
```rust
/// Command to remove an item from a task list
pub struct RemoveTaskItemCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    item_index: usize,
    removed_item: Option<ListItem>,
}

impl RemoveTaskItemCommand {
    /// Create a new command to remove an item from a task list
    pub fn new(document: Rc<RefCell<Document>>, node_index: usize, item_index: usize) -> Self{ ... }
}

impl Command for RemoveTaskItemCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/selection_format.rs
```rust
/// Command to apply formatting to the currently selected text
pub struct SelectionFormatCommand {
    document: Rc<RefCell<Document>>,
    formatting: TextFormatting,
    /// Store the original nodes for undo
    original_nodes: Vec<(usize, Node)>,
    /// Store the original selection
    original_selection: Option<Selection>,
}

impl SelectionFormatCommand {
    pub fn new(document: Rc<RefCell<Document>>, formatting: TextFormatting) -> Self{ ... }
}

impl Command for SelectionFormatCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/selection_indent.rs
```rust
/// Enum representing indentation direction
pub enum IndentDirection {
    /// Increase indentation level
    Increase,
    /// Decrease indentation level
    Decrease,
}

/// Command to indent or unindent selected content
pub struct SelectionIndentCommand {
    document: Rc<RefCell<Document>>,
    direction: IndentDirection,
    /// Store the original nodes for undo
    original_nodes: Vec<(usize, Node)>,
    /// Store the original selection
    original_selection: Option<Selection>,
}

impl SelectionIndentCommand {
    pub fn new(document: Rc<RefCell<Document>>, direction: IndentDirection) -> Self{ ... }
    fn list_types_compatible(type1: &ListType, type2: &ListType) -> bool { ... }
}

impl Command for SelectionIndentCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/sort_task_list.rs
```rust
/// Criteria for sorting task list items
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortCriteria {
    /// Sort alphabetically by task content
    Alphabetical,
    /// Sort with unchecked items first, then alphabetically
    UncheckedFirst,
    /// Sort with checked items first, then alphabetically
    CheckedFirst,
}

/// Command to sort task list items based on different criteria
#[derive(Debug)]
pub struct SortTaskListCommand {
    /// The document to operate on
    document: Document,
    /// Index of the task list node to sort
    node_idx: usize,
    /// Criteria for sorting
    criteria: SortCriteria,
    /// Original items for undo operation
    original_items: Option<Vec<ListItem>>,
}

impl SortTaskListCommand {
    /// Create a new sort task list command
    pub fn new(document: Document, node_idx: usize, criteria: SortCriteria) -> Self{ ... }
}

impl Command for SortTaskListCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/table_operations.rs
```rust
/// Types of table operations that can be performed
pub enum TableOperation {
    /// Add a row at the specified index (0 is first row after header)
    AddRow(usize),
    /// Remove the row at the specified index
    RemoveRow(usize),
    /// Add a column at the specified index (0 is first column)
    AddColumn(usize),
    /// Remove the column at the specified index
    RemoveColumn(usize),
    /// Change cell content at specified row and column
    SetCell {
        row: usize,
        column: usize,
        content: String,
        is_header: bool,
    },
    /// Set column alignment
    SetAlignment {
        column: usize,
        alignment: TableAlignment,
    },
    /// Set cell background color
    SetCellBackground {
        row: usize,
        column: usize,
        color: String,
        is_header: bool,
    },
    /// Set cell style
    SetCellStyle {
        row: usize,
        column: usize,
        style: String,
        is_header: bool,
    },
    /// Set cell span
    SetCellSpan {
        row: usize,
        column: usize,
        colspan: u32,
        rowspan: u32,
        is_header: bool,
    },
    /// Set table properties
    SetTableProperties(TableProperties),
}

/// Command to perform operations on an existing table
pub struct TableOperationsCommand {
    document: Rc<RefCell<Document>>,
    /// The index of the table node in the document
    node_index: usize,
    /// The operation to perform
    operation: TableOperation,
    /// Original node for undo
    original_node: Option<Node>,
}

impl TableOperationsCommand {
    /// Create a new table operations command
    pub fn new(
            document: Rc<RefCell<Document>>,
            node_index: usize,
            operation: TableOperation,
        ) -> Self{ ... }
}

impl Command for TableOperationsCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/commands/toggle_task.rs
```rust
/// Command to toggle the checked status of a task list item
pub struct ToggleTaskCommand {
    document: Rc<RefCell<Document>>,
    node_index: usize,
    item_index: usize,
    previous_state: Option<bool>,
}

impl ToggleTaskCommand {
    /// Create a new command to toggle a task list item
    pub fn new(document: Rc<RefCell<Document>>, node_index: usize, item_index: usize) -> Self{ ... }
}

impl Command for ToggleTaskCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn Any { ... }
}

```

## editor/mod.rs
```rust
/// Editor manages a document and provides operations to modify it
pub struct Editor {
    document: Rc<RefCell<Document>>,
    undo_stack: Vec<Box<dyn EditorCommand>>,
    redo_stack: Vec<Box<dyn EditorCommand>>,
    max_history: usize,
}

/// Enum representing node conversion types
pub enum NodeConversionType {
    /// Convert to paragraph
    Paragraph,
    /// Convert to heading with level
    Heading(u8),
    /// Convert to list with type
    List(ListType),
    /// Convert to code block with language
    CodeBlock(String),
    /// Convert to blockquote
    BlockQuote,
}

impl Editor {
    /// Creates a new editor instance with the given document
    pub fn new(document: Document) -> Self{ ... }
    /// Creates a new editor instance with a default empty document
    pub fn new_empty() -> Self{ ... }
    /// Get a reference to the current document
    pub fn document(&self) -> &Rc<RefCell<Document>>{ ... }
    /// Set the maximum number of operations to keep in history
    pub fn set_max_history(&mut self, max: usize){ ... }
    /// Delete text from a specific node
    pub fn delete_text(
            &mut self,
            node_index: usize,
            start: usize,
            end: usize,
        ) -> Result<(), EditError>{ ... }
    /// Merge two adjacent nodes of the same type
    pub fn merge_nodes(
            &mut self,
            first_index: usize,
            second_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Format text within a paragraph
    pub fn format_text(
            &mut self,
            node_index: usize,
            start: usize,
            end: usize,
            formatting: TextFormatting,
        ) -> Result<(), EditError>{ ... }
    /// Move a node from one position to another
    pub fn move_node(&mut self, from_index: usize, to_index: usize) -> Result<(), EditError>{ ... }
    /// Convert a node from one type to another
    pub fn convert_node_type(
            &mut self,
            node_index: usize,
            target_type: NodeConversionType,
        ) -> Result<(), EditError>{ ... }
    /// Delete a node entirely
    pub fn delete_node(&mut self, node_index: usize) -> Result<(), EditError>{ ... }
    /// Find and replace text across the document
    /// Returns the number of replacements made
    pub fn find_replace(&mut self, find: &str, replace: &str, case_sensitive: bool) -> usize{ ... }
    /// Undo the last operation
    pub fn undo(&mut self) -> Result<(), EditError>{ ... }
    /// Redo the last undone operation
    pub fn redo(&mut self) -> Result<(), EditError>{ ... }
    /// Execute a command and add it to the undo stack
    fn execute_command(&mut self, mut command: Box<dyn EditorCommand>) -> Result<(), EditError> { ... }
    /// Insert text at a specific position in a node
    pub fn insert_text(
            &mut self,
            node_index: usize,
            position: usize,
            text: &str,
        ) -> Result<(), EditError>{ ... }
    /// Insert a new node at a specific position in the document
    pub fn insert_node(&mut self, position: usize, node: Node) -> Result<(), EditError>{ ... }
    /// Insert a new paragraph with text at a specific position
    pub fn insert_paragraph(&mut self, position: usize, text: &str) -> Result<(), EditError>{ ... }
    /// Insert a new heading with text at a specific position
    pub fn insert_heading(
            &mut self,
            position: usize,
            level: u8,
            text: &str,
        ) -> Result<(), EditError>{ ... }
    /// Insert a new code block at a specific position
    pub fn insert_code_block(
            &mut self,
            position: usize,
            code: &str,
            language: &str,
        ) -> Result<(), EditError>{ ... }
    /// Duplicate a node at a specific index
    pub fn duplicate_node(&mut self, node_index: usize) -> Result<(), EditError>{ ... }
    /// Cut the currently selected content
    /// Returns a vector of nodes that were cut
    pub fn cut_selection(&mut self) -> Vec<Node>{ ... }
    /// Copy the currently selected content without modifying the document
    /// Returns a vector of nodes that were copied
    pub fn copy_selection(&mut self) -> Vec<Node>{ ... }
    /// Apply formatting to the selected text
    pub fn format_selection(&mut self, formatting: TextFormatting) -> Result<(), EditError>{ ... }
    /// Increase the indentation of the selected content
    pub fn indent_selection(&mut self) -> Result<(), EditError>{ ... }
    /// Decrease the indentation of the selected content
    pub fn unindent_selection(&mut self) -> Result<(), EditError>{ ... }
    /// Create a table of contents from document headings
    ///
    /// - `position`: The position in the document where the TOC should be inserted
    /// - `max_level`: The maximum heading level to include (1-6)
    pub fn create_table_of_contents(
            &mut self,
            position: usize,
            max_level: u8,
        ) -> Result<(), EditError>{ ... }
    /// Create an empty table with default alignments
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `columns`: The number of columns in the table
    /// - `rows`: The number of rows in the table (not including header)
    pub fn create_table(
            &mut self,
            position: usize,
            columns: usize,
            rows: usize,
        ) -> Result<(), EditError>{ ... }
    /// Create a table with custom column alignments
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `columns`: The number of columns in the table
    /// - `rows`: The number of rows in the table (not including header)
    /// - `alignments`: Column alignments (one per column)
    pub fn create_table_with_alignments(
            &mut self,
            position: usize,
            columns: usize,
            rows: usize,
            alignments: Vec<TableAlignment>,
        ) -> Result<(), EditError>{ ... }
    /// Create a table with predefined data
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `header`: Header row cells
    /// - `rows`: Table data (rows of cells)
    /// - `alignments`: Optional column alignments
    pub fn create_table_with_data(
            &mut self,
            position: usize,
            header: Vec<String>,
            rows: Vec<Vec<String>>,
            alignments: Option<Vec<TableAlignment>>,
        ) -> Result<(), EditError>{ ... }
    /// Create a table with custom properties
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `columns`: The number of columns in the table
    /// - `rows`: The number of rows in the table (not including header)
    /// - `properties`: Table styling and behavior properties
    pub fn create_table_with_properties(
            &mut self,
            position: usize,
            columns: usize,
            rows: usize,
            properties: TableProperties,
        ) -> Result<(), EditError>{ ... }
    /// Create a table with predefined data and custom properties
    ///
    /// - `position`: The position in the document where the table should be inserted
    /// - `header`: Header row cells
    /// - `rows`: Table data (rows of cells)
    /// - `alignments`: Optional column alignments
    /// - `properties`: Table styling and behavior properties
    pub fn create_table_with_data_and_properties(
            &mut self,
            position: usize,
            header: Vec<String>,
            rows: Vec<Vec<String>>,
            alignments: Option<Vec<TableAlignment>>,
            properties: TableProperties,
        ) -> Result<(), EditError>{ ... }
    /// Add a row to an existing table
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row_index`: The index where the new row should be inserted (0 is first row after header)
    pub fn add_table_row(&mut self, node_index: usize, row_index: usize) -> Result<(), EditError>{ ... }
    /// Remove a row from an existing table
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row_index`: The index of the row to remove
    pub fn remove_table_row(
            &mut self,
            node_index: usize,
            row_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Add a column to an existing table
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `column_index`: The index where the new column should be inserted
    pub fn add_table_column(
            &mut self,
            node_index: usize,
            column_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Remove a column from an existing table
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `column_index`: The index of the column to remove
    pub fn remove_table_column(
            &mut self,
            node_index: usize,
            column_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Set the content of a table cell
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row`: The row index of the cell (ignored if is_header is true)
    /// - `column`: The column index of the cell
    /// - `content`: The new content for the cell
    /// - `is_header`: Whether the cell is in the header row
    pub fn set_table_cell(
            &mut self,
            node_index: usize,
            row: usize,
            column: usize,
            content: &str,
            is_header: bool,
        ) -> Result<(), EditError>{ ... }
    /// Set the alignment of a table column
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `column`: The column index
    /// - `alignment`: The alignment to set
    pub fn set_table_column_alignment(
            &mut self,
            node_index: usize,
            column: usize,
            alignment: TableAlignment,
        ) -> Result<(), EditError>{ ... }
    /// Group multiple nodes together
    ///
    /// - `node_indices`: Indices of nodes to group
    /// - `group_name`: Name or type of the group
    pub fn group_nodes(
            &mut self,
            node_indices: Vec<usize>,
            group_name: &str,
        ) -> Result<(), EditError>{ ... }
    /// Selects all content in the document
    pub fn select_all(&mut self) -> Result<(), EditError>{ ... }
    /// Selects a specific node by index
    pub fn select_node(&mut self, node_index: usize) -> Result<(), EditError>{ ... }
    /// Selects a range of nodes
    pub fn select_node_range(
            &mut self,
            start_index: usize,
            end_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Selects a specific range of text within a node
    pub fn select_text_range(
            &mut self,
            node_index: usize,
            start_offset: usize,
            end_offset: usize,
        ) -> Result<(), EditError>{ ... }
    /// Selects from one position to another across any nodes
    pub fn select_range(
            &mut self,
            start_node: usize,
            start_offset: usize,
            end_node: usize,
            end_offset: usize,
        ) -> Result<(), EditError>{ ... }
    /// Collapses the current selection to its start position
    pub fn collapse_selection_to_start(&mut self) -> Result<(), EditError>{ ... }
    /// Collapses the current selection to its end position
    pub fn collapse_selection_to_end(&mut self) -> Result<(), EditError>{ ... }
    /// Clears the current selection
    pub fn clear_selection(&mut self){ ... }
    /// Returns whether there is currently a selection
    pub fn has_selection(&self) -> bool{ ... }
    /// Returns whether the current selection spans multiple nodes
    pub fn has_multi_node_selection(&self) -> bool{ ... }
    /// Gets the currently selected text, if any
    pub fn get_selected_text(&self) -> Option<String>{ ... }
    /// Begin a transaction to group multiple operations into a single atomic change.
    ///
    /// Returns a Transaction object that can be used to build up a series of operations.
    /// The transaction is not applied to the document until it is committed and executed.
    ///
    /// # Example
    /// ```
    /// # use md_core::{Document, Editor, EditError};
    /// # fn example() -> Result<(), EditError> {
    /// # let doc = Document::new();
    /// # let mut editor = Editor::new(doc);
    /// let mut transaction = editor.begin_transaction();
    /// transaction.insert_text(0, 0, "Hello");
    /// editor.execute_transaction(transaction)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_transaction(&self) -> Transaction{ ... }
    /// Execute a transaction with a provided closure that builds the transaction.
    ///
    /// # Example
    /// ```
    /// # use md_core::{Document, Editor, TextFormatting, EditError};
    /// # fn example() -> Result<(), EditError> {
    /// # let doc = Document::new();
    /// # let mut editor = Editor::new(doc);
    /// editor.with_transaction(|mut transaction| {
    /// transaction
    /// .insert_text(0, 0, "Hello")
    /// .format_text(0, 0, 5, TextFormatting {
    /// bold: true,
    /// ..Default::default()
    /// });
    /// transaction
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_transaction<F>(&mut self, transaction_builder: F) -> Result<(), EditError>
        where
            F: FnOnce(Transaction) -> Transaction,{ ... }
    /// Execute a transaction, updating the undo stack as a single composite command.
    ///
    /// This method commits the transaction and applies the changes to the document.
    pub fn execute_transaction(&mut self, transaction: Transaction) -> Result<(), EditError>{ ... }
    /// Execute a list of commands from a transaction and add to undo stack.
    ///
    /// This is a lower-level method that's used by execute_transaction.
    pub fn execute_transaction_commands(
            &mut self,
            commands: Vec<Box<dyn EditorCommand>>,
        ) -> Result<(), EditError>{ ... }
    /// Set the background color of a table cell
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row`: The row index of the cell (0 is the first row after the header)
    /// - `column`: The column index of the cell (0 is the first column)
    /// - `color`: The background color in hex format (e.g., "#f5f5f5")
    /// - `is_header`: Whether to modify a header cell or a body cell
    pub fn set_table_cell_background(
            &mut self,
            node_index: usize,
            row: usize,
            column: usize,
            color: impl Into<String>,
            is_header: bool,
        ) -> Result<(), EditError>{ ... }
    /// Set custom CSS style for a table cell
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row`: The row index of the cell (0 is the first row after the header)
    /// - `column`: The column index of the cell (0 is the first column)
    /// - `style`: CSS style string (e.g., "font-weight: bold; color: red;")
    /// - `is_header`: Whether to modify a header cell or a body cell
    pub fn set_table_cell_style(
            &mut self,
            node_index: usize,
            row: usize,
            column: usize,
            style: impl Into<String>,
            is_header: bool,
        ) -> Result<(), EditError>{ ... }
    /// Set the spanning of a table cell
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `row`: The row index of the cell (0 is the first row after the header)
    /// - `column`: The column index of the cell (0 is the first column)
    /// - `colspan`: Number of columns this cell should span
    /// - `rowspan`: Number of rows this cell should span
    /// - `is_header`: Whether to modify a header cell or a body cell
    pub fn set_table_cell_span(
            &mut self,
            node_index: usize,
            row: usize,
            column: usize,
            colspan: u32,
            rowspan: u32,
            is_header: bool,
        ) -> Result<(), EditError>{ ... }
    /// Set table properties
    ///
    /// - `node_index`: The index of the table node in the document
    /// - `properties`: The table properties to set
    pub fn set_table_properties(
            &mut self,
            node_index: usize,
            properties: TableProperties,
        ) -> Result<(), EditError>{ ... }
    /// Toggle the checked status of a task list item
    pub fn toggle_task(&mut self, node_index: usize, item_index: usize) -> Result<(), EditError>{ ... }
    /// Increase the indentation level of a task list item
    pub fn indent_task_item(
            &mut self,
            node_index: usize,
            item_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Decrease the indentation level of a task list item
    pub fn dedent_task_item(
            &mut self,
            node_index: usize,
            item_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Add a new item to a task list
    pub fn add_task_item(
            &mut self,
            node_index: usize,
            position: usize,
            text: impl Into<String>,
            checked: bool,
        ) -> Result<(), EditError>{ ... }
    /// Remove an item from a task list
    pub fn remove_task_item(
            &mut self,
            node_index: usize,
            item_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Edit the text content of a task list item
    pub fn edit_task_item(
            &mut self,
            node_index: usize,
            item_index: usize,
            text: impl Into<String>,
        ) -> Result<(), EditError>{ ... }
    /// Move a task item within a task list
    ///
    /// # Arguments
    ///
    /// * `node_index` - The index of the task list node
    /// * `from_index` - The index of the item to move
    /// * `to_index` - The destination index
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful
    /// * `Err` with appropriate error if the operation failed
    pub fn move_task_item(
            &mut self,
            node_index: usize,
            from_index: usize,
            to_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Move a task item up in the task list (swap with the previous item)
    ///
    /// # Arguments
    ///
    /// * `node_index` - The index of the task list node
    /// * `item_index` - The index of the item to move up
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful
    /// * `Err` with appropriate error if the operation failed (e.g., already at the top)
    pub fn move_task_item_up(
            &mut self,
            node_index: usize,
            item_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Move a task item down in the task list (swap with the next item)
    ///
    /// # Arguments
    ///
    /// * `node_index` - The index of the task list node
    /// * `item_index` - The index of the item to move down
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful
    /// * `Err` with appropriate error if the operation failed (e.g., already at the bottom)
    pub fn move_task_item_down(
            &mut self,
            node_index: usize,
            item_index: usize,
        ) -> Result<(), EditError>{ ... }
    /// Sort the items in a task list according to specified criteria
    ///
    /// This method allows sorting task items within a task list by different criteria
    /// such as alphabetically or by completion status.
    ///
    /// # Arguments
    ///
    /// * `node_index` - The index of the task list node in the document
    /// * `criteria` - The criteria to use for sorting (alphabetical, unchecked first, or checked first)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was successful
    /// * `Err` with appropriate error if the operation failed
    ///
    /// # Example
    ///
    /// ```
    /// use md_core::{Document, Editor, ListItem, ListType, Node, SortCriteria};
    ///
    /// // Create a document with a task list
    /// let mut document = Document::new();
    /// let items = vec![
    /// ListItem::task("Buy groceries", true),
    /// ListItem::task("Call plumber", false),
    /// ListItem::task("Attend meeting", false),
    /// ];
    /// document.nodes.push(Node::List {
    /// list_type: ListType::Task,
    /// items,
    /// });
    ///
    /// let mut editor = Editor::new(document);
    ///
    /// // Sort the task list alphabetically
    /// editor.sort_task_list(0, SortCriteria::Alphabetical).unwrap();
    /// ```
    pub fn sort_task_list(
            &mut self,
            node_index: usize,
            criteria: SortCriteria,
        ) -> Result<(), EditError>{ ... }
}



impl EditorCommand for CompositeCommand {
    fn execute(&mut self) -> Result<(), EditError> { ... }
    fn undo(&mut self) -> Result<(), EditError> { ... }
    fn as_any(&self) -> &dyn std::any::Any { ... }
}

```

## editor/transaction.rs
```rust
/// A transaction that groups multiple document operations together as a single atomic change.
///
/// The transaction follows a builder pattern, allowing multiple operations to be chained
/// together before being committed as a single unit. This is useful for ensuring that
/// complex edits are applied atomically and can be undone as a single operation.
///
/// # Example
/// ```
/// # use md_core::{Document, Editor, EditError};
/// # fn example() -> Result<(), EditError> {
/// # let doc = Document::new();
/// # let mut editor = Editor::new(doc);
/// let mut transaction = editor.begin_transaction();
/// transaction
/// .insert_heading(0, 1, "Document Title")
/// .insert_paragraph(1, "First paragraph");
///
/// // Execute the transaction on the editor
/// editor.execute_transaction(transaction)?;
/// # Ok(())
/// # }
/// ```
///
/// Transactions are automatically rolled back if they are dropped without being committed,
/// ensuring that no changes are made to the document if a transaction is abandoned.
pub struct Transaction {
    document: Rc<RefCell<Document>>,
    commands: Vec<Box<dyn EditorCommand>>,
    committed: bool,
    selection: Option<Selection>,
}

impl Transaction {
    /// Creates a new transaction.
    ///
    /// This is typically called by the Editor's begin_transaction method.
    pub(crate) fn new(document: Rc<RefCell<Document>>) -> Self { ... }
    /// Add a command to the transaction.
    fn add_command<C: EditorCommand + 'static>(&mut self, command: C) { ... }
    /// Insert text at a specific position in a node.
    pub fn insert_text(&mut self, node_index: usize, position: usize, text: &str) -> &mut Self{ ... }
    /// Delete text from a specific node.
    pub fn delete_text(&mut self, node_index: usize, start: usize, end: usize) -> &mut Self{ ... }
    /// Format text within a paragraph.
    pub fn format_text(
            &mut self,
            node_index: usize,
            start: usize,
            end: usize,
            formatting: TextFormatting,
        ) -> &mut Self{ ... }
    /// Insert a new node at a specific position.
    pub fn insert_node(&mut self, position: usize, node: Node) -> &mut Self{ ... }
    /// Insert a paragraph with text.
    pub fn insert_paragraph(&mut self, position: usize, text: &str) -> &mut Self{ ... }
    /// Insert a heading with text.
    pub fn insert_heading(&mut self, position: usize, level: u8, text: &str) -> &mut Self{ ... }
    /// Insert a code block.
    pub fn insert_code_block(&mut self, position: usize, code: &str, language: &str) -> &mut Self{ ... }
    /// Delete a node.
    pub fn delete_node(&mut self, node_index: usize) -> &mut Self{ ... }
    /// Move a node from one position to another.
    pub fn move_node(&mut self, from_index: usize, to_index: usize) -> &mut Self{ ... }
    /// Convert a node from one type to another.
    pub fn convert_node_type(
            &mut self,
            node_index: usize,
            target_type: NodeConversionType,
        ) -> &mut Self{ ... }
    /// Merge two adjacent nodes.
    pub fn merge_nodes(&mut self, first_index: usize, second_index: usize) -> &mut Self{ ... }
    /// Duplicate a node.
    pub fn duplicate_node(&mut self, node_index: usize) -> &mut Self{ ... }
    /// Apply formatting to selection.
    pub fn format_selection(&mut self, formatting: TextFormatting) -> &mut Self{ ... }
    /// Indent selection.
    pub fn indent_selection(&mut self) -> &mut Self{ ... }
    /// Unindent selection.
    pub fn unindent_selection(&mut self) -> &mut Self{ ... }
    /// Create a table.
    pub fn create_table(
            &mut self,
            position: usize,
            columns: usize,
            rows: usize,
            with_header: bool,
        ) -> &mut Self{ ... }
    /// Add a table operation (add/remove row/column, set cell, etc.).
    pub fn table_operation(&mut self, node_index: usize, operation: TableOperation) -> &mut Self{ ... }
    /// Selects an entire node by index.
    pub fn select_node(&mut self, node_index: usize) -> &mut Self{ ... }
    /// Selects a range of text in a node.
    pub fn select_text_range(&mut self, node_index: usize, start: usize, end: usize) -> &mut Self{ ... }
    /// Clears the selection.
    pub fn clear_selection(&mut self) -> &mut Self{ ... }
    /// Commits all changes to the document as a single operation.
    ///
    /// Returns a Vec of all the commands for adding to the editor's undo stack.
    /// Note that this consumes the transaction, so it can only be committed once.
    pub fn commit(mut self) -> Result<Vec<Box<dyn EditorCommand>>, EditError>{ ... }
    /// Rolls back any executed commands in reverse order.
    fn rollback(&mut self) { ... }
    /// Discards the transaction without applying any changes.
    pub fn discard(mut self){ ... }
}

/// Implementing Drop to automatically discard uncommitted transactions.
impl Drop for Transaction {
    fn drop(&mut self) { ... }
}

```

## error.rs
```rust
/// Represents errors that can occur during parsing or serialization
#[derive(Debug, Error)]
pub enum ParseError {
    /// Error parsing markdown content
    Markdown(String),
    /// Error parsing HTML content
    Html(String),
    /// Error parsing JSON content
    Json(String),
    /// Generic parsing error
    Generic(String),
}

/// Represents errors that can occur during document editing operations
#[derive(Debug, Error)]
pub enum EditError {
    /// The index is out of bounds
    IndexOutOfBounds,
    /// The operation is not supported for the given node type
    UnsupportedOperation,
    /// The range is invalid (e.g., end before start)
    InvalidRange,
    /// The operation was attempted on an invalid node
    InvalidNode,
    /// The operation could not be completed successfully
    OperationFailed,
    /// Other error with a message
    Other(String),
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { ... }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { ... }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self { ... }
}

```

## lib.rs
```rust
```

## models/builder/basic.rs
```rust
impl DocumentBuilder {
    /// Creates a new document builder
    pub fn new() -> Self{ ... }
    /// Creates a new document builder from a markdown string
    pub fn from_markdown(markdown: impl Into<String>) -> Result<Self, ParseError>{ ... }
    /// Sets a title for the document, creating a title heading
    pub fn title(mut self, title: impl Into<String>) -> Self{ ... }
    /// Sets the author metadata for the document
    pub fn author(mut self, author: impl Into<String>) -> Self{ ... }
    /// Sets the date metadata for the document
    pub fn date(mut self, date: impl Into<String>) -> Self{ ... }
    /// Adds custom metadata to the document
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self{ ... }
    /// Adds a heading to the document
    pub fn heading(mut self, level: u8, text: impl Into<String>) -> Self{ ... }
    /// Adds an empty paragraph to the document
    pub fn empty_paragraph(mut self) -> Self{ ... }
    /// Adds a paragraph with text to the document
    pub fn paragraph(mut self, text: impl Into<String>) -> Self{ ... }
    /// Adds a code block to the document
    pub fn code_block(mut self, code: impl Into<String>, language: impl Into<String>) -> Self{ ... }
    /// Adds a blockquote to the document
    pub fn blockquote(mut self, text: impl Into<String>) -> Self{ ... }
    /// Adds an unordered list to the document
    pub fn unordered_list(mut self, items: Vec<impl Into<String>>) -> Self{ ... }
    /// Adds an ordered list to the document
    pub fn ordered_list(mut self, items: Vec<impl Into<String>>) -> Self{ ... }
    /// Adds a task list to the document
    pub fn task_list(mut self, items: Vec<(impl Into<String>, bool)>) -> Self{ ... }
    /// Adds a horizontal rule to the document
    pub fn horizontal_rule(mut self) -> Self{ ... }
    /// Adds a simple table to the document
    pub fn table(
            mut self,
            headers: Vec<impl Into<String>>,
            rows: Vec<Vec<impl Into<String>>>,
        ) -> Self{ ... }
    /// Adds a table with column alignments to the document
    pub fn table_with_alignments(
            mut self,
            headers: Vec<impl Into<String>>,
            rows: Vec<Vec<impl Into<String>>>,
            alignments: Vec<TableAlignment>,
        ) -> Self{ ... }
    /// Adds a math block to the document
    pub fn math_block(mut self, math: impl Into<String>) -> Self{ ... }
    /// Adds a footnote reference to the document
    pub fn footnote_reference(mut self, label: impl Into<String>) -> Self{ ... }
    /// Adds a footnote definition to the document
    pub fn footnote_definition(
            mut self,
            label: impl Into<String>,
            text: impl Into<String>,
        ) -> Self{ ... }
    /// Adds a group of nodes with a name to the document
    pub fn group(
            mut self,
            name: impl Into<String>,
            builder: impl FnOnce(DocumentBuilder) -> DocumentBuilder,
        ) -> Self{ ... }
    /// Builds the document
    pub fn build(self) -> Document{ ... }
}

impl Default for DocumentBuilder {
    fn default() -> Self { ... }
}

```

## models/builder/mod.rs
```rust
/// A builder for creating documents with a fluent API
pub struct DocumentBuilder {
    /// The document being built
    document: Document,
}

```

## models/builder/selection.rs
```rust
/// Extension methods for Document to help with selections
impl Document {
    /// Sets the selection to select all content in the document
    pub fn select_all(&mut self) -> bool{ ... }
    /// Selects a specific node by index
    pub fn select_node(&mut self, node_index: usize) -> bool{ ... }
    /// Selects a range between two nodes (inclusive)
    pub fn select_node_range(&mut self, start_index: usize, end_index: usize) -> bool{ ... }
    /// Selects a specific range within a single node
    pub fn select_text_range(
            &mut self,
            node_index: usize,
            start_offset: usize,
            end_offset: usize,
        ) -> bool{ ... }
    /// Select from one position to another across any nodes
    pub fn select_range(
            &mut self,
            start_node: usize,
            start_offset: usize,
            end_node: usize,
            end_offset: usize,
        ) -> bool{ ... }
    /// Collapse selection to its start
    pub fn collapse_selection_to_start(&mut self) -> bool{ ... }
    /// Collapse selection to its end
    pub fn collapse_selection_to_end(&mut self) -> bool{ ... }
    /// Clear the current selection
    pub fn clear_selection(&mut self){ ... }
    /// Returns true if there is an active selection
    pub fn has_selection(&self) -> bool{ ... }
    /// Returns true if there is an active selection that spans multiple nodes
    pub fn has_multi_node_selection(&self) -> bool{ ... }
    /// Returns the selected text as a string, if possible
    pub fn get_selected_text(&self) -> Option<String>{ ... }
}

```

## models/document.rs
```rust
/// The main document structure, containing a list of block nodes
/// and selection state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Document {
    /// The document's block nodes
    pub nodes: Vec<Node>,
    /// Optional selection state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<Selection>,
    /// Document metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DocumentMetadata>,
}

/// Contains metadata about the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    /// Document title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Document author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Document date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Other metadata as key-value pairs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub custom: Vec<(String, String)>,
}

impl Document {
    /// Creates a new empty document
    pub fn new() -> Self{ ... }
    /// Creates a new document with a title heading
    pub fn with_title(title: impl Into<String>) -> Self{ ... }
    /// Adds a heading to the document
    pub fn add_heading(&mut self, level: u8, text: impl Into<String>) -> usize{ ... }
    /// Adds a paragraph to the document
    pub fn add_paragraph(&mut self) -> usize{ ... }
    /// Adds a paragraph with text to the document
    pub fn add_paragraph_with_text(&mut self, text: impl Into<String>) -> usize{ ... }
    /// Adds a paragraph with custom inline nodes to the document
    pub fn add_paragraph_with_inlines(&mut self, inlines: Vec<InlineNode>) -> usize{ ... }
    /// Adds a code block to the document
    pub fn add_code_block(
            &mut self,
            code: impl Into<String>,
            language: impl Into<String>,
        ) -> usize{ ... }
    /// Adds an unordered list to the document
    pub fn add_unordered_list(&mut self, items: Vec<impl Into<String>>) -> usize{ ... }
    /// Adds an ordered list to the document
    pub fn add_ordered_list(&mut self, items: Vec<impl Into<String>>) -> usize{ ... }
    /// Adds a task list to the document
    pub fn add_task_list(&mut self, items: Vec<(impl Into<String>, bool)>) -> usize{ ... }
    /// Adds a footnote reference to the document
    pub fn add_footnote_reference(&mut self, label: impl Into<String>) -> usize{ ... }
    /// Adds a footnote definition to the document
    pub fn add_footnote_definition(
            &mut self,
            label: impl Into<String>,
            content: impl Into<String>,
        ) -> usize{ ... }
    /// Adds a complex footnote definition with multiple nodes as content
    pub fn add_complex_footnote_definition(
            &mut self,
            label: impl Into<String>,
            content: Vec<Node>,
        ) -> usize{ ... }
    /// Adds a math block to the document
    pub fn add_math_block(&mut self, math: impl Into<String>) -> usize{ ... }
    /// Adds a definition list to the document
    pub fn add_definition_list(&mut self, items: Vec<(String, Vec<String>)>) -> usize{ ... }
    /// Adds a paragraph with inline math expression
    pub fn add_paragraph_with_math(
            &mut self,
            text: impl Into<String>,
            math: impl Into<String>,
        ) -> usize{ ... }
    /// Adds a paragraph with emojis
    pub fn add_paragraph_with_emoji(
            &mut self,
            text: impl Into<String>,
            emoji_shortcode: impl Into<String>,
        ) -> usize{ ... }
    /// Adds a paragraph with user mention
    pub fn add_paragraph_with_mention(
            &mut self,
            text: impl Into<String>,
            username: impl Into<String>,
        ) -> usize{ ... }
    /// Adds a paragraph with issue reference
    pub fn add_paragraph_with_issue(
            &mut self,
            text: impl Into<String>,
            issue: impl Into<String>,
        ) -> usize{ ... }
    /// Inserts text at the specified location
    pub fn insert_text(
            &mut self,
            node_index: usize,
            offset: usize,
            text: impl Into<String>,
        ) -> bool{ ... }
    /// Splits a node at the specified location
    pub fn split_node(&mut self, node_index: usize, offset: usize) -> bool{ ... }
    /// Returns a string representation of the document structure
    pub fn debug_structure(&self) -> String{ ... }
}

impl Deref for Document {
    fn deref(&self) -> &Self::Target { ... }
}

impl DerefMut for Document {
    fn deref_mut(&mut self) -> &mut Self::Target { ... }
}

impl AsRef<Document> for Document {
    fn as_ref(&self) -> &Document { ... }
}

```

## models/formatting.rs
```rust
/// Represents text formatting options for text nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TextFormatting {
    /// Whether the text is bold
    pub bold: bool,
    /// Whether the text is italic
    pub italic: bool,
    /// Whether the text has strikethrough
    pub strikethrough: bool,
    /// Whether the text is code (monospace)
    pub code: bool,
}

impl TextFormatting {
    /// Creates a new default formatting (no styling)
    pub fn new() -> Self{ ... }
    /// Creates bold formatting
    pub fn bold() -> Self{ ... }
    /// Creates italic formatting
    pub fn italic() -> Self{ ... }
    /// Creates code formatting
    pub fn code() -> Self{ ... }
    /// Creates strikethrough formatting
    pub fn strikethrough() -> Self{ ... }
    /// Adds bold to existing formatting
    pub fn with_bold(mut self) -> Self{ ... }
    /// Adds italic to existing formatting
    pub fn with_italic(mut self) -> Self{ ... }
    /// Adds code to existing formatting
    pub fn with_code(mut self) -> Self{ ... }
    /// Adds strikethrough to existing formatting
    pub fn with_strikethrough(mut self) -> Self{ ... }
}

```

## models/inline.rs
```rust
/// Represents a text node with formatting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextNode {
    /// The text content
    pub text: String,
    /// Formatting applied to the text
    pub formatting: TextFormatting,
}

/// Represents inline content within a block node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InlineNode {
    /// Regular text with optional formatting
    #[serde(rename = "text")]
    Text(TextNode),

    /// A hyperlink
    #[serde(rename = "link")]
    Link {
        /// URL the link points to
        url: String,
        /// Optional title for the link
        title: Option<String>,
        /// The link's content/children
        children: Vec<InlineNode>,
    },

    /// An inline image
    #[serde(rename = "image")]
    Image {
        /// URL of the image
        url: String,
        /// Alt text for the image
        alt: String,
        /// Optional title for the image
        title: Option<String>,
    },

    /// Inline code span
    #[serde(rename = "code_span")]
    CodeSpan {
        /// The code content
        code: String,
    },

    /// Autolink (URL or email that's automatically linked)
    #[serde(rename = "autolink")]
    AutoLink {
        /// The URL or email address
        url: String,
        /// Whether this is an email address
        is_email: bool,
    },

    /// An inline footnote reference
    #[serde(rename = "footnote_ref")]
    FootnoteRef {
        /// The footnote reference label
        label: String,
    },

    /// An inline footnote (content directly in the reference)
    #[serde(rename = "inline_footnote")]
    InlineFootnote {
        /// The footnote content
        children: Vec<InlineNode>,
    },

    /// A mention (like @username in GitHub)
    #[serde(rename = "mention")]
    Mention {
        /// The username or reference being mentioned
        name: String,
        /// The type of mention (user, issue, etc.)
        mention_type: String,
    },

    /// Inline math expression
    #[serde(rename = "math")]
    Math {
        /// The math content in TeX notation
        math: String,
    },

    /// Emoji shortcode
    #[serde(rename = "emoji")]
    Emoji {
        /// The emoji shortcode (e.g., "smile")
        shortcode: String,
    },

    /// Hard break
    HardBreak,

    /// Soft break
    SoftBreak,
}

impl TextNode {
    /// Creates a new text node with the given text and default formatting
    pub fn new(text: impl Into<String>) -> Self{ ... }
    /// Creates a new text node with the given text and formatting
    pub fn with_formatting(text: impl Into<String>, formatting: TextFormatting) -> Self{ ... }
    /// Creates a bold text node
    pub fn bold(text: impl Into<String>) -> Self{ ... }
    /// Creates an italic text node
    pub fn italic(text: impl Into<String>) -> Self{ ... }
    /// Creates a code text node
    pub fn code(text: impl Into<String>) -> Self{ ... }
    /// Creates a strikethrough text node
    pub fn strikethrough(text: impl Into<String>) -> Self{ ... }
}

impl InlineNode {
    /// Creates a plain text node
    pub fn text(text: impl Into<String>) -> Self{ ... }
    /// Creates a bold text node
    pub fn bold_text(text: impl Into<String>) -> Self{ ... }
    /// Creates an italic text node
    pub fn italic_text(text: impl Into<String>) -> Self{ ... }
    /// Creates a strikethrough text node
    pub fn strikethrough_text(text: impl Into<String>) -> Self{ ... }
    /// Creates a code span
    pub fn code_span(code: impl Into<String>) -> Self{ ... }
    /// Creates a link with the given URL and text
    pub fn link(url: impl Into<String>, text: impl Into<String>) -> Self{ ... }
    /// Creates a link with the given URL, title, and text
    pub fn link_with_title(
            url: impl Into<String>,
            title: impl Into<String>,
            text: impl Into<String>,
        ) -> Self{ ... }
    /// Creates an image with the given URL and alt text
    pub fn image(url: impl Into<String>, alt: impl Into<String>) -> Self{ ... }
    /// Creates an image with the given URL, alt text, and title
    pub fn image_with_title(
            url: impl Into<String>,
            alt: impl Into<String>,
            title: impl Into<String>,
        ) -> Self{ ... }
    /// Creates an autolink for a URL
    pub fn autolink_url(url: impl Into<String>) -> Self{ ... }
    /// Creates an autolink for an email address
    pub fn autolink_email(email: impl Into<String>) -> Self{ ... }
    /// Creates a footnote reference
    pub fn footnote_ref(label: impl Into<String>) -> Self{ ... }
    /// Creates an inline footnote
    pub fn inline_footnote(content: impl Into<String>) -> Self{ ... }
    /// Creates a user mention (like @username)
    pub fn user_mention(username: impl Into<String>) -> Self{ ... }
    /// Creates an issue mention (like #123)
    pub fn issue_mention(issue: impl Into<String>) -> Self{ ... }
    /// Creates an inline math expression
    pub fn math(math: impl Into<String>) -> Self{ ... }
    /// Creates an emoji from a shortcode
    pub fn emoji(shortcode: impl Into<String>) -> Self{ ... }
    /// Creates a hard break
    pub fn hard_break() -> Self{ ... }
    pub fn as_text(&self) -> Option<&str>{ ... }
}

```

## models/mod.rs
```rust
```

## models/node.rs
```rust
/// Alignment options for table columns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TableAlignment {
    /// Left-aligned column
    Left,
    /// Center-aligned column
    Center,
    /// Right-aligned column
    Right,
    /// Default alignment
    None,
    /// Text justify alignment
    Justify,
    /// Top vertical alignment
    Top,
    /// Middle vertical alignment
    Middle,
    /// Bottom vertical alignment
    Bottom,
}

/// Properties for table styling and behavior
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableProperties {
    /// Whether the table has a header
    #[serde(default = "default_true")]
    pub has_header: bool,
    /// Whether to render the table with borders
    #[serde(default = "default_true")]
    pub has_borders: bool,
    /// Whether rows should have alternating background colors
    #[serde(default = "default_false")]
    pub striped_rows: bool,
    /// Whether table cells should be highlighted on hover
    #[serde(default = "default_false")]
    pub hoverable: bool,
    /// Custom CSS class for the table
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    /// Custom CSS style for the table
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Caption for the table (shown at the top or bottom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// Whether the caption should be displayed at the bottom
    #[serde(default = "default_false")]
    pub caption_at_bottom: bool,
}

/// Type of list: ordered, unordered, or task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListType {
    /// Ordered list (1. 2. 3. etc.)
    Ordered,
    /// Unordered list (*, -, + etc.)
    Unordered,
    /// Task list (- [ ], - [x] etc.)
    Task,
}

/// Represents a list item with children nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListItem {
    /// Child nodes of this list item
    pub children: Vec<Node>,
    /// Whether this item is checked (for task lists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
}

/// Represents a table cell
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCell {
    /// Content of the cell
    pub content: Vec<InlineNode>,
    /// Number of columns this cell spans
    #[serde(default = "default_span", skip_serializing_if = "is_default_span")]
    pub colspan: u32,
    /// Number of rows this cell spans
    #[serde(default = "default_span", skip_serializing_if = "is_default_span")]
    pub rowspan: u32,
    /// Background color in hex format (e.g., "#f5f5f5")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    /// Custom CSS class names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    /// Custom styles as CSS properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Whether this is a header cell (th vs td)
    #[serde(
        default = "default_is_header",
        skip_serializing_if = "is_default_is_header"
    )]
    pub is_header: bool,
}

/// A footnote reference in the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FootnoteReference {
    /// The label of the footnote
    pub label: String,
    /// Optional identifier for the footnote
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
}

/// A footnote definition in the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FootnoteDefinition {
    /// The label of the footnote
    pub label: String,
    /// The content of the footnote
    pub content: Vec<Node>,
}

/// Definition term and descriptions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionItem {
    /// The term being defined
    pub term: Vec<InlineNode>,
    /// The descriptions/definitions of the term
    pub descriptions: Vec<Vec<Node>>,
}

/// Properties for advanced code block rendering
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CodeBlockProperties {
    /// Whether to show line numbers
    #[serde(default = "default_false")]
    pub show_line_numbers: bool,

    /// The theme to use for syntax highlighting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,

    /// The starting line number (when line numbers are shown)
    #[serde(default = "default_line_number")]
    pub start_line: u32,

    /// Whether to highlight specific lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_lines: Option<Vec<u32>>,

    /// Whether to enable the copy button
    #[serde(default = "default_true")]
    pub show_copy_button: bool,

    /// Custom CSS class for the code block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,

    /// Custom CSS style for the code block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Maximum height before scrolling (e.g., "500px")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<String>,
}

/// Represents a block-level node in the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    /// A heading (h1-h6)
    #[serde(rename = "heading")]
    Heading {
        /// Heading level (1-6)
        level: u8,
        /// Inline content of the heading
        children: Vec<InlineNode>,
    },

    /// A paragraph
    #[serde(rename = "paragraph")]
    Paragraph {
        /// Inline content of the paragraph
        children: Vec<InlineNode>,
    },

    /// A list (ordered, unordered, or tasks)
    #[serde(rename = "list")]
    List {
        /// Type of the list
        list_type: ListType,
        /// List items
        items: Vec<ListItem>,
    },

    /// A code block
    #[serde(rename = "code_block")]
    CodeBlock {
        /// Programming language of the code
        language: String,
        /// The code content
        code: String,
        /// Advanced code block properties for rendering
        #[serde(default)]
        properties: CodeBlockProperties,
    },

    /// A block quote
    #[serde(rename = "blockquote")]
    BlockQuote {
        /// Child nodes of the blockquote
        children: Vec<Node>,
    },

    /// A horizontal rule (thematic break)
    #[serde(rename = "thematic_break")]
    ThematicBreak,

    /// A table with enhanced features
    #[serde(rename = "table")]
    Table {
        /// Table header row (vector of cells)
        header: Vec<TableCell>,
        /// Table body rows (vector of rows, each row is a vector of cells)
        rows: Vec<Vec<TableCell>>,
        /// Column alignments
        alignments: Vec<TableAlignment>,
        /// Table styling and behavior properties
        #[serde(default)]
        properties: TableProperties,
    },

    /// A group of nodes treated as a single unit
    #[serde(rename = "group")]
    Group {
        /// Name or type of the group
        name: String,
        /// Child nodes in the group
        children: Vec<Node>,
    },

    /// A footnote reference
    #[serde(rename = "footnote_reference")]
    FootnoteReference(FootnoteReference),

    /// A footnote definition
    #[serde(rename = "footnote_definition")]
    FootnoteDefinition(FootnoteDefinition),

    /// A definition list
    #[serde(rename = "definition_list")]
    DefinitionList {
        /// List of definition items
        items: Vec<DefinitionItem>,
    },

    /// A mathematical expression
    #[serde(rename = "math_block")]
    MathBlock {
        /// The math content in TeX notation
        math: String,
    },

    /// Temporary variants for parsing stack
    #[doc(hidden)]
    TempListItem(ListItem),
    #[doc(hidden)]
    TempTableCell(TableCell),
}

impl Default for TableAlignment {
    fn default() -> Self { ... }
}

impl Default for TableProperties {
    fn default() -> Self { ... }
}

impl TableProperties {
    /// Create a new TableProperties with default values
    pub fn new() -> Self{ ... }
    /// Set whether the table has a header row
    pub fn with_header(mut self, has_header: bool) -> Self{ ... }
    /// Set whether the table has visible borders
    pub fn with_borders(mut self, has_borders: bool) -> Self{ ... }
    /// Set whether the table has striped rows
    pub fn with_striped_rows(mut self, striped_rows: bool) -> Self{ ... }
    /// Set whether the table has hover effects
    pub fn with_hover(mut self, hoverable: bool) -> Self{ ... }
    /// Set the CSS class for the table
    pub fn with_css_class(mut self, css_class: impl Into<String>) -> Self{ ... }
    /// Set custom CSS style for the table
    pub fn with_style(mut self, style: impl Into<String>) -> Self{ ... }
    /// Set the caption for the table
    pub fn with_caption(mut self, caption: impl Into<String>, at_bottom: bool) -> Self{ ... }
}

impl ListItem {
    /// Creates a new list item with the given children
    pub fn new(children: Vec<Node>) -> Self{ ... }
    /// Creates a new list item with a single paragraph
    pub fn paragraph(text: impl Into<String>) -> Self{ ... }
    /// Creates a new task list item (with checkbox)
    pub fn task(text: impl Into<String>, checked: bool) -> Self{ ... }
    pub fn as_text(&self) -> Option<&str>{ ... }
}

impl TableCell {
    /// Creates a new table cell with the given content
    pub fn new(content: Vec<InlineNode>) -> Self{ ... }
    /// Creates a new table cell with text content
    pub fn text(text: impl Into<String>) -> Self{ ... }
    /// Creates a new table cell with the given content and column span
    pub fn with_colspan(content: Vec<InlineNode>, colspan: u32) -> Self{ ... }
    /// Creates a new table cell with the given content and row span
    pub fn with_rowspan(content: Vec<InlineNode>, rowspan: u32) -> Self{ ... }
    /// Creates a new table cell with the given content and spans
    pub fn with_spans(content: Vec<InlineNode>, colspan: u32, rowspan: u32) -> Self{ ... }
    /// Creates a header cell with text content
    pub fn header(text: impl Into<String>) -> Self{ ... }
    /// Set background color for the cell
    pub fn with_background_color(mut self, color: impl Into<String>) -> Self{ ... }
    /// Set CSS class for the cell
    pub fn with_css_class(mut self, class: impl Into<String>) -> Self{ ... }
    /// Set custom CSS style for the cell
    pub fn with_style(mut self, style: impl Into<String>) -> Self{ ... }
    /// Set whether this is a header cell
    pub fn with_header(mut self, is_header: bool) -> Self{ ... }
}

impl Default for TableCell {
    fn default() -> Self { ... }
}

impl FootnoteReference {
    /// Creates a new footnote reference
    pub fn new(label: impl Into<String>) -> Self{ ... }
    /// Creates a new footnote reference with an identifier
    pub fn with_identifier(label: impl Into<String>, identifier: impl Into<String>) -> Self{ ... }
}

impl FootnoteDefinition {
    /// Creates a new footnote definition
    pub fn new(label: impl Into<String>, content: Vec<Node>) -> Self{ ... }
    /// Creates a new footnote definition with a single paragraph
    pub fn paragraph(label: impl Into<String>, text: impl Into<String>) -> Self{ ... }
}

impl DefinitionItem {
    /// Creates a new definition item
    pub fn new(term: Vec<InlineNode>, descriptions: Vec<Vec<Node>>) -> Self{ ... }
    /// Creates a new definition item with a single description
    pub fn single(term: impl Into<String>, description: impl Into<String>) -> Self{ ... }
}

impl Default for CodeBlockProperties {
    fn default() -> Self { ... }
}

impl CodeBlockProperties {
    /// Create a new default CodeBlockProperties
    pub fn new() -> Self{ ... }
    /// Set whether to show line numbers
    pub fn with_line_numbers(mut self, show: bool) -> Self{ ... }
    /// Set the highlighting theme
    pub fn with_theme(mut self, theme: impl Into<String>) -> Self{ ... }
    /// Set the starting line number
    pub fn with_start_line(mut self, line: u32) -> Self{ ... }
    /// Set which lines to highlight
    pub fn with_highlight_lines(mut self, lines: Vec<u32>) -> Self{ ... }
    /// Set whether to show the copy button
    pub fn with_copy_button(mut self, show: bool) -> Self{ ... }
    /// Set a custom CSS class
    pub fn with_css_class(mut self, class: impl Into<String>) -> Self{ ... }
    /// Set custom CSS styles
    pub fn with_style(mut self, style: impl Into<String>) -> Self{ ... }
    /// Set maximum height before scrolling
    pub fn with_max_height(mut self, height: impl Into<String>) -> Self{ ... }
}

impl Node {
    /// Creates a new heading node
    pub fn heading(level: u8, text: impl Into<String>) -> Self{ ... }
    /// Creates a new paragraph node
    pub fn paragraph(text: impl Into<String>) -> Self{ ... }
    /// Creates a new paragraph with the given inline nodes
    pub fn paragraph_with_inlines(children: Vec<InlineNode>) -> Self{ ... }
    /// Creates a new code block
    pub fn code_block(code: impl Into<String>, language: impl Into<String>) -> Self{ ... }
    /// Creates a new code block with custom properties
    pub fn code_block_with_properties(
            code: impl Into<String>,
            language: impl Into<String>,
            properties: CodeBlockProperties,
        ) -> Self{ ... }
    /// Creates a new blockquote with a paragraph
    pub fn blockquote(text: impl Into<String>) -> Self{ ... }
    /// Creates a new unordered list
    pub fn unordered_list(items: Vec<impl Into<String>>) -> Self{ ... }
    /// Creates a new ordered list
    pub fn ordered_list(items: Vec<impl Into<String>>) -> Self{ ... }
    /// Creates a new task list
    pub fn task_list(items: Vec<(impl Into<String>, bool)>) -> Self{ ... }
    /// Creates a horizontal rule
    pub fn horizontal_rule() -> Self{ ... }
    /// Creates a simple table with headers and rows
    pub fn simple_table(
            headers: Vec<impl Into<String>>,
            rows: Vec<Vec<impl Into<String>>>,
        ) -> Self{ ... }
    /// Creates a table with specific alignments
    pub fn table_with_alignments(
            headers: Vec<impl Into<String>>,
            rows: Vec<Vec<impl Into<String>>>,
            alignments: Vec<TableAlignment>,
        ) -> Self{ ... }
    /// Creates a table with properties
    pub fn table_with_properties(
            headers: Vec<impl Into<String>>,
            rows: Vec<Vec<impl Into<String>>>,
            properties: TableProperties,
        ) -> Self{ ... }
    /// Creates a full-featured table with alignments and properties
    pub fn create_enhanced_table(
            headers: Vec<impl Into<String>>,
            rows: Vec<Vec<impl Into<String>>>,
            alignments: Vec<TableAlignment>,
            properties: TableProperties,
        ) -> Self{ ... }
    /// Creates a footnote reference
    pub fn footnote_reference(label: impl Into<String>) -> Self{ ... }
    /// Creates a footnote definition
    pub fn footnote_definition(label: impl Into<String>, text: impl Into<String>) -> Self{ ... }
    /// Creates a definition list
    pub fn definition_list(items: Vec<(String, Vec<String>)>) -> Self{ ... }
    /// Creates a math block
    pub fn math_block(math: impl Into<String>) -> Self{ ... }
    /// Creates a new group node
    pub fn group(name: impl Into<String>, children: Vec<Node>) -> Self{ ... }
    /// Returns this node as a heading if it is one
    pub fn as_heading(&self) -> Option<(u8, &Vec<InlineNode>)>{ ... }
    /// Returns this node as a paragraph if it is one
    pub fn as_paragraph(&self) -> Option<&Vec<InlineNode>>{ ... }
    /// Returns this node as a list if it is one
    pub fn as_list(&self) -> Option<(&ListType, &Vec<ListItem>)>{ ... }
    /// Returns this node as a code block if it is one
    pub fn as_code_block(&self) -> Option<(&str, &str)>{ ... }
    /// Get a reference to a code block's contents and properties
    pub fn as_code_block_with_properties(&self) -> Option<(&str, &str, &CodeBlockProperties)>{ ... }
    /// Returns this node as a blockquote if it is one
    pub fn as_blockquote(&self) -> Option<&Vec<Node>>{ ... }
    /// Returns this node as a table if it is one
    pub fn as_table(&self) -> Option<TableComponents>{ ... }
    /// Returns this node as a footnote reference if it is one
    pub fn as_footnote_reference(&self) -> Option<&FootnoteReference>{ ... }
    /// Returns this node as a footnote definition if it is one
    pub fn as_footnote_definition(&self) -> Option<&FootnoteDefinition>{ ... }
    /// Returns this node as a definition list if it is one
    pub fn as_definition_list(&self) -> Option<&Vec<DefinitionItem>>{ ... }
    /// Returns this node as a math block if it is one
    pub fn as_math_block(&self) -> Option<&str>{ ... }
    /// Returns whether this node is a thematic break
    pub fn is_thematic_break(&self) -> bool{ ... }
    /// Returns the group components if this node is a group
    pub fn as_group(&self) -> Option<(&str, &Vec<Node>)>{ ... }
}

```

## models/selection.rs
```rust
/// Represents a position within the document
/// Path is a series of indices to traverse the document tree
/// Offset is the character offset within the final node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Path to the node containing the position
    pub path: Vec<usize>,
    /// Character offset within the node
    pub offset: usize,
}

/// Represents a selection range within the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Start position of the selection
    pub start: Position,
    /// End position of the selection
    pub end: Position,
    /// Whether the selection is collapsed to a single point
    pub is_collapsed: bool,
}

impl Position {
    /// Creates a new position
    pub fn new(path: Vec<usize>, offset: usize) -> Self{ ... }
    /// Creates a position at the start of the document
    pub fn start() -> Self{ ... }
}

impl Selection {
    /// Creates a new selection
    pub fn new(start: Position, end: Position) -> Self{ ... }
    /// Creates a collapsed selection at the specified position
    pub fn collapsed(position: Position) -> Self{ ... }
    /// Creates a selection at the start of the document
    pub fn at_start() -> Self{ ... }
}

```
