#!/bin/bash

# Browser OS - QEMU Runner Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
KERNEL_IMAGE="$PROJECT_ROOT/target/x86_64-browser_os/release/bootimage-kernel.bin"

# 색상 정의
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 함수: 메시지 출력
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# QEMU 설치 확인
if ! command -v qemu-system-x86_64 &> /dev/null; then
    error "qemu-system-x86_64 not found. Please install QEMU."
    echo "  macOS: brew install qemu"
    echo "  Ubuntu: sudo apt install qemu-system-x86"
    exit 1
fi

# 커널 이미지 존재 확인
if [ ! -f "$KERNEL_IMAGE" ]; then
    warn "Kernel image not found. Building..."
    cd "$PROJECT_ROOT"
    make build
fi

# QEMU 옵션 설정
MEMORY="512M"
SERIAL="stdio"
NETWORK="-device virtio-net,netdev=net0 -netdev user,id=net0"

# 디버그 모드 확인
if [ "$1" = "debug" ]; then
    info "Starting QEMU in debug mode (GDB on port 1234)..."
    GDB_OPTIONS="-s -S"
else
    info "Starting QEMU..."
    GDB_OPTIONS=""
fi

# QEMU 실행
qemu-system-x86_64 \
    -drive format=raw,file="$KERNEL_IMAGE" \
    $NETWORK \
    -m "$MEMORY" \
    -serial "$SERIAL" \
    $GDB_OPTIONS
