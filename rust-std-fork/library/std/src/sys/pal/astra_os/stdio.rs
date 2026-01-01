// stdio stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/stdio.rs

use crate::io::{self, IoSlice, IoSliceMut};

// External functions provided by kernel
unsafe extern "C" {
    fn astra_os_serial_write(data: *const u8, len: usize);
    fn astra_os_serial_read(data: *mut u8, len: usize) -> usize;
}

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // For now, return empty (no input)
        // TODO: Integrate with keyboard driver
        Ok(0)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Write to serial port
        unsafe {
            astra_os_serial_write(buf.as_ptr(), buf.len());
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut total = 0;
        for buf in bufs {
            total += self.write(buf)?;
        }
        Ok(total)
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Also write to serial port (same as stdout)
        unsafe {
            astra_os_serial_write(buf.as_ptr(), buf.len());
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut total = 0;
        for buf in bufs {
            total += self.write(buf)?;
        }
        Ok(total)
    }
}

pub const STDIN_BUF_SIZE: usize = 0; // No buffering for stdin
pub const STDOUT_BUF_SIZE: usize = 0; // No buffering for stdout
pub const STDERR_BUF_SIZE: usize = 0; // No buffering for stderr

pub fn is_ebadf(_err: &io::Error) -> bool {
    false
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
