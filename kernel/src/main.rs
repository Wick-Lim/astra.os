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
mod html;

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

    // Phase 5: HTML 렌더링!
    use drivers::framebuffer::{fill_rect, WIDTH, HEIGHT};

    serial_println!("=== HTML Rendering Test ===");

    // 화면 초기화
    fill_rect(0, 0, WIDTH, HEIGHT, Rgb888::new(10, 10, 30));

    // HTML 파싱 테스트
    serial_println!("Testing HTML parser...");

    // Temporarily bypass parser - create DOM directly
    use alloc::string::String;
    use alloc::vec;
    use html::Node;

    serial_println!("Creating DOM directly...");

    // Test 1: Simple text in h1
    let text1 = String::from("ASTRA Browser");
    let dom1 = Node::Element {
        tag: String::from("h1"),
        children: vec![Node::Text(text1)],
    };

    serial_println!("DOM 1 created!");

    // Test 2: Paragraph
    let text2 = String::from("Testing ASCII: 0123456789");
    let dom2 = Node::Element {
        tag: String::from("p"),
        children: vec![Node::Text(text2)],
    };

    serial_println!("DOM 2 created!");

    serial_println!("Rendering to VGA...");
    let mut renderer = html::renderer::Renderer::new();
    renderer.render(&dom1);
    serial_println!("First element rendered!");
    renderer.render(&dom2);
    serial_println!("Second element rendered!");
    serial_println!("Rendering complete!");

    serial_println!("HTML displayed successfully!");
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
