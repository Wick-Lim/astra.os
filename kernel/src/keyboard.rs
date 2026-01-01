// Keyboard input buffer for userspace
// Stores keyboard input from interrupts and provides it to sys_read

use spin::Mutex;
use lazy_static::lazy_static;

/// Circular buffer for keyboard input
const BUFFER_SIZE: usize = 256;

pub struct KeyboardBuffer {
    buffer: [u8; BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

impl KeyboardBuffer {
    const fn new() -> Self {
        KeyboardBuffer {
            buffer: [0; BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }
    
    /// Push a character into the buffer
    pub fn push(&mut self, ch: u8) -> bool {
        if self.count >= BUFFER_SIZE {
            return false; // Buffer full
        }
        
        self.buffer[self.write_pos] = ch;
        self.write_pos = (self.write_pos + 1) % BUFFER_SIZE;
        self.count += 1;
        true
    }
    
    /// Pop a character from the buffer
    pub fn pop(&mut self) -> Option<u8> {
        if self.count == 0 {
            return None;
        }
        
        let ch = self.buffer[self.read_pos];
        self.read_pos = (self.read_pos + 1) % BUFFER_SIZE;
        self.count -= 1;
        Some(ch)
    }
    
    /// Read multiple characters into a buffer
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut bytes_read = 0;
        
        for i in 0..buf.len() {
            if let Some(ch) = self.pop() {
                buf[i] = ch;
                bytes_read += 1;
            } else {
                break;
            }
        }
        
        bytes_read
    }
    
    /// Check if buffer has data
    pub fn has_data(&self) -> bool {
        self.count > 0
    }
    
    /// Get number of bytes available
    pub fn available(&self) -> usize {
        self.count
    }
}

lazy_static! {
    pub static ref KEYBOARD_BUFFER: Mutex<KeyboardBuffer> = Mutex::new(KeyboardBuffer::new());
}

/// Scancode to ASCII conversion table (US keyboard, basic layout)
/// This is a simplified version - only supports basic ASCII characters
const SCANCODE_TO_ASCII: [u8; 128] = [
    0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', b'\x08', // 0x00-0x0E
    b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', b'\n', // 0x0F-0x1C
    0, // 0x1D - Left Ctrl
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', // 0x1E-0x29
    0, // 0x2A - Left Shift
    b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', // 0x2B-0x35
    0, // 0x36 - Right Shift
    b'*', // 0x37 - Keypad *
    0, // 0x38 - Left Alt
    b' ', // 0x39 - Space
    0, // 0x3A - Caps Lock
    // F1-F10
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x3B-0x44
    0, // 0x45 - Num Lock
    0, // 0x46 - Scroll Lock
    b'7', b'8', b'9', b'-', // 0x47-0x4A - Keypad
    b'4', b'5', b'6', b'+', // 0x4B-0x4E - Keypad
    b'1', b'2', b'3', b'0', b'.', // 0x4F-0x53 - Keypad
    // Rest are 0
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Added 2 more zeros
];

/// Convert scancode to ASCII (called from interrupt handler)
pub fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    if scancode >= 0x80 {
        // Key release event - ignore
        return None;
    }
    
    let ascii = SCANCODE_TO_ASCII[scancode as usize];
    if ascii == 0 {
        None
    } else {
        Some(ascii)
    }
}

/// Push scancode to keyboard buffer (called from interrupt)
pub fn push_scancode(scancode: u8) {
    if let Some(ascii) = scancode_to_ascii(scancode) {
        let mut buffer = KEYBOARD_BUFFER.lock();
        if !buffer.push(ascii) {
            // Buffer full - drop character
            crate::serial_println!("[KEYBOARD] Buffer full, dropping character");
        }
    }
}
