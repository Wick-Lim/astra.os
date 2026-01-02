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

- Linux host (macOS works for building, Linux for KVM acceleration)
- Git, Make, GCC, and standard build tools
- QEMU for testing

### Build

```bash
# Clone repository
git clone --recursive https://github.com/user/astra.os
cd astra.os

# Configure and build
./scripts/build.sh configure
./scripts/build.sh build
```

First build takes 30-60 minutes depending on your machine.

### Run in QEMU

```bash
# Text mode (serial console)
./scripts/run-qemu.sh

# Graphics mode
./scripts/run-qemu.sh graphics

# With GPU acceleration (requires virgl)
./scripts/run-qemu.sh graphics-gl
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
│   ├── build.sh        # Build script
│   └── run-qemu.sh     # QEMU runner
├── output/             # Build output (generated)
└── archive/            # Legacy kernel code
```

## Development

### Modify Configuration

```bash
# Buildroot configuration
./scripts/build.sh menuconfig

# Kernel configuration
./scripts/build.sh linux-menuconfig

# Save changes
./scripts/build.sh savedefconfig
```

### Clean Build

```bash
./scripts/build.sh clean
./scripts/build.sh build
```

## Roadmap

- [x] Phase 1: Basic bootable Linux
- [ ] Phase 2: Graphics stack (Mesa, Wayland, Cage)
- [ ] Phase 3: Servo integration
- [ ] Phase 4: Boot automation
- [ ] Phase 5: Real hardware support (UEFI, ISO)
- [ ] Phase 6: Optimization

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- [Buildroot](https://buildroot.org/) - Embedded Linux build system
- [Servo](https://servo.org/) - The parallel browser engine
- [Mesa](https://mesa3d.org/) - OpenGL/Vulkan implementation
- [Cage](https://github.com/cage-kiosk/cage) - Wayland kiosk compositor
