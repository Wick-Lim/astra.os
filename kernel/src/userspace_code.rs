// Userspace code that will run in Ring 3
// This is embedded in the kernel but executed in user mode

use core::arch::asm;
use alloc::string::String;
use crate::simple_html;

/// Entry point for userspace
/// This function will be called from kernel after switching to Ring 3
/// Now with interrupt test to verify TSS-based Ring 3 → Ring 0 transition
/// Using #[naked] to prevent compiler from generating prologue/epilogue
#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn userspace_main() -> ! {
    // Test Ring 3 with interrupts enabled
    use core::arch::naked_asm;
    unsafe {
        naked_asm!(
            // Enable interrupts
            "sti",

            // Infinite loop - wait for timer interrupt
            "1:",
            "hlt",  // Halt until interrupt (saves CPU)
            "jmp 1b",
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
