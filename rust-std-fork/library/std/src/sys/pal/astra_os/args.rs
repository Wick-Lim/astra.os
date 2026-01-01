// Command-line arguments stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/args.rs

use crate::ffi::OsString;

pub fn args() -> Args {
    Args { index: 0 }
}

pub struct Args {
    index: usize,
}

impl Args {
    pub fn len(&self) -> usize {
        1 // Just the program name
    }
}

impl Iterator for Args {
    type Item = OsString;

    fn next(&mut self) -> Option<OsString> {
        if self.index == 0 {
            self.index += 1;
            Some(OsString::from("kernel"))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.len() - self.index
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<OsString> {
        self.next()
    }
}
