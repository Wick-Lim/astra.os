// Userspace code that will run in Ring 3
// This is embedded in the kernel but executed in user mode

use core::arch::asm;

/// Entry point for userspace - Simple test
#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn userspace_main() -> ! {
    use core::arch::naked_asm;
    unsafe {
        naked_asm!(
            // sys_write(1, "Hello from Ring 3!\n", 20)
            "mov rax, 1",              
            "mov rdi, 1",              
            "lea rsi, [rip + msg]",    
            "mov rdx, 20",             
            "int 0x80",                

            // sys_exit(0)
            "mov rax, 60",             
            "mov rdi, 0",              
            "int 0x80",                

            "hlt",

            "msg:",
            ".ascii \"Hello from Ring 3!\\n\"",
        );
    }
}

/// Get the address of userspace_main
pub fn get_userspace_entry() -> u64 {
    userspace_main as u64
}
