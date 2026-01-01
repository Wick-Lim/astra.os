// ASTRA.OS Browser - Userspace Program (Embedded)
// This will be compiled and embedded into the kernel

#![no_std]
#![no_main]

// For now, we can't use std in embedded userspace
// We'll start with no_std and add std support later

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Entry point for userspace
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Make a syscall to print "Hello from userspace!"
    // syscall number 1 = write, fd=1 (stdout)

    let message = b"Hello from userspace!\n";

    unsafe {
        // System call: write(1, message, len)
        let _result: i64;
        core::arch::asm!(
            "mov rax, 1",           // syscall number: write
            "mov rdi, 1",           // fd: stdout
            "mov rsi, {msg}",       // buffer
            "mov rdx, {len}",       // length
            "int 0x80",             // trigger syscall
            msg = in(reg) message.as_ptr(),
            len = in(reg) message.len(),
            lateout("rax") _result,
        );
    }

    // Loop forever
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
