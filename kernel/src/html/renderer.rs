// HTML Rendering Engine for ASTRA.OS
// Converts DOM tree to screen output using font rendering

use alloc::string::String;
use alloc::vec::Vec;
use super::{Node, NodeType, ElementData};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};

/// Rendering context with screen position and styling
pub struct RenderContext {
    pub x: usize,
    pub y: usize,
    pub max_width: usize,
    pub line_height: usize,
    pub default_color: Rgb888,
}

impl RenderContext {
    pub fn new(max_width: usize) -> Self {
        RenderContext {
            x: 10,
            y: 10,
            max_width,
            line_height: 10,
            default_color: Rgb888::WHITE,
        }
    }
    
    /// Move to next line
    pub fn newline(&mut self) {
        self.x = 10;
        self.y += self.line_height;
    }
    
    /// Add vertical space
    pub fn add_spacing(&mut self, pixels: usize) {
        self.y += pixels;
    }
}

/// Render a DOM node to the framebuffer
pub fn render_node(node: &Node, ctx: &mut RenderContext) {
    match &node.node_type {
        NodeType::Text(text) => {
            render_text(text, ctx);
        }
        NodeType::Element(element) => {
            render_element(element, &node.children, ctx);
        }
        NodeType::Comment(_) => {
            // Comments are not rendered
        }
    }
}

/// Render text content
fn render_text(text: &str, ctx: &mut RenderContext) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    
    // Use framebuffer's draw_string function
    let end_x = crate::drivers::framebuffer::draw_string(
        trimmed, 
        ctx.x, 
        ctx.y, 
        ctx.default_color
    );
    
    ctx.x = end_x + 8; // Add space after text
    
    // Check if we need to wrap to next line
    if ctx.x >= ctx.max_width {
        ctx.newline();
    }
}

/// Render an HTML element
fn render_element(element: &ElementData, children: &[alloc::boxed::Box<Node>], ctx: &mut RenderContext) {
    match element.tag_name.as_str() {
        "h1" => {
            ctx.newline();
            ctx.add_spacing(5);
            // H1 is just larger spacing for now
            for child in children {
                render_node(child, ctx);
            }
            ctx.newline();
            ctx.add_spacing(5);
        }
        "h2" => {
            ctx.newline();
            ctx.add_spacing(3);
            for child in children {
                render_node(child, ctx);
            }
            ctx.newline();
            ctx.add_spacing(3);
        }
        "p" => {
            ctx.newline();
            for child in children {
                render_node(child, ctx);
            }
            ctx.newline();
        }
        "br" => {
            ctx.newline();
        }
        "div" => {
            ctx.newline();
            for child in children {
                render_node(child, ctx);
            }
            ctx.newline();
        }
        "span" => {
            // Inline element
            for child in children {
                render_node(child, ctx);
            }
        }
        "html" | "body" | "head" => {
            // Container elements - just render children
            for child in children {
                render_node(child, ctx);
            }
        }
        "title" => {
            // Title is not rendered (would go to window title)
        }
        _ => {
            // Unknown elements - render children anyway
            for child in children {
                render_node(child, ctx);
            }
        }
    }
}

/// Render entire HTML document
pub fn render_html(root: &Node, screen_width: usize) {
    // Clear screen first
    crate::drivers::framebuffer::clear_screen(Rgb888::BLACK);
    
    // Create rendering context
    let mut ctx = RenderContext::new(screen_width - 20); // 10px margin on each side
    
    // Render the DOM tree
    render_node(root, &mut ctx);
}

/// Render simple HTML string directly
pub fn render_html_string(html: &str, screen_width: usize) {
    use crate::html::parse_html;
    use alloc::string::ToString;
    
    let dom = parse_html(html.to_string());
    render_html(&dom, screen_width);
}
