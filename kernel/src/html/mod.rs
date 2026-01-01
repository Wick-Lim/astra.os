// Simple HTML parser for ASTRA.OS
// Supports: <h1>, <p>, <div>, <span>, text nodes

#![allow(dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;

pub mod renderer;

pub enum Node {
    Element {
        tag: String,
        children: Vec<Box<Node>>,
    },
    Text(String),
}

// REMOVED Drop impl - causes crashes during struct initialization!

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(html: &'a str) -> Self {
        Parser {
            input: html,
            pos: 0,
        }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.remaining().starts_with(s)
    }

    fn consume_char(&mut self) -> Option<char> {
        let remaining = self.remaining();
        let mut chars = remaining.chars();
        match chars.next() {
            Some(ch) => {
                self.pos += ch.len_utf8();
                Some(ch)
            }
            None => None,
        }
    }

    fn consume_while<F>(&mut self, test: F) -> &'a str
    where
        F: Fn(char) -> bool,
    {
        let start = self.pos;
        while let Some(ch) = self.remaining().chars().next() {
            if !test(ch) {
                break;
            }
            self.pos += ch.len_utf8();
        }
        &self.input[start..self.pos]
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    fn parse_tag_name(&mut self) -> &'a str {
        self.consume_while(|c| c.is_alphanumeric())
    }

    // Safe iterative parser with depth limit
    pub fn parse(html: &str) -> Node {
        let mut parser = Parser::new(html);
        parser.consume_whitespace();

        // If starts with <, try to parse as element
        if !parser.eof() && parser.starts_with("<") {
            match parser.try_parse_element() {
                Some(node) => node,
                None => {
                    Node::Element {
                        tag: String::from("html"),
                        children: alloc::vec![
                            Box::new(Node::Text(String::from("Parse error")))
                        ],
                    }
                }
            }
        } else {
            Node::Text(String::from(html))
        }
    }

    // Non-panicking element parser
    fn try_parse_element(&mut self) -> Option<Node> {
        // Consume '<'
        if self.consume_char()? != '<' {
            return None;
        }

        let tag_name = self.parse_tag_name();
        if tag_name.is_empty() {
            return None;
        }

        // Skip attributes
        self.consume_while(|c| c != '>');

        // Consume '>'
        if self.consume_char()? != '>' {
            return None;
        }

        // Parse children (non-recursive, depth=1 only)
        let mut children = Vec::with_capacity(4);

        loop {
            self.consume_whitespace();

            if self.eof() {
                break;
            }

            // Check for closing tag
            if self.starts_with("</") {
                break;
            }

            // Check for nested tag
            if self.starts_with("<") {
                // For now, skip nested tags to avoid recursion
                self.consume_char(); // consume '<'
                self.consume_while(|c| c != '>');
                self.consume_char(); // consume '>'
                continue;
            }

            // Parse text
            let text = self.consume_while(|c| c != '<');
            if !text.is_empty() {
                children.push(Box::new(Node::Text(String::from(text))));
            }
        }

        // Try to consume closing tag
        if self.starts_with("</") {
            self.consume_char(); // '<'
            self.consume_char(); // '/'
            let _close_tag = self.parse_tag_name();
            self.consume_while(|c| c != '>');
            self.consume_char(); // '>'
        }

        // NOTE: Using actual parsed children - will leak memory on drop
        // TODO: Fix heap allocator deallocation bug
        Some(Node::Element {
            tag: String::from(tag_name),
            children,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let html = "<h1>Hello</h1><p>World</p>";
        let dom = Parser::parse(html);
        // Just make sure it doesn't panic
    }
}
