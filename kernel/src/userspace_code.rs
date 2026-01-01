// Userspace code that will run in Ring 3
// This is embedded in the kernel but executed in user mode

use core::arch::asm;
use alloc::string::String;
use crate::simple_html;

/// Entry point for userspace
/// This function will be called from kernel after switching to Ring 3
/// ULTRA MINIMAL VERSION - just infinite loop to test Ring 3 transition
/// Using #[naked] to prevent compiler from generating prologue/epilogue
#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn userspace_main() -> ! {
    // Do absolutely nothing except loop forever
    // This tests if Ring 3 transition itself works
    use core::arch::naked_asm;
    unsafe {
        naked_asm!(
            "2:",
            "nop",
            "nop",
            "nop",
            "nop",
            "jmp 2b",
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
