pub mod allocator;

use bootloader::BootInfo;
use x86_64::structures::paging::{
    OffsetPageTable, PageTable, Page, PhysFrame, Mapper, Size4KiB, PageTableFlags,
    mapper::MapToError,
};
use x86_64::{VirtAddr, PhysAddr};

/// 부트로더에서 전달받은 물리 메모리 오프셋을 저장
static mut PHYSICAL_MEMORY_OFFSET: Option<VirtAddr> = None;

/// 메모리 초기화
pub fn init(boot_info: &'static BootInfo) {
    use crate::serial_println;
    use bootloader::bootinfo::MemoryRegionType;

    serial_println!("  Setting physical memory offset...");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    unsafe {
        PHYSICAL_MEMORY_OFFSET = Some(phys_mem_offset);
    }
    serial_println!("  Physical memory offset set: {:#x}", phys_mem_offset.as_u64());

    // 메모리 맵 출력
    serial_println!("  Analyzing memory map...");
    let mut total_usable = 0u64;
    let mut usable_regions = 0usize;
    for region in boot_info.memory_map.iter() {
        let size = region.range.end_addr() - region.range.start_addr();
        if region.region_type == MemoryRegionType::Usable {
            total_usable += size;
            usable_regions += 1;
        }
    }
    let usable_mb = total_usable / (1024 * 1024);
    serial_println!("  Usable regions: {}", usable_regions);
    serial_println!("  Total usable: {} MB", usable_mb);

    // 페이지 테이블 설정
    serial_println!("  Creating page table mapper...");
    let mut mapper = unsafe { mapper(phys_mem_offset) };
    serial_println!("  Page table mapper created");

    // 힙 할당자 초기화
    serial_println!("  Initializing heap allocator...");
    allocator::init_heap(&mut mapper, &boot_info.memory_map)
        .expect("heap initialization failed");
    serial_println!("  Heap allocator initialized");

    // VGA 메모리 매핑
    serial_println!("  Mapping VGA memory...");
    map_vga_memory(&mut mapper, &boot_info.memory_map)
        .expect("VGA memory mapping failed");
    serial_println!("  VGA memory mapped");
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

/// VGA 메모리 영역을 identity mapping
fn map_vga_memory(
    mapper: &mut impl Mapper<Size4KiB>,
    memory_map: &'static bootloader::bootinfo::MemoryMap,
) -> Result<(), MapToError<Size4KiB>> {
    use crate::serial_println;

    // VGA 그래픽 모드 메모리: 0xA0000 - 0xBFFFF (128KB)
    // VGA 텍스트 모드 메모리: 0xB8000 - 0xBFFFF (32KB, 이미 포함됨)
    let vga_start = 0xA0000u64;
    let vga_end = 0xC0000u64; // 끝 주소 (exclusive)

    serial_println!("    VGA memory range: {:#x} - {:#x}", vga_start, vga_end);

    // 페이지 단위로 매핑
    let start_page: Page = Page::containing_address(VirtAddr::new(vga_start));
    let end_page: Page = Page::containing_address(VirtAddr::new(vga_end - 1));
    let page_range = Page::range_inclusive(start_page, end_page);

    let mut frame_allocator = unsafe {
        allocator::BootInfoFrameAllocator::new(memory_map)
    };

    serial_println!("    Mapping {} VGA pages...", page_range.count());

    for page in page_range {
        // 이미 매핑되어 있는지 확인
        if let Ok(_) = mapper.translate_page(page) {
            // 이미 매핑되어 있으면 스킵
            serial_println!("      Page {:#x} already mapped, skipping", page.start_address().as_u64());
            continue;
        }

        // Identity mapping: 가상 주소 = 물리 주소
        let frame = PhysFrame::containing_address(PhysAddr::new(page.start_address().as_u64()));

        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_CACHE; // VGA 메모리는 캐시 비활성화

        unsafe {
            mapper
                .map_to(page, frame, flags, &mut frame_allocator)?
                .flush();
        }
    }

    serial_println!("    VGA pages mapped successfully");
    Ok(())
}

/// Mark a code page as USER accessible (for Ring 3) - executable, read-only
/// This marks ALL levels of the page table hierarchy as USER_ACCESSIBLE
pub unsafe fn mark_code_page_user_accessible(page: Page<Size4KiB>) {
    use crate::serial_println;
    use x86_64::structures::paging::PageTableIndex;

    let phys_offset = physical_memory_offset();
    serial_println!("    Marking page {:#x} as USER (code, executable) at ALL levels", page.start_address().as_u64());

    // Get the page table hierarchy
    let level_4_table = active_level_4_table(phys_offset);

    let p4_index = page.p4_index();
    let p3_index = page.p3_index();
    let p2_index = page.p2_index();
    let p1_index = page.p1_index();

    // Mark Level 4 entry as USER_ACCESSIBLE
    let p4_entry = &mut level_4_table[p4_index];
    let mut flags = p4_entry.flags();
    flags |= PageTableFlags::USER_ACCESSIBLE;
    p4_entry.set_flags(flags);

    // Get Level 3 table
    let p3_table_addr = phys_offset + p4_entry.addr().as_u64();
    let p3_table: &mut PageTable = &mut *(p3_table_addr.as_mut_ptr());

    // Mark Level 3 entry as USER_ACCESSIBLE
    let p3_entry = &mut p3_table[p3_index];
    let mut flags = p3_entry.flags();
    flags |= PageTableFlags::USER_ACCESSIBLE;
    p3_entry.set_flags(flags);

    // Get Level 2 table
    let p2_table_addr = phys_offset + p3_entry.addr().as_u64();
    let p2_table: &mut PageTable = &mut *(p2_table_addr.as_mut_ptr());

    // Mark Level 2 entry as USER_ACCESSIBLE
    let p2_entry = &mut p2_table[p2_index];
    let mut flags = p2_entry.flags();
    flags |= PageTableFlags::USER_ACCESSIBLE;
    p2_entry.set_flags(flags);

    // Get Level 1 table
    let p1_table_addr = phys_offset + p2_entry.addr().as_u64();
    let p1_table: &mut PageTable = &mut *(p1_table_addr.as_mut_ptr());

    // Mark Level 1 entry as USER_ACCESSIBLE and EXECUTABLE (no NO_EXECUTE)
    let p1_entry = &mut p1_table[p1_index];
    let mut flags = p1_entry.flags();
    flags |= PageTableFlags::USER_ACCESSIBLE;
    flags.remove(PageTableFlags::NO_EXECUTE);
    p1_entry.set_flags(flags);

    // Flush TLB
    use x86_64::instructions::tlb;
    tlb::flush(page.start_address());

    serial_println!("    Page {:#x} now USER accessible (executable) at all levels", page.start_address().as_u64());
}

/// Mark a data page as USER accessible (for Ring 3) - writable, non-executable
/// This marks ALL levels of the page table hierarchy as USER_ACCESSIBLE
pub unsafe fn mark_data_page_user_accessible(page: Page<Size4KiB>) {
    use crate::serial_println;
    use x86_64::structures::paging::PageTableIndex;

    let phys_offset = physical_memory_offset();
    serial_println!("    Marking page {:#x} as USER (data, writable, NX) at ALL levels", page.start_address().as_u64());

    // Get the page table hierarchy
    let level_4_table = active_level_4_table(phys_offset);

    let p4_index = page.p4_index();
    let p3_index = page.p3_index();
    let p2_index = page.p2_index();
    let p1_index = page.p1_index();

    // Mark Level 4 entry as USER_ACCESSIBLE
    let p4_entry = &mut level_4_table[p4_index];
    let mut flags = p4_entry.flags();
    flags |= PageTableFlags::USER_ACCESSIBLE;
    p4_entry.set_flags(flags);

    // Get Level 3 table
    let p3_table_addr = phys_offset + p4_entry.addr().as_u64();
    let p3_table: &mut PageTable = &mut *(p3_table_addr.as_mut_ptr());

    // Mark Level 3 entry as USER_ACCESSIBLE
    let p3_entry = &mut p3_table[p3_index];
    let mut flags = p3_entry.flags();
    flags |= PageTableFlags::USER_ACCESSIBLE;
    p3_entry.set_flags(flags);

    // Get Level 2 table
    let p2_table_addr = phys_offset + p3_entry.addr().as_u64();
    let p2_table: &mut PageTable = &mut *(p2_table_addr.as_mut_ptr());

    // Mark Level 2 entry as USER_ACCESSIBLE
    let p2_entry = &mut p2_table[p2_index];
    let mut flags = p2_entry.flags();
    flags |= PageTableFlags::USER_ACCESSIBLE;
    p2_entry.set_flags(flags);

    // Get Level 1 table
    let p1_table_addr = phys_offset + p2_entry.addr().as_u64();
    let p1_table: &mut PageTable = &mut *(p1_table_addr.as_mut_ptr());

    // Mark Level 1 entry as USER_ACCESSIBLE, WRITABLE, and NO_EXECUTE
    let p1_entry = &mut p1_table[p1_index];
    let mut flags = p1_entry.flags();
    flags |= PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
    p1_entry.set_flags(flags);

    // Flush TLB
    use x86_64::instructions::tlb;
    tlb::flush(page.start_address());

    serial_println!("    Page {:#x} now USER accessible (writable, NX) at all levels", page.start_address().as_u64());
}

/// Ensure a page is writable at ALL page table levels (for kernel use)
/// This does NOT set USER_ACCESSIBLE - for kernel-only pages like kernel stacks
pub unsafe fn ensure_page_writable(page: Page<Size4KiB>) {
    let phys_offset = physical_memory_offset();

    // Get the page table hierarchy
    let level_4_table = active_level_4_table(phys_offset);

    let p4_index = page.p4_index();
    let p3_index = page.p3_index();
    let p2_index = page.p2_index();
    let p1_index = page.p1_index();

    // Ensure Level 4 entry is WRITABLE
    let p4_entry = &mut level_4_table[p4_index];
    let mut flags = p4_entry.flags();
    flags |= PageTableFlags::WRITABLE;
    p4_entry.set_flags(flags);

    // Get Level 3 table
    let p3_table_addr = phys_offset + p4_entry.addr().as_u64();
    let p3_table: &mut PageTable = &mut *(p3_table_addr.as_mut_ptr());

    // Ensure Level 3 entry is WRITABLE
    let p3_entry = &mut p3_table[p3_index];
    let mut flags = p3_entry.flags();
    flags |= PageTableFlags::WRITABLE;
    p3_entry.set_flags(flags);

    // Get Level 2 table
    let p2_table_addr = phys_offset + p3_entry.addr().as_u64();
    let p2_table: &mut PageTable = &mut *(p2_table_addr.as_mut_ptr());

    // Ensure Level 2 entry is WRITABLE
    let p2_entry = &mut p2_table[p2_index];
    let mut flags = p2_entry.flags();
    flags |= PageTableFlags::WRITABLE;
    p2_entry.set_flags(flags);

    // Get Level 1 table
    let p1_table_addr = phys_offset + p2_entry.addr().as_u64();
    let p1_table: &mut PageTable = &mut *(p1_table_addr.as_mut_ptr());

    // Ensure Level 1 entry is WRITABLE
    let p1_entry = &mut p1_table[p1_index];
    let mut flags = p1_entry.flags();
    flags |= PageTableFlags::WRITABLE;
    p1_entry.set_flags(flags);

    // Flush TLB
    use x86_64::instructions::tlb;
    tlb::flush(page.start_address());
}
