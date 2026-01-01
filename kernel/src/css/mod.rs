// CSS module for ASTRA.OS Browser
// Provides CSS parsing and style computation

pub mod parser;
pub mod selector;

pub use parser::{Color, PropertyValue, Declaration, Rule, Stylesheet, parse_css};
pub use selector::{SelectorType, parse_selector, matches_selector, compute_style, ComputedStyle};
