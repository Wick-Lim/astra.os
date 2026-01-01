// Alloc implementation for ASTRA.OS
// Forward to kernel's allocator via FFI

use crate::alloc::{GlobalAlloc, Layout, System};

unsafe extern "C" {
    fn astra_os_alloc(size: usize, align: usize) -> *mut u8;
    fn astra_os_dealloc(ptr: *mut u8, size: usize, align: usize);
    fn astra_os_realloc(ptr: *mut u8, old_size: usize, align: usize, new_size: usize) -> *mut u8;
}

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { astra_os_alloc(layout.size(), layout.align()) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { astra_os_dealloc(ptr, layout.size(), layout.align()) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { astra_os_realloc(ptr, layout.size(), layout.align(), new_size) }
    }
}
