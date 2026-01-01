# ASTRA.OS - Advanced System for Tomorrow's Revolutionary Applications

Rust로 커널부터 만드는 차세대 OS. 브라우저 중심 인터페이스와 현대적 시스템 아키텍처를 목표로 합니다.

## 현재 상태: Phase 3 (부분 완료)

### ✅ 구현된 기능

**Phase 1: Core Kernel**
- ✅ UEFI 부팅 (bootloader 0.9)
- ✅ 메모리 관리 (페이징, 1MB 힙 할당자)
- ✅ 인터럽트 핸들링 (타이머, 키보드)
- ✅ 시리얼 포트 디버깅

**Phase 2: Graphics & UI**
- ✅ VGA 텍스트 모드 그래픽 드라이버
- ✅ 색상 지원 및 도형 그리기 API
- ✅ PS/2 마우스 드라이버 (구현됨, 현재 비활성화)
- ✅ UI 위젯 시스템 (Button)

**Phase 3: Network Stack**
- ✅ smoltcp TCP/IP 스택 통합
- ✅ 네트워크 디바이스 추상화 레이어
- ✅ TCP 에코 서버 (포트 7)
- ⚠️ 현재 안정성 이슈로 비활성화

### ⚠️ 알려진 이슈
- 마우스 드라이버: QEMU에서 PS/2 초기화 시 멈춤
- 네트워크 스택: 힙 할당 중 크래시 발생 (디버깅 필요)

## 빌드 및 실행

### 빌드
\`\`\`bash
cd kernel
cargo build --release
cargo bootimage --release
\`\`\`

### 실행
\`\`\`bash
qemu-system-x86_64 \\
    -drive format=raw,file=target/x86_64-browser_os/release/bootimage-kernel.bin \\
    -serial stdio
\`\`\`

## 기술 스택

- \`bootloader\` (0.9) - BIOS/UEFI 부팅
- \`x86_64\` (0.15) - CPU 제어, 페이징
- \`smoltcp\` (0.11) - TCP/IP 스택
- \`linked_list_allocator\` - 힙 할당자

---

**ASTRA.OS** - *Advancing Systems Through Rust Architecture*
