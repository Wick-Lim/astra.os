use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::serial_println;

/// PIC 오프셋 설정
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// 인터럽트 인덱스
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Mouse = PIC_2_OFFSET + 4, // IRQ12
    Syscall = 0x80,  // System call interrupt
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

/// PIC 컨트롤러
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // 예외 핸들러
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);

        // 인터럽트 핸들러
        idt[InterruptIndex::Timer.as_u8()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse.as_u8()]
            .set_handler_fn(mouse_interrupt_handler);

        // System call handler - must be accessible from Ring 3
        idt[InterruptIndex::Syscall.as_u8()]
            .set_handler_fn(syscall_handler)
            .set_privilege_level(x86_64::PrivilegeLevel::Ring3);

        idt
    };
}

/// 인터럽트 초기화
pub fn init() {
    IDT.load();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

// === 예외 핸들러 ===

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
    serial_println!("EXCEPTION: PAGE FAULT");
    serial_println!("Accessed Address: {:?}", x86_64::registers::control::Cr2::read());
    serial_println!("Error Code: {:?}", error_code);
    serial_println!("{:#?}", stack_frame);

    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// === 하드웨어 인터럽트 핸들러 ===

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // 타이머 인터럽트는 매우 자주 발생하므로 로그를 남기지 않음

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    serial_println!("Keyboard: scancode = {}", scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::drivers::mouse::handle_interrupt();

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
    }
}

// === System Call Handler ===

extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
    unsafe {
        // Read syscall number and arguments from registers
        let syscall_num: u64;
        let arg1: u64;
        let arg2: u64;
        let arg3: u64;

        core::arch::asm!(
            "nop",  // Placeholder - registers already have the values
            lateout("rax") syscall_num,
            lateout("rdi") arg1,
            lateout("rsi") arg2,
            lateout("rdx") arg3,
        );

        // Call syscall handler
        let result = crate::syscall::handle_syscall(
            syscall_num,
            arg1,
            arg2,
            arg3,
            0, // arg4
            0, // arg5
        );

        // Return value in rax
        core::arch::asm!(
            "nop",
            in("rax") result,
        );
    }
}
