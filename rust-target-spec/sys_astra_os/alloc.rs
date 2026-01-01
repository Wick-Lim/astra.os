// Memory allocation stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/alloc.rs

use crate::alloc::{GlobalAlloc, Layout, System};

// Use the kernel's allocator directly
// The kernel already has a heap allocator implemented

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Call kernel's allocator
        extern "C" {
            fn astra_os_alloc(size: usize, align: usize) -> *mut u8;
        }
        astra_os_alloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Call kernel's deallocator
        extern "C" {
            fn astra_os_dealloc(ptr: *mut u8, size: usize, align: usize);
        }
        astra_os_dealloc(ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Call kernel's reallocator (if available)
        extern "C" {
            fn astra_os_realloc(ptr: *mut u8, old_size: usize, align: usize, new_size: usize) -> *mut u8;
        }
        astra_os_realloc(ptr, layout.size(), layout.align(), new_size)
    }
}
