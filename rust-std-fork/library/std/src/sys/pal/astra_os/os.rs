// os.rs - OS constants and functions for ASTRA.OS
use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::io;
use crate::path::{Path, PathBuf};

pub const FAMILY: &str = "unix";
pub const OS: &str = "astra_os";
pub const DLL_PREFIX: &str = "lib";
pub const DLL_SUFFIX: &str = ".so";
pub const DLL_EXTENSION: &str = "so";
pub const EXE_SUFFIX: &str = "";
pub const EXE_EXTENSION: &str = "";

// Re-export commonly needed functions from other modules
pub use super::env::{getenv, getpid, current_exe, exit};
pub use super::fs::{getcwd, chdir};

// Path splitting/joining - simplified versions
pub fn split_paths(unparsed: &OsStr) -> SplitPaths<'_> {
    SplitPaths { inner: unparsed.as_ref() }
}

pub struct SplitPaths<'a> {
    inner: &'a OsStr,
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        if self.inner.is_empty() {
            None
        } else {
            let path = PathBuf::from(self.inner);
            self.inner = OsStr::new("");
            Some(path)
        }
    }
}

pub fn join_paths<I, T>(paths: I) -> Result<OsString, JoinPathsError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    let mut result = OsString::new();
    for (i, path) in paths.enumerate() {
        if i > 0 {
            result.push(":");
        }
        result.push(path.as_ref());
    }
    Ok(result)
}

pub struct JoinPathsError;

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "join paths error")
    }
}

impl fmt::Debug for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl crate::error::Error for JoinPathsError {}

// Re-export from env module
pub fn home_dir() -> Option<PathBuf> {
    super::env::home_dir()
}

pub fn temp_dir() -> PathBuf {
    super::env::temp_dir()
}

// Error handling functions
pub fn errno() -> i32 {
    0 // No errno on ASTRA.OS
}

pub fn is_interrupted(_errno: i32) -> bool {
    false // No interrupts
}

pub fn error_string(errno: i32) -> String {
    format!("error code {}", errno)
}
