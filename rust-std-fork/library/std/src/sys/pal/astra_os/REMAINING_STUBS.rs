// Remaining stub modules for ASTRA.OS std implementation
// Each section below should go in its own file

// ============ path.rs ============
// rust/library/std/src/sys/astra_os/path.rs

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

// ============ os.rs ============
// rust/library/std/src/sys/astra_os/os.rs

pub const FAMILY: &str = "unix";
pub const OS: &str = "astra_os";
pub const DLL_PREFIX: &str = "lib";
pub const DLL_SUFFIX: &str = ".so";
pub const DLL_EXTENSION: &str = "so";
pub const EXE_SUFFIX: &str = "";
pub const EXE_EXTENSION: &str = "";

// ============ os_str.rs ============
// rust/library/std/src/sys/astra_os/os_str.rs

use crate::ffi::{OsStr, OsString};
use crate::mem;

#[derive(Clone, Hash)]
pub struct Buf {
    pub inner: Vec<u8>,
}

impl Buf {
    pub fn from_string(s: String) -> Buf {
        Buf {
            inner: s.into_bytes(),
        }
    }

    pub fn as_slice(&self) -> &Slice {
        unsafe { mem::transmute(&*self.inner) }
    }

    pub fn into_string(self) -> Result<String, Buf> {
        String::from_utf8(self.inner).map_err(|p| Buf {
            inner: p.into_bytes(),
        })
    }

    pub fn push_slice(&mut self, s: &Slice) {
        self.inner.extend_from_slice(&s.inner);
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), crate::collections::TryReserveError> {
        self.inner.try_reserve(additional)
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity);
    }

    pub fn into_box(self) -> Box<Slice> {
        unsafe { mem::transmute(self.inner.into_boxed_slice()) }
    }

    pub fn from_box(boxed: Box<Slice>) -> Buf {
        let inner: Box<[u8]> = unsafe { mem::transmute(boxed) };
        Buf {
            inner: inner.into_vec(),
        }
    }

    pub fn into_arc(&self) -> Arc<Slice> {
        self.as_slice().into_arc()
    }

    pub fn into_rc(&self) -> Rc<Slice> {
        self.as_slice().into_rc()
    }
}

#[derive(Hash)]
#[repr(transparent)]
pub struct Slice {
    pub inner: [u8],
}

impl Slice {
    pub fn from_str(s: &str) -> &Slice {
        unsafe { mem::transmute(s.as_bytes()) }
    }

    pub fn from_encoded_bytes_unchecked(s: &[u8]) -> &Slice {
        unsafe { mem::transmute(s) }
    }

    pub fn as_encoded_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn to_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.inner).ok()
    }

    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.inner)
    }

    pub fn to_owned(&self) -> Buf {
        Buf {
            inner: self.inner.to_vec(),
        }
    }

    pub fn into_box(&self) -> Box<Slice> {
        let boxed: Box<[u8]> = self.inner.into();
        unsafe { mem::transmute(boxed) }
    }

    pub fn empty_box() -> Box<Slice> {
        let boxed: Box<[u8]> = Default::default();
        unsafe { mem::transmute(boxed) }
    }

    pub fn into_arc(&self) -> Arc<Slice> {
        let arc: Arc<[u8]> = Arc::from(&self.inner);
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const Slice) }
    }

    pub fn into_rc(&self) -> Rc<Slice> {
        let rc: Rc<[u8]> = Rc::from(&self.inner);
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const Slice) }
    }
}

use crate::borrow::Cow;
use crate::rc::Rc;
use crate::sync::Arc;

// ============ pipe.rs ============
// rust/library/std/src/sys/astra_os/pipe.rs

use crate::io;
use crate::sys::unsupported;

pub struct AnonPipe;

impl AnonPipe {
    pub fn read(&self, _buf: &mut [u8]) -> io::Result<usize> {
        unsupported!()
    }

    pub fn write(&self, _buf: &[u8]) -> io::Result<usize> {
        unsupported!()
    }
}

pub fn anon_pipe() -> io::Result<(AnonPipe, AnonPipe)> {
    unsupported!()
}

// ============ cmath.rs ============
// rust/library/std/src/sys/astra_os/cmath.rs

// Minimal C math stubs
// TODO: Link with libm or implement in assembly

pub fn acos(n: f64) -> f64 {
    n // Stub
}

pub fn asin(n: f64) -> f64 {
    n // Stub
}

pub fn atan(n: f64) -> f64 {
    n // Stub
}

pub fn atan2(a: f64, b: f64) -> f64 {
    a / b // Stub
}

pub fn cbrt(n: f64) -> f64 {
    n // Stub
}

pub fn cos(n: f64) -> f64 {
    n // Stub
}

pub fn exp(n: f64) -> f64 {
    n // Stub
}

pub fn log(n: f64) -> f64 {
    n // Stub
}

pub fn log10(n: f64) -> f64 {
    n // Stub
}

pub fn pow(n: f64, p: f64) -> f64 {
    n * p // Stub
}

pub fn sin(n: f64) -> f64 {
    n // Stub
}

pub fn tan(n: f64) -> f64 {
    n // Stub
}

// ============ locks.rs ============
// rust/library/std/src/sys/astra_os/locks/mod.rs

pub mod condvar;
pub mod mutex;
pub mod rwlock;

// These use spin locks since we have no real threading

// ============ thread_local_key.rs ============
// rust/library/std/src/sys/astra_os/thread_local_key.rs

pub type Key = usize;
pub type Dtor = unsafe extern "C" fn(*mut u8);

// Thread-local storage (stub)
pub unsafe fn create(_dtor: Option<Dtor>) -> Key {
    0
}

pub unsafe fn set(_key: Key, _value: *mut u8) {}

pub unsafe fn get(_key: Key) -> *mut u8 {
    core::ptr::null_mut()
}

pub unsafe fn destroy(_key: Key) {}

pub fn requires_synchronized_create() -> bool {
    false
}

// ============ memchr.rs ============
// rust/library/std/src/sys/astra_os/memchr.rs

pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

pub fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().rposition(|&b| b == needle)
}
