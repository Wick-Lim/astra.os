// CSS Box Model Implementation
// Defines dimensions, rectangles, and edge sizes for layout calculation

/// Rectangle with position and size
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect { x, y, width, height }
    }

    /// Get the bottom-right corner
    pub fn bottom_right(&self) -> (i32, i32) {
        (self.x + self.width, self.y + self.height)
    }

    /// Check if a point is inside the rectangle
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }
}

/// Edge sizes (for margin, border, padding)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EdgeSizes {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl EdgeSizes {
    pub fn new(top: i32, right: i32, bottom: i32, left: i32) -> Self {
        EdgeSizes { left, right, top, bottom }
    }

    pub fn uniform(size: i32) -> Self {
        EdgeSizes {
            left: size,
            right: size,
            top: size,
            bottom: size,
        }
    }

    /// Total horizontal space (left + right)
    pub fn horizontal(&self) -> i32 {
        self.left + self.right
    }

    /// Total vertical space (top + bottom)
    pub fn vertical(&self) -> i32 {
        self.top + self.bottom
    }
}

/// CSS Box Model Dimensions
///
/// ```
/// +--margin--+
/// | +border+ |
/// | |+pad+ | |
/// | ||con|| |
/// | |+---+ | |
/// | +-----+ |
/// +---------+
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Dimensions {
    /// Position of content area relative to document origin
    pub content: Rect,

    /// Surrounding edges
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

impl Dimensions {
    pub fn new() -> Self {
        Dimensions::default()
    }

    /// Total box width including padding and border (but not margin)
    pub fn border_box_width(&self) -> i32 {
        self.content.width + self.padding.horizontal() + self.border.horizontal()
    }

    /// Total box height including padding and border (but not margin)
    pub fn border_box_height(&self) -> i32 {
        self.content.height + self.padding.vertical() + self.border.vertical()
    }

    /// Total box width including margin
    pub fn margin_box_width(&self) -> i32 {
        self.border_box_width() + self.margin.horizontal()
    }

    /// Total box height including margin
    pub fn margin_box_height(&self) -> i32 {
        self.border_box_height() + self.margin.vertical()
    }

    /// Get the padding box rectangle
    pub fn padding_box(&self) -> Rect {
        Rect {
            x: self.content.x - self.padding.left,
            y: self.content.y - self.padding.top,
            width: self.content.width + self.padding.horizontal(),
            height: self.content.height + self.padding.vertical(),
        }
    }

    /// Get the border box rectangle
    pub fn border_box(&self) -> Rect {
        let padding_box = self.padding_box();
        Rect {
            x: padding_box.x - self.border.left,
            y: padding_box.y - self.border.top,
            width: padding_box.width + self.border.horizontal(),
            height: padding_box.height + self.border.vertical(),
        }
    }

    /// Get the margin box rectangle
    pub fn margin_box(&self) -> Rect {
        let border_box = self.border_box();
        Rect {
            x: border_box.x - self.margin.left,
            y: border_box.y - self.margin.top,
            width: border_box.width + self.margin.horizontal(),
            height: border_box.height + self.margin.vertical(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 100, 50);
        assert!(rect.contains(50, 30));
        assert!(!rect.contains(5, 30));
        assert!(!rect.contains(50, 5));
    }

    #[test]
    fn test_edge_sizes() {
        let edges = EdgeSizes::uniform(10);
        assert_eq!(edges.horizontal(), 20);
        assert_eq!(edges.vertical(), 20);
    }

    #[test]
    fn test_dimensions() {
        let mut dims = Dimensions::new();
        dims.content = Rect::new(0, 0, 100, 50);
        dims.padding = EdgeSizes::uniform(5);
        dims.border = EdgeSizes::uniform(2);
        dims.margin = EdgeSizes::uniform(10);

        assert_eq!(dims.border_box_width(), 100 + 10 + 4); // content + padding + border
        assert_eq!(dims.margin_box_width(), 100 + 10 + 4 + 20); // + margin
    }
}
