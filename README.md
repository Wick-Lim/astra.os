# ASTRA.OS - Advanced System for Tomorrow's Revolutionary Applications

**Rust로 처음부터 만드는 차세대 브라우저 OS**

Servo 브라우저 엔진을 네이티브로 실행하는 것을 목표로, 커널부터 직접 구현하는 실험적인 운영체제 프로젝트입니다.

---

## 📊 현재 상태: Phase 5 완료 (Ring 3 Userspace)

**최종 업데이트**: 2026-01-01
**진행도**: 약 40% (Servo 통합까지)

### 🎯 프로젝트 비전

```
┌─────────────────────────────────────────┐
│    ASTRA.OS Architecture (최종 목표)    │
├─────────────────────────────────────────┤
│  Applications                           │
│  ┌──────────────────────────────────┐  │
│  │   Servo Browser Engine           │  │
│  │   (HTML/CSS/JS Rendering)        │  │
│  └──────────────────────────────────┘  │
├─────────────────────────────────────────┤
│  System Libraries                       │
│  ┌──────────┐ ┌──────────┐ ┌────────┐ │
│  │ Rust std │ │  libc    │ │ POSIX  │ │
│  └──────────┘ └──────────┘ └────────┘ │
├─────────────────────────────────────────┤
│  Kernel Services                        │
│  ┌──────────┐ ┌──────────┐ ┌────────┐ │
│  │ Syscalls │ │ Scheduler│ │   FS   │ │
│  │ Process  │ │ Memory   │ │  Net   │ │
│  └──────────┘ └──────────┘ └────────┘ │
├─────────────────────────────────────────┤
│  Hardware Abstraction                   │
│  VGA│Serial│Timer│Keyboard│Mouse│NIC   │
└─────────────────────────────────────────┘
```

---

## ✅ 완료된 기능 (Phase 1-5)

### **Phase 1: Core Kernel** ✅ 완료
커널의 기본 인프라 구축

**부팅 시스템**
- ✅ UEFI 부팅 (bootloader 0.9)
- ✅ BIOS/UEFI 양쪽 지원
- ✅ Multiboot2 준수
- ✅ 커널 엔트리 포인트 구현

**메모리 관리**
- ✅ 4-레벨 페이징 (x86-64)
- ✅ Identity mapping (물리 메모리 직접 매핑)
- ✅ 힙 할당자 (linked_list_allocator)
- ✅ 256MB 힙 공간
- ✅ Page table 동적 조작
- ✅ USER_ACCESSIBLE 플래그 지원

**인터럽트 핸들링**
- ✅ IDT (Interrupt Descriptor Table) 구성
- ✅ PIC (Programmable Interrupt Controller) 초기화
- ✅ Timer interrupt (IRQ 0)
- ✅ Keyboard interrupt (IRQ 1)
- ✅ Exception handlers (Page Fault, Double Fault 등)
- ✅ IST (Interrupt Stack Table) 지원

**디버깅 지원**
- ✅ Serial port (COM1) 초기화
- ✅ `serial_print!`, `serial_println!` 매크로
- ✅ QEMU `-serial stdio`로 디버그 출력

---

### **Phase 2: Graphics & UI** ✅ 완료
기본 그래픽 인터페이스 구현

**VGA 텍스트 모드**
- ✅ 80x25 텍스트 모드 드라이버
- ✅ 16색 컬러 지원
- ✅ 스크롤링 구현
- ✅ 커서 제어

**그래픽 API**
- ✅ embedded-graphics 통합
- ✅ DrawTarget trait 구현
- ✅ 도형 그리기 (선, 사각형, 원)
- ✅ 색상 변환 시스템

**입력 장치**
- ✅ PS/2 마우스 드라이버
- ✅ 마우스 이동/클릭 이벤트
- ✅ QEMU 환경 완벽 호환
- ✅ 마우스 커서 표시

**UI 시스템**
- ✅ 기본 위젯 프레임워크
- ✅ Button 위젯 구현
- ✅ 클릭 이벤트 처리
- ✅ 레이아웃 시스템 기초

---

### **Phase 3: Network Stack** ✅ 완료
TCP/IP 네트워킹 기초

**네트워크 스택**
- ✅ smoltcp 0.11 통합 (no_std)
- ✅ TCP/IP 프로토콜 지원
- ✅ 패킷 송수신 구조
- ✅ 네트워크 디바이스 추상화

**네트워크 관리**
- ✅ NetworkInfo 구조체
- ✅ IP 주소 설정 (10.0.2.15/24)
- ✅ MAC 주소 관리
- ✅ QEMU 네트워크 호환

---

### **Phase 4: Pixel Graphics** ✅ 완료
실제 픽셀 그래픽 렌더링

**VGA Mode 13h 구현**
- ✅ 320x200 해상도
- ✅ 256색 팔레트 모드
- ✅ VGA 레지스터 직접 프로그래밍
- ✅ Mode 13h 전환 완료

**메모리 매핑**
- ✅ VGA 메모리 (0xA0000-0xBFFFF) Identity mapping
- ✅ `write_volatile`로 안정적인 메모리 접근
- ✅ 전체 화면 렌더링 검증

**색상 시스템**
- ✅ 3-3-2 RGB 팔레트 (256색)
- ✅ RGB888 → 8비트 팔레트 변환
- ✅ DAC 레지스터 프로그래밍
- ✅ 커스텀 컬러 매핑

**렌더링 파이프라인**
- ✅ embedded-graphics DrawTarget 구현
- ✅ 도형, 텍스트, 이미지 렌더링
- ✅ 320x200 전체 영역 테스트 완료
- ✅ 크래시 없는 안정적 렌더링

---

### **Phase 5: Ring 3 Userspace Support** ✅ 완료! (2026-01-01)
x86-64 특권 레벨 분리 및 시스템 호출 구조

**GDT (Global Descriptor Table)**
- ✅ 7개 엔트리 커스텀 GDT 구성
  - 0x00: Null descriptor
  - 0x08: Kernel code segment (Ring 0)
  - 0x10: Kernel data segment (Ring 0)
  - 0x18: User code segment (Ring 3, DPL=3)
  - 0x20: User data segment (Ring 3, DPL=3)
  - 0x28-0x30: TSS descriptor (2 entries)
- ✅ 64-bit long mode segments
- ✅ RPL (Requested Privilege Level) 설정

**TSS (Task State Segment)**
- ✅ 64-bit TSS 구조체 구현
- ✅ Ring 0 kernel stack (rsp0) 설정
- ✅ IST (Interrupt Stack Table) 구현
  - IST[0]: Double fault handler (16KB)
  - IST[1]: Timer interrupts from Ring 3 (16KB)
  - IST[2]: Syscalls from Ring 3 (16KB)
- ✅ TSS 로딩 및 검증

**Privilege Level Transition**
- ✅ Ring 0 → Ring 3 전환 (`iretq` 사용)
- ✅ IOPL=3 설정 (Ring 3에서 STI/CLI 허용)
- ✅ Interrupt Flag 제어
- ✅ Code/Stack segment 전환 검증

**Interrupt Handling from Ring 3**
- ✅ Timer interrupts (IRQ 0) from Ring 3
- ✅ IST-based stack switching (Ring 3 → Ring 0)
- ✅ Stack Segment Fault (#12) 해결
- ✅ 안정적인 interrupt return (iretq)
- ✅ 인터럽트 카운팅 및 로깅

**System Call Interface**
- ✅ `int 0x80` 시스템 호출 기본 구조
- ✅ Ring 3에서 호출 가능 (DPL=3)
- ✅ IST[2]를 통한 스택 전환
- ✅ Syscall handler 프레임워크
- ✅ 레지스터 기반 인자 전달 준비

**Memory Protection**
- ✅ User code pages (USER_ACCESSIBLE, EXECUTABLE)
- ✅ User stack pages (USER_ACCESSIBLE, WRITABLE, NX)
- ✅ Kernel pages (Ring 0 only)
- ✅ Page fault handler 동작 확인

**SSE Support**
- ✅ CR4.OSFXSR, CR4.OSXMMEXCPT 활성화
- ✅ 컴파일러 생성 SSE 코드 지원
- ✅ xmm 레지스터 사용 가능

**디버깅 및 검증**
- ✅ QEMU debug mode (`-d int,cpu_reset`)
- ✅ CPL=3 상태 확인
- ✅ Timer interrupt 발생 확인 (v=20, cpl=3)
- ✅ Syscall 발생 확인 (v=80, cpl=3)
- ✅ 10초 이상 안정적 실행

**해결된 주요 이슈**
- ✅ Stack Segment Fault (#12) - IST 사용으로 해결
- ✅ General Protection Fault (#13) - IOPL=3 설정으로 해결
- ✅ Invalid Opcode (#6) - SSE 활성화로 해결
- ✅ Triple fault 완전 제거

**관련 파일**
- `kernel/src/gdt.rs` - GDT, TSS, IST 구현
- `kernel/src/interrupts/mod.rs` - IDT, interrupt handlers
- `kernel/src/userspace_code.rs` - Ring 3 entry point
- `kernel/src/main.rs` - Ring 3 전환 로직

---

## 🚧 진행 중인 작업 (Phase 6)

### **Phase 6A: Syscall Interface** 🔨 진행 예정
실제 동작하는 시스템 호출 구현

**핵심 Syscalls (우선순위 높음)**
- ⏳ `sys_write` - 콘솔/파일 출력
- ⏳ `sys_read` - 콘솔/파일 입력
- ⏳ `sys_open` - 파일 열기
- ⏳ `sys_close` - 파일 닫기
- ⏳ `sys_brk` / `sys_sbrk` - 힙 메모리 할당
- ⏳ `sys_mmap` / `sys_munmap` - 메모리 매핑
- ⏳ `sys_exit` - 프로세스 종료
- ⏳ `sys_getpid` - 프로세스 ID
- ⏳ `sys_fork` - 프로세스 복제 (선택사항)
- ⏳ `sys_exec` - 프로그램 실행 (선택사항)

**Syscall Dispatcher**
- ⏳ RAX 레지스터로 syscall 번호 전달
- ⏳ RDI, RSI, RDX, R10, R8, R9로 인자 전달
- ⏳ RAX로 반환값 전달
- ⏳ Error handling (errno 구현)

**예상 소요 시간**: 2-3주

---

### **Phase 6B: Process Management** 🔨 진행 예정
멀티태스킹 및 프로세스 스케줄링

**프로세스 구조체**
- ⏳ Process Control Block (PCB)
- ⏳ Page table per process
- ⏳ Register state 저장/복원
- ⏳ Process state (Running, Ready, Blocked)

**스케줄러**
- ⏳ Round-robin 스케줄러
- ⏳ Timer-based preemption
- ⏳ Context switching 구현
- ⏳ Process queue 관리

**스레드 지원**
- ⏳ 기본 스레드 생성
- ⏳ 스레드 간 컨텍스트 스위칭
- ⏳ Mutex / Semaphore (기본)

**예상 소요 시간**: 3-4주

---

### **Phase 6C: File System** 🔨 진행 예정
파일 입출력 기능

**초기 구현 (RAM Disk)**
- ⏳ In-memory file system
- ⏳ VFS (Virtual File System) 레이어
- ⏳ 기본 파일 연산 (open, read, write, close)
- ⏳ 디렉토리 구조

**정적 파일 지원**
- ⏳ 컴파일 타임에 파일 임베딩
- ⏳ HTML/CSS/JS 파일 로딩
- ⏳ `/` 루트 디렉토리 구조

**확장 계획 (선택사항)**
- ⏳ FAT32 파일 시스템 (읽기 전용)
- ⏳ 디스크 I/O
- ⏳ 파일 캐싱

**예상 소요 시간**: 1-2주

---

## 🎯 다음 목표 (Phase 7-8)

### **Phase 7: Rust std Implementation**
Servo가 의존하는 Rust 표준 라이브러리 구현

**타겟 스펙**
- ⏳ `x86_64-astra_os.json` 커스텀 타겟
- ⏳ Rust 컴파일러 포크 및 통합
- ⏳ `#![no_std]` → `std` 전환

**핵심 모듈**
- ⏳ `std::fs` - 파일 시스템
- ⏳ `std::io` - 입출력
- ⏳ `std::thread` - 스레딩
- ⏳ `std::sync` - 동기화 primitives
- ⏳ `std::net` - 네트워킹 (TCP/UDP)
- ⏳ `std::time` - 시간 관리
- ⏳ `std::env` - 환경 변수
- ⏳ `std::process` - 프로세스 관리

**libc 인터페이스**
- ⏳ POSIX-like syscall wrapper
- ⏳ `malloc`, `free` 구현
- ⏳ 기본 C 라이브러리 함수

**예상 소요 시간**: 3-4주

---

### **Phase 8: Servo Integration** 🎯 최종 목표!
Servo 브라우저 엔진 포팅 및 통합

**8A: Minimal Servo Port (4-6주)**

**의존성 최소화**
- ⏳ Servo 코드베이스 분석
- ⏳ 불필요한 의존성 제거 (네트워킹, 멀티스레드)
- ⏳ 핵심 컴포넌트만 추출
  - HTML parser
  - CSS parser
  - Layout engine
  - Rendering pipeline

**크로스 컴파일**
- ⏳ `x86_64-astra_os` 타겟으로 빌드
- ⏳ no_std 호환 작업
- ⏳ Stub 구현 (파일, 네트워크, 스레드)

**첫 렌더링**
- ⏳ 하드코딩된 HTML 렌더링
  ```html
  <html>
    <body>
      <h1>Hello from Servo on ASTRA.OS!</h1>
    </body>
  </html>
  ```
- ⏳ VGA 320x200에 출력
- ⏳ 레이아웃 엔진 동작 확인

**8B: Full Servo (4-8주)**

**고해상도 그래픽**
- ⏳ VESA/VBE framebuffer (640x480 이상)
- ⏳ 16/32비트 컬러 지원
- ⏳ Double buffering

**파일 로딩**
- ⏳ 파일 시스템에서 HTML 읽기
- ⏳ CSS/JS 파일 로딩
- ⏳ 이미지 디코딩 (PNG, JPEG)

**멀티스레드 렌더링**
- ⏳ Servo의 병렬 렌더링 활성화
- ⏳ 스레드 풀 구현
- ⏳ 렌더링 성능 최적화

**네트워킹**
- ⏳ HTTP/HTTPS 프로토콜
- ⏳ DNS 리졸버
- ⏳ TLS/SSL (rustls)
- ⏳ 실제 웹페이지 로딩!

**인터랙션**
- ⏳ 마우스 클릭 이벤트
- ⏳ 스크롤
- ⏳ 폼 입력
- ⏳ 키보드 입력

**JavaScript (선택사항)**
- ⏳ SpiderMonkey 통합
- ⏳ DOM API
- ⏳ 기본 JS 실행

---

## 🛠️ 기술 스택

### **커널 레벨**
```toml
[dependencies]
bootloader = "0.9"           # UEFI/BIOS 부팅
x86_64 = "0.15"              # CPU 제어, 페이징
linked_list_allocator = "*"  # 힙 할당자
pic8259 = "*"                # PIC 인터럽트 컨트롤러
uart_16550 = "*"             # Serial port
spin = "0.9"                 # Spinlock
lazy_static = "1.4"          # Static initialization
```

### **그래픽**
```toml
embedded-graphics = "0.8"    # 2D 그래픽 라이브러리
```

### **네트워킹**
```toml
smoltcp = "0.11"             # TCP/IP 스택 (no_std)
```

### **향후 추가 예정**
```toml
servo = { git = "...", default-features = false }
rustls = { version = "*", default-features = false }
```

---

## 📈 프로젝트 타임라인

```
2025-12-01  Phase 1-4 완료 (Core + Graphics + Network)
2026-01-01  Phase 5 완료 (Ring 3 Userspace) ← 현재 위치!
2026-01-15  Phase 6A 완료 목표 (Syscalls)
2026-02-01  Phase 6B 완료 목표 (Process Management)
2026-02-15  Phase 6C 완료 목표 (File System)
2026-03-01  Phase 7 완료 목표 (Rust std)
2026-04-15  Phase 8A 완료 목표 (Servo Minimal - 첫 렌더링!)
2026-06-01  Phase 8B 완료 목표 (Full Servo)
```

**예상 최종 완성**: 2026년 6월

---

## 🚀 빌드 및 실행

### **개발 환경**
```bash
# 필수 도구
rustup target add x86_64-unknown-none
cargo install bootimage
cargo install cargo-xbuild

# QEMU 설치
brew install qemu  # macOS
sudo apt install qemu-system-x86  # Linux
```

### **빌드**
```bash
# 전체 빌드
make build

# 또는 수동 빌드
cd kernel
cargo build --release
cargo bootimage --release
```

### **실행**
```bash
# 기본 실행
make run

# 디버그 모드
make debug

# Serial 출력 포함
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-browser_os/release/bootimage-kernel.bin \
    -serial stdio \
    -display cocoa \
    -m 256M
```

### **디버깅**
```bash
# QEMU 인터럽트 디버깅
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-browser_os/release/bootimage-kernel.bin \
    -serial stdio \
    -nographic \
    -d int,cpu_reset \
    -D /tmp/qemu_debug.log

# GDB 디버깅
make gdb
```

---

## 📁 프로젝트 구조

```
astra.os/
├── kernel/                    # 커널 소스코드
│   ├── src/
│   │   ├── main.rs           # 커널 엔트리 포인트
│   │   ├── gdt.rs            # GDT, TSS, Ring 3 구현
│   │   ├── interrupts/       # 인터럽트 핸들러
│   │   │   └── mod.rs        # IDT, PIC, syscall handler
│   │   ├── memory/           # 메모리 관리
│   │   │   ├── mod.rs        # 페이징, 힙
│   │   │   └── allocator.rs  # 할당자
│   │   ├── drivers/          # 디바이스 드라이버
│   │   │   ├── vga.rs        # VGA 텍스트 모드
│   │   │   ├── framebuffer.rs # VGA Mode 13h
│   │   │   ├── serial.rs     # COM1 시리얼
│   │   │   ├── keyboard.rs   # PS/2 키보드
│   │   │   └── mouse.rs      # PS/2 마우스
│   │   ├── network/          # 네트워크 스택
│   │   │   └── mod.rs        # smoltcp 통합
│   │   ├── process/          # 프로세스 관리 (예정)
│   │   ├── syscall/          # 시스템 호출 (예정)
│   │   ├── fs/               # 파일 시스템 (예정)
│   │   ├── userspace_code.rs # Ring 3 코드
│   │   ├── simple_html.rs    # 간단한 HTML 파서
│   │   └── ui/               # UI 위젯
│   ├── Cargo.toml
│   └── x86_64-browser_os.json # 커스텀 타겟 스펙
├── rust-std-fork/            # Rust std 구현 (예정)
├── servo-minimal/            # Servo 포트 (예정)
├── Makefile                  # 빌드 스크립트
├── README.md                 # 이 파일
├── SERVO_INTEGRATION_PLAN.md # Servo 통합 계획
└── NEXT_STEPS.md             # 다음 단계 상세

생성 예정:
kernel/src/syscall/           # Syscall 구현
kernel/src/process/           # 스케줄러
kernel/src/fs/                # 파일 시스템
```

---

## 🐛 알려진 이슈

### **해결됨**
- ✅ VGA 렌더링 크래시 → `write_volatile` 사용으로 해결
- ✅ 마우스 드라이버 불안정 → PS/2 초기화 간소화로 해결
- ✅ Ring 3 Stack Segment Fault → IST 사용으로 해결
- ✅ Ring 3 인터럽트 GPF → IOPL=3 설정으로 해결

### **진행 중**
- ⚠️ Syscall 로깅 안 됨 - spinlock 문제로 추정 (기능은 정상 작동)
- ⚠️ 힙 할당자 일부 케이스 크래시 - 안정성 개선 필요

### **예정**
- ⏳ 멀티태스킹 미구현
- ⏳ 파일 시스템 미구현
- ⏳ 네트워크 프로토콜 불완전 (TCP/IP만)

---

## 📚 참고 자료

### **학습 리소스**
- [OSDev Wiki](https://wiki.osdev.org/) - OS 개발 백과사전
- [Writing an OS in Rust](https://os.phil-opp.com/) - Rust OS 튜토리얼
- [Intel x86-64 SDM](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) - CPU 레퍼런스
- [AMD64 Architecture](https://www.amd.com/en/support/tech-docs) - AMD 문서

### **Ring 3 관련**
- [x86-64 Privilege Levels](https://wiki.osdev.org/Security#Rings) - 특권 레벨
- [TSS in Long Mode](https://wiki.osdev.org/Task_State_Segment#Long_Mode) - TSS 구조
- [System Calls](https://wiki.osdev.org/System_Calls) - 시스템 호출 구현

### **Servo 관련**
- [Servo Browser Engine](https://github.com/servo/servo)
- [Servo Architecture](https://github.com/servo/servo/wiki/Design)

---

## 🤝 기여

이 프로젝트는 개인 학습 목적의 실험 프로젝트입니다.

---

## 📜 라이센스

MIT License

---

## 🎯 다음 작업

**즉시 시작 가능**:
1. Syscall dispatcher 구현 (`kernel/src/syscall/mod.rs`)
2. `sys_write` 구현 (콘솔 출력)
3. `sys_brk` 구현 (힙 메모리)
4. Process 구조체 설계 (`kernel/src/process/mod.rs`)

**병렬 진행 가능**:
- Track A: Servo 코드베이스 분석 및 의존성 파악
- Track B: Syscall 인터페이스 구현
- Track C: 기본 스케줄러 설계

---

**ASTRA.OS** - *Advancing Systems Through Rust Architecture*

*"From bare metal to browser in pure Rust"*
