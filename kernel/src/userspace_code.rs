// Userspace code that will run in Ring 3
// This is embedded in the kernel but executed in user mode

use core::arch::asm;
use alloc::string::String;
use crate::simple_html;

/// Entry point for userspace
/// This function will be called from kernel after switching to Ring 3
#[no_mangle]
pub extern "C" fn userspace_main() -> ! {
    syscall_write(1, b"\n========================================\n");
    syscall_write(1, b"  ASTRA.OS BROWSER - Ring 3 Userspace\n");
    syscall_write(1, b"========================================\n\n");

    syscall_write(1, b"Initializing HTML renderer...\n\n");

    // Test HTML content
    let html = r#"
    <html>
        <head>
            <title>ASTRA.OS Browser</title>
        </head>
        <body>
            <h1>Welcome to ASTRA.OS!</h1>
            <p>This is a browser running in Ring 3 userspace.</p>
            <p>HTML parsing is working!</p>
            <div>
                <p>Nested content works too.</p>
            </div>
        </body>
    </html>
    "#;

    syscall_write(1, b"Parsing HTML...\n");
    let dom = simple_html::parse_html(html);

    syscall_write(1, b"\nRendered output:\n");
    syscall_write(1, b"----------------\n");
    simple_html::render_html(&dom, 0);
    syscall_write(1, b"----------------\n\n");

    syscall_write(1, b"Browser is running in userspace!\n");
    syscall_write(1, b"TODO: Add Servo for full browser engine\n\n");

    // Main loop
    let mut counter = 0;
    loop {
        if counter % 10000000 == 0 {
            syscall_write(1, b"Browser heartbeat...\n");
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
