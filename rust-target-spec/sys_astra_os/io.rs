// IO stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/io.rs

use crate::io::ErrorKind;

pub const STDIN_BUF_SIZE: usize = 0;

pub fn decode_error_kind(errno: i32) -> ErrorKind {
    match errno {
        1 => ErrorKind::PermissionDenied,
        2 => ErrorKind::NotFound,
        11 => ErrorKind::WouldBlock,
        32 => ErrorKind::BrokenPipe,
        _ => ErrorKind::Uncategorized,
    }
}

pub fn error_string(_errno: i32) -> String {
    "OS error".to_string()
}

pub const ERROR_NO_MEM: i32 = 12;
