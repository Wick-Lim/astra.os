#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

mod drivers;
mod interrupts;
mod memory;
mod serial;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 시리얼 포트 초기화
    serial::init();
    serial_println!("Browser OS v0.1.0");
    serial_println!("Kernel starting...");
    serial_println!("Boot info physical_memory_offset: {:#x}", boot_info.physical_memory_offset);

    serial_println!("Initializing memory...");
    // 메모리 초기화
    memory::init(boot_info);
    serial_println!("Memory initialized");

    serial_println!("Initializing interrupts...");
    // 인터럽트 초기화
    interrupts::init();
    serial_println!("Interrupts initialized");

    serial_println!("Initializing framebuffer...");
    // 프레임버퍼 초기화
    drivers::framebuffer::init();
    serial_println!("Framebuffer initialized");

    // 화면을 파란색으로 채우기
    drivers::framebuffer::clear_screen(0x0000FF);
    serial_println!("Screen cleared to blue");

    // 화면에 환영 메시지 출력
    println!("===========================================");
    println!("    Browser OS v0.1.0 - Phase 1");
    println!("===========================================");
    println!();
    println!("Kernel Features:");
    println!("  [OK] Memory Management (100MB Heap)");
    println!("  [OK] Interrupt Handling");
    println!("  [OK] VGA Text Display");
    println!("  [OK] Serial Port Debug");
    println!();
    println!("Next: Phase 2 - Graphics & UI");
    println!();
    println!("System ready. Press Ctrl+C to exit QEMU.");

    serial_println!("Kernel initialized successfully!");
    serial_println!("Entering idle loop...");

    // 메인 루프
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
