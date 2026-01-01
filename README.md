# ASTRA.OS - Advanced System for Tomorrow's Revolutionary Applications

Rust로 커널부터 만드는 차세대 OS. 브라우저 중심 인터페이스와 현대적 시스템 아키텍처를 목표로 합니다.

## 현재 상태: Phase 4 완료 → Servo 통합 준비

### ✅ 구현된 기능

**Phase 1: Core Kernel**
- ✅ UEFI 부팅 (bootloader 0.9)
- ✅ 메모리 관리 (페이징, 256MB 힙 할당자 예정)
- ✅ 인터럽트 핸들링 (타이머, 키보드)
- ✅ 시리얼 포트 디버깅

**Phase 2: Graphics & UI**
- ✅ VGA 텍스트 모드 그래픽 드라이버
- ✅ 색상 지원 및 도형 그리기 API
- ✅ PS/2 마우스 드라이버 (QEMU 호환)
- ✅ UI 위젯 시스템 (Button)

**Phase 3: Network Stack**
- ✅ smoltcp TCP/IP 스택 통합
- ✅ 네트워크 디바이스 추상화 레이어
- ✅ 네트워크 정보 관리 (IP, MAC 주소)
- ✅ QEMU 환경에서 안정적 동작

**Phase 4: Pixel Graphics** ✅ **완료!**
- ✅ VGA 메모리 영역 Identity Mapping (0xA0000-0xBFFFF)
- ✅ VGA Mode 13h 레지스터 프로그래밍 구현
- ✅ 3-3-2 RGB 팔레트 설정 (256색)
- ✅ embedded-graphics DrawTarget 구현
- ✅ RGB888 → palette 색상 변환
- ✅ 전체 화면 렌더링 동작 확인 (320x200)
- ✅ write_volatile을 사용한 안정적인 VGA 메모리 접근

**Phase 5: Rust std Implementation** 🔨 **진행 중!**
- ✅ x86_64-astra_os 타겟 스펙 작성
- ✅ sys::astra_os 백엔드 구현 (18개 모듈)
  - ✅ fs (하드코딩된 HTML)
  - ✅ thread (즉시 실행)
  - ✅ time (PIT 타이머)
  - ✅ stdio (시리얼 포트)
  - ✅ net, env, args, process, io, alloc 등
- 🔨 Rust 컴파일러 포크 및 빌드 (준비 완료)
- ⏳ Servo 브라우저 엔진 통합 (2-3주 예정)

### ✅ 해결된 이슈
- ✅ VGA 렌더링 크래시: `write_bytes` → `write_volatile` 변경으로 완전 해결
- ✅ 전체 화면 렌더링: 320x200 전체 영역 정상 동작 확인
- ✅ 마우스 드라이버: PS/2 초기화 간소화로 QEMU 호환성 확보
- ✅ 네트워크 스택: NetworkInfo 구조체로 안정적 초기화 구현

### 🚀 다음 단계: Servo 브라우저 통합 (2-3주 목표)

**Week 1-2: Rust std 구현**
1. Rust 컴파일러 포크
2. x86_64-astra_os 타겟 추가
3. sys::astra_os 백엔드 통합
4. 커스텀 Rust 툴체인 빌드

**Week 3: Servo 첫 렌더링** 🎯
1. Servo 클론 및 빌드
2. ASTRA.OS 포트 구현
3. 커널 통합
4. 첫 HTML 렌더링 성공!

상세 계획: `SERVO_INTEGRATION_PLAN.md`, `NEXT_STEPS.md` 참고

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

**커널 레벨:**
- \`bootloader\` (0.9) - BIOS/UEFI 부팅
- \`x86_64\` (0.15) - CPU 제어, 페이징
- \`linked_list_allocator\` - 힙 할당자

**네트워크:**
- \`smoltcp\` (0.11) - TCP/IP 스택 (no_std)

**그래픽:**
- \`embedded-graphics\` (0.8) - 2D 그래픽 라이브러리 (no_std)
- VGA Mode 13h - 320x200, 256색 팔레트

---

**ASTRA.OS** - *Advancing Systems Through Rust Architecture*
