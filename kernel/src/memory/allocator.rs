use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, PhysFrame,
};
use x86_64::{VirtAddr, PhysAddr};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use spin::Mutex;

/// 힙의 시작 주소
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// 힙 크기 (2 MiB) - 960 페이지 제한으로 인해 이 이상 불가
pub const HEAP_SIZE: usize = 2 * 1024 * 1024;

/// Bump Allocator - 간단하고 안정적이지만 메모리 재사용 불가
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return null_mut(),
        };

        if alloc_end > bump.heap_end {
            null_mut() // out of memory
        } else {
            bump.next = alloc_end;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator는 deallocate를 하지 않음
        // 메모리 재사용 불가 - 순수 bump-only 할당
        // 아무것도 하지 않음
    }
}

pub struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

/// 힙을 초기화하는 함수
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    memory_map: &'static MemoryMap,
) -> Result<(), MapToError<Size4KiB>> {
    use crate::serial_println;

    serial_println!("    Calculating page range...");
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + (HEAP_SIZE as u64) - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    serial_println!("    Page range calculated");

    serial_println!("    Creating frame allocator...");
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(memory_map) };
    serial_println!("    Frame allocator created");

    serial_println!("    Mapping heap pages...");
    let mut count = 0;
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, &mut frame_allocator)?.flush();
        }
        count += 1;
        if count % 64 == 0 {
            serial_println!("      Mapped {} pages", count);
        }
    }
    serial_println!("    All {} pages mapped", count);

    serial_println!("    Initializing BumpAllocator...");
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    serial_println!("    BumpAllocator initialized");

    Ok(())
}

/// 부트로더의 메모리 맵으로부터 사용 가능한 프레임을 제공하는 FrameAllocator
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next_frame: PhysFrame,
}

impl BootInfoFrameAllocator {
    /// 부트로더의 메모리 맵으로부터 FrameAllocator를 생성
    pub unsafe fn new(memory_map: &'static MemoryMap) -> Self {
        // 첫 번째 사용 가능한 프레임 찾기
        let next_frame = memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| PhysFrame::containing_address(PhysAddr::new(r.range.start_addr())))
            .next()
            .expect("No usable memory regions found");

        BootInfoFrameAllocator {
            memory_map,
            next_frame,
        }
    }

    /// 프레임이 사용 가능한 영역에 있는지 확인
    fn frame_is_usable(&self, frame: PhysFrame) -> bool {
        let addr = frame.start_address().as_u64();
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .any(|r| r.range.start_addr() <= addr && addr < r.range.end_addr())
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // 현재 프레임이 사용 가능한지 확인
        let mut attempts = 0u64;
        while !self.frame_is_usable(self.next_frame) {
            // 다음 프레임으로 이동
            self.next_frame += 1;
            attempts += 1;

            // 너무 많이 시도하면 실패
            if attempts > 100000 {
                use crate::serial_println;
                serial_println!("      WARN: Frame allocator exhausted after {} attempts", attempts);
                serial_println!("      Last frame attempted: {:#x}", self.next_frame.start_address().as_u64());
                return None;
            }
        }

        let frame = self.next_frame;
        self.next_frame += 1;
        Some(frame)
    }
}

/// FFI functions for std library allocation
/// These forward to our global BumpAllocator

#[no_mangle]
pub unsafe extern "C" fn astra_os_alloc(size: usize, align: usize) -> *mut u8 {
    let layout = match Layout::from_size_align(size, align) {
        Ok(layout) => layout,
        Err(_) => return null_mut(),
    };
    unsafe { ALLOCATOR.alloc(layout) }
}

#[no_mangle]
pub unsafe extern "C" fn astra_os_dealloc(ptr: *mut u8, size: usize, align: usize) {
    let layout = match Layout::from_size_align(size, align) {
        Ok(layout) => layout,
        Err(_) => return,
    };
    unsafe { ALLOCATOR.dealloc(ptr, layout) }
}

#[no_mangle]
pub unsafe extern "C" fn astra_os_realloc(
    ptr: *mut u8,
    old_size: usize,
    align: usize,
    new_size: usize,
) -> *mut u8 {
    let old_layout = match Layout::from_size_align(old_size, align) {
        Ok(layout) => layout,
        Err(_) => return null_mut(),
    };
    unsafe { ALLOCATOR.realloc(ptr, old_layout, new_size) }
}
