use crate::InlineNode;
use serde::{Deserialize, Serialize};

/// Table column alignment
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
}

impl Default for TableAlignment {
    fn default() -> Self {
        Self::None
    }
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

impl ListItem {
    /// Creates a new list item with the given children
    pub fn new(children: Vec<Node>) -> Self {
        Self {
            children,
            checked: None,
        }
    }

    /// Creates a new list item with a single paragraph
    pub fn paragraph(text: impl Into<String>) -> Self {
        Self {
            children: vec![Node::paragraph(text)],
            checked: None,
        }
    }

    /// Creates a new task list item (with checkbox)
    pub fn task(text: impl Into<String>, checked: bool) -> Self {
        Self {
            children: vec![Node::paragraph(text)],
            checked: Some(checked),
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        self.children.first().and_then(|node| {
            node.as_paragraph()
                .and_then(|inlines| inlines.first().and_then(|inline| inline.as_text()))
        })
    }
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
}

fn default_span() -> u32 {
    1
}

fn is_default_span(span: &u32) -> bool {
    *span == 1
}

impl TableCell {
    /// Creates a new table cell with the given content
    pub fn new(content: Vec<InlineNode>) -> Self {
        Self {
            content,
            colspan: 1,
            rowspan: 1,
        }
    }

    /// Creates a new table cell with text content
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![InlineNode::text(text)],
            colspan: 1,
            rowspan: 1,
        }
    }

    /// Creates a new table cell with the given content and column span
    pub fn with_colspan(content: Vec<InlineNode>, colspan: u32) -> Self {
        Self {
            content,
            colspan,
            rowspan: 1,
        }
    }

    /// Creates a new table cell with the given content and row span
    pub fn with_rowspan(content: Vec<InlineNode>, rowspan: u32) -> Self {
        Self {
            content,
            colspan: 1,
            rowspan,
        }
    }

    /// Creates a new table cell with the given content and spans
    pub fn with_spans(content: Vec<InlineNode>, colspan: u32, rowspan: u32) -> Self {
        Self {
            content,
            colspan,
            rowspan,
        }
    }
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

impl FootnoteReference {
    /// Creates a new footnote reference
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            identifier: None,
        }
    }

    /// Creates a new footnote reference with an identifier
    pub fn with_identifier(label: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            identifier: Some(identifier.into()),
        }
    }
}

/// A footnote definition in the document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FootnoteDefinition {
    /// The label of the footnote
    pub label: String,
    /// The content of the footnote
    pub content: Vec<Node>,
}

impl FootnoteDefinition {
    /// Creates a new footnote definition
    pub fn new(label: impl Into<String>, content: Vec<Node>) -> Self {
        Self {
            label: label.into(),
            content,
        }
    }

    /// Creates a new footnote definition with a single paragraph
    pub fn paragraph(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            content: vec![Node::paragraph(text)],
        }
    }
}

/// Definition term and descriptions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionItem {
    /// The term being defined
    pub term: Vec<InlineNode>,
    /// The descriptions/definitions of the term
    pub descriptions: Vec<Vec<Node>>,
}

impl DefinitionItem {
    /// Creates a new definition item
    pub fn new(term: Vec<InlineNode>, descriptions: Vec<Vec<Node>>) -> Self {
        Self { term, descriptions }
    }

    /// Creates a new definition item with a single description
    pub fn single(term: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            term: vec![InlineNode::text(term)],
            descriptions: vec![vec![Node::paragraph(description)]],
        }
    }
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

/// Type alias for table components
pub type TableComponents<'a> = (
    &'a Vec<TableCell>,
    &'a Vec<Vec<TableCell>>,
    &'a Vec<TableAlignment>,
);

impl Node {
    /// Creates a new heading node
    pub fn heading(level: u8, text: impl Into<String>) -> Self {
        let level = level.clamp(1, 6);
        Self::Heading {
            level,
            children: vec![InlineNode::text(text)],
        }
    }

    /// Creates a new paragraph node
    pub fn paragraph(text: impl Into<String>) -> Self {
        Self::Paragraph {
            children: vec![InlineNode::text(text)],
        }
    }

    /// Creates a new paragraph with the given inline nodes
    pub fn paragraph_with_inlines(children: Vec<InlineNode>) -> Self {
        Self::Paragraph { children }
    }

    /// Creates a new code block
    pub fn code_block(code: impl Into<String>, language: impl Into<String>) -> Self {
        Self::CodeBlock {
            language: language.into(),
            code: code.into(),
        }
    }

    /// Creates a new blockquote with a paragraph
    pub fn blockquote(text: impl Into<String>) -> Self {
        Self::BlockQuote {
            children: vec![Self::paragraph(text)],
        }
    }

    /// Creates a new unordered list
    pub fn unordered_list(items: Vec<impl Into<String>>) -> Self {
        let list_items = items
            .into_iter()
            .map(|text| ListItem::paragraph(text))
            .collect();

        Self::List {
            list_type: ListType::Unordered,
            items: list_items,
        }
    }

    /// Creates a new ordered list
    pub fn ordered_list(items: Vec<impl Into<String>>) -> Self {
        let list_items = items
            .into_iter()
            .map(|text| ListItem::paragraph(text))
            .collect();

        Self::List {
            list_type: ListType::Ordered,
            items: list_items,
        }
    }

    /// Creates a new task list
    pub fn task_list(items: Vec<(impl Into<String>, bool)>) -> Self {
        let list_items = items
            .into_iter()
            .map(|(text, checked)| ListItem::task(text, checked))
            .collect();

        Self::List {
            list_type: ListType::Task,
            items: list_items,
        }
    }

    /// Creates a horizontal rule
    pub fn horizontal_rule() -> Self {
        Self::ThematicBreak
    }

    /// Creates a simple table with headers and rows
    pub fn simple_table(
        headers: Vec<impl Into<String>>,
        rows: Vec<Vec<impl Into<String>>>,
    ) -> Self {
        let header_cells: Vec<TableCell> = headers
            .into_iter()
            .map(|text| TableCell::text(text))
            .collect();

        let body_rows = rows
            .into_iter()
            .map(|row| row.into_iter().map(|text| TableCell::text(text)).collect())
            .collect();

        // Default alignments to None for all columns
        let alignments = vec![TableAlignment::None; header_cells.len()];

        Self::Table {
            header: header_cells,
            rows: body_rows,
            alignments,
        }
    }

    /// Creates a table with specific alignments
    pub fn table_with_alignments(
        headers: Vec<impl Into<String>>,
        rows: Vec<Vec<impl Into<String>>>,
        alignments: Vec<TableAlignment>,
    ) -> Self {
        let header_cells = headers
            .into_iter()
            .map(|text| TableCell::text(text))
            .collect();

        let body_rows = rows
            .into_iter()
            .map(|row| row.into_iter().map(|text| TableCell::text(text)).collect())
            .collect();

        Self::Table {
            header: header_cells,
            rows: body_rows,
            alignments,
        }
    }

    /// Creates a footnote reference
    pub fn footnote_reference(label: impl Into<String>) -> Self {
        Self::FootnoteReference(FootnoteReference::new(label))
    }

    /// Creates a footnote definition
    pub fn footnote_definition(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self::FootnoteDefinition(FootnoteDefinition::paragraph(label, text))
    }

    /// Creates a definition list
    pub fn definition_list(items: Vec<(String, Vec<String>)>) -> Self {
        let def_items = items
            .into_iter()
            .map(|(term, descriptions)| {
                let term_node = vec![InlineNode::text(term)];
                let desc_nodes = descriptions
                    .into_iter()
                    .map(|desc| vec![Node::paragraph(desc)])
                    .collect();

                DefinitionItem {
                    term: term_node,
                    descriptions: desc_nodes,
                }
            })
            .collect();

        Self::DefinitionList { items: def_items }
    }

    /// Creates a math block
    pub fn math_block(math: impl Into<String>) -> Self {
        Self::MathBlock { math: math.into() }
    }

    /// Returns this node as a heading if it is one
    pub fn as_heading(&self) -> Option<(u8, &Vec<InlineNode>)> {
        match self {
            Self::Heading { level, children } => Some((*level, children)),
            _ => None,
        }
    }

    /// Returns this node as a paragraph if it is one
    pub fn as_paragraph(&self) -> Option<&Vec<InlineNode>> {
        match self {
            Self::Paragraph { children } => Some(children),
            _ => None,
        }
    }

    /// Returns this node as a list if it is one
    pub fn as_list(&self) -> Option<(&ListType, &Vec<ListItem>)> {
        match self {
            Self::List { list_type, items } => Some((list_type, items)),
            _ => None,
        }
    }

    /// Returns this node as a code block if it is one
    pub fn as_code_block(&self) -> Option<(&str, &str)> {
        match self {
            Self::CodeBlock { language, code } => Some((language, code)),
            _ => None,
        }
    }

    /// Returns this node as a blockquote if it is one
    pub fn as_blockquote(&self) -> Option<&Vec<Node>> {
        match self {
            Self::BlockQuote { children } => Some(children),
            _ => None,
        }
    }

    /// Returns this node as a table if it is one
    pub fn as_table(&self) -> Option<TableComponents> {
        match self {
            Self::Table {
                header,
                rows,
                alignments,
            } => Some((header, rows, alignments)),
            _ => None,
        }
    }

    /// Returns this node as a footnote reference if it is one
    pub fn as_footnote_reference(&self) -> Option<&FootnoteReference> {
        match self {
            Self::FootnoteReference(reference) => Some(reference),
            _ => None,
        }
    }

    /// Returns this node as a footnote definition if it is one
    pub fn as_footnote_definition(&self) -> Option<&FootnoteDefinition> {
        match self {
            Self::FootnoteDefinition(definition) => Some(definition),
            _ => None,
        }
    }

    /// Returns this node as a definition list if it is one
    pub fn as_definition_list(&self) -> Option<&Vec<DefinitionItem>> {
        match self {
            Self::DefinitionList { items } => Some(items),
            _ => None,
        }
    }

    /// Returns this node as a math block if it is one
    pub fn as_math_block(&self) -> Option<&str> {
        match self {
            Self::MathBlock { math } => Some(math),
            _ => None,
        }
    }

    /// Returns whether this node is a thematic break
    pub fn is_thematic_break(&self) -> bool {
        matches!(self, Self::ThematicBreak)
    }
}
