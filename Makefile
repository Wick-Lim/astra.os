# ASTRA.OS Makefile
# Convenience wrapper for build scripts

.PHONY: all configure build clean menuconfig linux-menuconfig run help

all: build

configure:
	@./scripts/build.sh configure

build:
	@./scripts/build.sh build

clean:
	@./scripts/build.sh clean

menuconfig:
	@./scripts/build.sh menuconfig

linux-menuconfig:
	@./scripts/build.sh linux-menuconfig

savedefconfig:
	@./scripts/build.sh savedefconfig

run:
	@./scripts/run-qemu.sh

run-graphics:
	@./scripts/run-qemu.sh graphics

run-gl:
	@./scripts/run-qemu.sh graphics-gl

help:
	@echo "ASTRA.OS Build System"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@echo "  configure       - Configure Buildroot"
	@echo "  build           - Build the system"
	@echo "  clean           - Clean build output"
	@echo "  menuconfig      - Open Buildroot config"
	@echo "  linux-menuconfig - Open kernel config"
	@echo "  savedefconfig   - Save current config"
	@echo "  run             - Run in QEMU (text)"
	@echo "  run-graphics    - Run in QEMU (graphics)"
	@echo "  run-gl          - Run in QEMU (GPU accel)"
	@echo "  help            - Show this help"
