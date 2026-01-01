// CSS Parser for ASTRA.OS Browser
// Supports: colors, font-size, margin, padding

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// CSS Color value
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const RED: Color = Color::new(255, 0, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);
    pub const CYAN: Color = Color::new(0, 255, 255);
    pub const YELLOW: Color = Color::new(255, 255, 0);
    pub const MAGENTA: Color = Color::new(255, 0, 255);

    /// Parse hex color (#RRGGBB or #RGB)
    pub fn from_hex(s: &str) -> Option<Color> {
        if !s.starts_with('#') {
            return None;
        }

        let hex = &s[1..];
        match hex.len() {
            3 => {
                // #RGB format
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Color::new(r, g, b))
            }
            6 => {
                // #RRGGBB format
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::new(r, g, b))
            }
            _ => None,
        }
    }

    /// Parse named color (case-insensitive comparison)
    pub fn from_name(name: &str) -> Option<Color> {
        // Manual case-insensitive comparison
        let name_lower = name.as_bytes();
        if name_lower.eq_ignore_ascii_case(b"black") {
            Some(Color::BLACK)
        } else if name_lower.eq_ignore_ascii_case(b"white") {
            Some(Color::WHITE)
        } else if name_lower.eq_ignore_ascii_case(b"red") {
            Some(Color::RED)
        } else if name_lower.eq_ignore_ascii_case(b"green") {
            Some(Color::GREEN)
        } else if name_lower.eq_ignore_ascii_case(b"blue") {
            Some(Color::BLUE)
        } else if name_lower.eq_ignore_ascii_case(b"cyan") {
            Some(Color::CYAN)
        } else if name_lower.eq_ignore_ascii_case(b"yellow") {
            Some(Color::YELLOW)
        } else if name_lower.eq_ignore_ascii_case(b"magenta") {
            Some(Color::MAGENTA)
        } else {
            None
        }
    }
}

/// CSS property value
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Color(Color),
    Length(i32),        // pixels
    Keyword(String),
}

/// CSS declaration (property: value)
#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: PropertyValue,
}

/// CSS rule (selector { declarations })
#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<String>,
    pub declarations: Vec<Declaration>,
}

/// CSS Stylesheet
#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn new() -> Self {
        Stylesheet { rules: Vec::new() }
    }

    /// Get default stylesheet
    pub fn default_styles() -> Self {
        use alloc::vec;
        let mut stylesheet = Stylesheet::new();

        // Default body style
        let mut body_decls = Vec::new();
        body_decls.push(Declaration {
            property: String::from("color"),
            value: PropertyValue::Color(Color::WHITE),
        });
        body_decls.push(Declaration {
            property: String::from("background-color"),
            value: PropertyValue::Color(Color::BLACK),
        });

        let mut body_selectors = Vec::new();
        body_selectors.push(String::from("body"));
        stylesheet.rules.push(Rule {
            selectors: body_selectors,
            declarations: body_decls,
        });

        // Default h1 style
        let mut h1_decls = Vec::new();
        h1_decls.push(Declaration {
            property: String::from("font-size"),
            value: PropertyValue::Length(20),
        });
        h1_decls.push(Declaration {
            property: String::from("color"),
            value: PropertyValue::Color(Color::WHITE),
        });
        h1_decls.push(Declaration {
            property: String::from("margin"),
            value: PropertyValue::Length(10),
        });

        let mut h1_selectors = Vec::new();
        h1_selectors.push(String::from("h1"));
        stylesheet.rules.push(Rule {
            selectors: h1_selectors,
            declarations: h1_decls,
        });

        // Default p style
        let mut p_decls = Vec::new();
        p_decls.push(Declaration {
            property: String::from("font-size"),
            value: PropertyValue::Length(14),
        });
        p_decls.push(Declaration {
            property: String::from("margin"),
            value: PropertyValue::Length(5),
        });

        let mut p_selectors = Vec::new();
        p_selectors.push(String::from("p"));
        stylesheet.rules.push(Rule {
            selectors: p_selectors,
            declarations: p_decls,
        });

        stylesheet
    }
}

/// Simple CSS tokenizer
struct Tokenizer {
    input: Vec<char>,
    pos: usize,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn current(&self) -> char {
        self.input.get(self.pos).copied().unwrap_or('\0')
    }

    fn consume(&mut self) -> char {
        let c = self.current();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while !self.eof() && self.current().is_whitespace() {
            self.consume();
        }
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.current()) {
            result.push(self.consume());
        }
        result
    }
}

/// Parse CSS stylesheet
pub fn parse_css(source: &str) -> Stylesheet {
    let mut tokenizer = Tokenizer::new(source);
    let mut stylesheet = Stylesheet::new();

    while !tokenizer.eof() {
        tokenizer.skip_whitespace();
        if tokenizer.eof() {
            break;
        }

        // Parse rule
        if let Some(rule) = parse_rule(&mut tokenizer) {
            stylesheet.rules.push(rule);
        }
    }

    stylesheet
}

/// Parse a single CSS rule
fn parse_rule(tokenizer: &mut Tokenizer) -> Option<Rule> {
    tokenizer.skip_whitespace();

    // Parse selectors (comma-separated)
    let selectors = parse_selectors(tokenizer)?;

    tokenizer.skip_whitespace();

    // Expect '{'
    if tokenizer.current() != '{' {
        return None;
    }
    tokenizer.consume();

    // Parse declarations
    let declarations = parse_declarations(tokenizer)?;

    Some(Rule {
        selectors,
        declarations,
    })
}

/// Parse selector list
fn parse_selectors(tokenizer: &mut Tokenizer) -> Option<Vec<String>> {
    let mut selectors = Vec::new();

    loop {
        tokenizer.skip_whitespace();

        let selector = tokenizer.consume_while(|c| {
            c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '#'
        });

        if !selector.is_empty() {
            selectors.push(selector);
        }

        tokenizer.skip_whitespace();

        if tokenizer.current() == ',' {
            tokenizer.consume();
        } else {
            break;
        }
    }

    if selectors.is_empty() {
        None
    } else {
        Some(selectors)
    }
}

/// Parse declarations block
fn parse_declarations(tokenizer: &mut Tokenizer) -> Option<Vec<Declaration>> {
    let mut declarations = Vec::new();

    loop {
        tokenizer.skip_whitespace();

        if tokenizer.current() == '}' {
            tokenizer.consume();
            break;
        }

        if tokenizer.eof() {
            break;
        }

        // Parse declaration
        if let Some(decl) = parse_declaration(tokenizer) {
            declarations.push(decl);
        }

        // Skip semicolon
        tokenizer.skip_whitespace();
        if tokenizer.current() == ';' {
            tokenizer.consume();
        }
    }

    Some(declarations)
}

/// Parse a single declaration (property: value)
fn parse_declaration(tokenizer: &mut Tokenizer) -> Option<Declaration> {
    tokenizer.skip_whitespace();

    // Property name
    let property = tokenizer.consume_while(|c| c.is_alphanumeric() || c == '-');

    if property.is_empty() {
        return None;
    }

    tokenizer.skip_whitespace();

    // Expect ':'
    if tokenizer.current() != ':' {
        return None;
    }
    tokenizer.consume();

    tokenizer.skip_whitespace();

    // Value
    let value = parse_value(tokenizer)?;

    Some(Declaration { property, value })
}

/// Parse property value
fn parse_value(tokenizer: &mut Tokenizer) -> Option<PropertyValue> {
    tokenizer.skip_whitespace();

    let value_str = tokenizer.consume_while(|c| {
        !c.is_whitespace() && c != ';' && c != '}'
    });

    // Try to parse as color
    if value_str.starts_with('#') {
        if let Some(color) = Color::from_hex(&value_str) {
            return Some(PropertyValue::Color(color));
        }
    }

    // Try to parse as named color
    if let Some(color) = Color::from_name(&value_str) {
        return Some(PropertyValue::Color(color));
    }

    // Try to parse as length (number, assume pixels)
    if let Ok(num) = value_str.parse::<i32>() {
        return Some(PropertyValue::Length(num));
    }

    // Otherwise, treat as keyword
    Some(PropertyValue::Keyword(value_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parsing() {
        assert_eq!(Color::from_hex("#FF0000"), Some(Color::RED));
        assert_eq!(Color::from_hex("#F00"), Some(Color::RED));
        assert_eq!(Color::from_name("white"), Some(Color::WHITE));
    }

    #[test]
    fn test_css_parsing() {
        let css = "h1 { color: #FF0000; font-size: 20; }";
        let stylesheet = parse_css(css);
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors[0], "h1");
        assert_eq!(stylesheet.rules[0].declarations.len(), 2);
    }
}
