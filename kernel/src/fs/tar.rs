// TAR filesystem implementation (USTAR format)
// Provides read-only access to files embedded in TAR archives

use alloc::string::String;
use alloc::vec::Vec;
use core::str;

/// USTAR header (512 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UstarHeader {
    pub name: [u8; 100],        // File name
    pub mode: [u8; 8],          // File mode (octal)
    pub uid: [u8; 8],           // Owner user ID (octal)
    pub gid: [u8; 8],           // Owner group ID (octal)
    pub size: [u8; 12],         // File size (octal)
    pub mtime: [u8; 12],        // Modification time (octal)
    pub checksum: [u8; 8],      // Header checksum (octal)
    pub typeflag: u8,           // File type
    pub linkname: [u8; 100],    // Link name
    pub magic: [u8; 6],         // "ustar\0"
    pub version: [u8; 2],       // "00"
    pub uname: [u8; 32],        // Owner user name
    pub gname: [u8; 32],        // Owner group name
    pub devmajor: [u8; 8],      // Device major number
    pub devminor: [u8; 8],      // Device minor number
    pub prefix: [u8; 155],      // Filename prefix
    pub pad: [u8; 12],          // Padding
}

const USTAR_BLOCK_SIZE: usize = 512;

/// File type flags
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Normal = 0,      // '0' or '\0'
    Directory = 5,   // '5'
}

/// TAR file entry
#[derive(Debug, Clone)]
pub struct TarEntry {
    pub name: String,
    pub size: usize,
    pub offset: usize,  // Offset in TAR archive where data starts
    pub file_type: FileType,
}

/// TAR archive
pub struct TarArchive {
    data: &'static [u8],
    entries: Vec<TarEntry>,
}

impl TarArchive {
    /// Parse TAR archive from memory
    pub fn new(data: &'static [u8]) -> Result<Self, &'static str> {
        let mut entries = Vec::new();
        let mut offset = 0;

        crate::serial_println!("[TAR] Parsing TAR archive at {:#x}, size: {} bytes",
            data.as_ptr() as usize, data.len());

        while offset + USTAR_BLOCK_SIZE <= data.len() {
            // Check for end of archive (two zero blocks)
            if data[offset..offset + USTAR_BLOCK_SIZE].iter().all(|&b| b == 0) {
                crate::serial_println!("[TAR] Found end marker at offset {:#x}", offset);
                break;
            }

            // Parse header
            let header = unsafe {
                &*(data[offset..].as_ptr() as *const UstarHeader)
            };

            // Verify USTAR magic
            if &header.magic[0..5] != b"ustar" {
                crate::serial_println!("[TAR] Invalid magic at offset {:#x}", offset);
                break;
            }

            // Parse filename
            let name = parse_cstr(&header.name)?;

            // Parse size (octal)
            let size = parse_octal(&header.size)?;

            // Parse file type
            let file_type = match header.typeflag {
                b'0' | 0 => FileType::Normal,
                b'5' => FileType::Directory,
                _ => {
                    crate::serial_println!("[TAR] Unknown file type: {}", header.typeflag as char);
                    FileType::Normal
                }
            };

            // Data starts after header
            let data_offset = offset + USTAR_BLOCK_SIZE;

            crate::serial_println!("[TAR] Found: {} (type: {:?}, size: {} bytes, offset: {:#x})",
                name, file_type, size, data_offset);

            entries.push(TarEntry {
                name: String::from(name),
                size,
                offset: data_offset,
                file_type,
            });

            // Move to next entry (data is padded to 512-byte blocks)
            let data_blocks = (size + USTAR_BLOCK_SIZE - 1) / USTAR_BLOCK_SIZE;
            offset = data_offset + data_blocks * USTAR_BLOCK_SIZE;
        }

        crate::serial_println!("[TAR] Parsed {} entries", entries.len());

        Ok(TarArchive { data, entries })
    }

    /// Find file by path
    pub fn find_file(&self, path: &str) -> Option<&TarEntry> {
        self.entries.iter().find(|entry| {
            entry.name == path && entry.file_type == FileType::Normal
        })
    }

    /// List all files
    pub fn list_files(&self) -> impl Iterator<Item = &TarEntry> {
        self.entries.iter().filter(|e| e.file_type == FileType::Normal)
    }

    /// Read file data
    pub fn read_file(&self, entry: &TarEntry) -> &[u8] {
        let end = entry.offset + entry.size;
        if end > self.data.len() {
            crate::serial_println!("[TAR] Read beyond archive bounds!");
            return &[];
        }
        &self.data[entry.offset..end]
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Parse null-terminated C string
fn parse_cstr(bytes: &[u8]) -> Result<&str, &'static str> {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    str::from_utf8(&bytes[..len]).map_err(|_| "Invalid UTF-8")
}

/// Parse octal string
fn parse_octal(bytes: &[u8]) -> Result<usize, &'static str> {
    let s = parse_cstr(bytes)?;
    let s = s.trim();

    if s.is_empty() {
        return Ok(0);
    }

    let mut result = 0;
    for ch in s.chars() {
        if ch < '0' || ch > '7' {
            return Err("Invalid octal digit");
        }
        result = result * 8 + (ch as usize - '0' as usize);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_octal() {
        assert_eq!(parse_octal(b"755\0"), Ok(0o755));
        assert_eq!(parse_octal(b"0000644\0"), Ok(0o644));
        assert_eq!(parse_octal(b"100\0"), Ok(0o100));
    }
}
