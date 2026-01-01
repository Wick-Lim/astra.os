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

    serial_println!("Initializing mouse...");
    // 마우스 드라이버 초기화
    drivers::mouse::init();
    serial_println!("Mouse initialized");

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
    draw_str(2, 0, "Browser OS v0.1.0 - Phase 2: Graphics & UI", Color::White, Color::Blue);

    // 메인 컨텐츠 영역
    draw_str(2, 2, "=== Kernel Features ===", Color::Yellow, Color::Black);
    draw_str(2, 4, "[OK] Memory Management", Color::Green, Color::Black);
    draw_str(2, 5, "[OK] Interrupt Handling", Color::Green, Color::Black);
    draw_str(2, 6, "[OK] VGA Graphics Driver", Color::Green, Color::Black);
    draw_str(2, 7, "[OK] Serial Port Debug", Color::Green, Color::Black);

    // 그래픽 데모 박스
    draw_str(2, 9, "=== Graphics Demo ===", Color::Yellow, Color::Black);

    // 사각형 테두리 데모
    draw_rect(5, 11, 20, 5, Color::Cyan, Color::Black);
    draw_str(7, 12, "Box Demo 1", Color::Cyan, Color::Black);

    // 채워진 사각형 데모
    fill_rect(30, 11, 20, 5, Color::White, Color::Magenta);
    draw_str(32, 13, "Box Demo 2", Color::White, Color::Magenta);

    // UI 위젯 데모
    draw_str(2, 17, "=== UI Widgets Demo ===", Color::Yellow, Color::Black);

    // 버튼 생성
    let mut button1 = ui::Button::new(5, 19, 15, 3, "Click Me!");
    let mut button2 = ui::Button::new(25, 19, 15, 3, "Press Me!");
    let mut button3 = ui::Button::new(45, 19, 20, 3, "Interactive UI!");

    button1.draw();
    button2.draw();
    button3.draw();

    // 클릭 카운터 표시
    let mut click_count = 0;
    draw_str(5, 22, "Clicks: 0", Color::Cyan, Color::Black);

    // 상태 바
    fill_rect(0, 24, 80, 1, Color::Black, Color::LightGray);
    draw_str(2, 24, "Move mouse and click buttons!", Color::Black, Color::LightGray);

    serial_println!("Kernel initialized successfully!");
    serial_println!("Entering idle loop...");

    // 메인 루프 - 마우스 커서 및 UI 상호작용
    let mut last_mouse_x = 40;
    let mut last_mouse_y = 12;
    loop {
        x86_64::instructions::hlt();

        // 마우스 상태 확인
        let mouse_state = drivers::mouse::get_state();

        // 버튼 업데이트 및 클릭 처리
        if button1.update(&mouse_state) || button2.update(&mouse_state) || button3.update(&mouse_state) {
            click_count += 1;
            // 클릭 카운터 업데이트
            use alloc::format;
            let counter_str = format!("Clicks: {}", click_count);
            // 이전 텍스트 지우기
            fill_rect(5, 22, 15, 1, Color::Cyan, Color::Black);
            draw_str(5, 22, &counter_str, Color::Cyan, Color::Black);
        }

        // 버튼 다시 그리기 (눌림 상태 표시)
        button1.draw();
        button2.draw();
        button3.draw();

        // 마우스 커서 업데이트
        if mouse_state.x != last_mouse_x || mouse_state.y != last_mouse_y {
            // 새 커서 그리기
            let cursor_char = if mouse_state.left_button {
                "X"
            } else if mouse_state.right_button {
                "O"
            } else {
                "+"
            };

            draw_str(
                mouse_state.x as usize,
                mouse_state.y as usize,
                cursor_char,
                Color::White,
                Color::Black,
            );

            last_mouse_x = mouse_state.x;
            last_mouse_y = mouse_state.y;
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
