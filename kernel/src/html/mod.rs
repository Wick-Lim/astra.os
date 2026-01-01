// Simple HTML parser for ASTRA.OS
// Supports: <h1>, <p>, <div>, <span>, text nodes

#![allow(dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;

pub mod renderer;

#[derive(Debug, Clone)]
pub enum Node {
    Element {
        tag: String,
        children: Vec<Node>,
    },
    Text(String),
}

pub struct Parser {
    pos: usize,
    input: Vec<char>,
}

impl Parser {
    pub fn new(html: &str) -> Self {
        Parser {
            pos: 0,
            input: html.chars().collect(),
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn next_char(&self) -> char {
        self.input[self.pos]
    }

    fn starts_with(&self, s: &str) -> bool {
        let remaining = &self.input[self.pos..];
        let mut chars = s.chars();
        for (i, ch) in remaining.iter().enumerate() {
            match chars.next() {
                Some(expected) if *ch == expected => continue,
                Some(_) => return false,
                None => return true,
            }
        }
        chars.next().is_none()
    }

    fn consume_char(&mut self) -> char {
        let c = self.input[self.pos];
        self.pos += 1;
        c
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| c.is_alphanumeric())
    }

    fn parse_node(&mut self) -> Node {
        if self.starts_with("<") {
            self.parse_element()
        } else {
            self.parse_text()
        }
    }

    fn parse_text(&mut self) -> Node {
        Node::Text(self.consume_while(|c| c != '<'))
    }

    fn parse_element(&mut self) -> Node {
        // Opening tag
        assert_eq!(self.consume_char(), '<');
        let tag_name = self.parse_tag_name();
        self.consume_while(|c| c != '>');
        assert_eq!(self.consume_char(), '>');

        // Contents
        let children = self.parse_nodes();

        // Closing tag
        if self.starts_with("</") {
            assert_eq!(self.consume_char(), '<');
            assert_eq!(self.consume_char(), '/');
            let close_tag = self.parse_tag_name();
            assert_eq!(close_tag, tag_name);  // Simplified assertion
            self.consume_while(|c| c != '>');
            assert_eq!(self.consume_char(), '>');
        }

        Node::Element {
            tag: tag_name,
            children,
        }
    }

    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    // Simplified parse that doesn't use recursion heavily
    pub fn parse(html: &str) -> Node {
        use crate::serial_println;
        serial_println!("  parse: creating parser");

        // For now, just create a simple result without full parsing
        // to test if allocation is the issue
        Node::Element {
            tag: String::from("html"),
            children: alloc::vec![
                Node::Element {
                    tag: String::from("h1"),
                    children: alloc::vec![
                        Node::Text(String::from("Hello"))
                    ],
                }
            ],
        }
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
