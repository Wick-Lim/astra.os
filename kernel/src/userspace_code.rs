// Userspace code that will run in Ring 3
// This is embedded in the kernel but executed in user mode

use core::arch::asm;
use alloc::string::String;
use crate::simple_html;

/// Entry point for userspace
/// This function will be called from kernel after switching to Ring 3
/// Now with interrupt test to verify TSS-based Ring 3 → Ring 0 transition
#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn userspace_main() -> ! {
    // Ring 3 entry point - must use naked to avoid stack frame setup
    // iretq already set up CS and SS correctly
    use core::arch::naked_asm;
    unsafe {
        naked_asm!(
            // Test syscall (int 0x80) from Ring 3 WITHOUT enabling interrupts first
            "1:",
            "mov rax, 42",        // Syscall number (arbitrary)
            "int 0x80",           // Trigger syscall
            "nop",
            "jmp 1b",             // Loop to repeat syscall test
        );
    }
}

/// Syscall wrapper: write
fn syscall_write(fd: i32, message: &[u8]) {
    unsafe {
        asm!(
            "mov rax, 1",           // syscall number: write
            "int 0x80",             // trigger syscall
            in("rdi") fd as u64,
            in("rsi") message.as_ptr(),
            in("rdx") message.len(),
            lateout("rax") _,
        );
    }
}

/// Get the address of userspace_main
pub fn get_userspace_entry() -> u64 {
    userspace_main as u64
}
