// System call interface for ASTRA.OS
// Implements POSIX-like syscalls for Ring 3 userspace

/// Syscall numbers (Linux-compatible)
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum SyscallNumber {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Exit = 60,
    Brk = 12,
    Mmap = 9,
    Munmap = 11,
    GetPid = 39,
}

impl SyscallNumber {
    pub fn from_usize(n: usize) -> Option<Self> {
        match n {
            0 => Some(Self::Read),
            1 => Some(Self::Write),
            2 => Some(Self::Open),
            3 => Some(Self::Close),
            60 => Some(Self::Exit),
            12 => Some(Self::Brk),
            9 => Some(Self::Mmap),
            11 => Some(Self::Munmap),
            39 => Some(Self::GetPid),
            _ => None,
        }
    }
}

/// Syscall arguments extracted from registers
#[derive(Debug)]
pub struct SyscallArgs {
    pub syscall_num: usize,  // RAX
    pub arg1: usize,          // RDI
    pub arg2: usize,          // RSI
    pub arg3: usize,          // RDX
    pub arg4: usize,          // R10
    pub arg5: usize,          // R8
    pub arg6: usize,          // R9
}

/// Syscall dispatcher - called from interrupt handler
///
/// This function dispatches syscall to appropriate handler based on
/// arguments extracted from registers.
///
/// # Arguments
/// - syscall_num: RAX - syscall number
/// - arg1: RDI - first argument
/// - arg2: RSI - second argument
/// - arg3: RDX - third argument
/// - arg4: R10 - fourth argument
/// - arg5: R8 - fifth argument
/// - arg6: R9 - sixth argument
///
/// # Returns
/// Return value to be placed in RAX for userspace
pub fn handle_syscall(
    syscall_num: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    let args = SyscallArgs {
        syscall_num,
        arg1,
        arg2,
        arg3,
        arg4,
        arg5,
        arg6,
    };

    dispatch_syscall(&args)
}

/// Dispatch syscall to appropriate handler
fn dispatch_syscall(args: &SyscallArgs) -> isize {
    match SyscallNumber::from_usize(args.syscall_num) {
        Some(SyscallNumber::Write) => sys_write(args.arg1, args.arg2, args.arg3),
        Some(SyscallNumber::Read) => sys_read(args.arg1, args.arg2, args.arg3),
        Some(SyscallNumber::Exit) => sys_exit(args.arg1 as i32),
        Some(SyscallNumber::Brk) => sys_brk(args.arg1),
        Some(SyscallNumber::GetPid) => sys_getpid(),
        Some(_) => {
            crate::serial_println!("[SYSCALL] Unimplemented: {:?}", args.syscall_num);
            -1 // ENOSYS
        }
        None => {
            crate::serial_println!("[SYSCALL] Invalid syscall number: {}", args.syscall_num);
            -1 // EINVAL
        }
    }
}

// =============================================================================
// Syscall Implementations
// =============================================================================

/// sys_write - write to file descriptor
///
/// # Arguments
/// - fd: file descriptor (1 = stdout, 2 = stderr)
/// - buf: pointer to buffer (userspace address)
/// - count: number of bytes to write
///
/// # Returns
/// Number of bytes written, or negative error code
fn sys_write(fd: usize, buf: usize, count: usize) -> isize {
    // Only support stdout/stderr for now
    if fd != 1 && fd != 2 {
        return -9; // EBADF
    }

    if count == 0 {
        return 0;
    }

    // TODO: Validate userspace pointer is accessible
    // For now, trust the pointer (UNSAFE!)

    unsafe {
        let slice = core::slice::from_raw_parts(buf as *const u8, count);

        // Try to convert to UTF-8 string
        match core::str::from_utf8(slice) {
            Ok(s) => {
                crate::serial_print!("{}", s);
                count as isize
            }
            Err(_) => {
                // Not valid UTF-8, print as bytes
                for &b in slice {
                    crate::serial_print!("{:02x} ", b);
                }
                crate::serial_println!();
                count as isize
            }
        }
    }
}

/// sys_read - read from file descriptor
///
/// # Arguments
/// - fd: file descriptor
/// - buf: pointer to buffer (userspace address)
/// - count: number of bytes to read
///
/// # Returns
/// Number of bytes read, or negative error code
fn sys_read(_fd: usize, _buf: usize, _count: usize) -> isize {
    // TODO: Implement keyboard input
    crate::serial_println!("[SYSCALL] sys_read not implemented");
    -1 // ENOSYS
}

/// sys_exit - terminate process
///
/// # Arguments
/// - status: exit status code
///
/// # Returns
/// Does not return
fn sys_exit(status: i32) -> ! {
    crate::serial_println!("[SYSCALL] Process exiting with status {}", status);

    // For now, just halt the CPU
    // In a full implementation, this would:
    // 1. Clean up process resources
    // 2. Remove from scheduler
    // 3. Switch to next process
    loop {
        x86_64::instructions::hlt();
    }
}

/// sys_brk - change data segment size (heap allocation)
///
/// # Arguments
/// - addr: new break address (0 = query current break)
///
/// # Returns
/// New break address, or negative error code
fn sys_brk(addr: usize) -> isize {
    static mut HEAP_END: usize = 0x50000000; // Start at 1.25 GB

    unsafe {
        if addr == 0 {
            // Query current break
            return HEAP_END as isize;
        }

        // TODO: Validate address is in valid range
        // TODO: Map pages if necessary

        let old_end = HEAP_END;
        HEAP_END = addr;

        crate::serial_println!("[SYSCALL] brk: {:#x} -> {:#x}", old_end, HEAP_END);

        HEAP_END as isize
    }
}

/// sys_getpid - get process ID
///
/// # Returns
/// Process ID (always 1 for now)
fn sys_getpid() -> isize {
    1 // Single process for now
}

// =============================================================================
// Helper functions
// =============================================================================

/// Validate that a userspace pointer is accessible
///
/// TODO: Implement proper validation by checking page tables
#[allow(dead_code)]
fn validate_user_ptr(_ptr: usize, _len: usize) -> bool {
    // For now, assume all pointers are valid
    // This is VERY UNSAFE and should be fixed!
    true
}
