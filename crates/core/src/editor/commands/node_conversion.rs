use crate::editor::command::Command;
use crate::{Document, EditError, InlineNode, ListType, Node, NodeConversionType};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

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
    ) -> Self {
        Self {
            document,
            node_index,
            target_type,
            original_node: None,
        }
    }
}

impl Command for ConvertNodeTypeCommand {
    fn execute(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        // Store the original node for undo
        self.original_node = Some(document.nodes[self.node_index].clone());

        // Extract text content from the node
        let inline_content = match &document.nodes[self.node_index] {
            Node::Paragraph { children } => Some(children.clone()),
            Node::Heading { children, .. } => Some(children.clone()),
            Node::List { items, .. } => {
                // For lists, gather text from all items
                let mut content = Vec::new();
                for item in items {
                    if let Some(Node::Paragraph { children }) = item.children.first() {
                        content.extend_from_slice(children);
                    }
                }
                if content.is_empty() {
                    None
                } else {
                    Some(content)
                }
            }
            Node::CodeBlock { code, .. } => {
                // Convert code block to paragraph text
                Some(vec![InlineNode::text(code)])
            }
            Node::BlockQuote { children } => {
                // Extract text from the first paragraph in the blockquote
                children.iter().find_map(|node| match node {
                    Node::Paragraph { children } => Some(children.clone()),
                    _ => None,
                })
            }
            _ => None, // Other node types not supported for now
        };

        let inline_content = inline_content.ok_or(EditError::UnsupportedOperation)?;

        // Create the new node with the extracted content
        let new_node = match &self.target_type {
            NodeConversionType::Paragraph => Node::Paragraph {
                children: inline_content,
            },
            NodeConversionType::Heading(level) => Node::Heading {
                level: *level,
                children: inline_content,
            },
            NodeConversionType::List(list_type) => {
                // For conversion to list, create a single item
                let item_text = extract_text_from_inline_nodes(&inline_content);
                match list_type {
                    ListType::Unordered => Node::unordered_list(vec![item_text]),
                    ListType::Ordered => Node::ordered_list(vec![item_text]),
                    ListType::Task => Node::task_list(vec![(item_text, false)]),
                }
            }
            NodeConversionType::CodeBlock(language) => {
                // For conversion to code block, convert inline content to plain text
                let code_text = extract_text_from_inline_nodes(&inline_content);
                Node::code_block(code_text, language)
            }
            NodeConversionType::BlockQuote => {
                // For conversion to blockquote, wrap the content in a paragraph
                Node::BlockQuote {
                    children: vec![Node::Paragraph {
                        children: inline_content,
                    }],
                }
            }
        };

        // Replace the original node with the new one
        document.nodes[self.node_index] = new_node;

        Ok(())
    }

    fn undo(&mut self) -> Result<(), EditError> {
        let mut document = self.document.borrow_mut();

        if self.node_index >= document.nodes.len() {
            return Err(EditError::IndexOutOfBounds);
        }

        if let Some(original) = &self.original_node {
            // Restore the original node
            document.nodes[self.node_index] = original.clone();
            Ok(())
        } else {
            Err(EditError::OperationFailed)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Helper function to extract plain text from inline nodes
fn extract_text_from_inline_nodes(nodes: &[InlineNode]) -> String {
    let mut result = String::new();

    for node in nodes {
        match node {
            InlineNode::Text(text_node) => {
                result.push_str(&text_node.text);
            }
            InlineNode::Link { children, .. } => {
                // For links, recursively extract text from children
                result.push_str(&extract_text_from_inline_nodes(children));
            }
            InlineNode::Image { alt, .. } => {
                // For images, use the alt text
                result.push_str(alt);
            }
            InlineNode::Math { math, .. } => {
                // For math nodes, use the math content
                result.push_str(math);
            }
            InlineNode::FootnoteRef { label, .. } => {
                // For footnotes, use the label
                result.push_str(label);
            }
            InlineNode::HardBreak => {
                // For hard breaks, add a newline
                result.push('\n');
            }
            InlineNode::SoftBreak => {
                // For soft breaks, add a space
                result.push(' ');
            }
            InlineNode::Emoji { shortcode, .. } => {
                // For emojis, use the shortcode
                result.push_str(shortcode);
            }
            InlineNode::CodeSpan { code, .. } => {
                // For code spans, use the code content
                result.push_str(code);
            }
            InlineNode::AutoLink { url, .. } => {
                // For auto links, use the URL
                result.push_str(url);
            }
            InlineNode::InlineFootnote { children, .. } => {
                // For inline footnotes, recursively extract text from children
                result.push_str(&extract_text_from_inline_nodes(children));
            }
            InlineNode::Mention { name, .. } => {
                // For mentions, use the name
                result.push_str(name);
            }
        }
    }

    result
}
