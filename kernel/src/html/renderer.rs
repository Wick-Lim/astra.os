// Simple HTML renderer for ASTRA.OS
// Renders HTML to VGA framebuffer

use super::Node;
use crate::drivers::framebuffer::{fill_rect, draw_pixel, WIDTH, HEIGHT};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use alloc::string::String;

pub struct Renderer {
    x: usize,
    y: usize,
    line_height: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            x: 10,
            y: 10,
            line_height: 16,
        }
    }

    pub fn render(&mut self, node: &Node) {
        match node {
            Node::Element { tag, children } => {
                match tag.as_str() {
                    "h1" => {
                        self.render_heading(children, Rgb888::new(0, 200, 255), 2);
                    }
                    "h2" => {
                        self.render_heading(children, Rgb888::new(100, 200, 255), 1);
                    }
                    "p" => {
                        self.render_paragraph(children);
                    }
                    "div" | "html" | "body" => {
                        for child in children {
                            self.render(child);
                        }
                    }
                    _ => {
                        for child in children {
                            self.render(child);
                        }
                    }
                }
            }
            Node::Text(text) => {
                self.render_text(text, Rgb888::WHITE);
            }
        }
    }

    fn render_heading(&mut self, children: &[Node], color: Rgb888, size: usize) {
        // Add some vertical space before heading
        self.y += self.line_height;

        // Draw background bar
        fill_rect(0, self.y, WIDTH, self.line_height * size, Rgb888::new(20, 20, 60));

        self.y += 5;
        self.x = 20;

        for child in children {
            match child {
                Node::Text(text) => {
                    self.render_text(text, color);
                }
                _ => self.render(child),
            }
        }

        self.y += self.line_height * size + 5;
        self.x = 10;
    }

    fn render_paragraph(&mut self, children: &[Node]) {
        self.y += 10;
        self.x = 20;

        for child in children {
            self.render(child);
        }

        self.y += self.line_height;
        self.x = 10;
    }

    fn render_text(&mut self, text: &str, color: Rgb888) {
        // Simple 8x8 pixel character rendering
        for ch in text.chars() {
            if ch == '\n' || self.x + 8 > WIDTH {
                self.x = 20;
                self.y += self.line_height;
            }

            if self.y + 8 > HEIGHT {
                return; // Out of screen space
            }

            if ch != '\n' && ch != '\r' {
                self.render_char(ch, self.x, self.y, color);
                self.x += 8;
            }
        }
    }

    fn render_char(&self, ch: char, x: usize, y: usize, color: Rgb888) {
        // Ultra-simple ASCII rendering
        // Just draw a rectangle for now (will improve later)
        let font_data = get_simple_font(ch);

        for row in 0..8 {
            for col in 0..8 {
                if font_data[row] & (1 << (7 - col)) != 0 {
                    draw_pixel(x + col, y + row, color);
                }
            }
        }
    }
}

// Ultra-simple font data (just a few characters)
fn get_simple_font(ch: char) -> [u8; 8] {
    match ch {
        'A' => [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00],
        'S' => [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00],
        'T' => [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
        'R' => [0x7C, 0x66, 0x66, 0x7C, 0x78, 0x6C, 0x66, 0x00],
        'O' => [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
        'H' => [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
        'e' => [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0x00],
        'l' => [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00],
        'o' => [0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0x00],
        'W' => [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00],
        'r' => [0x00, 0x00, 0x7C, 0x66, 0x60, 0x60, 0x60, 0x00],
        'd' => [0x06, 0x06, 0x06, 0x3E, 0x66, 0x66, 0x3E, 0x00],
        '!' => [0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        _   => [0xFF, 0x81, 0x81, 0x81, 0x81, 0x81, 0xFF, 0x00], // Unknown char
    }
}
