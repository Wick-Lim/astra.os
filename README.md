# Browser OS - Rust 기반 브라우저 중심 운영체제

Rust로 커널부터 만드는 미니멀 OS. 최종 목표는 브라우저(Servo 컴포넌트 기반)가 메인 UI인 운영체제입니다.

## 현재 상태: Phase 1 완료

### 구현된 기능
- ✅ UEFI 부팅 (bootloader_api)
- ✅ 프레임버퍼 기반 그래픽 출력
- ✅ 메모리 관리 (페이징, 힙 할당자)
- ✅ 인터럽트 핸들링 (타이머, 키보드)
- ✅ 시리얼 포트 디버깅

## 요구사항

### 필수 도구
```bash
# Rust nightly 툴체인
rustup override set nightly
rustup component add rust-src llvm-tools-preview

# bootimage 도구
cargo install bootimage

# QEMU (가상화 에뮬레이터)
# macOS
brew install qemu

# Ubuntu/Debian
sudo apt install qemu-system-x86
```

## 빌드 및 실행

### 방법 1: Makefile 사용 (권장)

```bash
# 빌드 및 실행
make run

# 빌드만
make build

# 클린
make clean

# 디버그 모드 (GDB 연결 가능)
make debug
```

### 방법 2: 수동 빌드

```bash
# 커널 빌드
cd kernel
cargo build --release

# 부팅 이미지 생성
cargo bootimage --release

# QEMU 실행
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-browser_os/release/bootimage-kernel.bin \
    -device virtio-net,netdev=net0 \
    -netdev user,id=net0 \
    -m 512M \
    -serial stdio
```

### 방법 3: 실행 스크립트 사용

```bash
# 일반 모드
./tools/run-qemu.sh

# 디버그 모드
./tools/run-qemu.sh debug
```

## 프로젝트 구조

```
browser-os/
├── kernel/                    # 커널 코드
│   ├── src/
│   │   ├── main.rs           # 커널 엔트리포인트
│   │   ├── drivers/          # 하드웨어 드라이버
│   │   │   └── framebuffer.rs
│   │   ├── interrupts/       # 인터럽트 핸들러
│   │   ├── memory/           # 메모리 관리
│   │   │   ├── allocator.rs # 힙 할당자
│   │   │   └── mod.rs
│   │   └── serial.rs         # 시리얼 디버깅
│   ├── Cargo.toml
│   └── x86_64-browser_os.json # 커스텀 타겟
├── Cargo.toml                # 워크스페이스
├── Makefile
└── tools/
    └── run-qemu.sh           # QEMU 실행 스크립트
```

## 기술 스택

### 커널 레벨
- `bootloader_api` (v0.11) - UEFI 부팅
- `x86_64` - CPU 제어, 페이징
- `pic8259` - 인터럽트 컨트롤러
- `pc-keyboard` - PS/2 키보드
- `uart_16550` - 시리얼 디버깅
- `linked_list_allocator` - 힙 할당자

## 현재 동작

1. 부팅 시 프레임버퍼를 파란색으로 채웁니다
2. 시리얼 포트로 부팅 로그를 출력합니다
3. 키보드 입력 시 스캔코드를 시리얼로 출력합니다

## 다음 단계 (Phase 2)

- [ ] embedded-graphics 통합
- [ ] 폰트 렌더링 (fontdue)
- [ ] 마우스 드라이버
- [ ] 기본 UI 위젯

## 참고 자료

- [blog_os](https://os.phil-opp.com/) - Rust OS 개발 가이드
- [Redox OS](https://gitlab.redox-os.org/redox-os/redox) - Rust OS 참고 구현
- [Servo](https://github.com/servo/servo) - Rust 브라우저 엔진

## 라이선스

MIT License
