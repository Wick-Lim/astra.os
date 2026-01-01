// sys backend for ASTRA.OS

#![forbid(unsafe_op_in_unsafe_fn)]

use crate::io as std_io;

pub mod alloc;
pub mod args;
pub mod env;
pub mod fs;
pub mod os;
pub mod path;
pub mod process;
pub mod random;
pub mod stdio;
pub mod thread;
pub mod time;

// Note: We don't use common::alloc, we have our own in sys/alloc/astra_os.rs

#[inline]
pub const fn unsupported<T>() -> std_io::Result<T> {
    Err(unsupported_err())
}

#[inline]
pub const fn unsupported_err() -> std_io::Error {
    std_io::const_error!(std_io::ErrorKind::Unsupported, "operation not supported on ASTRA.OS")
}

pub fn decode_error_kind(_code: i32) -> std_io::ErrorKind {
    std_io::ErrorKind::Uncategorized
}

pub fn abort_internal() -> ! {
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt", options(noreturn, nomem, nostack));
        }
    }
}

#[cfg(not(test))]
pub fn init(_argc: isize, _argv: *const *const u8, _sigpipe: u8) {}

#[cfg(not(test))]
pub unsafe fn cleanup() {}

pub fn hashmap_random_keys() -> (u64, u64) {
    let mut buf = [0u8; 16];
    random::fill_bytes(&mut buf);
    let k1 = u64::from_ne_bytes(buf[0..8].try_into().unwrap());
    let k2 = u64::from_ne_bytes(buf[8..16].try_into().unwrap());
    (k1, k2)
}
