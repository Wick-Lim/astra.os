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

    use alloc::string::String;
    use alloc::vec::Vec;
    use alloc::boxed::Box;
    use html::Node;

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

    // Create <h1>ASTRA.OS</h1> DOM
    use core::mem;
    let mut children2 = Vec::new();
    children2.push(Box::new(Node::Text(String::from("ASTRA.OS"))));
    let dom = Node::Element {
        tag: String::from("h1"),
        children: children2,
    };

    // Simple manual rendering without Renderer struct
    use crate::drivers::framebuffer::{fill_rect, draw_pixel};
    use embedded_graphics::pixelcolor::{Rgb888, RgbColor};

    // Clear screen
    fill_rect(0, 0, 320, 200, Rgb888::BLACK);

    // Draw heading background
    fill_rect(0, 26, 320, 32, Rgb888::new(20, 20, 60));

    // Draw simple "ASTRA.OS" using rectangles (no font needed)
    let color = Rgb888::new(0, 200, 255);
    let y_pos = 30;

    // A
    fill_rect(20, y_pos, 3, 20, color);
    fill_rect(23, y_pos, 8, 3, color);
    fill_rect(31, y_pos, 3, 20, color);
    fill_rect(23, y_pos+10, 8, 3, color);

    // S
    fill_rect(40, y_pos, 10, 3, color);
    fill_rect(40, y_pos, 3, 10, color);
    fill_rect(40, y_pos+10, 10, 3, color);
    fill_rect(47, y_pos+10, 3, 10, color);
    fill_rect(40, y_pos+17, 10, 3, color);

    // T
    fill_rect(56, y_pos, 10, 3, color);
    fill_rect(59, y_pos, 3, 20, color);

    mem::forget(dom);

    serial_println!("OK");

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
