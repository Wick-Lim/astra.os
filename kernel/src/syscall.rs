// System call interface for ASTRA.OS
// Provides communication between kernel and userspace

use x86_64::VirtAddr;

/// System call numbers
#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum SyscallNumber {
    Exit = 0,
    Write = 1,
    Read = 2,
    Open = 3,
    Close = 4,
    Mmap = 5,
    Munmap = 6,
    // Graphics/Display
    DrawPixel = 100,
    DrawRect = 101,
    Flush = 102,
}

impl SyscallNumber {
    pub fn from_u64(n: u64) -> Option<Self> {
        match n {
            0 => Some(Self::Exit),
            1 => Some(Self::Write),
            2 => Some(Self::Read),
            3 => Some(Self::Open),
            4 => Some(Self::Close),
            5 => Some(Self::Mmap),
            6 => Some(Self::Munmap),
            100 => Some(Self::DrawPixel),
            101 => Some(Self::DrawRect),
            102 => Some(Self::Flush),
            _ => None,
        }
    }
}

/// System call handler
/// Called from syscall interrupt handler
pub fn handle_syscall(
    syscall_num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> u64 {
    let syscall = match SyscallNumber::from_u64(syscall_num) {
        Some(s) => s,
        None => return u64::MAX, // Invalid syscall
    };

    match syscall {
        SyscallNumber::Exit => {
            syscall_exit(arg1 as i32)
        }
        SyscallNumber::Write => {
            syscall_write(arg1 as i32, arg2 as *const u8, arg3 as usize)
        }
        SyscallNumber::Read => {
            syscall_read(arg1 as i32, arg2 as *mut u8, arg3 as usize)
        }
        SyscallNumber::DrawPixel => {
            syscall_draw_pixel(arg1 as u32, arg2 as u32, arg3 as u32)
        }
        SyscallNumber::DrawRect => {
            syscall_draw_rect(arg1 as u32, arg2 as u32, arg3 as u32, arg4 as u32, arg5 as u32)
        }
        SyscallNumber::Flush => {
            syscall_flush()
        }
        _ => u64::MAX, // Not implemented
    }
}

// System call implementations

fn syscall_exit(code: i32) -> u64 {
    use crate::process::SCHEDULER;

    serial_println!("Process exiting with code {}", code);

    let mut scheduler = SCHEDULER.lock();
    if let Some(process) = scheduler.current_process_mut() {
        process.state = crate::process::ProcessState::Terminated;
    }

    0
}

fn syscall_write(fd: i32, buf: *const u8, count: usize) -> u64 {
    // For now, only support stdout (fd=1) and stderr (fd=2)
    if fd != 1 && fd != 2 {
        return u64::MAX;
    }

    // Safety: Validate buffer is in userspace memory
    // TODO: Add proper memory validation
    if buf.is_null() || count == 0 {
        return 0;
    }

    unsafe {
        let slice = core::slice::from_raw_parts(buf, count);
        if let Ok(s) = core::str::from_utf8(slice) {
            serial_print!("{}", s);
            count as u64
        } else {
            // Non-UTF8 data, print as bytes
            for &byte in slice {
                serial_print!("{}", byte as char);
            }
            count as u64
        }
    }
}

fn syscall_read(fd: i32, buf: *mut u8, count: usize) -> u64 {
    // Not implemented yet
    0
}

fn syscall_draw_pixel(x: u32, y: u32, color: u32) -> u64 {
    // TODO: Call VGA driver to draw pixel
    // This will interface with the graphics system
    0
}

fn syscall_draw_rect(x: u32, y: u32, width: u32, height: u32, color: u32) -> u64 {
    // TODO: Call VGA driver to draw rectangle
    0
}

fn syscall_flush() -> u64 {
    // TODO: Flush framebuffer to display
    0
}

use crate::serial_print;
use crate::serial_println;
