// sys backend for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/mod.rs

pub mod alloc;
pub mod args;
pub mod cmath;
pub mod env;
pub mod fs;
pub mod io;
pub mod locks;
pub mod net;
pub mod os;
pub mod os_str;
pub mod path;
pub mod pipe;
pub mod process;
pub mod stdio;
pub mod thread;
pub mod thread_local_key;
pub mod time;

// Unsupported modules (stub with errors)
pub mod condvar;
pub mod memchr;
pub mod mutex;
pub mod rwlock;

// Required for std to compile
pub use crate::sys_common::os_str_bytes as os_str_bytes;

// Important: Define unsupported! macro for unimplemented features
#[macro_export]
macro_rules! unsupported {
    () => {
        return Err(io::const_io_error!(
            io::ErrorKind::Unsupported,
            "operation not supported on ASTRA.OS",
        ))
    };
}

// Export unsupported macro
pub(crate) use unsupported;
