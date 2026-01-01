use crate::drivers::framebuffer::{Color, draw_str, fill_rect, draw_rect};
use crate::drivers::mouse::MouseState;

#[derive(Debug, Clone, Copy)]
pub struct Button {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub label: &'static str,
    pub fg_color: Color,
    pub bg_color: Color,
    pub border_color: Color,
    pub is_pressed: bool,
}

impl Button {
    pub const fn new(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        label: &'static str,
    ) -> Self {
        Button {
            x,
            y,
            width,
            height,
            label,
            fg_color: Color::Black,
            bg_color: Color::LightGray,
            border_color: Color::DarkGray,
            is_pressed: false,
        }
    }

    /// 버튼에 마우스가 있는지 확인
    pub fn contains(&self, mouse_x: i32, mouse_y: i32) -> bool {
        let mx = mouse_x as usize;
        let my = mouse_y as usize;
        mx >= self.x
            && mx < self.x + self.width
            && my >= self.y
            && my < self.y + self.height
    }

    /// 버튼 그리기
    pub fn draw(&self) {
        // 배경 색상 (눌렸을 때 다르게 표시)
        let bg = if self.is_pressed {
            Color::DarkGray
        } else {
            self.bg_color
        };

        // 버튼 배경 채우기
        fill_rect(self.x, self.y, self.width, self.height, self.fg_color, bg);

        // 버튼 테두리 그리기
        draw_rect(
            self.x,
            self.y,
            self.width,
            self.height,
            self.border_color,
            bg,
        );

        // 라벨 중앙에 그리기
        let label_len = self.label.len();
        let label_x = self.x + (self.width.saturating_sub(label_len)) / 2;
        let label_y = self.y + self.height / 2;

        draw_str(label_x, label_y, self.label, self.fg_color, bg);
    }

    /// 마우스 상태에 따라 버튼 업데이트
    pub fn update(&mut self, mouse_state: &MouseState) -> bool {
        let was_pressed = self.is_pressed;
        let contains_mouse = self.contains(mouse_state.x, mouse_state.y);

        self.is_pressed = contains_mouse && mouse_state.left_button;

        // 버튼이 눌렸다가 떼졌을 때 클릭으로 간주
        was_pressed && !self.is_pressed && contains_mouse
    }
}
