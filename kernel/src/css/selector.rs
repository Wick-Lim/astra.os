// CSS Selector Matching Engine
// Supports: tag selectors, class selectors, ID selectors

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::html::ElementData;
use super::parser::{Stylesheet, Rule, PropertyValue};

/// Selector type
#[derive(Debug, Clone, PartialEq)]
pub enum SelectorType {
    Tag(String),        // h1, p, div
    Class(String),      // .my-class
    Id(String),         // #my-id
}

/// Parse selector string into SelectorType
pub fn parse_selector(selector: &str) -> SelectorType {
    if selector.starts_with('#') {
        SelectorType::Id(String::from(&selector[1..]))
    } else if selector.starts_with('.') {
        SelectorType::Class(String::from(&selector[1..]))
    } else {
        SelectorType::Tag(String::from(selector))
    }
}

/// Check if selector matches element
pub fn matches_selector(element: &ElementData, selector: &SelectorType) -> bool {
    match selector {
        SelectorType::Tag(tag) => &element.tag_name == tag,
        SelectorType::Class(class) => {
            // Check if element has this class
            element.attributes.iter()
                .find(|(key, _)| key == "class")
                .map(|(_, classes)| {
                    classes.split_whitespace().any(|c| c == class)
                })
                .unwrap_or(false)
        }
        SelectorType::Id(id) => {
            // Check if element has this ID
            element.attributes.iter()
                .find(|(key, _)| key == "id")
                .map(|(_, element_id)| element_id == id)
                .unwrap_or(false)
        }
    }
}

/// Specificity for selector sorting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    pub id: u32,        // ID selectors
    pub class: u32,     // Class selectors
    pub tag: u32,       // Tag selectors
}

impl Specificity {
    pub fn new(selector: &SelectorType) -> Self {
        match selector {
            SelectorType::Id(_) => Specificity { id: 1, class: 0, tag: 0 },
            SelectorType::Class(_) => Specificity { id: 0, class: 1, tag: 0 },
            SelectorType::Tag(_) => Specificity { id: 0, class: 0, tag: 1 },
        }
    }
}

/// Matched rule with specificity
#[derive(Debug, Clone)]
pub struct MatchedRule<'a> {
    pub specificity: Specificity,
    pub rule: &'a Rule,
}

/// Find all rules that match an element
pub fn matching_rules<'a>(element: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    let mut matched = Vec::new();

    for rule in &stylesheet.rules {
        for selector_str in &rule.selectors {
            let selector = parse_selector(selector_str);

            if matches_selector(element, &selector) {
                matched.push(MatchedRule {
                    specificity: Specificity::new(&selector),
                    rule,
                });
                break; // One match per rule is enough
            }
        }
    }

    // Sort by specificity (higher specificity = higher priority)
    matched.sort_by(|a, b| b.specificity.cmp(&a.specificity));

    matched
}

/// Computed style properties
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub properties: BTreeMap<String, PropertyValue>,
}

impl ComputedStyle {
    pub fn new() -> Self {
        ComputedStyle {
            properties: BTreeMap::new(),
        }
    }

    /// Get property value
    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.properties.get(name)
    }

    /// Set property value
    pub fn set(&mut self, name: String, value: PropertyValue) {
        self.properties.insert(name, value);
    }
}

/// Compute style for an element
pub fn compute_style(element: &ElementData, stylesheet: &Stylesheet) -> ComputedStyle {
    let mut style = ComputedStyle::new();

    // Get matching rules (already sorted by specificity)
    let matched = matching_rules(element, stylesheet);

    // Apply rules in order (lower specificity first, so higher specificity overwrites)
    for matched_rule in matched.iter().rev() {
        for declaration in &matched_rule.rule.declarations {
            style.set(
                declaration.property.clone(),
                declaration.value.clone(),
            );
        }
    }

    // Apply inline styles (highest priority)
    if let Some((_, style_str)) = element.attributes.iter().find(|(key, _)| key == "style") {
        parse_inline_style(style_str, &mut style);
    }

    style
}

/// Parse inline style attribute
fn parse_inline_style(style_str: &str, computed: &mut ComputedStyle) {
    // Simple inline style parser (property: value; property: value;)
    for part in style_str.split(';') {
        let mut parts = part.splitn(2, ':');
        if let (Some(property), Some(value)) = (parts.next(), parts.next()) {
            let property = String::from(property.trim());
            let value_str = value.trim();

            // Parse value (simplified)
            if let Some(parsed_value) = parse_inline_value(value_str) {
                computed.set(property, parsed_value);
            }
        }
    }
}

/// Parse inline style value
fn parse_inline_value(value: &str) -> Option<PropertyValue> {
    use super::parser::Color;

    // Try hex color
    if value.starts_with('#') {
        if let Some(color) = Color::from_hex(value) {
            return Some(PropertyValue::Color(color));
        }
    }

    // Try named color
    if let Some(color) = Color::from_name(value) {
        return Some(PropertyValue::Color(color));
    }

    // Try number (pixels)
    if let Ok(num) = value.parse::<i32>() {
        return Some(PropertyValue::Length(num));
    }

    // Remove 'px' suffix if present
    if value.ends_with("px") {
        if let Ok(num) = value[..value.len() - 2].parse::<i32>() {
            return Some(PropertyValue::Length(num));
        }
    }

    Some(PropertyValue::Keyword(String::from(value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_parsing() {
        assert_eq!(parse_selector("h1"), SelectorType::Tag(String::from("h1")));
        assert_eq!(parse_selector(".my-class"), SelectorType::Class(String::from("my-class")));
        assert_eq!(parse_selector("#my-id"), SelectorType::Id(String::from("my-id")));
    }

    #[test]
    fn test_selector_matching() {
        let attrs = Vec::new();
        let element = ElementData {
            tag_name: String::from("div"),
            attributes: attrs,
        };

        assert!(matches_selector(&element, &SelectorType::Tag(String::from("div"))));
        assert!(!matches_selector(&element, &SelectorType::Tag(String::from("p"))));

        let mut attrs = Vec::new();
        attrs.push((String::from("class"), String::from("foo bar")));
        let element = ElementData {
            tag_name: String::from("div"),
            attributes: attrs,
        };
        assert!(matches_selector(&element, &SelectorType::Class(String::from("foo"))));
        assert!(matches_selector(&element, &SelectorType::Class(String::from("bar"))));
        assert!(!matches_selector(&element, &SelectorType::Class(String::from("baz"))));
    }
}
