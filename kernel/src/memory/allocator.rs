use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, PhysFrame,
};
use x86_64::{VirtAddr, PhysAddr};

/// 힙의 시작 주소
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// 힙 크기 (1 MiB) - 나중에 늘릴 수 있음
pub const HEAP_SIZE: usize = 1 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

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

    serial_println!("    Initializing allocator...");
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }
    serial_println!("    Allocator initialized");

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
        while !self.frame_is_usable(self.next_frame) {
            // 다음 프레임으로 이동
            self.next_frame += 1;
        }

        let frame = self.next_frame;
        self.next_frame += 1;
        Some(frame)
    }
}
