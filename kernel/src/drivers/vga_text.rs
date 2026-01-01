use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// 특정 위치에 문자 쓰기
    pub fn write_char_at(&mut self, x: usize, y: usize, byte: u8, fg: Color, bg: Color) {
        if x >= BUFFER_WIDTH || y >= BUFFER_HEIGHT {
            return;
        }
        let color_code = ColorCode::new(fg, bg);
        self.buffer.chars[y][x].write(ScreenChar {
            ascii_character: byte,
            color_code,
        });
    }

    /// 사각형 그리기 (테두리만)
    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color, ch: u8) {
        for i in 0..width {
            if x + i < BUFFER_WIDTH {
                if y < BUFFER_HEIGHT {
                    self.write_char_at(x + i, y, ch, fg, bg);
                }
                if y + height.saturating_sub(1) < BUFFER_HEIGHT {
                    self.write_char_at(x + i, y + height.saturating_sub(1), ch, fg, bg);
                }
            }
        }
        for i in 1..height.saturating_sub(1) {
            if y + i < BUFFER_HEIGHT {
                if x < BUFFER_WIDTH {
                    self.write_char_at(x, y + i, ch, fg, bg);
                }
                if x + width.saturating_sub(1) < BUFFER_WIDTH {
                    self.write_char_at(x + width.saturating_sub(1), y + i, ch, fg, bg);
                }
            }
        }
    }

    /// 채워진 사각형 그리기
    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color, ch: u8) {
        for dy in 0..height {
            for dx in 0..width {
                if x + dx < BUFFER_WIDTH && y + dy < BUFFER_HEIGHT {
                    self.write_char_at(x + dx, y + dy, ch, fg, bg);
                }
            }
        }
    }

    /// 특정 위치에 문자열 쓰기
    pub fn write_str_at(&mut self, x: usize, y: usize, s: &str, fg: Color, bg: Color) {
        let mut col = x;
        for byte in s.bytes() {
            if col >= BUFFER_WIDTH {
                break;
            }
            match byte {
                0x20..=0x7e => {
                    self.write_char_at(col, y, byte, fg, bg);
                    col += 1;
                }
                _ => {
                    self.write_char_at(col, y, 0xfe, fg, bg);
                    col += 1;
                }
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear_screen(&mut self, color: u32) {
        let background = match color {
            0x0000FF => Color::Blue,
            0x00FF00 => Color::Green,
            0xFF0000 => Color::Red,
            _ => Color::Black,
        };

        self.color_code = ColorCode::new(Color::White, background);
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub fn init() {
    // VGA 버퍼 초기화 (이미 lazy_static으로 초기화됨)
}

pub fn clear_screen(color: u32) {
    WRITER.lock().clear_screen(color);
}

/// 특정 위치에 문자열 그리기
pub fn draw_str(x: usize, y: usize, s: &str, fg: Color, bg: Color) {
    WRITER.lock().write_str_at(x, y, s, fg, bg);
}

/// 사각형 테두리 그리기
pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color) {
    WRITER.lock().draw_rect(x, y, width, height, fg, bg, b'#');
}

/// 채워진 사각형 그리기
pub fn fill_rect(x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color) {
    WRITER.lock().fill_rect(x, y, width, height, fg, bg, b' ');
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
