#!/bin/bash
# ASTRA.OS Build Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILDROOT_DIR="$PROJECT_ROOT/buildroot"
OUTPUT_DIR="$PROJECT_ROOT/output"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check dependencies
check_deps() {
    info "Checking dependencies..."

    local deps=(make gcc g++ patch wget cpio unzip rsync bc)
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            error "Missing dependency: $dep"
        fi
    done

    info "All dependencies found"
}

# Configure buildroot
configure() {
    info "Configuring Buildroot..."

    if [ ! -d "$BUILDROOT_DIR" ]; then
        error "Buildroot directory not found. Run: git submodule update --init"
    fi

    cd "$BUILDROOT_DIR"

    # Set BR2_EXTERNAL
    export BR2_EXTERNAL="$PROJECT_ROOT"

    # Create output directory
    mkdir -p "$OUTPUT_DIR"

    # Load defconfig
    make O="$OUTPUT_DIR" BR2_EXTERNAL="$PROJECT_ROOT" astra_defconfig

    info "Configuration complete"
}

# Build
build() {
    info "Building ASTRA.OS..."

    cd "$BUILDROOT_DIR"

    # Parallel build with available cores
    local cores=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

    make O="$OUTPUT_DIR" BR2_EXTERNAL="$PROJECT_ROOT" -j"$cores"

    info "Build complete!"
    info "Kernel: $OUTPUT_DIR/images/bzImage"
    info "Rootfs: $OUTPUT_DIR/images/rootfs.ext4"
}

# Clean
clean() {
    info "Cleaning build..."

    if [ -d "$OUTPUT_DIR" ]; then
        rm -rf "$OUTPUT_DIR"
        info "Cleaned"
    else
        info "Nothing to clean"
    fi
}

# Menuconfig
menuconfig() {
    info "Opening menuconfig..."

    cd "$BUILDROOT_DIR"
    make O="$OUTPUT_DIR" BR2_EXTERNAL="$PROJECT_ROOT" menuconfig
}

# Linux menuconfig
linux_menuconfig() {
    info "Opening Linux menuconfig..."

    cd "$BUILDROOT_DIR"
    make O="$OUTPUT_DIR" BR2_EXTERNAL="$PROJECT_ROOT" linux-menuconfig
}

# Save defconfig
savedefconfig() {
    info "Saving defconfig..."

    cd "$BUILDROOT_DIR"
    make O="$OUTPUT_DIR" BR2_EXTERNAL="$PROJECT_ROOT" savedefconfig

    # Copy to configs
    cp "$OUTPUT_DIR/defconfig" "$PROJECT_ROOT/configs/astra_defconfig"

    info "Saved to configs/astra_defconfig"
}

# Help
show_help() {
    echo "ASTRA.OS Build Script"
    echo ""
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  configure     Configure Buildroot with astra_defconfig"
    echo "  build         Build the entire system"
    echo "  clean         Remove build output"
    echo "  menuconfig    Open Buildroot configuration menu"
    echo "  linux-menuconfig  Open Linux kernel configuration menu"
    echo "  savedefconfig Save current config to astra_defconfig"
    echo "  help          Show this help message"
    echo ""
    echo "First time setup:"
    echo "  $0 configure"
    echo "  $0 build"
    echo "  ../scripts/run-qemu.sh"
}

# Main
case "${1:-help}" in
    configure)
        check_deps
        configure
        ;;
    build)
        build
        ;;
    clean)
        clean
        ;;
    menuconfig)
        menuconfig
        ;;
    linux-menuconfig)
        linux_menuconfig
        ;;
    savedefconfig)
        savedefconfig
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        error "Unknown command: $1"
        ;;
esac
