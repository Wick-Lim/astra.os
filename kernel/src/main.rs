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
mod ui;
mod network;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 시리얼 포트 초기화
    serial::init();
    serial_println!("ASTRA.OS v0.1.0");
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

    serial_println!("Initializing mouse...");
    // 마우스 드라이버 초기화
    drivers::mouse::init();
    serial_println!("Mouse initialized");

    serial_println!("Initializing network stack...");
    // 네트워크 스택 초기화
    network::init();
    serial_println!("Network stack initialized");

    serial_println!("Initializing framebuffer...");
    // 프레임버퍼 초기화
    drivers::framebuffer::init();
    serial_println!("Framebuffer initialized");

    // 화면을 검은색으로 채우기
    drivers::framebuffer::clear_screen(0x000000);
    serial_println!("Screen cleared");

    // Phase 2 그래픽 데모
    use drivers::framebuffer::{Color, draw_str, draw_rect, fill_rect};

    // 타이틀 바
    fill_rect(0, 0, 80, 1, Color::White, Color::Blue);
    draw_str(2, 0, "ASTRA.OS v0.1.0 - Phase 3: Network Stack", Color::White, Color::Blue);

    // 메인 컨텐츠 영역
    draw_str(2, 2, "=== Kernel Features ===", Color::Yellow, Color::Black);
    draw_str(2, 4, "[OK] Memory Management", Color::Green, Color::Black);
    draw_str(2, 5, "[OK] Interrupt Handling", Color::Green, Color::Black);
    draw_str(2, 6, "[OK] VGA Graphics Driver", Color::Green, Color::Black);
    draw_str(2, 7, "[OK] Serial Port Debug", Color::Green, Color::Black);
    draw_str(2, 8, "[OK] Network Stack (Ready)", Color::Green, Color::Black);

    // 네트워크 정보 섹션
    draw_str(2, 10, "=== Network Configuration ===", Color::Yellow, Color::Black);
    draw_str(2, 12, "IP Address:  10.0.2.15/24", Color::Cyan, Color::Black);
    draw_str(2, 13, "MAC Address: 02:00:00:00:00:01", Color::Cyan, Color::Black);
    draw_str(2, 14, "Status:      Ready (DummyDevice)", Color::Green, Color::Black);

    // 프레임으로 네트워크 정보 강조
    draw_rect(1, 9, 50, 7, Color::DarkGray, Color::Black);

    // UI 위젯 데모
    draw_str(2, 17, "=== UI Widgets Demo ===", Color::Yellow, Color::Black);
    draw_str(2, 19, "Mouse: Ready (No PS/2 in QEMU)", Color::Yellow, Color::Black);
    draw_str(2, 20, "Network: Investigating smoltcp", Color::Yellow, Color::Black);

    // 상태 바
    fill_rect(0, 24, 80, 1, Color::Black, Color::LightGray);
    draw_str(2, 24, "Network Stack Ready | Press Ctrl+C to exit QEMU", Color::Black, Color::LightGray);

    serial_println!("Kernel initialized successfully!");
    serial_println!("Entering idle loop...");

    // 메인 루프 (마우스 비활성화됨)
    loop {
        x86_64::instructions::hlt();
        // TODO: 네트워크 폴링 추가
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
