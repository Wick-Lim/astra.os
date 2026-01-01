# Quick Start: Implementing ASTRA.OS std Backend

This guide helps you quickly understand and start implementing the Rust standard library backend for ASTRA.OS.

## Current Status (2026-01-01)

✅ **What's Working:**
- Basic file I/O stubs (hardcoded files)
- PIT-based monotonic time (Instant)
- Serial execution thread stubs
- stdout/stderr via serial port
- 7 syscalls implemented: read, write, open, close, brk, getpid, exit

⏳ **Immediate Next Steps:**
- VFS integration for real file I/O (Phase 1)
- Network syscalls for HTTP (Phase 2)
- Real threading with clone (Phase 3)

## Architecture in 30 Seconds

```
Servo → std::fs/net/thread → sys::pal::astra_os → Syscalls → Kernel
```

1. **Servo** uses standard Rust APIs (File::open, TcpStream, thread::spawn)
2. **std library** delegates to platform-specific code in `sys::pal::astra_os`
3. **Platform layer** makes syscalls (INT 0x80) with Linux-compatible numbers
4. **Kernel** handles syscalls and returns results

## File Locations

```
rust-std-fork/library/std/src/sys/pal/astra_os/
├── mod.rs          # Root module
├── fs.rs           # File I/O (PRIORITY: needs VFS integration)
├── net.rs          # TCP/UDP sockets (PRIORITY: needs implementation)
├── time.rs         # Instant/SystemTime (mostly done)
├── thread.rs       # Threading (stub, needs clone syscall)
├── sync.rs         # Mutex/RwLock (needs futex)
└── ... (11 more support modules)

kernel/src/
├── syscall/mod.rs  # Syscall dispatcher (7 implemented, ~15 needed)
├── fs/            # VFS layer (TAR filesystem ready)
├── network/       # Network stack (HTTP parser ready)
└── process.rs     # Thread scheduler (basic structure exists)
```

## Phase 1: VFS Integration (Start Here!)

**Goal:** Replace hardcoded files with real syscall-based I/O

**Time Estimate:** 1-2 days

### Step 1: Add sys_lseek to kernel (2 hours)

**File:** `kernel/src/syscall/mod.rs`

```rust
// Add to SyscallNumber enum
Lseek = 8,

// Add to dispatch_syscall
Some(SyscallNumber::Lseek) => sys_lseek(args.arg1, args.arg2 as isize, args.arg3),

// Implement handler
fn sys_lseek(fd: usize, offset: isize, whence: usize) -> isize {
    const SEEK_SET: usize = 0;
    const SEEK_CUR: usize = 1;
    const SEEK_END: usize = 2;

    match crate::fs::lseek(crate::fs::FileDescriptor(fd), offset, whence as u8) {
        Ok(new_pos) => new_pos as isize,
        Err(_) => -9, // EBADF
    }
}
```

**File:** `kernel/src/fs/vfs.rs`

```rust
pub fn lseek(fd: FileDescriptor, offset: isize, whence: u8) -> Result<usize, &'static str> {
    let mut table = FILE_TABLE.lock();
    let file = table.get_mut(fd.0).ok_or("Invalid FD")?;

    const SEEK_SET: u8 = 0;
    const SEEK_CUR: u8 = 1;
    const SEEK_END: u8 = 2;

    let new_pos = match whence {
        SEEK_SET => offset,
        SEEK_CUR => file.position as isize + offset,
        SEEK_END => file.data.len() as isize + offset,
        _ => return Err("Invalid whence"),
    };

    if new_pos < 0 {
        return Err("Negative seek position");
    }

    file.position = new_pos as usize;
    Ok(new_pos as usize)
}
```

### Step 2: Update std fs.rs (4 hours)

**File:** `rust-std-fork/library/std/src/sys/pal/astra_os/fs.rs`

Replace hardcoded File implementation with syscall-based:

```rust
pub struct File {
    fd: RawFd,
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        use crate::sys::pal::astra_os::cvt;

        // Convert path to C string
        let path_cstr = path.to_c_str()?;

        // Build flags
        let flags = if opts.write { 1 } else { 0 };

        // Make syscall
        let fd = unsafe {
            syscall!(OPEN, path_cstr.as_ptr(), flags)
        };

        cvt(fd)?;
        Ok(File { fd: fd as RawFd })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe {
            syscall!(READ, self.fd, buf.as_mut_ptr(), buf.len())
        };

        cvt(ret)?;
        Ok(ret as usize)
    }

    pub fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (whence, offset) = match pos {
            SeekFrom::Start(off) => (SEEK_SET, off as i64),
            SeekFrom::End(off) => (SEEK_END, off),
            SeekFrom::Current(off) => (SEEK_CUR, off),
        };

        let ret = unsafe {
            syscall!(LSEEK, self.fd, offset, whence)
        };

        cvt(ret)?;
        Ok(ret as u64)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            let _ = syscall!(CLOSE, self.fd);
        }
    }
}
```

### Step 3: Add syscall macro (1 hour)

**File:** `rust-std-fork/library/std/src/sys/pal/astra_os/mod.rs`

```rust
// Syscall numbers
pub const SYS_READ: usize = 0;
pub const SYS_WRITE: usize = 1;
pub const SYS_OPEN: usize = 2;
pub const SYS_CLOSE: usize = 3;
pub const SYS_LSEEK: usize = 8;

// Raw syscall functions
#[inline(always)]
pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "mov rax, {syscall_n}",
        "mov rdi, {arg1}",
        "mov rsi, {arg2}",
        "int 0x80",
        "mov {ret}, rax",
        syscall_n = in(reg) n,
        arg1 = in(reg) arg1,
        arg2 = in(reg) arg2,
        ret = out(reg) ret,
        lateout("rax") _,
        lateout("rdi") _,
        lateout("rsi") _,
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "mov rax, {syscall_n}",
        "mov rdi, {arg1}",
        "mov rsi, {arg2}",
        "mov rdx, {arg3}",
        "int 0x80",
        "mov {ret}, rax",
        syscall_n = in(reg) n,
        arg1 = in(reg) arg1,
        arg2 = in(reg) arg2,
        arg3 = in(reg) arg3,
        ret = out(reg) ret,
        lateout("rax") _,
        lateout("rdi") _,
        lateout("rsi") _,
        lateout("rdx") _,
    );
    ret
}

// Convenience macro
macro_rules! syscall {
    (READ, $fd:expr, $buf:expr, $count:expr) => {
        syscall3(SYS_READ, $fd as usize, $buf as usize, $count)
    };
    (WRITE, $fd:expr, $buf:expr, $count:expr) => {
        syscall3(SYS_WRITE, $fd as usize, $buf as usize, $count)
    };
    (OPEN, $path:expr, $flags:expr) => {
        syscall2(SYS_OPEN, $path as usize, $flags)
    };
    (CLOSE, $fd:expr) => {
        syscall1(SYS_CLOSE, $fd as usize)
    };
    (LSEEK, $fd:expr, $offset:expr, $whence:expr) => {
        syscall3(SYS_LSEEK, $fd as usize, $offset as usize, $whence)
    };
}
pub(crate) use syscall;

// Error conversion
#[inline]
pub fn cvt(result: isize) -> io::Result<isize> {
    if result < 0 {
        Err(io::Error::from_raw_os_error((-result) as i32))
    } else {
        Ok(result)
    }
}
```

### Step 4: Test (2 hours)

**Build and test:**

```bash
# Build kernel
cd kernel
cargo build --release

# Test in QEMU
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-astra_os/release/bootimage-astra_os.bin \
    -serial mon:stdio \
    -display none

# Expected output:
# [SYSCALL] sys_open: opening 'index.html'
# [SYSCALL] sys_open: opened as FD 3
# [SYSCALL] sys_read: read 523 bytes from FD 3
# [SYSCALL] sys_close: FD 3 closed successfully
```

**Write userspace test:**

```rust
// kernel/src/userspace_code.rs
fn test_file_io() {
    // This code runs in Ring 3
    extern "C" {
        fn sys_open(path: *const u8, flags: i32) -> i32;
        fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize;
        fn sys_close(fd: i32) -> i32;
    }

    let path = b"index.html\0";
    let fd = unsafe { sys_open(path.as_ptr(), 0) };

    if fd > 0 {
        let mut buffer = [0u8; 512];
        let n = unsafe { sys_read(fd, buffer.as_mut_ptr(), buffer.len()) };

        if n > 0 {
            serial_println!("Read {} bytes from index.html", n);
            // Parse as UTF-8 and print
        }

        unsafe { sys_close(fd) };
    }
}
```

## Phase 2: Networking (After Phase 1)

**Goal:** Enable HTTP requests for Servo

**Key syscalls to implement:**
- `sys_socket(41)` - Create socket
- `sys_connect(42)` - Connect to server
- `sys_send(44)` / `sys_write(1)` - Send data
- `sys_recv(45)` / `sys_read(0)` - Receive data

**Key files to modify:**
- `kernel/src/syscall/mod.rs` - Add 4 network syscalls
- `kernel/src/network/mod.rs` - Implement socket table
- `rust-std-fork/library/std/src/sys/pal/astra_os/net.rs` - Implement TcpStream

## Phase 3: Threading (After Phase 2)

**Goal:** Real parallelism with clone syscall

**Key syscalls to implement:**
- `sys_clone(56)` - Create thread
- `sys_futex(202)` - Synchronization primitive

**Key changes:**
- Thread creation in kernel
- Context switching
- Scheduler

## Common Issues & Solutions

### Issue: "Invalid syscall number"
**Solution:** Make sure syscall number matches between std and kernel:
```rust
// std side
pub const SYS_READ: usize = 0;

// kernel side
Read = 0,
```

### Issue: "EBADF (Bad file descriptor)"
**Solution:** Check FD table in kernel:
```rust
// kernel/src/fs/vfs.rs
serial_println!("FD table state: {:?}", FILE_TABLE.lock());
```

### Issue: "Page fault in userspace"
**Solution:** Make sure userspace pages are marked USER_ACCESSIBLE:
```rust
// kernel/src/memory/mod.rs
mark_data_page_user_accessible(page);
```

### Issue: "Servo won't compile"
**Solution:** Check that sys::pal::astra_os exports match std's expectations:
```bash
# Compare with Unix backend
diff rust-std-fork/library/std/src/sys/pal/{unix,astra_os}/fs.rs
```

## Testing Strategy

1. **Unit tests in kernel:**
   ```rust
   #[test_case]
   fn test_sys_open() {
       let fd = sys_open(b"/index.html\0".as_ptr() as usize, 0);
       assert!(fd > 0);
   }
   ```

2. **Integration tests in userspace:**
   ```rust
   fn userspace_main() {
       test_file_io();
       test_timing();
       test_threading();
   }
   ```

3. **Servo tests:**
   ```bash
   cd servo
   cargo +astra_os test --target x86_64-astra_os
   ```

## Debugging Tips

1. **Enable verbose syscall logging:**
   ```rust
   // kernel/src/syscall/mod.rs
   serial_println!("[SYSCALL] {} called with args: {:?}", name, args);
   ```

2. **Trace execution flow:**
   ```rust
   serial_println!("[{}:{}] {}", file!(), line!(), msg);
   ```

3. **Use QEMU's built-in debugger:**
   ```bash
   qemu-system-x86_64 ... -s -S
   # In another terminal:
   gdb target/.../kernel
   (gdb) target remote :1234
   (gdb) break syscall_handler
   (gdb) continue
   ```

4. **Check register state:**
   ```rust
   // In syscall handler
   serial_println!("RAX={:#x} RDI={:#x} RSI={:#x}", rax, rdi, rsi);
   ```

## Performance Goals

| Operation          | Target Latency | Current |
|--------------------|----------------|---------|
| Syscall overhead   | < 1 µs         | ~0.1 µs ✅ |
| File open          | < 5 µs         | ~10 µs  |
| File read (1KB)    | < 2 µs         | ~3 µs   |
| TCP connect        | < 100 ms       | TBD     |
| Thread spawn       | < 100 µs       | TBD     |
| Context switch     | < 5 µs         | TBD     |

## Success Metrics

**Phase 1 Complete When:**
- ✅ Servo can open files from TAR filesystem
- ✅ File::read() works without errors
- ✅ File::seek() allows random access
- ✅ Error codes propagate correctly

**Phase 2 Complete When:**
- ✅ TcpStream::connect() succeeds
- ✅ HTTP GET request completes
- ✅ Response body is parsed correctly
- ✅ Servo can fetch external CSS/JS

**Phase 3 Complete When:**
- ✅ Multiple threads execute in parallel
- ✅ Mutex prevents data races
- ✅ Servo's parallel layout works
- ✅ Performance improves 2-4x

## Resources

- **Full Design Doc:** `/Users/wick/Documents/workspaces/astra.os/STD_BACKEND_DESIGN.md`
- **Architecture Diagram:** `/Users/wick/Documents/workspaces/astra.os/docs/std-backend-architecture.txt`
- **Kernel Source:** `kernel/src/`
- **std Backend:** `rust-std-fork/library/std/src/sys/pal/astra_os/`
- **Linux Syscall Reference:** https://man7.org/linux/man-pages/man2/syscalls.2.html

## Quick Reference: Syscall Numbers

```rust
// Priority 0 (implemented)
SYS_READ      = 0
SYS_WRITE     = 1
SYS_OPEN      = 2
SYS_CLOSE     = 3
SYS_BRK       = 12
SYS_GETPID    = 39
SYS_EXIT      = 60

// Priority 1 (next)
SYS_LSEEK     = 8   // Phase 1
SYS_FSTAT     = 5   // Phase 1
SYS_SOCKET    = 41  // Phase 2
SYS_CONNECT   = 42  // Phase 2
SYS_SEND      = 44  // Phase 2
SYS_RECV      = 45  // Phase 2

// Priority 2 (later)
SYS_CLONE     = 56   // Phase 3
SYS_GETTID    = 186  // Phase 3
SYS_FUTEX     = 202  // Phase 3
```

## Need Help?

1. Check the full design document for detailed specifications
2. Look at existing implementations in `sys/pal/unix/` for reference
3. Use serial_println! liberally for debugging
4. Test each syscall in isolation before integration

---

**Start with Phase 1, VFS integration!** This is the highest priority and will immediately enable Servo to load HTML files from the TAR filesystem.

Good luck! 🦀
