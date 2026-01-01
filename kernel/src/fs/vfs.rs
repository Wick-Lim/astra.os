// Virtual File System layer
// Provides file descriptor management and file operations

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use lazy_static::lazy_static;
use super::tar::{TarArchive, TarEntry};

const MAX_OPEN_FILES: usize = 128;

/// File descriptor
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileDescriptor(pub usize);

impl FileDescriptor {
    pub const STDIN: FileDescriptor = FileDescriptor(0);
    pub const STDOUT: FileDescriptor = FileDescriptor(1);
    pub const STDERR: FileDescriptor = FileDescriptor(2);
}

/// Open file state
#[derive(Debug, Clone)]
pub struct OpenFile {
    pub path: String,
    pub offset: usize,      // Current read position
    pub tar_entry: TarEntry,
}

/// File descriptor table (per-process, but we only have one process for now)
pub struct FileTable {
    files: [Option<OpenFile>; MAX_OPEN_FILES],
    next_fd: usize,
}

impl FileTable {
    const fn new() -> Self {
        FileTable {
            files: [const { None }; MAX_OPEN_FILES],
            next_fd: 3, // Start after stdin/stdout/stderr
        }
    }

    /// Allocate a new file descriptor
    pub fn alloc_fd(&mut self, file: OpenFile) -> Result<FileDescriptor, &'static str> {
        // Find free slot
        for fd in self.next_fd..MAX_OPEN_FILES {
            if self.files[fd].is_none() {
                self.files[fd] = Some(file);
                crate::serial_println!("[VFS] Allocated FD {}", fd);
                return Ok(FileDescriptor(fd));
            }
        }
        Err("Too many open files")
    }

    /// Get open file by descriptor
    pub fn get(&self, fd: FileDescriptor) -> Option<&OpenFile> {
        if fd.0 < MAX_OPEN_FILES {
            self.files[fd.0].as_ref()
        } else {
            None
        }
    }

    /// Get mutable reference to open file
    pub fn get_mut(&mut self, fd: FileDescriptor) -> Option<&mut OpenFile> {
        if fd.0 < MAX_OPEN_FILES {
            self.files[fd.0].as_mut()
        } else {
            None
        }
    }

    /// Close file descriptor
    pub fn close(&mut self, fd: FileDescriptor) -> Result<(), &'static str> {
        if fd.0 < 3 {
            return Err("Cannot close stdin/stdout/stderr");
        }
        if fd.0 >= MAX_OPEN_FILES {
            return Err("Invalid file descriptor");
        }

        if self.files[fd.0].is_some() {
            self.files[fd.0] = None;
            crate::serial_println!("[VFS] Closed FD {}", fd.0);
            Ok(())
        } else {
            Err("File descriptor not open")
        }
    }
}

lazy_static! {
    pub static ref FILE_TABLE: Mutex<FileTable> = Mutex::new(FileTable::new());
    pub static ref TAR_FS: Mutex<Option<TarArchive>> = Mutex::new(None);
}

/// Initialize VFS with TAR archive
pub fn init(tar_data: &'static [u8]) -> Result<(), &'static str> {
    crate::serial_println!("[VFS] Initializing with TAR archive...");

    let archive = TarArchive::new(tar_data)?;
    crate::serial_println!("[VFS] TAR archive contains {} entries", archive.entry_count());

    // List all files
    crate::serial_println!("[VFS] Files in archive:");
    for entry in archive.list_files() {
        crate::serial_println!("  - {} ({} bytes)", entry.name, entry.size);
    }

    *TAR_FS.lock() = Some(archive);
    Ok(())
}

/// Open a file (returns file descriptor)
pub fn open(path: &str) -> Result<FileDescriptor, &'static str> {
    crate::serial_println!("[VFS] Opening file: {}", path);

    // Get TAR filesystem
    let tar_fs = TAR_FS.lock();
    let tar = tar_fs.as_ref().ok_or("TAR filesystem not initialized")?;

    // Find file in TAR
    let entry = tar.find_file(path).ok_or("File not found")?;

    // Create open file
    let open_file = OpenFile {
        path: String::from(path),
        offset: 0,
        tar_entry: entry.clone(),
    };

    // Allocate file descriptor
    let mut file_table = FILE_TABLE.lock();
    file_table.alloc_fd(open_file)
}

/// Read from file descriptor
pub fn read(fd: FileDescriptor, buf: &mut [u8]) -> Result<usize, &'static str> {
    // Special handling for stdin (already implemented in keyboard.rs)
    if fd == FileDescriptor::STDIN {
        let bytes_read = crate::keyboard::KEYBOARD_BUFFER.lock().read(buf);
        return Ok(bytes_read);
    }

    // Get file from table
    let mut file_table = FILE_TABLE.lock();
    let file = file_table.get_mut(fd).ok_or("Invalid file descriptor")?;

    // Get TAR filesystem
    let tar_fs = TAR_FS.lock();
    let tar = tar_fs.as_ref().ok_or("TAR filesystem not initialized")?;

    // Read from TAR
    let file_data = tar.read_file(&file.tar_entry);
    let remaining = file.tar_entry.size.saturating_sub(file.offset);
    let to_read = core::cmp::min(buf.len(), remaining);

    if to_read == 0 {
        return Ok(0); // EOF
    }

    buf[..to_read].copy_from_slice(&file_data[file.offset..file.offset + to_read]);
    file.offset += to_read;

    crate::serial_println!("[VFS] Read {} bytes from FD {} (offset now: {})",
        to_read, fd.0, file.offset);

    Ok(to_read)
}

/// Close file descriptor
pub fn close(fd: FileDescriptor) -> Result<(), &'static str> {
    crate::serial_println!("[VFS] Closing FD {}", fd.0);
    let mut file_table = FILE_TABLE.lock();
    file_table.close(fd)
}
