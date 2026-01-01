// Global Descriptor Table (GDT) for ASTRA.OS
// Creates custom GDT with Ring 3 (userspace) segments and TSS

use x86_64::structures::gdt::SegmentSelector;
use x86_64::VirtAddr;
use spin::Mutex;

/// GDT with 7 entries: Null, Kernel Code, Kernel Data, User Code, User Data, TSS (2 entries)
static GDT: Mutex<Option<[u64; 7]>> = Mutex::new(None);

/// TSS (Task State Segment) for storing kernel stack pointer
/// x86-64 TSS is naturally aligned, no packing needed
#[repr(C)]
struct TaskStateSegment {
    _reserved1: u32,
    /// Ring 0 stack pointer - used when switching from Ring 3 to Ring 0
    rsp0: u64,
    _rsp1: u64,
    _rsp2: u64,
    _reserved2: u64,
    _ist: [u64; 7],
    _reserved3: u64,
    _reserved4: u16,
    _iomap_base: u16,
}

static mut TSS: TaskStateSegment = TaskStateSegment {
    _reserved1: 0,
    rsp0: 0,  // Will be set in init()
    _rsp1: 0,
    _rsp2: 0,
    _reserved2: 0,
    _ist: [0; 7],
    _reserved3: 0,
    _reserved4: 0,
    _iomap_base: 0,
};

// Kernel stack for interrupt handling (16KB, properly aligned)
#[repr(align(4096))]
struct KernelStack([u8; 16384]);
static KERNEL_STACK: KernelStack = KernelStack([0; 16384]);

// Double fault stack (separate stack for double fault handler)
#[repr(align(4096))]
struct DoubleFaultStack([u8; 16384]);
static DOUBLE_FAULT_STACK: DoubleFaultStack = DoubleFaultStack([0; 16384]);

// Timer interrupt stack (separate stack for timer interrupt from Ring 3)
#[repr(align(4096))]
struct TimerInterruptStack([u8; 16384]);
static TIMER_INTERRUPT_STACK: TimerInterruptStack = TimerInterruptStack([0; 16384]);

// Syscall stack (separate stack for syscalls from Ring 3)
#[repr(align(4096))]
struct SyscallStack([u8; 16384]);
static SYSCALL_STACK: SyscallStack = SyscallStack([0; 16384]);

/// Initialize GDT with custom entries including userspace segments and TSS
pub fn init() {
    crate::serial_println!("  Creating custom GDT with userspace segments and TSS...");

    // Enable SSE (required for compiler-generated code using xmm registers)
    crate::serial_println!("  Enabling SSE...");
    unsafe {
        // Enable SSE: set CR4.OSFXSR (bit 9) and CR4.OSXMMEXCPT (bit 10)
        core::arch::asm!(
            "mov rax, cr4",
            "or rax, 0x600",  // Set bits 9 and 10
            "mov cr4, rax",
            out("rax") _,
        );
    }
    crate::serial_println!("  SSE enabled");

    // Get kernel stack top address (stack grows downward from top)
    let kernel_stack_ptr = KERNEL_STACK.0.as_ptr() as u64;
    let kernel_stack_len = KERNEL_STACK.0.len() as u64;
    let kernel_stack_top = kernel_stack_ptr + kernel_stack_len;
    crate::serial_println!("  Kernel stack range calculated");
    crate::serial_println!("  Stack top: {:#x}", kernel_stack_top);

    // Initialize TSS with kernel stack pointer
    crate::serial_println!("  Setting TSS rsp0...");
    let tss_ptr = unsafe {
        TSS.rsp0 = kernel_stack_top;

        // Set IST[0] for double fault handler
        let double_fault_stack_top = DOUBLE_FAULT_STACK.0.as_ptr() as u64 + DOUBLE_FAULT_STACK.0.len() as u64;
        TSS._ist[0] = double_fault_stack_top;
        crate::serial_println!("  Double fault stack (IST[0]): {:#x}", double_fault_stack_top);

        // Set IST[1] for timer interrupt from Ring 3
        let timer_int_stack_top = TIMER_INTERRUPT_STACK.0.as_ptr() as u64 + TIMER_INTERRUPT_STACK.0.len() as u64;
        TSS._ist[1] = timer_int_stack_top;
        crate::serial_println!("  Timer interrupt stack (IST[1]): {:#x}", timer_int_stack_top);

        // Set IST[2] for syscalls from Ring 3
        let syscall_stack_top = SYSCALL_STACK.0.as_ptr() as u64 + SYSCALL_STACK.0.len() as u64;
        TSS._ist[2] = syscall_stack_top;
        crate::serial_println!("  Syscall stack (IST[2]): {:#x}", syscall_stack_top);

        &TSS as *const TaskStateSegment as u64
    };
    crate::serial_println!("  TSS initialized at {:#x}", tss_ptr);

    let tss_size = core::mem::size_of::<TaskStateSegment>() - 1;
    crate::serial_println!("  TSS at {:#x}, size: {} bytes", tss_ptr, tss_size);
    crate::serial_println!("  TSS rsp0 field: {:#x}", kernel_stack_top);

    // Create TSS descriptor (takes 2 GDT entries in 64-bit mode)
    let tss_low = {
        let mut desc: u64 = 0;
        // Limit (bits 0-15 and 48-51)
        desc |= (tss_size as u64 & 0xFFFF);
        desc |= ((tss_size as u64 & 0xF0000) << 32);
        // Base (bits 16-39 and 56-63)
        desc |= ((tss_ptr & 0xFFFFFF) << 16);
        desc |= ((tss_ptr & 0xFF000000) << 32);
        // Type = 0x9 (available 64-bit TSS), P=1, DPL=0
        desc |= 0x0089 << 40;
        desc
    };
    let tss_high = (tss_ptr >> 32) & 0xFFFFFFFF;

    // Create GDT with 7 entries (TSS takes 2)
    // x86-64 long mode segment descriptors
    // Use same format for kernel and user data, only difference is DPL
    let gdt = [
        0x0000000000000000,  // 0x00: Null descriptor
        0x00209a0000000000,  // 0x08: Kernel code (P=1, DPL=0, S=1, Type=0x0a, L=1)
        0x0000920000000000,  // 0x10: Kernel data (P=1, DPL=0, S=1, Type=0x02)
        0x0020fa0000000000,  // 0x18: User code   (P=1, DPL=3, S=1, Type=0x0a, L=1)
        0x0000f20000000000,  // 0x20: User data   (P=1, DPL=3, S=1, Type=0x02)
                             //      Same as kernel data but with DPL=3
                             //      Type 0x02 = Read/Write data segment
        tss_low,              // 0x28: TSS low
        tss_high,             // 0x30: TSS high
    ];

    // Store GDT globally
    *GDT.lock() = Some(gdt);

    // Load new GDT
    unsafe {
        let gdt_ptr = GDT.lock().as_ref().unwrap().as_ptr();
        let gdtr = GDTR {
            limit: (7 * 8 - 1) as u16,  // 7 entries * 8 bytes - 1
            base: gdt_ptr as u64,
        };

        let base = gdtr.base;
        let limit = gdtr.limit;
        crate::serial_println!("  Loading GDT at {:#x}, limit {:#x}", base, limit);

        // Load the new GDT
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &gdtr,
            options(nostack)
        );

        crate::serial_println!("  GDT loaded successfully");

        // Reload segment registers
        // CS is reloaded with a far return
        core::arch::asm!(
            "push 0x08",           // Push kernel code segment
            "lea {tmp}, [rip + 2f]", // Load address of label 2
            "push {tmp}",          // Push return address
            "retfq",               // Far return to reload CS
            "2:",                  // Label after far return
            // Now reload data segments
            "mov ax, 0x10",        // Kernel data segment
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            tmp = out(reg) _,
            out("ax") _,
        );

        crate::serial_println!("  Segment registers reloaded");

        // Load TSS (selector 0x28 = index 5)
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) 0x28u16,
        );
        crate::serial_println!("  TSS loaded");
    }

    // Verify GDT was loaded
    let gdtr = read_gdtr();
    let base = gdtr.base;
    let limit = gdtr.limit;
    crate::serial_println!("  New GDTR base: {:#x}, limit: {:#x}", base, limit);

    // Read current CS to verify
    let cs: u16;
    unsafe {
        core::arch::asm!("mov {0:x}, cs", out(reg) cs);
    }
    crate::serial_println!("  Current CS after reload: {:#x}", cs);

    crate::serial_println!("  GDT initialization complete with userspace segments");
}

#[repr(C, packed)]
struct GDTR {
    limit: u16,
    base: u64,
}

fn read_gdtr() -> GDTR {
    let mut gdtr = GDTR { limit: 0, base: 0 };
    unsafe {
        core::arch::asm!("sgdt [{}]", in(reg) &mut gdtr, options(nostack));
    }
    gdtr
}

/// Get user code segment selector (for Ring 3)
pub fn user_code_selector() -> SegmentSelector {
    // 0x18 | 3 = 0x1b (index 3, RPL 3)
    SegmentSelector::new(3, x86_64::PrivilegeLevel::Ring3)
}

/// Get user data segment selector (for Ring 3)
pub fn user_data_selector() -> SegmentSelector {
    // 0x20 | 3 = 0x23 (index 4, RPL 3)
    SegmentSelector::new(4, x86_64::PrivilegeLevel::Ring3)
}

/// Get TSS information (rsp0, ist0, ist1, ist2)
pub fn get_tss_info() -> (u64, u64, u64, u64) {
    unsafe {
        (TSS.rsp0, TSS._ist[0], TSS._ist[1], TSS._ist[2])
    }
}
