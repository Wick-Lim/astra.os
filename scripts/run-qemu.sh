#!/bin/bash
# ASTRA.OS QEMU Runner

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_ROOT/output"

KERNEL="$OUTPUT_DIR/images/bzImage"
ROOTFS="$OUTPUT_DIR/images/rootfs.ext4"

# Default settings
MEMORY="2G"
CPUS="2"
MODE="text"  # text, graphics, graphics-gl

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check if images exist
check_images() {
    if [ ! -f "$KERNEL" ]; then
        error "Kernel not found: $KERNEL\nRun ./scripts/build.sh build first"
    fi

    if [ ! -f "$ROOTFS" ]; then
        error "Root filesystem not found: $ROOTFS\nRun ./scripts/build.sh build first"
    fi
}

# Run in text mode (serial console)
run_text() {
    info "Starting ASTRA.OS in text mode..."

    qemu-system-x86_64 \
        -kernel "$KERNEL" \
        -drive file="$ROOTFS",format=raw,if=virtio \
        -append "root=/dev/vda console=ttyS0 rw" \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -nographic \
        -enable-kvm 2>/dev/null || \
    qemu-system-x86_64 \
        -kernel "$KERNEL" \
        -drive file="$ROOTFS",format=raw,if=virtio \
        -append "root=/dev/vda console=ttyS0 rw" \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -nographic
}

# Run with graphics (no GL acceleration)
run_graphics() {
    info "Starting ASTRA.OS with graphics..."

    qemu-system-x86_64 \
        -kernel "$KERNEL" \
        -drive file="$ROOTFS",format=raw,if=virtio \
        -append "root=/dev/vda console=tty0 rw" \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -device virtio-vga \
        -device virtio-keyboard-pci \
        -device virtio-mouse-pci \
        -display gtk \
        -enable-kvm 2>/dev/null || \
    qemu-system-x86_64 \
        -kernel "$KERNEL" \
        -drive file="$ROOTFS",format=raw,if=virtio \
        -append "root=/dev/vda console=tty0 rw" \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -device virtio-vga \
        -device virtio-keyboard-pci \
        -device virtio-mouse-pci \
        -display gtk
}

# Run with OpenGL acceleration (for Wayland/Servo)
run_graphics_gl() {
    info "Starting ASTRA.OS with GPU acceleration..."

    # Check for virgl support
    if ! qemu-system-x86_64 -device help 2>&1 | grep -q "virtio-gpu-gl"; then
        error "QEMU does not support virtio-gpu-gl. Install QEMU with virgl support."
    fi

    qemu-system-x86_64 \
        -kernel "$KERNEL" \
        -drive file="$ROOTFS",format=raw,if=virtio \
        -append "root=/dev/vda console=tty0 rw" \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -device virtio-gpu-gl-pci \
        -device virtio-keyboard-pci \
        -device virtio-mouse-pci \
        -display gtk,gl=on \
        -enable-kvm 2>/dev/null || \
    qemu-system-x86_64 \
        -kernel "$KERNEL" \
        -drive file="$ROOTFS",format=raw,if=virtio \
        -append "root=/dev/vda console=tty0 rw" \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -device virtio-gpu-gl-pci \
        -device virtio-keyboard-pci \
        -device virtio-mouse-pci \
        -display gtk,gl=on
}

# Help
show_help() {
    echo "ASTRA.OS QEMU Runner"
    echo ""
    echo "Usage: $0 [mode] [options]"
    echo ""
    echo "Modes:"
    echo "  text        Serial console only (default)"
    echo "  graphics    VGA graphics without GL"
    echo "  graphics-gl OpenGL accelerated (for Wayland)"
    echo ""
    echo "Options:"
    echo "  -m <size>   Memory size (default: 2G)"
    echo "  -c <count>  CPU count (default: 2)"
    echo ""
    echo "Examples:"
    echo "  $0                  # Text mode"
    echo "  $0 graphics         # Graphics mode"
    echo "  $0 graphics-gl -m 4G  # GL with 4GB RAM"
    echo ""
    echo "Exit QEMU:"
    echo "  Text mode: Ctrl+A, then X"
    echo "  Graphics: Close window"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        text)
            MODE="text"
            shift
            ;;
        graphics)
            MODE="graphics"
            shift
            ;;
        graphics-gl)
            MODE="graphics-gl"
            shift
            ;;
        -m)
            MEMORY="$2"
            shift 2
            ;;
        -c)
            CPUS="$2"
            shift 2
            ;;
        help|--help|-h)
            show_help
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Main
check_images

case "$MODE" in
    text)
        run_text
        ;;
    graphics)
        run_graphics
        ;;
    graphics-gl)
        run_graphics_gl
        ;;
esac
