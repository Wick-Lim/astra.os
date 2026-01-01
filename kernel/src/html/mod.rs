// Minimal HTML5 parser for ASTRA.OS
// This is a simplified version focusing on essential HTML parsing for a browser OS
// Based on html5ever concepts but adapted for no_std environment

pub mod renderer;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// HTML Node types
#[derive(Debug, Clone)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
    Comment(String),
}

/// Element data (tag name and attributes)
#[derive(Debug, Clone)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: Vec<(String, String)>,
}

/// HTML DOM Node
#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Box<Node>>,
}

impl Node {
    /// Create a new text node
    pub fn text(data: String) -> Box<Node> {
        Box::new(Node {
            node_type: NodeType::Text(data),
            children: Vec::new(),
        })
    }

    /// Create a new element node
    pub fn element(tag_name: String, attributes: Vec<(String, String)>, children: Vec<Box<Node>>) -> Box<Node> {
        Box::new(Node {
            node_type: NodeType::Element(ElementData { tag_name, attributes }),
            children,
        })
    }

    /// Create a new comment node
    pub fn comment(data: String) -> Box<Node> {
        Box::new(Node {
            node_type: NodeType::Comment(data),
            children: Vec::new(),
        })
    }
}

/// Minimal HTML Parser
pub struct HtmlParser {
    pos: usize,
    input: Vec<u8>,
}

impl HtmlParser {
    pub fn new(input: String) -> Self {
        HtmlParser {
            pos: 0,
            input: input.into_bytes(),
        }
    }

    /// Parse HTML and return DOM tree
    pub fn parse(&mut self) -> Box<Node> {
        let mut nodes = self.parse_nodes();

        // If we have a single root, return it
        if nodes.len() == 1 {
            nodes.swap_remove(0)
        } else {
            // Otherwise, create an implicit root
            Node::element("html".into(), Vec::new(), nodes)
        }
    }

    /// Parse a sequence of sibling nodes
    fn parse_nodes(&mut self) -> Vec<Box<Node>> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();

            if self.eof() || self.starts_with("</") {
                break;
            }

            if let Some(node) = self.parse_node() {
                nodes.push(node);
            }
        }
        nodes
    }

    /// Parse a single node
    fn parse_node(&mut self) -> Option<Box<Node>> {
        if self.current_char() == b'<' {
            self.parse_element()
        } else {
            self.parse_text()
        }
    }

    /// Parse an element tag
    fn parse_element(&mut self) -> Option<Box<Node>> {
        // Opening tag
        self.consume_char(); // consume '<'

        let tag_name = self.parse_tag_name();
        let attributes = self.parse_attributes();

        self.consume_char(); // consume '>'

        // Contents
        let children = self.parse_nodes();

        // Closing tag (simplified - doesn't validate tag names match)
        if self.starts_with("</") {
            self.consume_char(); // '<'
            self.consume_char(); // '/'
            self.parse_tag_name(); // consume closing tag name
            self.consume_char(); // '>'
        }

        Some(Node::element(tag_name, attributes, children))
    }

    /// Parse tag name
    fn parse_tag_name(&mut self) -> String {
        let mut name = String::new();
        while !self.eof() && self.current_char().is_ascii_alphanumeric() {
            name.push(self.consume_char() as char);
        }
        name
    }

    /// Parse attributes
    fn parse_attributes(&mut self) -> Vec<(String, String)> {
        let mut attributes = Vec::new();
        loop {
            self.consume_whitespace();

            if self.current_char() == b'>' || self.eof() {
                break;
            }

            let name = self.parse_tag_name();
            self.consume_whitespace();

            if self.current_char() == b'=' {
                self.consume_char();
                self.consume_whitespace();

                let value = if self.current_char() == b'"' || self.current_char() == b'\'' {
                    self.parse_quoted_string()
                } else {
                    String::new()
                };

                attributes.push((name, value));
            }
        }
        attributes
    }

    /// Parse quoted string
    fn parse_quoted_string(&mut self) -> String {
        let quote = self.consume_char();
        let mut value = String::new();

        while !self.eof() && self.current_char() != quote {
            value.push(self.consume_char() as char);
        }

        if !self.eof() {
            self.consume_char(); // consume closing quote
        }

        value
    }

    /// Parse text node
    fn parse_text(&mut self) -> Option<Box<Node>> {
        let mut text = String::new();
        while !self.eof() && self.current_char() != b'<' {
            text.push(self.consume_char() as char);
        }

        if text.is_empty() {
            None
        } else {
            Some(Node::text(text))
        }
    }

    /// Consume whitespace
    fn consume_whitespace(&mut self) {
        while !self.eof() && self.current_char().is_ascii_whitespace() {
            self.consume_char();
        }
    }

    /// Get current character without consuming
    fn current_char(&self) -> u8 {
        if self.eof() {
            0
        } else {
            self.input[self.pos]
        }
    }

    /// Consume current character and advance
    fn consume_char(&mut self) -> u8 {
        let c = self.current_char();
        self.pos += 1;
        c
    }

    /// Check if we're at end of input
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Check if input starts with string at current position
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s.as_bytes())
    }
}

/// Parse HTML string into DOM tree
pub fn parse_html(html: String) -> Box<Node> {
    HtmlParser::new(html).parse()
}
