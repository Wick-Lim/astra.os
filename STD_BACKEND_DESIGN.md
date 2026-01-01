# ASTRA.OS Rust Standard Library Backend Design

**Version:** 1.0
**Date:** 2026-01-01
**Status:** Design Phase - Implementation Ready

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [Module Dependency Graph](#module-dependency-graph)
4. [Syscall Interface](#syscall-interface)
5. [Module Specifications](#module-specifications)
6. [Implementation Roadmap](#implementation-roadmap)
7. [Code Examples](#code-examples)
8. [Testing Strategy](#testing-strategy)

---

## Executive Summary

This document defines the complete architecture for ASTRA.OS's Rust standard library backend (`sys/pal/astra_os`). The backend bridges Rust's `std` library with ASTRA.OS kernel syscalls, enabling Servo browser engine and other Rust applications to run natively.

**Key Goals:**
- Enable Servo to run with minimal modifications
- Provide POSIX-like syscall interface for compatibility
- Support VFS-based file I/O via TAR filesystem
- Implement monotonic time via PIT timer
- Enable basic threading (initially serial, then parallel)
- Network stack integration for HTTP/HTTPS

**Current State:**
- Basic stubs exist in `/rust-std-fork/library/std/src/sys/pal/astra_os/`
- Kernel has syscalls: read, write, open, close, exit, brk, getpid
- VFS layer with TAR filesystem is operational
- Network stack (URL parser, HTTP client) is ready
- PIT timer runs at 1000 Hz (1ms resolution)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Servo Browser Engine                     │
│                    (Rust Application Layer)                  │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            │ uses std::*
                            │
┌───────────────────────────▼─────────────────────────────────┐
│              Rust Standard Library (std)                     │
│  ┌─────────┬──────────┬─────────┬──────────┬──────────┐    │
│  │ std::fs │ std::net │std::time│std::thread│std::sync │    │
│  └────┬────┴────┬─────┴────┬────┴────┬─────┴────┬─────┘    │
└───────┼─────────┼──────────┼─────────┼──────────┼──────────┘
        │         │          │         │          │
        │         │          │         │          │
┌───────▼─────────▼──────────▼─────────▼──────────▼──────────┐
│          sys::pal::astra_os (Platform Abstraction)           │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐  │
│  │   fs.rs  │  net.rs  │ time.rs  │thread.rs │ sync.rs  │  │
│  └─────┬────┴─────┬────┴─────┬────┴─────┬────┴─────┬────┘  │
└────────┼──────────┼──────────┼──────────┼──────────┼───────┘
         │          │          │          │          │
         │ syscall  │ syscall  │ rdtsc/   │ syscall  │ futex
         │          │          │ PIT      │          │ (future)
         │          │          │          │          │
┌────────▼──────────▼──────────▼──────────▼──────────▼───────┐
│                    ASTRA.OS Kernel (Ring 0)                  │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐  │
│  │VFS (TAR) │  Network │   PIT    │ Process  │  Futex   │  │
│  │          │  Stack   │  Timer   │  Mgmt    │ (future) │  │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Design Principles:**
1. **Compatibility First**: Match POSIX semantics where possible
2. **Zero-Copy**: Use direct syscalls without buffering layers
3. **Minimal Overhead**: Thin abstraction over kernel services
4. **Progressive Enhancement**: Start with stubs, add real implementations incrementally
5. **Safety**: Maintain Rust's safety guarantees at std level

---

## Module Dependency Graph

```
Legend: [Module] --depends-on--> [Module]
        [Module] ==syscall===> [Kernel Feature]

┌─────────────────────────────────────────────────────────┐
│                      Core Modules                        │
└─────────────────────────────────────────────────────────┘

[mod.rs] (root)
    ├──> [os.rs]        (errno, signal handling)
    ├──> [path.rs]      (path manipulation)
    ├──> [args.rs]      (command line args)
    ├──> [env.rs]       (environment variables)
    └──> [random.rs]    (PRNG for HashMap keys)

┌─────────────────────────────────────────────────────────┐
│                    I/O Subsystem                         │
└─────────────────────────────────────────────────────────┘

[fs.rs]
    ├──> [os.rs]        (error mapping)
    └==> [VFS syscalls] (open, read, close, seek)

[stdio.rs]
    ├──> [fs.rs]        (File handles)
    └==> [write syscall] (stdout, stderr)

[io.rs]
    ├──> [fs.rs]
    └──> [stdio.rs]

┌─────────────────────────────────────────────────────────┐
│                 Concurrency Subsystem                    │
└─────────────────────────────────────────────────────────┘

[thread.rs]
    ├──> [time.rs]      (sleep, timeouts)
    └==> [clone syscall] (thread creation - Phase 2)

[sync.rs]
    ├──> [thread.rs]
    └==> [futex syscall] (Mutex, Condvar - Phase 3)

┌─────────────────────────────────────────────────────────┐
│                  Timing Subsystem                        │
└─────────────────────────────────────────────────────────┘

[time.rs]
    ├──> [os.rs]
    └==> [PIT timer]     (monotonic time)
    └==> [RTC syscall]   (wall clock - Phase 2)

┌─────────────────────────────────────────────────────────┐
│                 Networking Subsystem                     │
└─────────────────────────────────────────────────────────┘

[net.rs]
    ├──> [io.rs]
    ├──> [os.rs]
    └==> [socket syscalls] (socket, connect, send, recv)

┌─────────────────────────────────────────────────────────┐
│                 Process Management                       │
└─────────────────────────────────────────────────────────┘

[process.rs]
    ├──> [fs.rs]        (stdio redirection)
    ├──> [env.rs]
    └==> [fork, exec, wait syscalls] (process control)
```

**Critical Dependencies:**
- All I/O modules depend on proper errno mapping in `os.rs`
- Threading depends on time.rs for sleep/timeouts
- Network I/O needs both net.rs and io.rs coordination
- Process spawning needs both process.rs and thread.rs

---

## Syscall Interface

### Current Kernel Syscalls (Implemented)

| Syscall # | Name    | Arguments                    | Returns        | Status |
|-----------|---------|------------------------------|----------------|--------|
| 0         | read    | fd, *buf, count              | bytes_read     | ✅ VFS |
| 1         | write   | fd, *buf, count              | bytes_written  | ✅ Serial |
| 2         | open    | *path, flags                 | fd             | ✅ VFS |
| 3         | close   | fd                           | 0/-errno       | ✅ VFS |
| 12        | brk     | addr                         | new_brk        | ✅ Heap |
| 39        | getpid  | -                            | pid            | ✅ Basic |
| 60        | exit    | status                       | (no return)    | ✅ Basic |

### Required Syscalls for Full std Support

#### Phase 1: File I/O & Basic Operations (Current)
```c
// Already implemented
ssize_t sys_read(int fd, void *buf, size_t count);
ssize_t sys_write(int fd, const void *buf, size_t count);
int sys_open(const char *pathname, int flags);
int sys_close(int fd);
int sys_brk(void *addr);
pid_t sys_getpid(void);
void sys_exit(int status);
```

#### Phase 2: Enhanced File I/O
```c
// Syscall #8
off_t sys_lseek(int fd, off_t offset, int whence);
// Returns: new file offset, or -1 on error

// Syscall #4
struct stat {
    uint64_t st_size;
    uint64_t st_mode;
    uint64_t st_mtime;
};
int sys_stat(const char *pathname, struct stat *buf);
// Returns: 0 on success, -1 on error

// Syscall #5
int sys_fstat(int fd, struct stat *buf);

// Syscall #78
int sys_getdents(int fd, struct dirent *dirp, size_t count);
// For directory iteration
```

#### Phase 3: Time & Clock
```c
// Syscall #96
struct timespec {
    uint64_t tv_sec;   // seconds
    uint64_t tv_nsec;  // nanoseconds
};
int sys_clock_gettime(int clock_id, struct timespec *tp);
// clock_id: CLOCK_MONOTONIC=1, CLOCK_REALTIME=0

// Syscall #228
int sys_clock_getres(int clock_id, struct timespec *res);

// Syscall #35
int sys_nanosleep(const struct timespec *req, struct timespec *rem);
```

#### Phase 4: Threading
```c
// Syscall #56
pid_t sys_clone(unsigned long flags, void *stack,
                int *parent_tid, int *child_tid,
                unsigned long tls);
// Flags: CLONE_VM, CLONE_FS, CLONE_FILES, CLONE_THREAD, etc.
// Returns: thread ID in parent, 0 in child

// Syscall #202 (futex for synchronization)
int sys_futex(int *uaddr, int op, int val,
              const struct timespec *timeout,
              int *uaddr2, int val3);
// For Mutex, RwLock, Condvar implementation

// Syscall #186
int sys_gettid(void);
// Get thread ID

// Syscall #218
int sys_set_tid_address(int *tidptr);
```

#### Phase 5: Networking
```c
// Syscall #41
int sys_socket(int domain, int type, int protocol);
// domain: AF_INET=2, type: SOCK_STREAM=1

// Syscall #42
int sys_connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen);

// Syscall #43
int sys_accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen);

// Syscall #44
int sys_sendto(int sockfd, const void *buf, size_t len, int flags,
               const struct sockaddr *dest_addr, socklen_t addrlen);

// Syscall #45
int sys_recvfrom(int sockfd, void *buf, size_t len, int flags,
                 struct sockaddr *src_addr, socklen_t *addrlen);

// Syscall #48
int sys_shutdown(int sockfd, int how);

// Syscall #49
int sys_bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen);

// Syscall #50
int sys_listen(int sockfd, int backlog);
```

#### Phase 6: Process Management
```c
// Syscall #57
int sys_fork(void);
// Returns: child PID in parent, 0 in child

// Syscall #59
int sys_execve(const char *pathname, char *const argv[], char *const envp[]);

// Syscall #61
int sys_wait4(pid_t pid, int *status, int options, struct rusage *rusage);
```

### Syscall Mapping Table

| std Function             | Syscall(s)                           | Priority | Notes |
|--------------------------|--------------------------------------|----------|-------|
| **File I/O**             |                                      |          |       |
| File::open()             | open(2)                              | P0       | ✅ Done |
| File::read()             | read(0)                              | P0       | ✅ Done |
| File::write()            | write(1)                             | P0       | Serial only |
| File::seek()             | lseek(8)                             | P1       | Needed for HTTP |
| File::metadata()         | stat(4), fstat(5)                    | P1       | For file info |
| read_dir()               | getdents(78)                         | P2       | Directory listing |
| **Stdio**                |                                      |          |       |
| println!()               | write(1, STDOUT)                     | P0       | ✅ Done |
| eprintln!()              | write(1, STDERR)                     | P0       | ✅ Done |
| stdin.read()             | read(0, STDIN)                       | P1       | Keyboard input |
| **Time**                 |                                      |          |       |
| Instant::now()           | rdtsc or PIT counter                 | P0       | ✅ PIT done |
| SystemTime::now()        | clock_gettime(96, REALTIME)          | P1       | RTC needed |
| Duration                 | (computed)                           | P0       | ✅ Done |
| thread::sleep()          | nanosleep(35)                        | P1       | Busy-wait OK for now |
| **Threading**            |                                      |          |       |
| thread::spawn()          | clone(56)                            | P2       | Serial stub exists |
| thread::current()        | gettid(186)                          | P2       | Returns 1 for now |
| thread::yield_now()      | (hlt instruction)                    | P0       | ✅ Done |
| **Sync Primitives**      |                                      |          |       |
| Mutex::lock()            | futex(202, FUTEX_WAIT/WAKE)          | P2       | Spinlock stub OK |
| RwLock::read()           | futex(202)                           | P3       | Not critical |
| Condvar::wait()          | futex(202)                           | P3       | Not critical |
| **Networking**           |                                      |          |       |
| TcpStream::connect()     | socket(41), connect(42)              | P1       | HTTP needs this |
| TcpStream::read()        | recvfrom(45) or read(0)              | P1       | HTTP needs this |
| TcpStream::write()       | sendto(44) or write(1)               | P1       | HTTP needs this |
| TcpListener::bind()      | socket(41), bind(49), listen(50)     | P3       | Server mode |
| **Process**              |                                      |          |       |
| Command::spawn()         | fork(57), execve(59)                 | P3       | Not needed for Servo |
| process::exit()          | exit(60)                             | P0       | ✅ Done |
| process::id()            | getpid(39)                           | P0       | ✅ Done |
| **Memory**               |                                      |          |       |
| alloc/dealloc            | brk(12)                              | P0       | ✅ Done |
| mmap/munmap              | mmap(9), munmap(11)                  | P2       | Large allocations |

**Priority Levels:**
- **P0**: Critical for basic Servo operation (already implemented or stubbed)
- **P1**: Important for full Servo features (HTTP, file caching)
- **P2**: Nice to have for performance (real threads, mmap)
- **P3**: Future enhancements (server mode, process spawning)

---

## Module Specifications

### 1. fs.rs - File System Operations

**Purpose**: Bridge std::fs to VFS/TAR filesystem via syscalls

**Current State**:
- Hardcoded HTML files (index.html, test.html)
- Read-only operations
- No real VFS integration

**Syscall Dependencies**:
```rust
// Current
sys_open(pathname, flags) -> fd
sys_read(fd, buf, count) -> bytes_read
sys_close(fd) -> result

// Needed for Phase 1
sys_lseek(fd, offset, whence) -> new_offset
sys_fstat(fd, stat_buf) -> result
```

**Key Types**:
```rust
pub struct File {
    fd: RawFd,  // Change from in-memory buffer to real FD
}

pub struct FileAttr {
    size: u64,
    file_type: FileType,
    modified: SystemTime,
}

pub enum FileType {
    File,
    Dir,
    // Symlinks not supported initially
}

pub struct OpenOptions {
    read: bool,
    write: bool,
    // Other flags for future
}
```

**Implementation Plan**:

**Phase 1: VFS Integration** (Priority: HIGH)
```rust
impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        // Convert path to C string
        let path_cstr = path_to_cstr(path)?;

        // Build flags (O_RDONLY=0, O_WRONLY=1, O_RDWR=2)
        let flags = if opts.write { 1 } else { 0 };

        // Make syscall
        let fd = unsafe {
            syscall2(SYS_OPEN, path_cstr.as_ptr() as usize, flags)
        };

        if fd < 0 {
            return Err(io::Error::from_raw_os_error(-fd as i32));
        }

        Ok(File { fd: fd as RawFd })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result = unsafe {
            syscall3(SYS_READ, self.fd as usize, buf.as_ptr() as usize, buf.len())
        };

        if result < 0 {
            Err(io::Error::from_raw_os_error(-result as i32))
        } else {
            Ok(result as usize)
        }
    }

    // Similar for write, seek, etc.
}
```

**Phase 2: Directory Operations**
```rust
pub fn read_dir(path: &Path) -> io::Result<ReadDir> {
    // Needs sys_getdents syscall
    // Iterate through TAR filesystem entries
}
```

**Testing**:
```rust
// Test 1: Read existing TAR file
let mut file = File::open("/index.html")?;
let mut contents = String::new();
file.read_to_string(&mut contents)?;
assert!(contents.contains("<html>"));

// Test 2: Error handling
assert!(File::open("/nonexistent").is_err());

// Test 3: Seek operations
file.seek(SeekFrom::Start(10))?;
```

---

### 2. time.rs - Timing and Clocks

**Purpose**: Monotonic and wall-clock time via PIT timer and RTC

**Current State**:
- PIT-based Instant (monotonic)
- Hardcoded SystemTime (fixed at 2026)
- 1ms resolution from PIT running at 1000 Hz

**Syscall Dependencies**:
```rust
// Phase 1: PIT-based (current)
static TICKS: AtomicU64;  // Updated by timer interrupt

// Phase 2: Proper syscalls
sys_clock_gettime(CLOCK_MONOTONIC, &timespec) -> result
sys_clock_gettime(CLOCK_REALTIME, &timespec) -> result
sys_nanosleep(&timespec, &remaining) -> result
```

**Key Types**:
```rust
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    ticks: u64,  // PIT ticks since boot
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemTime {
    seconds: u64,  // Seconds since UNIX epoch
    nanos: u32,    // Nanosecond component
}
```

**Implementation Plan**:

**Phase 1: Improve PIT-based timing** (Current)
```rust
// Kernel timer interrupt handler updates this
#[no_mangle]
pub static TICKS: AtomicU64 = AtomicU64::new(0);

impl Instant {
    pub fn now() -> Instant {
        Instant {
            ticks: TICKS.load(Ordering::Relaxed),
        }
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        let tick_diff = self.ticks.saturating_sub(earlier.ticks);
        // PIT runs at 1000 Hz (1ms per tick)
        Duration::from_millis(tick_diff)
    }
}
```

**Phase 2: RTC integration**
```rust
impl SystemTime {
    pub fn now() -> SystemTime {
        // Read RTC via CMOS ports or syscall
        let mut ts = timespec { tv_sec: 0, tv_nsec: 0 };
        unsafe {
            syscall2(SYS_CLOCK_GETTIME, CLOCK_REALTIME, &mut ts as *mut _ as usize);
        }
        SystemTime {
            seconds: ts.tv_sec,
            nanos: ts.tv_nsec as u32,
        }
    }
}
```

**Kernel Integration**:
```rust
// In kernel/src/interrupts/mod.rs
extern "x86-interrupt" fn timer_interrupt_handler(_sf: InterruptStackFrame) {
    // Update std's tick counter
    unsafe {
        let ticks_ptr = 0xDEADBEEF as *mut u64;  // Link-time address
        *ticks_ptr = (*ticks_ptr).wrapping_add(1);
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
```

**Testing**:
```rust
// Test 1: Monotonic time
let start = Instant::now();
busy_wait(10_000);  // 10ms
let elapsed = start.elapsed();
assert!(elapsed >= Duration::from_millis(9));
assert!(elapsed <= Duration::from_millis(11));

// Test 2: SystemTime
let now = SystemTime::now();
let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
assert!(since_epoch.as_secs() > 1_600_000_000);  // After 2020
```

---

### 3. thread.rs - Threading Support

**Purpose**: Multi-threading via clone syscall

**Current State**:
- Serial stub (executes immediately, no real threads)
- No TLS support
- Returns dummy thread IDs

**Syscall Dependencies**:
```rust
// Phase 1: Serial execution (current stub)
// No syscalls needed

// Phase 2: Real threads
sys_clone(flags, stack, parent_tid, child_tid, tls) -> tid
sys_gettid() -> tid
sys_futex(uaddr, op, val, timeout, uaddr2, val3) -> result
```

**Key Types**:
```rust
pub struct Thread {
    id: ThreadId,
    name: Option<CString>,
}

pub type ThreadId = NonZeroU64;

pub struct Builder {
    name: Option<CString>,
    stack_size: usize,
}
```

**Implementation Plan**:

**Phase 1: Keep serial stub** (Current - OK for initial Servo demo)
```rust
pub unsafe fn spawn<F>(f: F, stack_size: usize) -> io::Result<Thread>
where
    F: FnOnce() + 'static,
{
    // Execute immediately (no real parallelism)
    f();

    Ok(Thread {
        id: ThreadId::new(NEXT_ID.fetch_add(1, Ordering::Relaxed)),
        name: None,
    })
}
```

**Phase 2: Real threading with clone**
```rust
pub unsafe fn spawn<F>(f: F, stack_size: usize) -> io::Result<Thread>
where
    F: FnOnce() + 'static,
{
    // Allocate stack for new thread
    let stack = alloc_stack(stack_size)?;

    // Box the closure to pass to new thread
    let closure_box = Box::into_raw(Box::new(f));

    // Clone syscall with thread flags
    let flags = CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_THREAD | CLONE_SIGHAND;
    let tid = syscall5(
        SYS_CLONE,
        flags,
        stack.top() as usize,
        0,  // parent_tid
        0,  // child_tid
        0,  // tls
    );

    if tid < 0 {
        // Free stack and closure box on error
        dealloc_stack(stack);
        drop(Box::from_raw(closure_box));
        return Err(io::Error::from_raw_os_error(-tid as i32));
    }

    if tid == 0 {
        // Child thread: execute closure
        let f = Box::from_raw(closure_box);
        f();
        syscall1(SYS_EXIT, 0);  // Thread exit
        unreachable!();
    } else {
        // Parent thread: return handle
        Ok(Thread {
            id: ThreadId::new(tid as u64).unwrap(),
            name: None,
        })
    }
}
```

**Kernel Requirements** (Phase 2):
```rust
// kernel/src/syscall/mod.rs
fn sys_clone(flags: usize, stack: usize, parent_tid: usize,
             child_tid: usize, tls: usize) -> isize {
    let mut scheduler = SCHEDULER.lock();

    // Create new thread with shared address space
    let current = scheduler.current_process().unwrap();
    let new_thread = Thread {
        pid: scheduler.next_tid(),
        page_table: current.page_table.clone(),  // Shared!
        stack_pointer: VirtAddr::new(stack as u64),
        registers: current.registers.clone(),
    };

    scheduler.add_thread(new_thread);
    new_thread.pid as isize
}
```

**Testing**:
```rust
// Test 1: Serial execution (Phase 1)
let counter = Arc::new(AtomicUsize::new(0));
let c = counter.clone();
thread::spawn(move || {
    c.fetch_add(1, Ordering::SeqCst);
});
assert_eq!(counter.load(Ordering::SeqCst), 1);

// Test 2: Parallel execution (Phase 2)
let handles: Vec<_> = (0..4).map(|i| {
    thread::spawn(move || i * 2)
}).collect();
let results: Vec<_> = handles.into_iter()
    .map(|h| h.join().unwrap())
    .collect();
assert_eq!(results, vec![0, 2, 4, 6]);
```

---

### 4. net.rs - Network I/O

**Purpose**: TCP/UDP sockets for HTTP/HTTPS

**Current State**:
- No implementation (stub returns unsupported)
- Kernel has URL parser and HTTP request builder
- DummyDevice network stack exists

**Syscall Dependencies**:
```rust
sys_socket(domain, type, protocol) -> fd
sys_connect(sockfd, addr, addrlen) -> result
sys_send(sockfd, buf, len, flags) -> bytes_sent
sys_recv(sockfd, buf, len, flags) -> bytes_received
sys_close(sockfd) -> result
```

**Key Types**:
```rust
pub struct TcpStream {
    fd: RawFd,
}

pub struct TcpListener {
    fd: RawFd,
}

pub struct SocketAddr {
    ip: IpAddr,
    port: u16,
}

pub enum IpAddr {
    V4([u8; 4]),
    V6([u8; 16]),
}
```

**Implementation Plan**:

**Phase 1: Basic TCP client**
```rust
impl TcpStream {
    pub fn connect(addr: SocketAddr) -> io::Result<TcpStream> {
        // Create socket
        let fd = unsafe {
            syscall3(SYS_SOCKET, AF_INET, SOCK_STREAM, 0)
        };

        if fd < 0 {
            return Err(io::Error::from_raw_os_error(-fd as i32));
        }

        // Build sockaddr_in
        let sockaddr = match addr.ip {
            IpAddr::V4(octets) => {
                sockaddr_in {
                    sin_family: AF_INET,
                    sin_port: addr.port.to_be(),
                    sin_addr: u32::from_ne_bytes(octets),
                    sin_zero: [0; 8],
                }
            }
            _ => return Err(io::Error::new(ErrorKind::Unsupported, "IPv6 not supported")),
        };

        // Connect
        let result = unsafe {
            syscall3(
                SYS_CONNECT,
                fd as usize,
                &sockaddr as *const _ as usize,
                size_of::<sockaddr_in>(),
            )
        };

        if result < 0 {
            unsafe { syscall1(SYS_CLOSE, fd as usize) };
            return Err(io::Error::from_raw_os_error(-result as i32));
        }

        Ok(TcpStream { fd: fd as RawFd })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Use sys_recv or sys_read
        let result = unsafe {
            syscall3(SYS_READ, self.fd as usize, buf.as_ptr() as usize, buf.len())
        };

        if result < 0 {
            Err(io::Error::from_raw_os_error(-result as i32))
        } else {
            Ok(result as usize)
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result = unsafe {
            syscall3(SYS_WRITE, self.fd as usize, buf.as_ptr() as usize, buf.len())
        };

        if result < 0 {
            Err(io::Error::from_raw_os_error(-result as i32))
        } else {
            Ok(result as usize)
        }
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        unsafe {
            syscall1(SYS_CLOSE, self.fd as usize);
        }
    }
}
```

**Kernel Requirements**:
```rust
// kernel/src/syscall/mod.rs
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum SyscallNumber {
    // Existing...
    Socket = 41,
    Connect = 42,
    Sendto = 44,
    Recvfrom = 45,
    Shutdown = 48,
}

fn sys_socket(domain: usize, sock_type: usize, protocol: usize) -> isize {
    // Allocate network socket in kernel
    let socket = network::create_socket(domain, sock_type, protocol)?;

    // Register in process FD table
    let fd = register_fd(FileDescriptor::Socket(socket));
    fd as isize
}

fn sys_connect(fd: usize, addr: usize, addrlen: usize) -> isize {
    let socket = get_socket(fd)?;

    // Parse sockaddr
    let sockaddr = unsafe {
        &*(addr as *const SockaddrIn)
    };

    // Use kernel network stack to connect
    network::tcp_connect(socket, sockaddr.ip(), sockaddr.port())?;

    0
}
```

**Testing**:
```rust
// Test 1: HTTP GET request
let stream = TcpStream::connect("93.184.216.34:80".parse().unwrap())?;
stream.write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")?;
let mut response = Vec::new();
stream.read_to_end(&mut response)?;
assert!(response.starts_with(b"HTTP/1.1 200"));

// Test 2: Error handling
assert!(TcpStream::connect("0.0.0.0:1".parse().unwrap()).is_err());
```

---

### 5. sync.rs - Synchronization Primitives

**Purpose**: Mutex, RwLock, Condvar for thread synchronization

**Current State**:
- No implementation (not in existing files)
- Will need futex syscall for real threads

**Syscall Dependencies**:
```rust
// Phase 1: Spinlock (no syscall needed)
// Use atomic operations

// Phase 2: Futex-based
sys_futex(uaddr, FUTEX_WAIT, val, timeout, NULL, 0) -> result
sys_futex(uaddr, FUTEX_WAKE, count, NULL, NULL, 0) -> result
```

**Key Types**:
```rust
pub struct Mutex {
    futex: AtomicU32,  // 0 = unlocked, 1 = locked, 2+ = contended
}

pub struct RwLock {
    state: AtomicU32,  // High bit = writer, low bits = reader count
}

pub struct Condvar {
    futex: AtomicU32,
}
```

**Implementation Plan**:

**Phase 1: Spinlock Mutex** (for single-threaded / serial mode)
```rust
pub struct Mutex {
    locked: AtomicBool,
}

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex {
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) {
        // Simple spinlock
        while self.locked.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_err() {
            // Spin (or yield)
            core::hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}
```

**Phase 2: Futex-based Mutex** (for real multi-threading)
```rust
pub struct Mutex {
    futex: AtomicU32,
}

impl Mutex {
    pub fn lock(&self) {
        // Fast path: try to acquire
        if self.futex.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            return;
        }

        // Slow path: contended
        loop {
            // Mark as contended
            let old = self.futex.swap(2, Ordering::Acquire);

            if old == 0 {
                // We got the lock
                return;
            }

            // Wait in kernel
            unsafe {
                syscall6(SYS_FUTEX, &self.futex as *const _ as usize, FUTEX_WAIT, 2, 0, 0, 0);
            }
        }
    }

    pub fn unlock(&self) {
        if self.futex.swap(0, Ordering::Release) == 2 {
            // There were waiters, wake one
            unsafe {
                syscall6(SYS_FUTEX, &self.futex as *const _ as usize, FUTEX_WAKE, 1, 0, 0, 0);
            }
        }
    }
}
```

**Testing**:
```rust
// Test 1: Basic locking
let mutex = Mutex::new();
mutex.lock();
// Critical section
mutex.unlock();

// Test 2: Cross-thread (Phase 2 only)
let counter = Arc::new(Mutex::new(0));
let handles: Vec<_> = (0..10).map(|_| {
    let c = counter.clone();
    thread::spawn(move || {
        for _ in 0..100 {
            *c.lock() += 1;
        }
    })
}).collect();

for h in handles {
    h.join().unwrap();
}

assert_eq!(*counter.lock(), 1000);
```

---

### 6. io.rs - I/O Traits

**Purpose**: Common I/O trait implementations (Read, Write, Seek)

**Current State**:
- Minimal stub
- Relies on fs.rs and net.rs

**Implementation**:
```rust
// Already implemented by File and TcpStream via Read/Write traits
// Just need to ensure they implement the traits correctly

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read(buf)  // Delegate to File::read
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush()
    }
}

// Similar for TcpStream
```

---

### 7. stdio.rs - Standard I/O

**Purpose**: stdin, stdout, stderr

**Current State**:
- stdout/stderr use sys_write to serial port
- stdin not implemented

**Implementation**:
```rust
pub struct Stdin {
    // Use FD 0
}

pub struct Stdout {
    // Use FD 1
}

pub struct Stderr {
    // Use FD 2
}

impl Stdin {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let result = syscall3(SYS_READ, 0, buf.as_ptr() as usize, buf.len());
            if result < 0 {
                Err(io::Error::from_raw_os_error(-result as i32))
            } else {
                Ok(result as usize)
            }
        }
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            let result = syscall3(SYS_WRITE, 1, buf.as_ptr() as usize, buf.len());
            if result < 0 {
                Err(io::Error::from_raw_os_error(-result as i32))
            } else {
                Ok(result as usize)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())  // Serial port is unbuffered
    }
}

// Similar for Stderr (FD 2)
```

---

### 8. Supporting Modules

#### os.rs - OS-specific functionality
```rust
// Error number mapping
pub fn errno() -> i32 {
    // TODO: Store in thread-local storage
    0
}

pub fn is_interrupted(errno: i32) -> bool {
    errno == EINTR  // 4
}

// Error code constants
pub const EINTR: i32 = 4;
pub const EBADF: i32 = 9;
pub const ENOENT: i32 = 2;
pub const EACCES: i32 = 13;
// ... etc
```

#### path.rs - Path manipulation
```rust
// Use standard Unix path semantics
// Already implemented in existing stub
```

#### args.rs - Command line arguments
```rust
pub fn args() -> Args {
    // For now, return empty or hardcoded args
    Args {
        inner: vec![
            OsString::from("/servo"),
            OsString::from("--url"),
            OsString::from("file:///index.html"),
        ].into_iter()
    }
}
```

#### env.rs - Environment variables
```rust
pub fn vars() -> Vars {
    // Return empty for now
    Vars { inner: HashMap::new().into_iter() }
}

pub fn var(key: &str) -> Result<String, VarError> {
    // Hardcode a few useful ones
    match key {
        "PATH" => Ok("/bin:/usr/bin".to_string()),
        "HOME" => Ok("/".to_string()),
        _ => Err(VarError::NotPresent),
    }
}
```

#### random.rs - PRNG
```rust
pub fn fill_bytes(buf: &mut [u8]) {
    // Use RDRAND if available, else use timer + address randomness
    #[cfg(target_arch = "x86_64")]
    unsafe {
        for chunk in buf.chunks_mut(8) {
            let mut rand: u64;
            core::arch::asm!(
                "rdrand {}",
                out(reg) rand,
                options(nomem, nostack),
            );
            chunk.copy_from_slice(&rand.to_ne_bytes()[..chunk.len()]);
        }
    }
}
```

---

## Implementation Roadmap

### Phase 0: Current State (DONE ✅)
**Goal**: Basic stubs for Servo compilation

**Completed**:
- [x] mod.rs structure
- [x] fs.rs with hardcoded files
- [x] time.rs with PIT-based Instant
- [x] thread.rs with serial execution
- [x] stdio.rs with serial output
- [x] args.rs, env.rs, path.rs stubs
- [x] Kernel syscalls: read, write, open, close, exit, brk, getpid

**Status**: Servo can compile against this backend but won't run properly yet

---

### Phase 1: VFS Integration (PRIORITY)
**Goal**: Real file I/O via kernel VFS/TAR

**Timeline**: 1-2 days

**Tasks**:
1. **Update fs.rs** (6 hours)
   - [ ] Replace hardcoded files with syscall-based File
   - [ ] Implement File::open() using sys_open
   - [ ] Implement File::read() using sys_read
   - [ ] Implement File::seek() using sys_lseek
   - [ ] Add proper error handling with errno

2. **Add sys_lseek to kernel** (2 hours)
   ```rust
   // kernel/src/syscall/mod.rs
   fn sys_lseek(fd: usize, offset: isize, whence: usize) -> isize {
       match crate::fs::lseek(FileDescriptor(fd), offset, whence) {
           Ok(new_pos) => new_pos as isize,
           Err(_) => -1,
       }
   }
   ```

3. **Update VFS for seek** (2 hours)
   ```rust
   // kernel/src/fs/vfs.rs
   pub fn lseek(fd: FileDescriptor, offset: isize, whence: u8) -> Result<usize, &'static str> {
       let mut table = FILE_TABLE.lock();
       let file = table.get_mut(fd.0)?;

       let new_pos = match whence {
           SEEK_SET => offset,
           SEEK_CUR => file.position as isize + offset,
           SEEK_END => file.data.len() as isize + offset,
           _ => return Err("Invalid whence"),
       };

       file.position = new_pos as usize;
       Ok(new_pos as usize)
   }
   ```

4. **Testing** (2 hours)
   - [ ] Test reading index.html from TAR
   - [ ] Test seeking within files
   - [ ] Test error cases (nonexistent files)

**Success Criteria**:
- Servo can load HTML files from TAR filesystem via std::fs::File
- Seeking works for partial reads
- Error messages are propagated correctly

---

### Phase 2: Networking (HTTP Support)
**Goal**: Enable Servo to fetch resources over HTTP

**Timeline**: 3-4 days

**Tasks**:
1. **Implement net.rs basics** (8 hours)
   - [ ] TcpStream::connect()
   - [ ] TcpStream::read() / Write trait
   - [ ] SocketAddr parsing
   - [ ] Error handling

2. **Add socket syscalls to kernel** (8 hours)
   ```rust
   // kernel/src/syscall/mod.rs
   Socket = 41,
   Connect = 42,
   Send = 44,
   Recv = 45,

   fn sys_socket(domain: usize, type: usize, protocol: usize) -> isize;
   fn sys_connect(fd: usize, addr: usize, addrlen: usize) -> isize;
   fn sys_send(fd: usize, buf: usize, len: usize, flags: usize) -> isize;
   fn sys_recv(fd: usize, buf: usize, len: usize, flags: usize) -> isize;
   ```

3. **Integrate with kernel network stack** (12 hours)
   - [ ] Socket FD type in VFS
   - [ ] TCP connection state machine
   - [ ] Send/recv buffer management
   - [ ] Integration with existing HTTP parser

4. **Testing** (4 hours)
   - [ ] Manual HTTP GET to example.com
   - [ ] Parse HTTP response
   - [ ] Handle connection errors

**Success Criteria**:
- Can establish TCP connection to remote server
- Can send HTTP GET request
- Can receive and parse HTTP response
- Servo can fetch external resources

---

### Phase 3: Real Threading
**Goal**: Parallel execution for performance

**Timeline**: 5-7 days

**Tasks**:
1. **Implement sys_clone in kernel** (12 hours)
   - [ ] Thread creation with shared address space
   - [ ] Thread scheduler (round-robin)
   - [ ] Context switching
   - [ ] Per-thread stacks

2. **Update thread.rs** (8 hours)
   - [ ] Replace serial stub with real clone
   - [ ] Stack allocation
   - [ ] Thread-local storage (basic)
   - [ ] JoinHandle with wait

3. **Implement futex syscall** (8 hours)
   - [ ] Futex wait queue per address
   - [ ] FUTEX_WAIT operation
   - [ ] FUTEX_WAKE operation
   - [ ] Timeout support

4. **Implement sync.rs** (8 hours)
   - [ ] Futex-based Mutex
   - [ ] RwLock
   - [ ] Condvar

5. **Testing** (8 hours)
   - [ ] Multi-threaded counter test
   - [ ] Mutex contention test
   - [ ] Servo parallel layout

**Success Criteria**:
- Multiple threads execute in parallel
- Mutex prevents data races
- Servo can use parallel layout engine

---

### Phase 4: Polish & Optimization
**Goal**: Production-ready std backend

**Timeline**: 3-5 days

**Tasks**:
1. **RTC integration** (4 hours)
   - [ ] Read CMOS RTC
   - [ ] sys_clock_gettime
   - [ ] Proper SystemTime::now()

2. **Directory operations** (6 hours)
   - [ ] sys_getdents
   - [ ] read_dir() iterator
   - [ ] File metadata

3. **Error handling** (4 hours)
   - [ ] Complete errno mapping
   - [ ] Thread-local errno storage
   - [ ] Better error messages

4. **Memory management** (4 hours)
   - [ ] sys_mmap / sys_munmap
   - [ ] Large allocations
   - [ ] Guard pages

5. **Performance tuning** (6 hours)
   - [ ] Reduce syscall overhead
   - [ ] Buffer I/O where appropriate
   - [ ] Profile Servo startup time

6. **Documentation** (4 hours)
   - [ ] Inline documentation for all modules
   - [ ] Architecture guide
   - [ ] Porting guide for other apps

**Success Criteria**:
- All Servo tests pass
- Performance is acceptable (within 2x of Linux)
- Code is well-documented

---

## Code Examples

### Example 1: Reading a File from TAR Filesystem

```rust
// Userspace code using std::fs
use std::fs::File;
use std::io::Read;

fn main() -> std::io::Result<()> {
    // Open file from TAR filesystem via VFS
    let mut file = File::open("/index.html")?;

    // Read entire contents
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("Read {} bytes", contents.len());
    println!("Content: {}", contents);

    Ok(())
}
```

**Syscall trace**:
```
sys_open("/index.html", O_RDONLY) = 3
sys_read(3, buf, 4096) = 523  (first read)
sys_read(3, buf, 4096) = 0    (EOF)
sys_close(3) = 0
```

---

### Example 2: HTTP GET Request

```rust
use std::net::TcpStream;
use std::io::{Write, Read};

fn fetch_url(url: &str) -> std::io::Result<String> {
    // Parse URL
    let host = "example.com";
    let port = 80;

    // Connect
    let mut stream = TcpStream::connect((host, port))?;

    // Send HTTP request
    let request = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        host
    );
    stream.write_all(request.as_bytes())?;

    // Read response
    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    Ok(response)
}
```

**Syscall trace**:
```
sys_socket(AF_INET, SOCK_STREAM, 0) = 4
sys_connect(4, {sin_family=AF_INET, sin_port=80, sin_addr=93.184.216.34}, 16) = 0
sys_write(4, "GET / HTTP/1.1\r\n...", 47) = 47
sys_read(4, buf, 4096) = 1256
sys_read(4, buf, 4096) = 0
sys_close(4) = 0
```

---

### Example 3: Multi-threaded Counter

```rust
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    // Spawn 4 threads
    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut num = counter.lock().unwrap();
                *num += 1;
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}
```

**Syscall trace** (Phase 3, with real threads):
```
sys_clone(CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_THREAD, stack=0x..., ...) = 1001
sys_clone(...) = 1002
sys_clone(...) = 1003
sys_clone(...) = 1004
sys_futex(&mutex, FUTEX_WAIT, 2, NULL) = 0  (blocking wait)
sys_futex(&mutex, FUTEX_WAKE, 1, NULL) = 1  (wake waiting thread)
...
```

---

### Example 4: Measuring Time

```rust
use std::time::Instant;
use std::thread;
use std::time::Duration;

fn benchmark() {
    let start = Instant::now();

    // Do some work
    for _ in 0..1000 {
        // Computation
    }

    let elapsed = start.elapsed();
    println!("Took {:?}", elapsed);

    // Sleep for 100ms
    thread::sleep(Duration::from_millis(100));
}
```

**Implementation**:
```
Instant::now() -> reads TICKS atomic variable (updated by timer interrupt)
thread::sleep() -> busy-wait loop checking elapsed time
```

---

## Testing Strategy

### Unit Tests (in rust-std-fork)

**Location**: `library/std/src/sys/pal/astra_os/tests/`

```rust
#[test]
fn test_file_read() {
    let mut file = File::open("/test.html").unwrap();
    let mut buf = [0u8; 100];
    let n = file.read(&mut buf).unwrap();
    assert!(n > 0);
    assert!(buf.starts_with(b"<!DOCTYPE html>"));
}

#[test]
fn test_instant() {
    let start = Instant::now();
    busy_wait(1000); // 1ms
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() >= 1);
}

#[test]
fn test_tcp_connect() {
    // Requires network
    let stream = TcpStream::connect("127.0.0.1:8080");
    assert!(stream.is_ok());
}
```

### Integration Tests (in kernel)

**Location**: `kernel/tests/std_backend_tests.rs`

```rust
#[test_case]
fn test_syscall_open_read_close() {
    // Test VFS integration
    let fd = syscall::sys_open("/index.html", 0);
    assert!(fd > 0);

    let mut buf = [0u8; 100];
    let n = syscall::sys_read(fd, &mut buf);
    assert!(n > 0);

    syscall::sys_close(fd);
}
```

### Manual Testing with Servo

```bash
# Build custom Rust toolchain with astra_os target
cd rust-std-fork
./x.py build --target x86_64-astra_os library/std

# Build kernel with std backend
cd kernel
cargo build --release

# Build Servo against custom toolchain
cd servo
cargo +astra_os build --target x86_64-astra_os

# Run in QEMU
make run
```

**Test Cases**:
1. **Servo startup**: Does it compile and link?
2. **File loading**: Can it load index.html from TAR?
3. **HTML parsing**: Does the DOM tree build correctly?
4. **Layout**: Does the layout engine run?
5. **Rendering**: Does content appear on screen?
6. **HTTP**: Can it fetch resources from network? (Phase 2)
7. **Threading**: Does parallel layout work? (Phase 3)

---

## Appendix A: Syscall Numbers

Complete syscall number reference (Linux-compatible):

| Number | Name           | Signature                                                      |
|--------|----------------|----------------------------------------------------------------|
| 0      | read           | (fd: int, buf: *mut u8, count: usize) -> ssize_t              |
| 1      | write          | (fd: int, buf: *const u8, count: usize) -> ssize_t            |
| 2      | open           | (path: *const u8, flags: int) -> int                          |
| 3      | close          | (fd: int) -> int                                              |
| 4      | stat           | (path: *const u8, statbuf: *mut stat) -> int                  |
| 5      | fstat          | (fd: int, statbuf: *mut stat) -> int                          |
| 8      | lseek          | (fd: int, offset: off_t, whence: int) -> off_t                |
| 9      | mmap           | (addr: *mut u8, len: usize, prot: int, flags: int, fd: int, offset: off_t) -> *mut u8 |
| 11     | munmap         | (addr: *mut u8, len: usize) -> int                            |
| 12     | brk            | (addr: *mut u8) -> *mut u8                                    |
| 35     | nanosleep      | (req: *const timespec, rem: *mut timespec) -> int             |
| 39     | getpid         | () -> pid_t                                                   |
| 41     | socket         | (domain: int, type: int, protocol: int) -> int                |
| 42     | connect        | (sockfd: int, addr: *const sockaddr, addrlen: socklen_t) -> int |
| 44     | sendto         | (sockfd: int, buf: *const u8, len: usize, flags: int, dest_addr: *const sockaddr, addrlen: socklen_t) -> ssize_t |
| 45     | recvfrom       | (sockfd: int, buf: *mut u8, len: usize, flags: int, src_addr: *mut sockaddr, addrlen: *mut socklen_t) -> ssize_t |
| 56     | clone          | (flags: ulong, stack: *mut u8, parent_tid: *mut int, child_tid: *mut int, tls: ulong) -> pid_t |
| 60     | exit           | (status: int) -> !                                            |
| 78     | getdents       | (fd: int, dirp: *mut dirent, count: uint) -> int              |
| 96     | clock_gettime  | (clockid: clockid_t, tp: *mut timespec) -> int                |
| 186    | gettid         | () -> pid_t                                                   |
| 202    | futex          | (uaddr: *mut u32, op: int, val: u32, timeout: *const timespec, uaddr2: *mut u32, val3: u32) -> int |

---

## Appendix B: File Locations

```
rust-std-fork/library/std/src/sys/
├── pal/
│   └── astra_os/               # Platform abstraction layer
│       ├── mod.rs              # Root module, exports all submodules
│       ├── os.rs               # errno, signals, OS constants
│       ├── fs.rs               # File, OpenOptions, FileAttr
│       ├── stdio.rs            # Stdin, Stdout, Stderr
│       ├── io.rs               # I/O trait implementations
│       ├── net.rs              # TcpStream, TcpListener, UdpSocket
│       ├── time.rs             # Instant, SystemTime
│       ├── thread.rs           # Thread, spawn, JoinHandle
│       ├── sync.rs             # Mutex, RwLock, Condvar
│       ├── path.rs             # Path handling
│       ├── args.rs             # Command line arguments
│       ├── env.rs              # Environment variables
│       ├── process.rs          # Command, Child, ExitStatus
│       ├── random.rs           # RNG for HashMap keys
│       └── REMAINING_STUBS.rs  # Placeholder for unsupported features
│
├── alloc/
│   └── astra_os.rs             # Heap allocator (brk-based)
│
└── common/                     # Shared utilities
    ├── small_c_string.rs       # Efficient CString for syscalls
    └── ...

kernel/src/
├── syscall/
│   └── mod.rs                  # Syscall dispatcher and handlers
├── fs/
│   ├── mod.rs                  # VFS layer
│   ├── tar.rs                  # TAR filesystem driver
│   └── vfs.rs                  # File descriptor table
├── network/
│   ├── mod.rs                  # Network stack
│   ├── http.rs                 # HTTP client
│   ├── url.rs                  # URL parser
│   └── device.rs               # Network device driver
├── process.rs                  # Process/thread management
├── interrupts/
│   └── mod.rs                  # Timer interrupt (for TICKS)
└── main.rs                     # Kernel entry point
```

---

## Appendix C: Priority Matrix

| Module    | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Complexity | Impact |
|-----------|---------|---------|---------|---------|------------|--------|
| fs.rs     | ✅      | -       | -       | Polish  | Medium     | High   |
| time.rs   | ✅      | RTC     | -       | -       | Low        | High   |
| stdio.rs  | ✅      | -       | -       | -       | Low        | High   |
| net.rs    | -       | ✅      | -       | -       | High       | High   |
| thread.rs | Stub✅  | -       | ✅      | -       | Very High  | Medium |
| sync.rs   | -       | -       | ✅      | -       | High       | Medium |
| process.rs| Stub✅  | -       | -       | ✅      | High       | Low    |
| io.rs     | ✅      | -       | -       | -       | Low        | High   |
| os.rs     | ✅      | -       | -       | ✅      | Medium     | High   |
| path.rs   | ✅      | -       | -       | -       | Low        | Medium |
| args.rs   | ✅      | -       | -       | -       | Low        | Low    |
| env.rs    | ✅      | -       | -       | -       | Low        | Low    |
| random.rs | ✅      | -       | -       | -       | Low        | Medium |

**Legend**:
- ✅ = Implemented (at least as stub)
- - = No work needed this phase
- RTC = Real-time clock integration
- Polish = Cleanup and optimization

---

## Summary

This design document provides a comprehensive blueprint for implementing a production-ready Rust standard library backend for ASTRA.OS. The phased approach ensures that:

1. **Phase 1** delivers immediate value (VFS integration)
2. **Phase 2** enables critical features (HTTP networking)
3. **Phase 3** improves performance (real threading)
4. **Phase 4** polishes the implementation

Each module is designed with clear syscall dependencies, implementation strategies, and testing plans. The architecture prioritizes compatibility with existing Rust code (especially Servo) while maintaining clean separation between userspace (std) and kernel.

**Next Steps**:
1. Review this design with the team
2. Begin Phase 1 implementation (VFS integration)
3. Set up continuous integration tests
4. Document lessons learned for future OS developers

---

**Document Version**: 1.0
**Last Updated**: 2026-01-01
**Authors**: Claude (AI Assistant)
**License**: MIT (same as ASTRA.OS)
