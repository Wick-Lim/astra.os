// Thread stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/thread.rs

use crate::ffi::CStr;
use crate::io;
use crate::num::NonZeroUsize;
use crate::sys::unsupported;
use crate::time::Duration;

pub struct Thread {
    id: usize,
}

impl Thread {
    // The main thread has ID 1, spawned threads get sequential IDs
    pub fn new() -> Thread {
        static mut NEXT_ID: usize = 2;
        unsafe {
            let id = NEXT_ID;
            NEXT_ID += 1;
            Thread { id }
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

pub const DEFAULT_MIN_STACK_SIZE: usize = 2 * 1024 * 1024; // 2MB

pub mod guard {
    pub type Guard = !;
    pub unsafe fn current() -> Option<Guard> {
        None
    }
    pub unsafe fn init() -> Option<Guard> {
        None
    }
}

// Spawning threads - for now, we execute immediately (no real parallelism)
// This is a STUB for initial Servo demo. Real threads will be implemented later.
pub unsafe fn spawn<'a, F>(f: F, _stack: usize) -> io::Result<Thread>
where
    F: FnOnce() + 'a,
{
    // WARNING: This is NOT a real thread!
    // We execute the function immediately in the current context.
    // This will make Servo run serially, which is slow but functional.

    // Execute the thread function immediately
    f();

    // Return a dummy thread handle
    Ok(Thread::new())
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    // Report 1 CPU for now (no real parallelism yet)
    Ok(unsafe { NonZeroUsize::new_unchecked(1) })
}

pub fn current() -> Thread {
    // Return the main thread (ID 1)
    Thread { id: 1 }
}

pub fn yield_now() {
    // Call HLT instruction to yield to hardware
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("hlt");
    }
}

pub fn sleep(dur: Duration) {
    // Simple busy-wait sleep using PIT timer
    // TODO: Integrate with kernel's timer interrupt
    let start = crate::sys::time::Instant::now();
    loop {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            break;
        }
        yield_now();
    }
}

// Thread-local storage key (dummy implementation)
pub type Key = usize;

#[inline]
pub unsafe fn create(_dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    // Return dummy key
    0
}

#[inline]
pub unsafe fn set(_key: Key, _value: *mut u8) {
    // Stub: Do nothing
}

#[inline]
pub unsafe fn get(_key: Key) -> *mut u8 {
    // Stub: Return null
    core::ptr::null_mut()
}

#[inline]
pub unsafe fn destroy(_key: Key) {
    // Stub: Do nothing
}

pub fn min_stack() -> usize {
    DEFAULT_MIN_STACK_SIZE
}

// JoinHandle implementation
pub struct JoinHandle<T> {
    result: Option<T>,
}

impl<T> JoinHandle<T> {
    pub fn new(result: T) -> Self {
        JoinHandle {
            result: Some(result),
        }
    }

    pub fn join(mut self) -> io::Result<T> {
        // Since we executed immediately, just return the result
        self.result.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "thread already joined")
        })
    }
}

// Thread parking (stub)
pub fn park() {
    // Just yield
    yield_now();
}

pub fn park_timeout(dur: Duration) {
    sleep(dur);
}

pub fn unpark(_thread: &Thread) {
    // Stub: Do nothing (no real threads to wake)
}

// Thread naming (stub)
pub fn set_name(_name: &CStr) {
    // Stub: Ignore thread names for now
}

pub fn get_name() -> Option<&'static CStr> {
    None
}
