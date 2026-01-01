// ASTRA.OS - Browser OS Kernel with Userspace Support
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

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

    use alloc::string::String;
    use alloc::vec::Vec;
    use alloc::boxed::Box;
    use simple_html::Node;

    // Test 1: String
    { let s = String::from("test"); }
    serial_println!("Test 1 OK");

    // Test 2: Vec<Box<Node>>
    { let mut v = Vec::new(); v.push(Box::new(Node::Text(String::from("A")))); v.push(Box::new(Node::Text(String::from("B")))); }
    serial_println!("Test 2 OK");

    // Test 3: Node::Text
    { let _text = Node::Text(String::from("X")); }
    serial_println!("Test 3 OK");

    // Test 4: Empty Element
    { let _element = Node::Element { tag: String::from("p"), children: Vec::new(), }; }
    serial_println!("Test 4 OK");

    serial_println!("=== All basic tests passed! ===");

    // TODO: Fix allocator issue - additional allocations cause triple fault
    // serial_println!("=== Testing std library features ===");

    serial_println!("\n=== Skipping additional tests - going straight to Ring 3 ===\n");

    // Jump to userspace (Ring 3)
    jump_to_userspace();
}

/// Jump from Ring 0 (kernel) to Ring 3 (userspace)
fn jump_to_userspace() -> ! {
    use x86_64::VirtAddr;

    // Get userspace entry point
    let userspace_entry = userspace_code::get_userspace_entry();
    serial_println!("Userspace entry point: {:#x}", userspace_entry);

    // Allocate user stack (for now, use a static array)
    static mut USER_STACK: [u8; 8192] = [0; 8192];
    let user_stack_top = unsafe {
        VirtAddr::from_ptr(USER_STACK.as_ptr()).as_u64() + USER_STACK.len() as u64
    };
    serial_println!("User stack: {:#x}", user_stack_top);

    // Get user segment selectors
    let user_cs = gdt::user_code_selector().0 as u64;
    let user_ss = gdt::user_data_selector().0 as u64;

    serial_println!("User CS: {:#x}, User SS: {:#x}", user_cs, user_ss);
    serial_println!("Executing iretq to Ring 3...");

    unsafe {
        core::arch::asm!(
            // Push values for iretq (in reverse order)
            "push {ss}",          // SS (user data segment)
            "push {rsp}",         // RSP (user stack pointer)
            "pushfq",             // RFLAGS
            "or qword ptr [rsp], 0x200",  // Set IF (interrupt flag) in RFLAGS
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
