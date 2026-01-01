// path.rs - Path handling for ASTRA.OS
use crate::ffi::OsStr;
use crate::path::Path;

pub const MAIN_SEP_STR: &str = "/";
pub const MAIN_SEP: char = '/';

pub fn is_sep_byte(b: u8) -> bool {
    b == b'/'
}

pub fn is_verbatim_sep(b: u8) -> bool {
    b == b'/'
}

pub fn parse_prefix(_path: &OsStr) -> Option<!> {
    None
}
