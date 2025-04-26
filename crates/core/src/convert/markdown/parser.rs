use crate::{
    Document, FootnoteDefinition, InlineNode, ListItem, ListType, Node, ParseError, TableAlignment,
    TableCell, TextFormatting, TextNode,
};
use pulldown_cmark::{
    Alignment, CodeBlockKind, Event, HeadingLevel, /* LinkType, */ Options, Parser, Tag,
    TagEnd,
};
use std::collections::HashMap;

/// Converts a pulldown-cmark Alignment to our TableAlignment
fn convert_alignment(alignment: Alignment) -> TableAlignment {
    match alignment {
        Alignment::Left => TableAlignment::Left,
        Alignment::Center => TableAlignment::Center,
        Alignment::Right => TableAlignment::Right,
        Alignment::None => TableAlignment::None,
    }
}

/// Convert a heading level to u8
fn level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Represents the current parsing context (e.g., inside a list, blockquote).
#[allow(dead_code)]
#[derive(Debug)]
enum Context {
    Document,
    Paragraph,
    Heading(u8),
    BlockQuote,
    List(ListType, Option<u64>), // Type and start number
    ListItem,
    Table(Vec<TableAlignment>),
    TableHead,
    TableRow,
    TableCell,
    FootnoteDefinition(String),
}

/// Helper struct to manage the parsing stack and accumulated nodes.
struct ParserStack {
    // Stack of contexts and the nodes accumulated within them.
    stack: Vec<(Context, Vec<Node>)>,
    // Accumulator for inline nodes within the current context.
    inline_accumulator: Vec<InlineNode>,
    // Current text formatting state.
    formatting: TextFormatting,
    // Footnote definitions found.
    footnotes: HashMap<String, FootnoteDefinition>,
    // Temporary storage for current code block language
    current_code_language: Option<String>,
    // Temporary storage for TaskListMarker status
    pending_task_status: Option<bool>,
    // Temporary storage for last link index
    last_link_index: Option<usize>,
    // Temporary storage for code block handling
    in_code_block: bool,
    // Temporary storage for code block text
    code_block_text: String,
}

impl ParserStack {
    fn new() -> Self {
        ParserStack {
            stack: vec![(Context::Document, Vec::new())], // Start with Document context
            inline_accumulator: Vec::new(),
            formatting: TextFormatting::default(),
            footnotes: HashMap::new(),
            current_code_language: None,
            pending_task_status: None, // Initialize
            last_link_index: None,
            in_code_block: false,
            code_block_text: String::new(),
        }
    }

    /// Get a mutable reference to the nodes of the current context.
    fn current_nodes(&mut self) -> &mut Vec<Node> {
        &mut self
            .stack
            .last_mut()
            .expect("Stack should never be empty")
            .1
    }

    /// Get the current context.
    fn current_context(&self) -> &Context {
        &self.stack.last().expect("Stack should never be empty").0
    }

    /// Start a new context (e.g., entering a list).
    fn push_context(&mut self, context: Context) {
        self.flush_inline_accumulator(); // Flush any pending inline nodes
        self.stack.push((context, Vec::new()));
    }

    /// End the current context, returning the constructed Node.
    fn pop_context(&mut self) -> Option<Node> {
        // Paragraphs and Headings need to consume the inline accumulator.
        // Other contexts might flush first if they contained loose inlines.
        let inlines_for_context = match self.current_context() {
            Context::Paragraph | Context::Heading(_) | Context::TableCell => {
                // Take the inlines directly, don't flush.
                Some(std::mem::take(&mut self.inline_accumulator))
            }
            _ => {
                // For other contexts, ensure any loose inlines are flushed first.
                self.flush_inline_accumulator();
                None // Indicate that inlines were not directly taken for this context type.
            }
        };

        // Reset formatting when a context expecting inlines is popped.
        if inlines_for_context.is_some() {
            self.formatting = TextFormatting::default();
        }

        if self.stack.len() <= 1 {
            return None; // Cannot pop the base Document context
        }

        let (context, children) = self.stack.pop().expect("Stack already checked > 1");

        match context {
            Context::Paragraph => Some(Node::Paragraph {
                children: inlines_for_context
                    .expect("Inlines should have been taken for Paragraph"),
            }),
            Context::Heading(level) => Some(Node::Heading {
                level,
                children: inlines_for_context.expect("Inlines should have been taken for Heading"),
            }),
            Context::BlockQuote => {
                // BlockQuote children are added via flush_inline_accumulator or popping child contexts.
                Some(Node::BlockQuote { children })
            }
            Context::List(list_type, _) => {
                // Extract ListItem structs from TempListItem nodes
                let items = children
                    .into_iter()
                    .filter_map(|node| {
                        if let Node::TempListItem(item) = node {
                            Some(item)
                        } else {
                            eprintln!(
                                "Warning: Non-TempListItem node found in List context: {:?}",
                                node
                            );
                            None
                        }
                    })
                    .collect();
                Some(Node::List { list_type, items })
            }
            Context::ListItem => {
                // ListItem children are added via flush_inline_accumulator or popping child contexts.
                let mut list_item = ListItem::new(children);
                if let Some(checked) = self.pending_task_status.take() {
                    list_item.checked = Some(checked);
                }
                Some(Node::TempListItem(list_item))
            }
            Context::FootnoteDefinition(label) => {
                self.footnotes
                    .insert(label.clone(), FootnoteDefinition::new(label, children));
                None
            }
            Context::Table(_) => {
                eprintln!("Warning: Popping Table context via pop_context");
                None
            }
            Context::TableHead => None,
            Context::TableRow => {
                eprintln!(
                    "Warning: pop_context called for TableRow - should be handled in TagEnd::TableRow"
                );
                None
            }
            Context::TableCell => {
                // TableCell children are formed from the inline accumulator.
                Some(Node::TempTableCell(TableCell::new(
                    inlines_for_context.expect("Inlines should have been taken for TableCell"),
                )))
            }
            Context::Document => None,
        }
    }

    /// Add an inline node to the current accumulator.
    fn push_inline(&mut self, inline: InlineNode) {
        match self.current_context() {
            // If current context expects block nodes, wrap inline in a paragraph
            Context::Document
            | Context::BlockQuote
            | Context::ListItem
            | Context::FootnoteDefinition(_) => {
                self.inline_accumulator.push(inline);
            }
            // If context expects inlines, add directly
            Context::Paragraph | Context::Heading(_) | Context::TableCell => {
                self.inline_accumulator.push(inline);
            }
            // Other contexts might not directly accept inlines
            _ => {
                eprintln!(
                    "Warning: Pushing inline {:?} into unexpected context {:?}",
                    inline,
                    self.current_context()
                );
                // Decide: Wrap in paragraph? Add to accumulator anyway?
                self.inline_accumulator.push(inline);
            }
        }
    }

    /// Flush accumulated inline nodes into a Paragraph node if applicable.
    fn flush_inline_accumulator(&mut self) {
        if !self.inline_accumulator.is_empty() {
            let inlines = std::mem::take(&mut self.inline_accumulator);
            let node = match self.current_context() {
                // Only create Paragraphs if the context expects block nodes.
                Context::Document
                | Context::BlockQuote
                | Context::ListItem
                | Context::FootnoteDefinition(_) => Some(Node::paragraph_with_inlines(inlines)),
                // Do nothing if context is already Paragraph/Heading/TableCell
                // as pop_context will handle the inlines.
                Context::Paragraph | Context::Heading(_) | Context::TableCell => None,
                _ => {
                    eprintln!(
                        "Warning: Flushing inlines in unexpected context {:?}",
                        self.current_context()
                    );
                    None
                }
            };
            if let Some(n) = node {
                self.current_nodes().push(n);
            }
        }
        // Do not reset formatting here
    }

    /// Handle Text events.
    fn handle_text(&mut self, text: String) {
        if text.trim().is_empty() {
            // Handle potentially significant whitespace depending on context?
            // For now, just push if not completely empty.
            if !text.is_empty() {
                self.push_inline(InlineNode::Text(TextNode {
                    text,
                    formatting: self.formatting.clone(),
                }));
            }
        } else {
            self.push_inline(InlineNode::Text(TextNode {
                text,
                formatting: self.formatting.clone(),
            }));
        }
    }
}

/// Helper to convert a Vec<Node> (assumed to contain Paragraphs with inlines)
/// back into a Vec<InlineNode> for contexts like Heading/Paragraph.
fn _convert_nodes_to_inlines(nodes: Vec<Node>) -> Option<Vec<InlineNode>> {
    if nodes.len() == 1 {
        if let Node::Paragraph { children } = nodes.into_iter().next().unwrap() {
            Some(children)
        } else {
            eprintln!(
                "Warning: Expected single Paragraph node for inline conversion, found different node type"
            );
            None
        }
    } else if nodes.is_empty() {
        Some(Vec::new())
    } else {
        eprintln!(
            "Warning: Expected single Paragraph node for inline conversion, found multiple nodes"
        );
        // Attempt to merge? For now, return None or first paragraph?
        None
    }
}

/// Parse Markdown text into a Document using a stack-based approach.
pub(crate) fn parse_markdown(markdown: &str) -> Result<Document, ParseError> {
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);
    let mut stack = ParserStack::new();
    let mut current_table_state: Option<TableState> = None;

    let mut events = parser.peekable();

    while let Some(event) = events.next() {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    // Flush before starting a new paragraph context *if needed* by the parent context.
                    match stack.current_context() {
                        Context::ListItem
                        | Context::BlockQuote
                        | Context::FootnoteDefinition(_) => stack.flush_inline_accumulator(),
                        _ => {}
                    }
                    stack.push_context(Context::Paragraph)
                }
                Tag::Heading { level, .. } => {
                    stack.push_context(Context::Heading(level_to_u8(level)))
                }
                Tag::BlockQuote(_) => {
                    stack.flush_inline_accumulator(); // Flush before block node
                    stack.push_context(Context::BlockQuote);
                }
                Tag::CodeBlock(kind) => {
                    stack.flush_inline_accumulator(); // Ensure pending text becomes a node
                    let language = match kind {
                        CodeBlockKind::Fenced(lang) => lang.into_string(),
                        CodeBlockKind::Indented => String::new(),
                    };
                    // Store language before pushing context for text
                    stack.current_code_language = Some(language);
                    // Use a specialized context for code blocks to handle them correctly
                    stack.in_code_block = true;
                    stack.code_block_text = String::new();
                }
                Tag::List(start) => {
                    let list_type = match start {
                        Some(_) => ListType::Ordered,
                        None => ListType::Unordered, // Initial assumption, may change to Task
                    };
                    stack.push_context(Context::List(list_type, start));
                }
                Tag::Item => stack.push_context(Context::ListItem),
                Tag::FootnoteDefinition(label) => {
                    stack.push_context(Context::FootnoteDefinition(label.into_string()))
                }
                Tag::Table(alignments) => {
                    stack.flush_inline_accumulator();
                    // Add type annotation
                    let alignments_converted: Vec<TableAlignment> =
                        alignments.into_iter().map(convert_alignment).collect();
                    stack.push_context(Context::Table(alignments_converted.clone()));
                    current_table_state = Some(TableState::new(alignments_converted));
                }
                Tag::TableHead => {
                    if let Some(ref mut table) = current_table_state {
                        table.in_header = true;
                        stack.push_context(Context::TableHead);
                    } else {
                        eprintln!("Warning: TableHead outside of Table context");
                    }
                }
                Tag::TableRow => {
                    if current_table_state.is_some() {
                        stack.push_context(Context::TableRow);
                    } else {
                        eprintln!("Warning: TableRow outside of Table context");
                    }
                }
                Tag::TableCell => {
                    if current_table_state.is_some() {
                        stack.push_context(Context::TableCell);
                    } else {
                        eprintln!("Warning: TableCell outside of Table context");
                    }
                }
                Tag::Emphasis => stack.formatting = stack.formatting.clone().with_italic(),
                Tag::Strong => stack.formatting = stack.formatting.clone().with_bold(),
                Tag::Strikethrough => {
                    stack.formatting = stack.formatting.clone().with_strikethrough()
                }
                // Ignore link_type and id for now
                Tag::Link {
                    dest_url, title, ..
                } => {
                    // Store link information for later, when we handle TagEnd::Link
                    let link_info = InlineNode::Link {
                        url: dest_url.into_string(),
                        title: if title.is_empty() {
                            None
                        } else {
                            Some(title.into_string())
                        },
                        children: vec![], // Will be populated later
                    };

                    // Store the link and its index in the accumulator for later
                    let link_index = stack.inline_accumulator.len();
                    stack.inline_accumulator.push(link_info);
                    stack.last_link_index = Some(link_index);
                }
                // Ignore link_type and id for now
                Tag::Image {
                    dest_url, title, ..
                } => {
                    stack.flush_inline_accumulator();
                    let mut alt_text = String::new();
                    // Peek ahead for the Text event containing alt text
                    if let Some(Event::Text(alt)) = events.peek() {
                        alt_text = alt.to_string();
                        events.next(); // Consume the peeked Text event
                    }
                    // Expect End(Image) next, consume it if present
                    if let Some(Event::End(TagEnd::Image)) = events.peek() {
                        events.next();
                    } else {
                        eprintln!("Warning: Expected End(Image) after Image start/alt text");
                    }

                    // Create the Image node directly
                    let image_node = InlineNode::Image {
                        url: dest_url.into_string(),
                        alt: alt_text,
                        title: if title.is_empty() {
                            None
                        } else {
                            Some(title.into_string())
                        },
                    };

                    // Add image. Should it be wrapped in paragraph? Depends on context.
                    // Add to accumulator for now, let flush handle paragraphing.
                    stack.push_inline(image_node);
                }
                // Add catch-all for other Tag types
                _ => { /* Optional: Log unhandled Start tags */ }
            },
            Event::End(tag_end) => {
                // When a block ends, pop its context and add the constructed node to the parent.
                match tag_end {
                    TagEnd::Paragraph => {
                        // Paragraph end always pops the Paragraph context.
                        if let Some(node) = stack.pop_context() {
                            stack.current_nodes().push(node);
                        }
                    }
                    TagEnd::Heading(_) => {
                        // Heading end pops the Heading context.
                        if let Some(node) = stack.pop_context() {
                            stack.current_nodes().push(node);
                        }
                    }
                    TagEnd::TableCell => {
                        // TableCell end pops the TableCell context.
                        if let Some(node) = stack.pop_context() {
                            // The popped node is Node::TempTableCell
                            stack.current_nodes().push(node);
                        } else { /* Warning */
                        }
                    }
                    TagEnd::BlockQuote(_) => {
                        // BlockQuote end pops the BlockQuote context.
                        if let Some(node) = stack.pop_context() {
                            stack.current_nodes().push(node);
                        }
                    }
                    TagEnd::List(_) => {
                        if let Some(node) = stack.pop_context() {
                            stack.current_nodes().push(node);
                        }
                    }
                    TagEnd::Item => {
                        if let Some(node) = stack.pop_context() {
                            stack.current_nodes().push(node);
                        }
                    }
                    TagEnd::FootnoteDefinition => {
                        let _ = stack.pop_context();
                    }
                    TagEnd::Table => {
                        stack.flush_inline_accumulator();
                        if let Some(table_state) = current_table_state.take() {
                            if let Context::Table(_) = *stack.current_context() {
                                let (_, _children) =
                                    stack.stack.pop().expect("Table context should be on stack");
                                let table_node = Node::Table {
                                    header: table_state.header,
                                    rows: table_state.rows,
                                    alignments: table_state.alignments,
                                };
                                stack.current_nodes().push(table_node);
                            } else {
                                eprintln!(
                                    "Warning: TableEnd encountered without Table context on stack"
                                );
                            }
                        } else {
                            eprintln!("Warning: TableEnd encountered without active table state");
                        }
                    }
                    TagEnd::TableHead => {
                        if let Context::TableHead = *stack.current_context() {
                            // Pop TableHead context - pop_context returns None
                            let _ = stack.pop_context();
                        } else {
                            eprintln!("Warning: TableHeadEnd without TableHead context");
                        }
                        if let Some(ref mut table) = current_table_state {
                            table.in_header = false;
                        }
                    }
                    TagEnd::TableRow => {
                        if let Context::TableRow = *stack.current_context() {
                            // Pop TableRow - pop_context now returns None
                            // Cell collection happens here instead.
                            let (_, children) =
                                stack.stack.pop().expect("TableRow should be on stack");
                            let table_cells: Vec<TableCell> = children.into_iter().filter_map(|node| {
                                if let Node::TempTableCell(cell) = node {
                                    Some(cell)
                                } else {
                                    eprintln!("Warning: Non-TempTableCell found when finalizing TableRow");
                                    None
                                }
                            }).collect();

                            if let Some(ref mut table) = current_table_state {
                                if table.in_header {
                                    table.header = table_cells;
                                } else {
                                    table.rows.push(table_cells);
                                }
                            } else {
                                eprintln!(
                                    "Warning: Finalizing TableRow without active table state"
                                );
                            }
                        } else {
                            eprintln!(
                                "Warning: TagEnd::TableRow without TableRow context on stack"
                            );
                        }
                    }
                    TagEnd::Emphasis => stack.formatting.italic = false,
                    TagEnd::Strong => stack.formatting.bold = false,
                    TagEnd::Strikethrough => stack.formatting.strikethrough = false,
                    TagEnd::Link => {
                        // If we have a last link index, we need to update the link's children
                        if let Some(link_index) = stack.last_link_index.take() {
                            if link_index < stack.inline_accumulator.len() {
                                // Get the accumulated text nodes that should become the link's children
                                let accumulated_since_link: Vec<InlineNode> =
                                    stack.inline_accumulator.drain(link_index + 1..).collect();

                                // Update the link's children
                                if let InlineNode::Link { children, .. } =
                                    &mut stack.inline_accumulator[link_index]
                                {
                                    *children = accumulated_since_link;
                                }
                            }
                        }
                    }
                    TagEnd::Image => { /* Handled at Start? */ }
                    TagEnd::CodeBlock => {
                        if stack.in_code_block {
                            let language = stack.current_code_language.take().unwrap_or_default();
                            // Trim trailing newline to match expected format
                            let mut code = std::mem::take(&mut stack.code_block_text);
                            if code.ends_with('\n') {
                                code.pop();
                            }

                            stack.in_code_block = false;

                            // Create the CodeBlock node
                            let code_block = Node::CodeBlock { language, code };
                            stack.current_nodes().push(code_block);
                        } else {
                            eprintln!("Warning: CodeBlock end without CodeBlock context");
                        }
                    }
                    // Add catch-all for other TagEnd types
                    _ => { /* Optional: Log unhandled End tags */ }
                }
            }
            Event::Text(text) => {
                if stack.in_code_block {
                    // If we're in a code block, append to the code block text
                    stack.code_block_text.push_str(&text);
                } else {
                    stack.handle_text(text.into_string())
                }
            }
            Event::Code(text) => stack.push_inline(InlineNode::code_span(text.into_string())),
            Event::Html(html) => {
                // Decide how to handle raw HTML. Convert to text? Special node?
                stack.handle_text(html.into_string()); // Treat as text for now
            }
            Event::FootnoteReference(label) => {
                stack.push_inline(InlineNode::footnote_ref(label.into_string()));
            }
            Event::SoftBreak => stack.handle_text(" ".to_string()), // Or handle based on context
            Event::HardBreak => stack.push_inline(InlineNode::hard_break()), // Use constructor
            Event::Rule => {
                stack.flush_inline_accumulator();
                stack.current_nodes().push(Node::horizontal_rule());
            }
            Event::TaskListMarker(checked) => {
                // Store the status temporarily. It will be applied when the ListItem context is popped.
                stack.pending_task_status = Some(checked);

                // The old logic tried to modify the stack directly, which was complex.
                // We still need to update the parent List type.
                let mut list_context_index = None;
                for (index, (context, _)) in stack.stack.iter().enumerate().rev() {
                    if let Context::List(_, _) = context {
                        list_context_index = Some(index);
                        break;
                    }
                }
                if let Some(idx) = list_context_index {
                    if let Some((Context::List(list_type, _), _)) = stack.stack.get_mut(idx) {
                        *list_type = ListType::Task;
                    }
                } else {
                    eprintln!(
                        "Warning: TaskListMarker found outside of a List context (in parent check)"
                    );
                }
            }
            // Add catch-all for other Event types
            _ => { /* Optional: Log unhandled Events */ }
        }
    }

    // Finalize the document
    if stack.stack.len() != 1 {
        eprintln!(
            "Warning: Stack not fully unwound. Remaining: {:?}",
            stack.stack
        );
        // Attempt to pop remaining contexts
        while stack.stack.len() > 1 {
            if let Some(node) = stack.pop_context() {
                stack.current_nodes().push(node);
            }
        }
    }
    stack.flush_inline_accumulator(); // Flush any remaining inlines at the end

    let (_doc_context, nodes) = stack.stack.pop().expect("Stack should have Document root");
    let mut document = Document::new();
    document.nodes = nodes;
    // Add footnotes? The original code didn't add them to the Document struct.
    // document.footnotes = stack.footnotes;
    Ok(document)
}

// Temporary struct for new table state mgmt (integrate with ParserStack)
#[derive(Debug)]
struct TableState {
    alignments: Vec<TableAlignment>,
    header: Vec<TableCell>,
    rows: Vec<Vec<TableCell>>,
    in_header: bool,
}

impl TableState {
    fn new(alignments: Vec<TableAlignment>) -> Self {
        TableState {
            alignments,
            header: Vec::new(),
            rows: Vec::new(),
            in_header: false,
        }
    }
}
