// Userspace code that will run in Ring 3
// This is embedded in the kernel but executed in user mode

use core::arch::asm;
use alloc::string::String;
use crate::simple_html;

/// Entry point for userspace
/// This function will be called from kernel after switching to Ring 3
/// MINIMAL VERSION - no allocations to test Ring 3 transition
#[no_mangle]
pub extern "C" fn userspace_main() -> ! {
    syscall_write(1, b"\n===== RING 3 ENTERED! =====\n");
    syscall_write(1, b"Userspace is running!\n");
    syscall_write(1, b"===========================\n\n");

    // Main loop
    let mut counter = 0u64;
    loop {
        if counter % 100000000 == 0 {
            syscall_write(1, b"Userspace heartbeat...\n");
        }
        counter += 1;

        unsafe { asm!("nop") };
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
