// Environment stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/env.rs

use crate::ffi::OsStr;
use crate::path::PathBuf;

pub fn env() -> Env {
    Env { _priv: () }
}

pub struct Env {
    _priv: (),
}

impl Iterator for Env {
    type Item = (OsString, OsString);

    fn next(&mut self) -> Option<Self::Item> {
        // No environment variables
        None
    }
}

pub fn getenv(_key: &OsStr) -> Option<OsString> {
    // No environment variables
    None
}

pub fn setenv(_key: &OsStr, _value: &OsStr) -> io::Result<()> {
    // Stub: Do nothing
    Ok(())
}

pub fn unsetenv(_key: &OsStr) -> io::Result<()> {
    // Stub: Do nothing
    Ok(())
}

pub fn temp_dir() -> PathBuf {
    PathBuf::from("/tmp")
}

pub fn home_dir() -> Option<PathBuf> {
    Some(PathBuf::from("/"))
}

pub fn exit(code: i32) -> ! {
    // Call kernel panic or halt
    unsafe extern "C" {
        fn astra_os_exit(code: i32) -> !;
    }
    unsafe { astra_os_exit(code) }
}

pub fn getpid() -> u32 {
    1 // We're the kernel, PID 1
}

use crate::ffi::OsString;
use crate::io;

pub fn current_exe() -> io::Result<PathBuf> {
    Ok(PathBuf::from("/kernel"))
}
