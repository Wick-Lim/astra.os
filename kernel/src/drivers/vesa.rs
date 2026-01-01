// VESA VBE (VESA BIOS Extensions) Driver
// Provides higher resolution graphics modes (640x480, 800x600, 1024x768, etc.)

/// VBE mode information structure
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct VbeModeInfo {
    pub attributes: u16,
    pub window_a: u8,
    pub window_b: u8,
    pub granularity: u16,
    pub window_size: u16,
    pub segment_a: u16,
    pub segment_b: u16,
    pub win_func_ptr: u32,
    pub pitch: u16,
    pub width: u16,
    pub height: u16,
    pub w_char: u8,
    pub y_char: u8,
    pub planes: u8,
    pub bpp: u8,
    pub banks: u8,
    pub memory_model: u8,
    pub bank_size: u8,
    pub image_pages: u8,
    pub reserved0: u8,
    pub red_mask: u8,
    pub red_position: u8,
    pub green_mask: u8,
    pub green_position: u8,
    pub blue_mask: u8,
    pub blue_position: u8,
    pub reserved_mask: u8,
    pub reserved_position: u8,
    pub direct_color_attributes: u8,
    pub framebuffer: u32,
    pub off_screen_mem_off: u32,
    pub off_screen_mem_size: u16,
    pub reserved1: [u8; 206],
}

/// Common VESA modes
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum VesaMode {
    Mode640x480x16 = 0x111,   // 640x480, 16 colors
    Mode640x480x256 = 0x101,  // 640x480, 256 colors
    Mode800x600x16 = 0x113,   // 800x600, 16 colors
    Mode800x600x256 = 0x103,  // 800x600, 256 colors
    Mode1024x768x16 = 0x115,  // 1024x768, 16 colors
    Mode1024x768x256 = 0x105, // 1024x768, 256 colors
    Mode640x480x32K = 0x110,  // 640x480, 32K colors (15-bit)
    Mode640x480x64K = 0x111,  // 640x480, 64K colors (16-bit)
    Mode640x480x16M = 0x112,  // 640x480, 16M colors (24-bit)
}

pub struct VesaDriver {
    pub mode_info: Option<VbeModeInfo>,
    pub framebuffer: Option<*mut u8>,
    pub width: u16,
    pub height: u16,
    pub bpp: u8,
    pub pitch: u16,
}

impl VesaDriver {
    pub const fn new() -> Self {
        VesaDriver {
            mode_info: None,
            framebuffer: None,
            width: 0,
            height: 0,
            bpp: 0,
            pitch: 0,
        }
    }

    /// Initialize VESA mode using BIOS int 0x10
    /// NOTE: This must be called BEFORE switching to protected/long mode
    /// For now, we'll document the interface and provide stub
    pub unsafe fn init(&mut self, mode: VesaMode) -> Result<(), &'static str> {
        // TODO: This needs to be called from bootloader or early init
        // VESA BIOS calls require real mode (16-bit)
        // Options:
        // 1. Use bootloader to set VESA mode before entering kernel
        // 2. Use multiboot2 framebuffer info
        // 3. Write real-mode stub that switches back temporarily

        crate::serial_println!("[VESA] Stub: Would set mode {:?}", mode);
        crate::serial_println!("[VESA] Note: VESA mode setting requires bootloader support");

        Err("VESA mode setting not implemented - needs bootloader support")
    }

    /// Set pixel at (x, y) with RGB color
    pub unsafe fn put_pixel(&mut self, x: u16, y: u16, r: u8, g: u8, b: u8) {
        if let Some(fb) = self.framebuffer {
            if x >= self.width || y >= self.height {
                return;
            }

            let offset = (y as usize * self.pitch as usize) + (x as usize * (self.bpp as usize / 8));
            let pixel_ptr = fb.add(offset);

            match self.bpp {
                8 => {
                    // 256-color mode - convert RGB to palette index
                    let color = ((r >> 5) << 5) | ((g >> 5) << 2) | (b >> 6);
                    *pixel_ptr = color;
                }
                16 => {
                    // 16-bit RGB (5-6-5)
                    let color = ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3);
                    *(pixel_ptr as *mut u16) = color;
                }
                24 => {
                    // 24-bit RGB
                    *pixel_ptr = b;
                    *pixel_ptr.add(1) = g;
                    *pixel_ptr.add(2) = r;
                }
                32 => {
                    // 32-bit RGBA (or RGBX)
                    *pixel_ptr = b;
                    *pixel_ptr.add(1) = g;
                    *pixel_ptr.add(2) = r;
                    *pixel_ptr.add(3) = 0xFF; // Alpha
                }
                _ => {}
            }
        }
    }

    /// Fill rectangle with color
    pub unsafe fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, r: u8, g: u8, b: u8) {
        for dy in 0..height {
            for dx in 0..width {
                self.put_pixel(x + dx, y + dy, r, g, b);
            }
        }
    }

    /// Clear screen with color
    pub unsafe fn clear(&mut self, r: u8, g: u8, b: u8) {
        self.fill_rect(0, 0, self.width, self.height, r, g, b);
    }
}

static mut VESA: VesaDriver = VesaDriver::new();

/// Initialize VESA driver
pub fn init() {
    crate::serial_println!("[VESA] Driver initialized (stub mode)");
    crate::serial_println!("[VESA] Real VESA requires bootloader framebuffer support");
    crate::serial_println!("[VESA] Alternative: Use bootloader crate's framebuffer feature");
}

/// Get mutable reference to VESA driver
pub fn get_driver() -> &'static mut VesaDriver {
    unsafe { &mut VESA }
}
