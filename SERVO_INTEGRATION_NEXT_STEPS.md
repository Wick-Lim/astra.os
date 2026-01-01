# Servo Integration - Next Steps

## 분석 완료 (2026-01-02)

### Servo 의존성 분석 결과

**JavaScript 엔진**: `mozjs = "0.14.4"` (SpiderMonkey Rust 바인딩)

**Servo가 사용하는 std 모듈 (사용 빈도순):**

| 모듈 | 사용 횟수 | 우선순위 | 설명 |
|------|-----------|----------|------|
| std::cell | 270 | HIGH | RefCell, Cell (이미 no_std에서 가능) |
| std::sync | 176 | HIGH | Arc, Mutex, RwLock - **멀티스레드 필수** |
| std::rc | 158 | HIGH | Rc (이미 no_std에서 가능) |
| std::collections | 143 | HIGH | HashMap, Vec (이미 alloc에서 가능) |
| std::time | 67 | **CRITICAL** | Instant, Duration - **Timer 필요** |
| std::thread | 41 | **CRITICAL** | spawn, sleep - **스레드 필수** |
| std::path | 41 | HIGH | PathBuf, Path |
| std::io | 32 | **CRITICAL** | Read, Write traits |
| std::ffi | 32 | HIGH | CString, OsStr |
| std::fs | 24 | **CRITICAL** | File, read_to_string |
| std::net | 18 | **CRITICAL** | TcpStream, UdpSocket |

## 작업 계획

### Phase 1: 필수 Syscall 구현 (1주)

Servo + mozjs를 실행하기 위한 최소 syscall:

```rust
// kernel/src/syscall/mod.rs

// 파일 I/O (std::fs)
sys_open(path, flags) -> fd
sys_close(fd)
sys_read(fd, buf, count) -> bytes_read
sys_write(fd, buf, count) -> bytes_written
sys_lseek(fd, offset, whence) -> new_offset

// 메모리 관리 (std::alloc, mozjs)
sys_brk(addr) -> new_brk
sys_mmap(addr, length, prot, flags) -> mapped_addr
sys_munmap(addr, length) -> result

// 스레드 (std::thread - CRITICAL!)
sys_clone(flags, stack, ...) -> pid
sys_exit(status)
sys_wait4(pid, status, options) -> pid

// 시간 (std::time - CRITICAL!)
sys_clock_gettime(clockid, timespec) -> result
sys_nanosleep(req, rem) -> result

// 네트워크 (std::net)
sys_socket(domain, type, protocol) -> fd
sys_connect(fd, addr, addrlen) -> result
sys_bind(fd, addr, addrlen) -> result
sys_listen(fd, backlog) -> result
sys_accept(fd, addr, addrlen) -> new_fd
sys_send(fd, buf, len, flags) -> bytes_sent
sys_recv(fd, buf, len, flags) -> bytes_received

// 기타
sys_getpid() -> pid
sys_ioctl(fd, request, argp) -> result
```

### Phase 2: std Backend 구현 (1-2주)

Rust std fork에 `sys/astra_os/` 구현:

**우선순위 1 - 즉시 구현 필요:**

1. **sys/astra_os/time.rs** (CRITICAL)
```rust
pub struct Instant { /* syscall::clock_gettime */ }
pub struct SystemTime { /* syscall::clock_gettime */ }
impl Instant {
    pub fn now() -> Self { /* sys_clock_gettime */ }
    pub fn elapsed(&self) -> Duration { /* ... */ }
}
```

2. **sys/astra_os/thread.rs** (CRITICAL)
```rust
pub fn spawn(f: Box<dyn FnOnce()>) -> io::Result<JoinHandle> {
    // sys_clone 사용
}
pub fn sleep(dur: Duration) {
    // sys_nanosleep 사용
}
```

3. **sys/astra_os/fs.rs** (CRITICAL)
```rust
pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
    // sys_open 사용
}
impl Read for File { /* sys_read */ }
impl Write for File { /* sys_write */ }
```

4. **sys/astra_os/net.rs** (CRITICAL)
```rust
pub struct TcpStream { /* sys_socket, sys_connect */ }
impl Read for TcpStream { /* sys_recv */ }
impl Write for TcpStream { /* sys_send */ }
```

**우선순위 2 - 기본 구현:**

5. **sys/astra_os/io.rs**
6. **sys/astra_os/sync.rs** (Mutex, RwLock)
7. **sys/astra_os/os.rs**
8. **sys/astra_os/path.rs**

### Phase 3: mozjs (SpiderMonkey) 포팅 (2-3주)

**mozjs 0.14.4 분석:**

```bash
# mozjs 크레이트 클론
git clone https://github.com/servo/mozjs.git
cd mozjs
git checkout 0.14.4

# 의존성 확인
cat Cargo.toml
# - SpiderMonkey C++ 소스코드 포함 (huge!)
# - libc, libz-sys 의존
# - 컴파일 시 C++ 빌드 필요
```

**과제:**
1. SpiderMonkey C++ 코드를 x86_64-astra_os용으로 크로스 컴파일
2. mozjs Rust 바인딩을 no_std 환경에 맞게 수정
3. JIT 컴파일러 지원 여부 결정
   - Option A: JIT 활성화 (빠름, 복잡)
   - Option B: Interpreter only (느림, 간단)

### Phase 4: Servo 빌드 (1주)

```bash
cd /Users/wick/Documents/workspaces/astra.os/servo

# 1. 커스텀 타겟으로 빌드 시도
cargo build \
    --target x86_64-astra_os.json \
    --no-default-features \
    --features="js"

# 2. 링커 에러 확인
# → 누락된 심볼 파악
# → syscall/std 추가 구현

# 3. 반복
```

**예상 문제:**
- 수백 개의 링커 에러
- 누락된 libc 함수
- 누락된 std 함수
- Platform-specific 코드 (Linux/Windows/macOS)

### Phase 5: Servo 통합 (1주)

```rust
// kernel/src/main.rs

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // ... 기존 초기화 ...

    serial_println!("Starting Servo browser engine...");

    // Servo 초기화
    extern "C" {
        fn servo_main();
    }

    unsafe {
        servo_main();
    }

    loop { hlt(); }
}
```

**첫 목표:**
```html
<!-- 메모리에 하드코딩된 HTML -->
<html>
  <head>
    <title>ASTRA.OS Browser</title>
  </head>
  <body>
    <h1>Hello from Servo!</h1>
    <p>JavaScript works:</p>
    <script>
      document.write("Date: " + new Date());
    </script>
  </body>
</html>
```

이게 렌더링되고 JavaScript가 실행되면 **성공**!

## 타임라인

| Week | Task | Deliverable |
|------|------|-------------|
| 1 | Syscall 구현 | sys_read, sys_write, sys_open, sys_brk, sys_mmap 동작 |
| 2 | std backend | std::fs::File, std::thread::spawn 동작 |
| 3-4 | mozjs 포팅 | SpiderMonkey 컴파일 성공 |
| 5 | Servo 빌드 | Servo 크로스 컴파일 성공 |
| 6 | 통합 및 테스트 | "Hello from Servo!" 렌더링 |
| 7 | JS 실행 | JavaScript 코드 실행 확인 |

**예상 완료일**: 2026년 2월 중순

## 즉시 시작 가능한 작업

### 1. Syscall Interface 구현

```bash
cd /Users/wick/Documents/workspaces/astra.os/kernel/src
mkdir -p syscall

# 파일 생성
touch syscall/mod.rs
touch syscall/fs.rs
touch syscall/memory.rs
touch syscall/process.rs
touch syscall/time.rs
touch syscall/network.rs
```

### 2. 기존 불필요 코드 정리

```bash
# Phase 1-5에서 만든 중복 코드 제거
git rm -r kernel/src/html
git rm -r kernel/src/css
git rm -r kernel/src/layout
git rm kernel/src/network/http.rs
git rm kernel/src/network/url.rs
git rm -r kernel/src/resource

git commit -m "Remove redundant browser components (will use Servo)"
```

### 3. rust-std-fork 시작

```bash
cd /Users/wick/Documents/workspaces/astra.os
git clone https://github.com/rust-lang/rust.git rust-std-fork
cd rust-std-fork

# sys/astra_os 백엔드 생성
mkdir -p library/std/src/sys/astra_os
```

## 다음 작업

**병렬로 진행 가능:**
- Track A: Syscall 구현 (커널)
- Track B: std backend (Rust fork)
- Track C: mozjs 분석 (SpiderMonkey)

**시작 명령:**
```bash
# Track A
cd kernel/src && mkdir syscall && vim syscall/mod.rs

# Track B
cd rust-std-fork/library/std/src/sys && mkdir astra_os

# Track C
git clone https://github.com/servo/mozjs.git && cd mozjs
```

---

**목표: JavaScript가 동작하는 Servo 브라우저 on ASTRA.OS!** 🚀
