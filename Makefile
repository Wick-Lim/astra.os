.PHONY: build run clean kernel

# 기본 타겟
all: build

# 커널 빌드
kernel:
	@echo "Building kernel..."
	cd kernel && cargo build --release

# 부팅 이미지 생성
build: kernel
	@echo "Creating bootable image..."
	cd kernel && cargo bootimage --release

# QEMU로 실행
run: build
	@echo "Starting QEMU..."
	qemu-system-x86_64 \
		-drive format=raw,file=target/x86_64-browser_os/release/bootimage-kernel.bin \
		-device virtio-net,netdev=net0 \
		-netdev user,id=net0 \
		-m 256M \
		-serial stdio \
		-display cocoa

# 클린
clean:
	@echo "Cleaning build artifacts..."
	cd kernel && cargo clean
	rm -rf target/

# 디버그 모드로 QEMU 실행
debug: build
	@echo "Starting QEMU in debug mode..."
	qemu-system-x86_64 \
		-drive format=raw,file=target/x86_64-browser_os/release/bootimage-kernel.bin \
		-device virtio-net,netdev=net0 \
		-netdev user,id=net0 \
		-m 512M \
		-serial stdio \
		-s -S

# 헬프
help:
	@echo "Browser OS - Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all     - Build the kernel and create bootable image (default)"
	@echo "  kernel  - Build the kernel only"
	@echo "  build   - Create bootable image"
	@echo "  run     - Run the OS in QEMU"
	@echo "  debug   - Run the OS in QEMU with GDB support"
	@echo "  clean   - Remove build artifacts"
	@echo "  help    - Show this help message"
