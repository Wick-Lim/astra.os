// File system stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/fs.rs

use crate::ffi::OsString;
use crate::fmt;
use crate::io::{self, Error, ErrorKind, IoSlice, IoSliceMut, SeekFrom};
use crate::path::{Path, PathBuf};
use crate::sys::time::SystemTime;
use crate::sys::unsupported;

// Hardcoded HTML content for initial Servo demo
const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <title>ASTRA.OS</title>
    <style>
        body {
            font-family: sans-serif;
            background: #1a1a2e;
            color: #eee;
            padding: 20px;
        }
        h1 {
            color: #00d9ff;
            font-size: 48px;
        }
        p {
            font-size: 20px;
            line-height: 1.6;
        }
    </style>
</head>
<body>
    <h1>ASTRA.OS</h1>
    <p>Advanced System for Tomorrow's Revolutionary Applications</p>
    <p>Powered by <strong>Servo</strong> browser engine</p>
    <p>Built with Rust 🦀 from kernel to userspace</p>
</body>
</html>
"#;

pub struct File {
    path: PathBuf,
    content: Vec<u8>,
    position: usize,
}

impl File {
    pub fn open(path: &Path, _opts: &OpenOptions) -> io::Result<File> {
        // For now, only support reading hardcoded files
        let content = match path.to_str() {
            Some("/index.html") | Some("index.html") => INDEX_HTML.as_bytes().to_vec(),
            Some("/test.html") | Some("test.html") => {
                b"<!DOCTYPE html><html><body><h1>Test Page</h1></body></html>".to_vec()
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("file not found: {:?}", path),
                ))
            }
        };

        Ok(File {
            path: path.to_path_buf(),
            content,
            position: 0,
        })
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        Ok(FileAttr {
            size: self.content.len() as u64,
        })
    }

    pub fn fsync(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn datasync(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn truncate(&mut self, _size: u64) -> io::Result<()> {
        unsupported!()
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.content.len() - self.position;
        let to_read = buf.len().min(remaining);

        buf[..to_read].copy_from_slice(&self.content[self.position..self.position + to_read]);
        self.position += to_read;

        Ok(to_read)
    }

    pub fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut total = 0;
        for buf in bufs {
            let n = self.read(buf)?;
            total += n;
            if n < buf.len() {
                break;
            }
        }
        Ok(total)
    }

    pub fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn write_vectored(&mut self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    pub fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => self.content.len() as i64 + offset,
            SeekFrom::Current(offset) => self.position as i64 + offset,
        };

        if new_pos < 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "invalid seek to negative position",
            ));
        }

        self.position = new_pos as usize;
        Ok(self.position as u64)
    }

    pub fn duplicate(&self) -> io::Result<File> {
        Ok(File {
            path: self.path.clone(),
            content: self.content.clone(),
            position: 0,
        })
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        unsupported!()
    }
}

#[derive(Clone)]
pub struct FileAttr {
    size: u64,
}

impl FileAttr {
    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn perm(&self) -> FilePermissions {
        FilePermissions {}
    }

    pub fn file_type(&self) -> FileType {
        FileType::File
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        Ok(SystemTime::UNIX_EPOCH)
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        Ok(SystemTime::UNIX_EPOCH)
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        Ok(SystemTime::UNIX_EPOCH)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FileType {
    File,
    Dir,
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        matches!(self, FileType::Dir)
    }

    pub fn is_file(&self) -> bool {
        matches!(self, FileType::File)
    }

    pub fn is_symlink(&self) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FilePermissions {}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        true // Everything is readonly for now
    }

    pub fn set_readonly(&mut self, _readonly: bool) {}
}

pub struct ReadDir {
    entries: Vec<PathBuf>,
    index: usize,
}

impl ReadDir {
    fn new() -> ReadDir {
        ReadDir {
            entries: vec![
                PathBuf::from("index.html"),
                PathBuf::from("test.html"),
            ],
            index: 0,
        }
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        if self.index >= self.entries.len() {
            return None;
        }

        let entry = DirEntry {
            path: self.entries[self.index].clone(),
        };
        self.index += 1;

        Some(Ok(entry))
    }
}

pub struct DirEntry {
    path: PathBuf,
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn file_name(&self) -> OsString {
        self.path.file_name().unwrap().to_owned()
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        Ok(FileAttr { size: 0 })
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(FileType::File)
    }
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    create: bool,
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            create: false,
        }
    }

    pub fn read(&mut self, read: bool) {
        self.read = read;
    }

    pub fn write(&mut self, write: bool) {
        self.write = write;
    }

    pub fn append(&mut self, _append: bool) {}

    pub fn truncate(&mut self, _truncate: bool) {}

    pub fn create(&mut self, create: bool) {
        self.create = create;
    }

    pub fn create_new(&mut self, _create_new: bool) {}
}

// File system operations
pub fn readdir(_p: &Path) -> io::Result<ReadDir> {
    Ok(ReadDir::new())
}

pub fn unlink(_p: &Path) -> io::Result<()> {
    unsupported!()
}

pub fn rename(_old: &Path, _new: &Path) -> io::Result<()> {
    unsupported!()
}

pub fn set_perm(_p: &Path, _perm: FilePermissions) -> io::Result<()> {
    unsupported!()
}

pub fn rmdir(_p: &Path) -> io::Result<()> {
    unsupported!()
}

pub fn remove_dir_all(_path: &Path) -> io::Result<()> {
    unsupported!()
}

pub fn try_exists(_path: &Path) -> io::Result<bool> {
    Ok(false)
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    unsupported!()
}

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported!()
}

pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    unsupported!()
}

pub fn stat(_p: &Path) -> io::Result<FileAttr> {
    Ok(FileAttr { size: 0 })
}

pub fn lstat(_p: &Path) -> io::Result<FileAttr> {
    Ok(FileAttr { size: 0 })
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    unsupported!()
}

pub fn copy(_from: &Path, _to: &Path) -> io::Result<u64> {
    unsupported!()
}

pub fn getcwd() -> io::Result<PathBuf> {
    // Return root directory as current working directory
    Ok(PathBuf::from("/"))
}

pub fn chdir(_p: &Path) -> io::Result<()> {
    // Stub: pretend to change directory
    Ok(())
}
