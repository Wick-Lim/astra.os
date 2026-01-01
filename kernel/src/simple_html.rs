// Simple no_std HTML parser for ASTRA.OS
// This is a minimal parser for demonstration
// Will be replaced with Servo later when std is available

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::boxed::Box;

#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    Element {
        tag: String,
        children: Vec<Box<Node>>,
    },
}

/// Parse simple HTML (very basic implementation)
pub fn parse_html(html: &str) -> Vec<Box<Node>> {
    let mut nodes = Vec::new();
    let mut current_pos = 0;
    let bytes = html.as_bytes();

    while current_pos < bytes.len() {
        if bytes[current_pos] == b'<' {
            // Parse tag
            if let Some((tag, end_pos)) = parse_tag(html, current_pos) {
                nodes.push(Box::new(Node::Element {
                    tag,
                    children: Vec::new(),
                }));
                current_pos = end_pos;
            } else {
                current_pos += 1;
            }
        } else {
            // Parse text
            if let Some((text, end_pos)) = parse_text(html, current_pos) {
                if !text.is_empty() {
                    nodes.push(Box::new(Node::Text(text)));
                }
                current_pos = end_pos;
            } else {
                current_pos += 1;
            }
        }
    }

    nodes
}

fn parse_tag(html: &str, start: usize) -> Option<(String, usize)> {
    let bytes = html.as_bytes();
    if bytes[start] != b'<' {
        return None;
    }

    let mut pos = start + 1;

    // Skip closing tags
    if pos < bytes.len() && bytes[pos] == b'/' {
        while pos < bytes.len() && bytes[pos] != b'>' {
            pos += 1;
        }
        return Some((String::new(), pos + 1));
    }

    // Find tag name
    let tag_start = pos;
    while pos < bytes.len() && bytes[pos] != b'>' && bytes[pos] != b' ' {
        pos += 1;
    }

    let tag_name = core::str::from_utf8(&bytes[tag_start..pos])
        .unwrap_or("")
        .to_string();

    // Skip to end of tag
    while pos < bytes.len() && bytes[pos] != b'>' {
        pos += 1;
    }

    if pos < bytes.len() {
        Some((tag_name, pos + 1))
    } else {
        None
    }
}

fn parse_text(html: &str, start: usize) -> Option<(String, usize)> {
    let bytes = html.as_bytes();
    let mut pos = start;

    while pos < bytes.len() && bytes[pos] != b'<' {
        pos += 1;
    }

    let text = core::str::from_utf8(&bytes[start..pos])
        .unwrap_or("")
        .trim()
        .to_string();

    Some((text, pos))
}

/// Render HTML to text output (via syscalls)
pub fn render_html(nodes: &[Box<Node>], depth: usize) {
    for node in nodes {
        render_node(node, depth);
    }
}

fn render_node(node: &Node, depth: usize) {
    match node {
        Node::Text(text) => {
            if !text.is_empty() {
                // Syscall to print text
                syscall_write(1, text.as_bytes());
                syscall_write(1, b"\n");
            }
        }
        Node::Element { tag, children } => {
            if !tag.is_empty() {
                // Print indentation
                for _ in 0..depth {
                    syscall_write(1, b"  ");
                }

                syscall_write(1, b"<");
                syscall_write(1, tag.as_bytes());
                syscall_write(1, b">\n");

                // Render children
                render_html(children, depth + 1);
            }
        }
    }
}

fn syscall_write(fd: i32, message: &[u8]) {
    unsafe {
        core::arch::asm!(
            "mov rax, 1",
            "int 0x80",
            in("rdi") fd as u64,
            in("rsi") message.as_ptr(),
            in("rdx") message.len(),
            lateout("rax") _,
        );
    }
}
