#!/bin/bash
# ASTRA.OS Docker Build Script
# For macOS users who can't run Buildroot natively

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

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

# Check Docker
check_docker() {
    if ! command -v docker &> /dev/null; then
        error "Docker not found. Install Docker Desktop first."
    fi

    if ! docker info &> /dev/null; then
        error "Docker daemon not running. Start Docker Desktop."
    fi

    info "Docker is ready"
}

# Build Docker image
build_image() {
    info "Building Docker image..."
    cd "$PROJECT_ROOT"
    docker-compose build
    info "Docker image ready"
}

# Run build in container
run_build() {
    info "Starting build in Docker container..."
    info "This will take 60-90 minutes on first run"

    cd "$PROJECT_ROOT"

    docker-compose run --rm builder bash -c "
        cd /astra

        # Configure if not done
        if [ ! -f output/.config ]; then
            echo '[Docker] Configuring Buildroot...'
            cd buildroot
            make O=/astra/output BR2_EXTERNAL=/astra defconfig BR2_DEFCONFIG=/astra/configs/astra_defconfig
            cd /astra
        fi

        # Build
        echo '[Docker] Building...'
        cd buildroot
        make O=/astra/output BR2_EXTERNAL=/astra -j\$(nproc)

        echo '[Docker] Build complete!'
        echo 'Kernel: output/images/bzImage'
        echo 'Rootfs: output/images/rootfs.ext4'
    "
}

# Run menuconfig in container
run_menuconfig() {
    info "Opening menuconfig in Docker..."

    cd "$PROJECT_ROOT"

    docker-compose run --rm builder bash -c "
        cd /astra/buildroot
        make O=/astra/output BR2_EXTERNAL=/astra menuconfig
    "
}

# Run Linux menuconfig
run_linux_menuconfig() {
    info "Opening Linux menuconfig in Docker..."

    cd "$PROJECT_ROOT"

    docker-compose run --rm builder bash -c "
        cd /astra/buildroot
        make O=/astra/output BR2_EXTERNAL=/astra linux-menuconfig
    "
}

# Enter shell
run_shell() {
    info "Entering Docker shell..."

    cd "$PROJECT_ROOT"
    docker-compose run --rm builder bash
}

# Clean
clean() {
    info "Cleaning build output..."

    cd "$PROJECT_ROOT"
    rm -rf output

    info "Cleaned"
}

# Clean Docker volumes
clean_all() {
    info "Cleaning everything including Docker volumes..."

    cd "$PROJECT_ROOT"
    rm -rf output
    docker-compose down -v

    info "Cleaned"
}

# Help
show_help() {
    echo "ASTRA.OS Docker Build Script"
    echo ""
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  build           Build the system in Docker"
    echo "  menuconfig      Open Buildroot config menu"
    echo "  linux-menuconfig Open Linux kernel config"
    echo "  shell           Enter Docker shell"
    echo "  clean           Remove build output"
    echo "  clean-all       Remove output + Docker volumes"
    echo "  help            Show this help"
    echo ""
    echo "First time:"
    echo "  $0 build        # Takes 60-90 minutes"
    echo ""
    echo "After build:"
    echo "  ./scripts/run-qemu.sh graphics-gl"
}

# Main
cd "$PROJECT_ROOT"

case "${1:-help}" in
    build)
        check_docker
        build_image
        run_build
        ;;
    menuconfig)
        check_docker
        run_menuconfig
        ;;
    linux-menuconfig)
        check_docker
        run_linux_menuconfig
        ;;
    shell)
        check_docker
        build_image
        run_shell
        ;;
    clean)
        clean
        ;;
    clean-all)
        clean_all
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        error "Unknown command: $1"
        ;;
esac
