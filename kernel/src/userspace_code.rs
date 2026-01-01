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
    // Ring 3 entry point - Hello World from userspace!
    use core::arch::naked_asm;
    unsafe {
        naked_asm!(
            // sys_write(1, "Hello from Ring 3!\n", 20)
            "mov rax, 1",              // syscall number: write
            "mov rdi, 1",              // fd: stdout
            "lea rsi, [rip + msg]",    // buffer: pointer to message
            "mov rdx, 20",             // count: message length
            "int 0x80",                // invoke syscall

            // sys_exit(0)
            "mov rax, 60",             // syscall number: exit
            "mov rdi, 0",              // status: 0
            "int 0x80",                // invoke syscall

            // Should never reach here
            "hlt",

            "msg:",
            ".ascii \"Hello from Ring 3!\\n\"",
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
