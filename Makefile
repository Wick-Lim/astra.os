# ASTRA.OS Makefile

.PHONY: build clean menuconfig linux-menuconfig shell run help

# Build (always uses Docker)
build:
	@./scripts/docker-build.sh build

clean:
	@./scripts/docker-build.sh clean-all

menuconfig:
	@./scripts/docker-build.sh menuconfig

linux-menuconfig:
	@./scripts/docker-build.sh linux-menuconfig

shell:
	@./scripts/docker-build.sh shell

# Run in QEMU
run:
	@./scripts/run-qemu.sh

run-graphics:
	@./scripts/run-qemu.sh graphics

run-gl:
	@./scripts/run-qemu.sh graphics-gl

# Help
help:
	@echo "ASTRA.OS Build System"
	@echo ""
	@echo "Build:"
	@echo "  make build      - Build in Docker (60-90 min)"
	@echo "  make clean      - Clean build output"
	@echo "  make menuconfig - Buildroot config"
	@echo "  make shell      - Enter build shell"
	@echo ""
	@echo "Run:"
	@echo "  make run        - QEMU text mode"
	@echo "  make run-gl     - QEMU with GPU"
