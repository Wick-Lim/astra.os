pub mod allocator;

use bootloader::BootInfo;
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::VirtAddr;

/// 부트로더에서 전달받은 물리 메모리 오프셋을 저장
static mut PHYSICAL_MEMORY_OFFSET: Option<VirtAddr> = None;

/// 메모리 초기화
pub fn init(boot_info: &'static BootInfo) {
    use crate::serial_println;

    serial_println!("  Setting physical memory offset...");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    unsafe {
        PHYSICAL_MEMORY_OFFSET = Some(phys_mem_offset);
    }
    serial_println!("  Physical memory offset set: {:#x}", phys_mem_offset.as_u64());

    // 페이지 테이블 설정
    serial_println!("  Creating page table mapper...");
    let mut mapper = unsafe { mapper(phys_mem_offset) };
    serial_println!("  Page table mapper created");

    // 힙 할당자 초기화
    serial_println!("  Initializing heap allocator...");
    allocator::init_heap(&mut mapper, &boot_info.memory_map)
        .expect("heap initialization failed");
    serial_println!("  Heap allocator initialized");
}

/// 물리 메모리 오프셋을 반환
pub unsafe fn physical_memory_offset() -> VirtAddr {
    PHYSICAL_MEMORY_OFFSET.expect("Physical memory offset not initialized")
}

/// 활성 레벨 4 페이지 테이블에 대한 가변 참조를 반환
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// OffsetPageTable 인스턴스를 생성
unsafe fn mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}
