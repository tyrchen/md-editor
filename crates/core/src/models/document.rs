use crate::{InlineNode, Node, Selection, TextNode};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

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
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new document with a title heading
    pub fn with_title(title: impl Into<String>) -> Self {
        let title_str = title.into();
        let mut doc = Self::default();
        doc.nodes.push(Node::heading(1, title_str.clone()));
        doc.metadata = Some(DocumentMetadata {
            title: Some(title_str),
            ..Default::default()
        });
        doc
    }

    /// Adds a heading to the document
    pub fn add_heading(&mut self, level: u8, text: impl Into<String>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::heading(level, text));
        index
    }

    /// Adds a paragraph to the document
    pub fn add_paragraph(&mut self) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::Paragraph {
            children: Vec::new(),
        });
        index
    }

    /// Adds a paragraph with text to the document
    pub fn add_paragraph_with_text(&mut self, text: impl Into<String>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::paragraph(text));
        index
    }

    /// Adds a paragraph with custom inline nodes to the document
    pub fn add_paragraph_with_inlines(&mut self, inlines: Vec<InlineNode>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::paragraph_with_inlines(inlines));
        index
    }

    /// Adds a code block to the document
    pub fn add_code_block(
        &mut self,
        code: impl Into<String>,
        language: impl Into<String>,
    ) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::code_block(code, language));
        index
    }

    /// Adds an unordered list to the document
    pub fn add_unordered_list(&mut self, items: Vec<impl Into<String>>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::unordered_list(items));
        index
    }

    /// Adds an ordered list to the document
    pub fn add_ordered_list(&mut self, items: Vec<impl Into<String>>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::ordered_list(items));
        index
    }

    /// Adds a task list to the document
    pub fn add_task_list(&mut self, items: Vec<(impl Into<String>, bool)>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::task_list(items));
        index
    }

    /// Inserts text at the specified location
    pub fn insert_text(
        &mut self,
        node_index: usize,
        offset: usize,
        text: impl Into<String>,
    ) -> bool {
        if node_index >= self.nodes.len() {
            return false;
        }

        match &mut self.nodes[node_index] {
            Node::Paragraph { children } => {
                // Simple case: if children is empty, just add a new text node
                if children.is_empty() {
                    children.push(InlineNode::text(text));
                    return true;
                }

                // Find the right text node to insert into
                let mut current_offset = 0;
                for child in children.iter_mut() {
                    match child {
                        InlineNode::Text(TextNode { text: content, .. }) => {
                            let next_offset = current_offset + content.len();

                            // Insert inside this text node
                            if offset >= current_offset && offset <= next_offset {
                                let insertion_point = offset - current_offset;
                                content.insert_str(insertion_point, &text.into());
                                return true;
                            }

                            current_offset = next_offset;
                        }
                        // For other node types, just count their length for now
                        // A more sophisticated implementation would handle insertion
                        // within these nodes too
                        _ => {
                            // Simplified: just consider each non-text node as length 1
                            current_offset += 1;
                        }
                    }
                }

                // If we get here, append to the last text node or create a new one
                if let Some(InlineNode::Text(TextNode { text: content, .. })) = children.last_mut()
                {
                    content.push_str(&text.into());
                } else {
                    children.push(InlineNode::text(text));
                }

                true
            }
            // Currently only handling insertion into paragraphs
            // A complete implementation would handle other node types
            _ => false,
        }
    }

    /// Splits a node at the specified location
    pub fn split_node(&mut self, node_index: usize, offset: usize) -> bool {
        if node_index >= self.nodes.len() {
            return false;
        }

        match &self.nodes[node_index] {
            Node::Paragraph { children } => {
                // Create two new paragraphs: before and after the split
                let mut before_children = Vec::new();
                let mut after_children = Vec::new();

                let mut current_offset = 0;
                let mut found_split = false;

                for child in children {
                    match child {
                        InlineNode::Text(TextNode { text, formatting }) => {
                            let next_offset = current_offset + text.len();

                            // Split inside this text node
                            if offset >= current_offset && offset <= next_offset && !found_split {
                                let split_point = offset - current_offset;

                                // Add text before split to first paragraph
                                if split_point > 0 {
                                    before_children.push(InlineNode::Text(TextNode {
                                        text: text[..split_point].to_string(),
                                        formatting: formatting.clone(),
                                    }));
                                }

                                // Add text after split to second paragraph
                                if split_point < text.len() {
                                    after_children.push(InlineNode::Text(TextNode {
                                        text: text[split_point..].to_string(),
                                        formatting: formatting.clone(),
                                    }));
                                }

                                found_split = true;
                            } else if !found_split {
                                before_children.push(child.clone());
                            } else {
                                after_children.push(child.clone());
                            }

                            current_offset = next_offset;
                        }
                        // For other node types, simplified approach
                        _ => {
                            if !found_split {
                                before_children.push(child.clone());
                            } else {
                                after_children.push(child.clone());
                            }
                            current_offset += 1;
                        }
                    }
                }

                // If we didn't find a split point, just split at the end
                if !found_split {
                    before_children = children.clone();
                }

                // Replace the original node with the split nodes
                let mut new_nodes = self.nodes[..node_index].to_vec();
                new_nodes.push(Node::Paragraph {
                    children: before_children,
                });
                new_nodes.push(Node::Paragraph {
                    children: after_children,
                });
                new_nodes.extend_from_slice(&self.nodes[node_index + 1..]);

                self.nodes = new_nodes;
                true
            }
            // Currently only handling splitting paragraphs
            // A complete implementation would handle other node types
            _ => false,
        }
    }

    /// Returns a string representation of the document structure
    pub fn debug_structure(&self) -> String {
        let mut result = String::new();

        for (i, node) in self.nodes.iter().enumerate() {
            let node_type = match node {
                Node::Heading { level, .. } => format!("Heading (level {})", level),
                Node::Paragraph { .. } => "Paragraph".to_string(),
                Node::List { list_type, .. } => format!("List ({:?})", list_type),
                Node::CodeBlock { language, .. } => format!("CodeBlock ({})", language),
                Node::BlockQuote { .. } => "BlockQuote".to_string(),
                Node::ThematicBreak => "ThematicBreak".to_string(),
                Node::Table { .. } => "Table".to_string(),
                Node::Group { name, .. } => format!("Group ({})", name),
                Node::FootnoteReference { .. } => "FootnoteReference".to_string(),
                Node::FootnoteDefinition { .. } => "FootnoteDefinition".to_string(),
                Node::DefinitionList { .. } => "DefinitionList".to_string(),
                Node::MathBlock { .. } => "MathBlock".to_string(),
                Node::TempListItem(_) => "TempListItem (Internal)".to_string(),
                Node::TempTableCell(_) => "TempTableCell (Internal)".to_string(),
            };

            result.push_str(&format!("{}. {}\n", i, node_type));
        }

        if let Some(selection) = &self.selection {
            result.push_str(&format!("\nSelection: {:?}", selection));
        }

        result
    }
}

// Allow using Document as if it were a Vec<Node>
impl Deref for Document {
    type Target = Vec<Node>;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl DerefMut for Document {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}

impl AsRef<Document> for Document {
    fn as_ref(&self) -> &Document {
        self
    }
}
