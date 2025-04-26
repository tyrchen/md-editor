use crate::TextFormatting;
use serde::{Deserialize, Serialize};

/// Represents a text node with formatting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextNode {
    /// The text content
    pub text: String,
    /// Formatting applied to the text
    pub formatting: TextFormatting,
}

impl TextNode {
    /// Creates a new text node with the given text and default formatting
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            formatting: TextFormatting::default(),
        }
    }

    /// Creates a new text node with the given text and formatting
    pub fn with_formatting(text: impl Into<String>, formatting: TextFormatting) -> Self {
        Self {
            text: text.into(),
            formatting,
        }
    }

    /// Creates a bold text node
    pub fn bold(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            formatting: TextFormatting::bold(),
        }
    }

    /// Creates an italic text node
    pub fn italic(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            formatting: TextFormatting::italic(),
        }
    }

    /// Creates a code text node
    pub fn code(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            formatting: TextFormatting::code(),
        }
    }

    /// Creates a strikethrough text node
    pub fn strikethrough(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            formatting: TextFormatting::strikethrough(),
        }
    }
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

impl InlineNode {
    /// Creates a plain text node
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(TextNode::new(text))
    }

    /// Creates a bold text node
    pub fn bold_text(text: impl Into<String>) -> Self {
        Self::Text(TextNode::bold(text))
    }

    /// Creates an italic text node
    pub fn italic_text(text: impl Into<String>) -> Self {
        Self::Text(TextNode::italic(text))
    }

    /// Creates a strikethrough text node
    pub fn strikethrough_text(text: impl Into<String>) -> Self {
        Self::Text(TextNode::strikethrough(text))
    }

    /// Creates a code span
    pub fn code_span(code: impl Into<String>) -> Self {
        Self::CodeSpan { code: code.into() }
    }

    /// Creates a link with the given URL and text
    pub fn link(url: impl Into<String>, text: impl Into<String>) -> Self {
        Self::Link {
            url: url.into(),
            title: None,
            children: vec![Self::text(text)],
        }
    }

    /// Creates a link with the given URL, title, and text
    pub fn link_with_title(
        url: impl Into<String>,
        title: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self::Link {
            url: url.into(),
            title: Some(title.into()),
            children: vec![Self::text(text)],
        }
    }

    /// Creates an image with the given URL and alt text
    pub fn image(url: impl Into<String>, alt: impl Into<String>) -> Self {
        Self::Image {
            url: url.into(),
            alt: alt.into(),
            title: None,
        }
    }

    /// Creates an image with the given URL, alt text, and title
    pub fn image_with_title(
        url: impl Into<String>,
        alt: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self::Image {
            url: url.into(),
            alt: alt.into(),
            title: Some(title.into()),
        }
    }

    /// Creates an autolink for a URL
    pub fn autolink_url(url: impl Into<String>) -> Self {
        Self::AutoLink {
            url: url.into(),
            is_email: false,
        }
    }

    /// Creates an autolink for an email address
    pub fn autolink_email(email: impl Into<String>) -> Self {
        Self::AutoLink {
            url: email.into(),
            is_email: true,
        }
    }

    /// Creates a footnote reference
    pub fn footnote_ref(label: impl Into<String>) -> Self {
        Self::FootnoteRef {
            label: label.into(),
        }
    }

    /// Creates an inline footnote
    pub fn inline_footnote(content: impl Into<String>) -> Self {
        Self::InlineFootnote {
            children: vec![Self::text(content)],
        }
    }

    /// Creates a user mention (like @username)
    pub fn user_mention(username: impl Into<String>) -> Self {
        Self::Mention {
            name: username.into(),
            mention_type: "user".to_string(),
        }
    }

    /// Creates an issue mention (like #123)
    pub fn issue_mention(issue: impl Into<String>) -> Self {
        Self::Mention {
            name: issue.into(),
            mention_type: "issue".to_string(),
        }
    }

    /// Creates an inline math expression
    pub fn math(math: impl Into<String>) -> Self {
        Self::Math { math: math.into() }
    }

    /// Creates an emoji from a shortcode
    pub fn emoji(shortcode: impl Into<String>) -> Self {
        Self::Emoji {
            shortcode: shortcode.into(),
        }
    }

    /// Creates a hard break
    pub fn hard_break() -> Self {
        Self::HardBreak
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(&text.text),
            _ => None,
        }
    }
}
