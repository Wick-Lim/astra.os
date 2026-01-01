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
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);

        // Double fault handler uses IST[0] for a separate stack
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(0);  // Use IST[0]
        }

        // 인터럽트 핸들러
        // Timer interrupt uses IST[1] for Ring 3 -> Ring 0 transitions
        unsafe {
            idt[InterruptIndex::Timer.as_u8()]
                .set_handler_fn(timer_interrupt_handler)
                .set_stack_index(1);  // Use IST[1] for separate stack
        }
        idt[InterruptIndex::Keyboard.as_u8()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse.as_u8()]
            .set_handler_fn(mouse_interrupt_handler);

        // System call handler - must be accessible from Ring 3, uses IST[2]
        unsafe {
            idt[InterruptIndex::Syscall.as_u8()]
                .set_handler_fn(syscall_handler)
                .set_privilege_level(x86_64::PrivilegeLevel::Ring3)
                .set_stack_index(2);  // Use IST[2] for Ring 3 → Ring 0 transition
        }

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

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    serial_println!("EXCEPTION: GENERAL PROTECTION FAULT");
    serial_println!("Error Code: {:#x}", error_code);
    serial_println!("Stack Frame: {:#?}", stack_frame);
    serial_println!("This likely means:");
    serial_println!("  - Segment selector error");
    serial_println!("  - Privilege level violation");
    serial_println!("  - Or IDT/GDT misconfiguration");

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

extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: InterruptStackFrame) {
    // Check if interrupt came from Ring 3 (userspace)
    static mut RING3_INTERRUPT_COUNT: u64 = 0;
    static mut TOTAL_INTERRUPT_COUNT: u64 = 0;

    unsafe {
        TOTAL_INTERRUPT_COUNT += 1;

        // CS register's lower 2 bits indicate CPL (Current Privilege Level)
        let cs = stack_frame.code_segment.0;
        let cpl = cs & 0x3;

        if cpl == 3 {
            RING3_INTERRUPT_COUNT += 1;

            // Log first Ring 3 interrupt and then every 100th to avoid spam
            if RING3_INTERRUPT_COUNT == 1 || RING3_INTERRUPT_COUNT % 100 == 0 {
                crate::serial_println!("[TIMER] Ring 3 interrupt #{} (total: {}) - TSS working! CS={:#x}",
                                      RING3_INTERRUPT_COUNT, TOTAL_INTERRUPT_COUNT, cs);
            }
        }

        // Always send EOI
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
    static mut SYSCALL_COUNT: u64 = 0;

    unsafe {
        SYSCALL_COUNT += 1;

        // Print first syscall and then every 100th to reduce spam
        if SYSCALL_COUNT == 1 || SYSCALL_COUNT % 100 == 0 {
            let cs = stack_frame.code_segment.0;
            let cpl = cs & 0x3;
            crate::serial_println!("[SYSCALL] #{} from Ring {} (CS={:#x}) - TSS working!",
                                  SYSCALL_COUNT, cpl, cs);
        }
    }
    // Just return for now - register reading needs a different approach
}
