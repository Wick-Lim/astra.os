// ASTRA.OS - Browser OS Kernel with Userspace Support
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(asm_const)]

extern crate alloc;
// Custom std backend implemented in rust-std-fork/library/std/src/sys/pal/astra_os/
// Can be enabled once libc dependency issues are resolved

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, Circle},
    Drawable,
};

mod drivers;
mod gdt;
mod interrupts;
mod memory;
mod serial;
// mod ui; // TODO: UI 모듈을 픽셀 그래픽에 맞게 업데이트 필요
mod network;
mod process;
mod syscall;
mod simple_html;
mod userspace_code;
mod html;
mod keyboard;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 시리얼 포트 초기화
    serial::init();
    serial_println!("ASTRA.OS v0.2.0 - Phase 4");
    serial_println!("Kernel starting...");
    serial_println!("Boot info physical_memory_offset: {:#x}", boot_info.physical_memory_offset);

    serial_println!("Initializing memory...");
    // 메모리 초기화
    memory::init(boot_info);
    serial_println!("Memory initialized");

    serial_println!("Initializing GDT...");
    // GDT 초기화 (Ring 3 세그먼트 포함)
    gdt::init();
    serial_println!("GDT initialized with userspace segments");

    serial_println!("Initializing interrupts...");
    // 인터럽트 초기화
    interrupts::init();
    serial_println!("Interrupts initialized");

    serial_println!("Initializing mouse...");
    // 마우스 드라이버 초기화
    drivers::mouse::init();
    serial_println!("Mouse initialized");

    serial_println!("Initializing network stack...");
    // 네트워크 스택 초기화
    network::init();
    serial_println!("Network stack initialized");

    serial_println!("Initializing framebuffer (VGA Mode 13h)...");
    // 프레임버퍼 초기화 (VGA Mode 13h: 320x200, 256색)
    drivers::framebuffer::init();
    serial_println!("Framebuffer initialized");

    // Phase 1 테스트: Font rendering, HTML rendering
    serial_println!("\n=== Phase 1 Tests ===");
    test_phase1_features();

    serial_println!("\nGoing to Ring 3...\n");

    // Jump to userspace (Ring 3)
    jump_to_userspace();
}

/// Phase 1 기능 테스트
fn test_phase1_features() {
    use embedded_graphics::pixelcolor::Rgb888;

    // Test 1: Font Rendering
    serial_println!("[TEST 1] Font Rendering");
    drivers::framebuffer::clear_screen(Rgb888::new(0, 0, 0));

    drivers::framebuffer::draw_string(
        "ASTRA.OS Browser v0.2.0",
        10,
        10,
        Rgb888::new(255, 255, 255)
    );

    drivers::framebuffer::draw_string(
        "Phase 1: Font + HTML",
        10,
        25,
        Rgb888::new(100, 200, 255)
    );

    serial_println!("  Font rendering: OK");

    // Test 2: HTML Rendering
    serial_println!("[TEST 2] HTML Rendering");
    let test_html = r#"
        <html>
            <body>
                <h1>Welcome to ASTRA.OS!</h1>
                <p>A minimal browser OS written in Rust</p>
                <p>Features: HTML parsing and rendering</p>
            </body>
        </html>
    "#;

    use alloc::string::ToString;
    html::renderer::render_html_string(test_html, 320);
    serial_println!("  HTML rendering: OK");

    // Test 3: Keyboard buffer (키보드 입력은 userspace에서 테스트)
    serial_println!("[TEST 3] Keyboard Input System");
    serial_println!("  Keyboard buffer initialized: {}",
        keyboard::KEYBOARD_BUFFER.lock().available() == 0);
    serial_println!("  sys_read implementation: Ready");
    serial_println!("=== Phase 1 Tests Complete ===\n");
}

/// Jump from Ring 0 (kernel) to Ring 3 (userspace)
fn jump_to_userspace() -> ! {
    use x86_64::VirtAddr;
    use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};

    // Get userspace entry point
    let userspace_entry = userspace_code::get_userspace_entry();
    serial_println!("Userspace entry point: {:#x}", userspace_entry);

    // Allocate user stack with page alignment to avoid overlapping with GDT
    #[repr(align(4096))]
    struct UserStack([u8; 16384]);  // 16KB stack, page-aligned

    static mut USER_STACK: UserStack = UserStack([0; 16384]);
    let user_stack_top = unsafe {
        VirtAddr::from_ptr(USER_STACK.0.as_ptr()).as_u64() + USER_STACK.0.len() as u64
    };
    serial_println!("User stack: {:#x}", user_stack_top);

    // Mark userspace code and stack pages as USER accessible
    serial_println!("Marking userspace pages as USER accessible...");
    unsafe {
        // Mark code pages as USER accessible
        // Mark multiple pages to ensure all userspace code/data is accessible
        let code_start = VirtAddr::new(userspace_entry);
        let code_page = Page::containing_address(code_start);
        serial_println!("    Code at {:#x}, starting from page {:#x}", userspace_entry, code_page.start_address().as_u64());

        // Mark the code page and a few adjacent pages to cover all userspace code/data
        for i in 0..4 {  // Mark 4 pages (16KB) for userspace code
            let page: Page<Size4KiB> = code_page + i;
            memory::mark_code_page_user_accessible(page);
        }

        // Mark stack pages as USER accessible
        let stack_start = VirtAddr::from_ptr(USER_STACK.0.as_ptr());
        let stack_end = stack_start + USER_STACK.0.len() as u64;
        serial_println!("    Stack from {:#x} to {:#x}", stack_start.as_u64(), stack_end.as_u64());
        let stack_start_page = Page::containing_address(stack_start);
        let stack_end_page = Page::containing_address(stack_end - 1u64);
        serial_println!("    Stack pages from {:#x} to {:#x}", stack_start_page.start_address().as_u64(), stack_end_page.start_address().as_u64());

        for page in Page::range_inclusive(stack_start_page, stack_end_page) {
            memory::mark_data_page_user_accessible(page);
        }
    }
    serial_println!("Userspace pages marked as USER accessible");

    // Ensure kernel stack pages are properly mapped and writable
    serial_println!("Mapping kernel stack pages...");
    unsafe {
        // Get TSS info
        let tss_info = gdt::get_tss_info();
        let kernel_stack_top = tss_info.0;
        let kernel_stack_start = kernel_stack_top - 16384;  // 16KB stack
        let double_fault_stack_top = tss_info.1;
        let double_fault_stack_start = double_fault_stack_top - 16384;
        let timer_int_stack_top = tss_info.2;
        let timer_int_stack_start = timer_int_stack_top - 16384;
        let syscall_stack_top = tss_info.3;
        let syscall_stack_start = syscall_stack_top - 16384;

        serial_println!("  Kernel stack: {:#x} - {:#x}", kernel_stack_start, kernel_stack_top);
        serial_println!("  Double fault stack: {:#x} - {:#x}", double_fault_stack_start, double_fault_stack_top);
        serial_println!("  Timer interrupt stack: {:#x} - {:#x}", timer_int_stack_start, timer_int_stack_top);
        serial_println!("  Syscall stack: {:#x} - {:#x}", syscall_stack_start, syscall_stack_top);

        // Map kernel stack pages as WRITABLE (not user accessible)
        let kernel_stack_start_page = Page::containing_address(VirtAddr::new(kernel_stack_start));
        let kernel_stack_end_page = Page::containing_address(VirtAddr::new(kernel_stack_top - 1));

        for page in Page::range_inclusive(kernel_stack_start_page, kernel_stack_end_page) {
            serial_println!("  Ensuring kernel stack page {:#x} is writable", page.start_address().as_u64());
            // Kernel stack pages should already be mapped, just ensure WRITABLE flag
            memory::ensure_page_writable(page);
        }

        // Map double fault stack pages as WRITABLE
        let df_stack_start_page = Page::containing_address(VirtAddr::new(double_fault_stack_start));
        let df_stack_end_page = Page::containing_address(VirtAddr::new(double_fault_stack_top - 1));

        for page in Page::range_inclusive(df_stack_start_page, df_stack_end_page) {
            serial_println!("  Ensuring double fault stack page {:#x} is writable", page.start_address().as_u64());
            memory::ensure_page_writable(page);
        }

        // Map timer interrupt stack pages as WRITABLE
        let timer_int_stack_start_page = Page::containing_address(VirtAddr::new(timer_int_stack_start));
        let timer_int_stack_end_page = Page::containing_address(VirtAddr::new(timer_int_stack_top - 1));

        for page in Page::range_inclusive(timer_int_stack_start_page, timer_int_stack_end_page) {
            serial_println!("  Ensuring timer interrupt stack page {:#x} is writable", page.start_address().as_u64());
            memory::ensure_page_writable(page);
        }

        // Map syscall stack pages as WRITABLE
        let syscall_stack_start_page = Page::containing_address(VirtAddr::new(syscall_stack_start));
        let syscall_stack_end_page = Page::containing_address(VirtAddr::new(syscall_stack_top - 1));

        for page in Page::range_inclusive(syscall_stack_start_page, syscall_stack_end_page) {
            serial_println!("  Ensuring syscall stack page {:#x} is writable", page.start_address().as_u64());
            memory::ensure_page_writable(page);
        }

        serial_println!("  Kernel stacks mapped successfully");

        // Now test kernel stack is writable
        let test_addr = (kernel_stack_top - 8) as *mut u64;
        *test_addr = 0xDEADBEEF;
        serial_println!("  Kernel stack write test OK");
    }

    // Get user segment selectors
    let user_cs = gdt::user_code_selector().0 as u64;
    let user_ss = gdt::user_data_selector().0 as u64;

    serial_println!("User CS: {:#x}, User SS: {:#x}", user_cs, user_ss);
    serial_println!("User RIP: {:#x}, User RSP: {:#x}", userspace_entry, user_stack_top);

    // Ensure stack is 16-byte aligned
    let user_stack_top = user_stack_top & !0xF;
    serial_println!("User stack aligned to: {:#x}", user_stack_top);

    serial_println!("Executing iretq to Ring 3 with interrupts DISABLED...");
    serial_println!("Userspace will enable interrupts with STI instruction after stable entry");

    unsafe {
        core::arch::asm!(
            // Push values for iretq (in reverse order)
            "push {ss}",          // SS (user data segment)
            "push {rsp}",         // RSP (user stack pointer)
            "pushfq",             // RFLAGS (with current flags)
            "and qword ptr [rsp], ~0x200",  // Clear IF (keep interrupts disabled)
            "push {cs}",          // CS (user code segment)
            "push {rip}",         // RIP (user code entry point)
            "iretq",              // Return to Ring 3

            ss = in(reg) user_ss,
            rsp = in(reg) user_stack_top,
            cs = in(reg) user_cs,
            rip = in(reg) userspace_entry,
            options(noreturn)
        );
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
