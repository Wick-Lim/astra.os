// Layout Tree Construction and Layout Algorithm
// Converts styled HTML nodes into positioned boxes

use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;
use crate::html::{Node, NodeType, ElementData};
use crate::css::{ComputedStyle, PropertyValue, compute_style, Stylesheet};
use super::box_model::{Dimensions, Rect, EdgeSizes};

/// Type of box for layout
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    Block,
    Inline,
    Anonymous,  // For wrapping inline children of block elements
}

/// Layout box representing a rendered element
#[derive(Debug)]
pub struct LayoutBox {
    pub box_type: BoxType,
    pub dimensions: Dimensions,
    pub style: ComputedStyle,
    pub children: Vec<Box<LayoutBox>>,

    // For debugging and rendering
    pub element_name: Option<String>,
}

impl LayoutBox {
    pub fn new(box_type: BoxType, style: ComputedStyle) -> Self {
        LayoutBox {
            box_type,
            dimensions: Dimensions::new(),
            style,
            children: Vec::new(),
            element_name: None,
        }
    }

    /// Get a style property value
    fn get_style_value(&self, name: &str) -> Option<&PropertyValue> {
        self.style.get(name)
    }

    /// Get style value as i32 (for lengths)
    fn get_style_length(&self, name: &str, default: i32) -> i32 {
        match self.get_style_value(name) {
            Some(PropertyValue::Length(n)) => *n,
            _ => default,
        }
    }
}

/// Build layout tree from styled HTML node
pub fn build_layout_tree(node: &Node, stylesheet: &Stylesheet) -> Option<Box<LayoutBox>> {
    match &node.node_type {
        NodeType::Element(element) => {
            // Compute style for this element
            let style = compute_style(element, stylesheet);

            // Determine box type based on display property
            let box_type = match style.get("display") {
                Some(PropertyValue::Keyword(ref s)) if s == "inline" => BoxType::Inline,
                Some(PropertyValue::Keyword(ref s)) if s == "none" => return None,
                _ => BoxType::Block,  // Default to block
            };

            let mut layout_box = Box::new(LayoutBox::new(box_type, style));
            layout_box.element_name = Some(element.tag_name.clone());

            // Recursively build children
            for child in &node.children {
                if let Some(child_box) = build_layout_tree(child, stylesheet) {
                    layout_box.children.push(child_box);
                }
            }

            Some(layout_box)
        }
        NodeType::Text(_text) => {
            // For now, skip text nodes in layout
            // In a full implementation, text would create inline boxes
            None
        }
        NodeType::Comment(_) => None,
    }
}

/// Calculate layout for the entire tree
pub fn layout_tree(layout_box: &mut LayoutBox, containing_block: Dimensions) {
    match layout_box.box_type {
        BoxType::Block => layout_block(layout_box, containing_block),
        BoxType::Inline => layout_inline(layout_box, containing_block),
        BoxType::Anonymous => layout_block(layout_box, containing_block),
    }
}

/// Layout a block-level box
fn layout_block(layout_box: &mut LayoutBox, containing_block: Dimensions) {
    // Calculate width
    calculate_block_width(layout_box, containing_block);

    // Calculate position
    calculate_block_position(layout_box, containing_block);

    // Layout children
    layout_block_children(layout_box);

    // Calculate height based on children
    calculate_block_height(layout_box);
}

/// Calculate block box width
fn calculate_block_width(layout_box: &mut LayoutBox, containing_block: Dimensions) {
    let auto = PropertyValue::Keyword(String::from("auto"));

    // Get style values
    let width = layout_box.get_style_value("width").unwrap_or(&auto);

    let margin_left = layout_box.get_style_length("margin-left", 0);
    let margin_right = layout_box.get_style_length("margin-right", 0);

    let border_left = layout_box.get_style_length("border-left-width", 0);
    let border_right = layout_box.get_style_length("border-right-width", 0);

    let padding_left = layout_box.get_style_length("padding-left", 0);
    let padding_right = layout_box.get_style_length("padding-right", 0);

    // Try margin shorthand
    let margin = layout_box.get_style_length("margin", 0);
    let margin_left = if margin_left == 0 { margin } else { margin_left };
    let margin_right = if margin_right == 0 { margin } else { margin_right };

    // Try padding shorthand
    let padding = layout_box.get_style_length("padding", 0);
    let padding_left = if padding_left == 0 { padding } else { padding_left };
    let padding_right = if padding_right == 0 { padding } else { padding_right };

    // Total width of non-auto properties
    let total = margin_left + margin_right + border_left + border_right +
                padding_left + padding_right;

    // If width is not auto, use it; otherwise fill the containing block
    let width = match width {
        PropertyValue::Length(w) => *w,
        _ => containing_block.content.width - total,
    };

    // Set dimensions
    layout_box.dimensions.content.width = width;

    layout_box.dimensions.padding.left = padding_left;
    layout_box.dimensions.padding.right = padding_right;

    layout_box.dimensions.border.left = border_left;
    layout_box.dimensions.border.right = border_right;

    layout_box.dimensions.margin.left = margin_left;
    layout_box.dimensions.margin.right = margin_right;
}

/// Calculate block box position
fn calculate_block_position(layout_box: &mut LayoutBox, containing_block: Dimensions) {
    // Get style values for vertical spacing
    let margin_top = layout_box.get_style_length("margin-top", 0);
    let margin_bottom = layout_box.get_style_length("margin-bottom", 0);

    let border_top = layout_box.get_style_length("border-top-width", 0);
    let border_bottom = layout_box.get_style_length("border-bottom-width", 0);

    let padding_top = layout_box.get_style_length("padding-top", 0);
    let padding_bottom = layout_box.get_style_length("padding-bottom", 0);

    // Try shorthand properties
    let margin = layout_box.get_style_length("margin", 0);
    let margin_top = if margin_top == 0 { margin } else { margin_top };
    let margin_bottom = if margin_bottom == 0 { margin } else { margin_bottom };

    let padding = layout_box.get_style_length("padding", 0);
    let padding_top = if padding_top == 0 { padding } else { padding_top };
    let padding_bottom = if padding_bottom == 0 { padding } else { padding_bottom };

    layout_box.dimensions.padding.top = padding_top;
    layout_box.dimensions.padding.bottom = padding_bottom;

    layout_box.dimensions.border.top = border_top;
    layout_box.dimensions.border.bottom = border_bottom;

    layout_box.dimensions.margin.top = margin_top;
    layout_box.dimensions.margin.bottom = margin_bottom;

    // Position the box below the previous boxes in the containing block
    layout_box.dimensions.content.x = containing_block.content.x +
                                       layout_box.dimensions.margin.left +
                                       layout_box.dimensions.border.left +
                                       layout_box.dimensions.padding.left;

    layout_box.dimensions.content.y = containing_block.content.y +
                                       containing_block.content.height +
                                       layout_box.dimensions.margin.top +
                                       layout_box.dimensions.border.top +
                                       layout_box.dimensions.padding.top;
}

/// Layout block children
fn layout_block_children(layout_box: &mut LayoutBox) {
    let mut height = 0;

    for child in &mut layout_box.children {
        // Use current dimensions as containing block for children
        let mut containing_block = layout_box.dimensions;
        containing_block.content.height = height;

        layout_tree(child, containing_block);

        // Accumulate child heights
        height += child.dimensions.margin_box_height();
    }

    layout_box.dimensions.content.height = height;
}

/// Calculate block box height
fn calculate_block_height(layout_box: &mut LayoutBox) {
    // If height is explicitly set, use it
    if let Some(PropertyValue::Length(h)) = layout_box.get_style_value("height") {
        layout_box.dimensions.content.height = *h;
    }
    // Otherwise height is already set by layout_block_children
}

/// Layout an inline box (simplified - just treat as block for now)
fn layout_inline(layout_box: &mut LayoutBox, containing_block: Dimensions) {
    // Simplified: treat inline as small block
    layout_box.dimensions.content.x = containing_block.content.x;
    layout_box.dimensions.content.y = containing_block.content.y + containing_block.content.height;

    // Get padding and margin
    let padding = layout_box.get_style_length("padding", 2);
    let margin = layout_box.get_style_length("margin", 2);

    layout_box.dimensions.padding = EdgeSizes::uniform(padding);
    layout_box.dimensions.margin = EdgeSizes::uniform(margin);

    // Inline boxes size to content (simplified)
    layout_box.dimensions.content.width = 50;  // Default width
    layout_box.dimensions.content.height = 20; // Default height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    #[test]
    fn test_box_type() {
        let style = ComputedStyle::new();
        let layout_box = LayoutBox::new(BoxType::Block, style);
        assert_eq!(layout_box.box_type, BoxType::Block);
    }

    #[test]
    fn test_layout_dimensions() {
        let mut style = ComputedStyle::new();
        style.set(String::from("width"), PropertyValue::Length(100));
        style.set(String::from("margin"), PropertyValue::Length(10));

        let mut layout_box = LayoutBox::new(BoxType::Block, style);

        let containing_block = Dimensions {
            content: Rect::new(0, 0, 800, 600),
            ..Dimensions::default()
        };

        calculate_block_width(&mut layout_box, containing_block);

        assert_eq!(layout_box.dimensions.content.width, 100);
        assert_eq!(layout_box.dimensions.margin.left, 10);
    }
}
