#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, Circle},
    Drawable,
};

mod drivers;
mod interrupts;
mod memory;
mod serial;
// mod ui; // TODO: UI 모듈을 픽셀 그래픽에 맞게 업데이트 필요
mod network;

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

    // Phase 4: 픽셀 그래픽 데모
    use drivers::framebuffer::{with_framebuffer, fill_rect, draw_rect, WIDTH, HEIGHT};

    serial_println!("=== fill_rect 테스트 시작 ===");

    // 100x100 테스트
    serial_println!("  Testing 100x100...");
    fill_rect(0, 0, 100, 100, Rgb888::new(255, 0, 0));
    serial_println!("  100x100 OK!");

    // 전체 화면 테스트
    serial_println!("  Testing full screen clear...");
    fill_rect(0, 0, WIDTH, HEIGHT, Rgb888::BLACK);
    serial_println!("  Full screen OK!");

    serial_println!("  Testing full width (320x20)...");
    fill_rect(0, 0, WIDTH, 20, Rgb888::BLACK);
    serial_println!("  320x20 OK");

    serial_println!("  Testing full screen...");
    fill_rect(0, 0, WIDTH, HEIGHT, Rgb888::BLACK);
    serial_println!("  Full screen OK");

    // 타이틀 바 (파란색)
    fill_rect(0, 0, WIDTH, 20, Rgb888::new(0, 0, 180));
    serial_println!("  Title bar drawn");

    // 상태 박스들 (녹색)
    fill_rect(20, 30, 60, 30, Rgb888::new(0, 150, 0));
    fill_rect(90, 30, 60, 30, Rgb888::new(0, 150, 0));
    fill_rect(160, 30, 60, 30, Rgb888::new(0, 150, 0));
    fill_rect(230, 30, 60, 30, Rgb888::new(0, 150, 0));
    serial_println!("  Status boxes drawn");

    // 하단 상태 바 (회색)
    fill_rect(0, 180, WIDTH, 20, Rgb888::new(80, 80, 80));
    serial_println!("  Status bar drawn");

    serial_println!("UI drawn successfully!");
    serial_println!("Resolution: {}x{}", WIDTH, HEIGHT);
    serial_println!("Pixel graphics enabled!");
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
