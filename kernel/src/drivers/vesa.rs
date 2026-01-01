// VESA VBE (VESA BIOS Extensions) graphics driver
// Provides higher resolution graphics modes beyond VGA Mode 13h

use x86_64::instructions::port::Port;

/// VESA VBE mode information
#[derive(Debug, Clone, Copy)]
pub struct VesaModeInfo {
    pub width: u16,
    pub height: u16,
    pub bpp: u8,  // bits per pixel
    pub framebuffer_addr: u64,
    pub pitch: u16,  // bytes per scanline
}

/// Common VESA modes
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum VesaMode {
    Mode640x480x16 = 0x111,
    Mode800x600x16 = 0x114,
    Mode1024x768x16 = 0x117,
    Mode1280x1024x16 = 0x11A,
}

/// VESA VBE Driver
pub struct VesaDriver {
    mode_info: Option<VesaModeInfo>,
}

impl VesaDriver {
    pub const fn new() -> Self {
        VesaDriver { mode_info: None }
    }

    /// Initialize VESA VBE mode
    /// Note: This requires BIOS calls which are only available during boot
    /// For now, we'll use a pre-configured mode or rely on bootloader
    pub fn init(&mut self, mode: VesaMode) -> Result<(), &'static str> {
        // In a real implementation, this would use BIOS INT 10h to:
        // 1. Get VBE controller info (AX=0x4F00)
        // 2. Get mode info (AX=0x4F01)
        // 3. Set VBE mode (AX=0x4F02)

        // For now, return an error indicating BIOS calls are not available
        // In practice, the bootloader should set up the graphics mode
        Err("VESA VBE initialization requires bootloader support")
    }

    /// Set mode info (to be called by bootloader integration)
    pub fn set_mode_info(&mut self, info: VesaModeInfo) {
        self.mode_info = Some(info);
    }

    /// Get current mode info
    pub fn get_mode_info(&self) -> Option<&VesaModeInfo> {
        self.mode_info.as_ref()
    }

    /// Write pixel at (x, y) with color (assuming 8-bit indexed color for now)
    pub fn put_pixel(&self, framebuffer: &mut [u8], x: u16, y: u16, color: u8) {
        if let Some(info) = &self.mode_info {
            if x < info.width && y < info.height {
                let offset = (y as usize * info.pitch as usize) + x as usize;
                if offset < framebuffer.len() {
                    framebuffer[offset] = color;
                }
            }
        }
    }

    /// Fill rectangle with color
    pub fn fill_rect(&self, framebuffer: &mut [u8], x: u16, y: u16, width: u16, height: u16, color: u8) {
        for dy in 0..height {
            for dx in 0..width {
                self.put_pixel(framebuffer, x + dx, y + dy, color);
            }
        }
    }

    /// Clear screen with color
    pub fn clear_screen(&self, framebuffer: &mut [u8], color: u8) {
        if let Some(info) = &self.mode_info {
            self.fill_rect(framebuffer, 0, 0, info.width, info.height, color);
        }
    }
}

/// Global VESA driver instance (will be initialized later)
pub static mut VESA_DRIVER: VesaDriver = VesaDriver::new();

/// Initialize VESA driver with mode info from bootloader
pub unsafe fn init_with_mode_info(info: VesaModeInfo) {
    VESA_DRIVER.set_mode_info(info);
    crate::serial_println!("VESA VBE initialized: {}x{}x{} @ {:#x}",
        info.width, info.height, info.bpp, info.framebuffer_addr);
}
