# ASTRA.OS

**A**dvanced **S**ystem for **T**omorrow's **R**evolutionary **A**pplications

A minimal Linux-based operating system designed to run the Servo browser engine with GPU acceleration.

## Architecture

```
┌─────────────────────────────────────┐
│         Servo Browser               │
├─────────────────────────────────────┤
│    Cage (Wayland compositor)        │
├─────────────────────────────────────┤
│     Mesa (Vulkan/OpenGL) + DRM      │
├─────────────────────────────────────┤
│         Linux Kernel 6.6            │
├─────────────────────────────────────┤
│           Hardware                  │
└─────────────────────────────────────┘
```

## Features

- **Minimal footprint**: ~300MB ISO image
- **Fast boot**: ~5 seconds in QEMU
- **GPU support**: Intel, AMD, NVIDIA (nouveau), QEMU (virgl)
- **Wayland native**: No X11 overhead

## Quick Start

### Prerequisites

- Docker (for macOS) or Linux host
- QEMU for testing
- Git

### Build on macOS (Docker)

```bash
# Clone repository
git clone --recursive https://github.com/user/astra.os
cd astra.os

# Build in Docker (60-90 min first time)
make docker

# Test in QEMU
make run-gl
```

### Build on Linux (Native)

```bash
# Clone repository
git clone --recursive https://github.com/user/astra.os
cd astra.os

# Configure and build
make configure
make build

# Test in QEMU
make run-gl
```

### Run in QEMU

```bash
make run          # Text mode (serial console)
make run-graphics # VGA graphics
make run-gl       # GPU acceleration (virgl)
```

Exit QEMU: `Ctrl+A`, then `X` (text mode) or close window (graphics).

## Project Structure

```
astra.os/
├── buildroot/          # Buildroot (git submodule)
├── configs/
│   ├── astra_defconfig # Buildroot configuration
│   └── linux.config    # Kernel configuration
├── overlay/            # Root filesystem overlay
│   └── etc/init.d/     # Init scripts
├── packages/           # Custom Buildroot packages
│   └── servo/          # Servo browser package
├── scripts/
│   ├── build.sh        # Native build script
│   ├── docker-build.sh # Docker build script
│   └── run-qemu.sh     # QEMU runner
├── Dockerfile          # Build environment
├── docker-compose.yml
└── output/             # Build output (generated)
```

## Development

### Modify Configuration

```bash
# macOS (Docker)
make docker-menuconfig
make docker-linux-menuconfig

# Linux (Native)
make menuconfig
make linux-menuconfig
```

### Enter Build Shell

```bash
# macOS
make docker-shell

# Then inside container:
cd buildroot
make O=/astra/output BR2_EXTERNAL=/astra menuconfig
```

### Clean Build

```bash
make docker-clean  # macOS: clean + remove Docker volumes
make clean         # Linux: clean output only
```

## Roadmap

- [x] Phase 1: Buildroot environment
- [x] Phase 2: Graphics stack (Mesa, Wayland, Cage)
- [ ] Phase 3: Servo integration
- [ ] Phase 4: Boot automation
- [ ] Phase 5: Real hardware (UEFI, ISO)
- [ ] Phase 6: Optimization

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- [Buildroot](https://buildroot.org/) - Embedded Linux build system
- [Servo](https://servo.org/) - The parallel browser engine
- [Mesa](https://mesa3d.org/) - OpenGL/Vulkan implementation
- [Cage](https://github.com/cage-kiosk/cage) - Wayland kiosk compositor
