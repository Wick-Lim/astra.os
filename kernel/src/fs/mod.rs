// File system module
// Provides TAR-based read-only filesystem

pub mod tar;
pub mod vfs;

pub use vfs::{FileDescriptor, init, open, read, close};
