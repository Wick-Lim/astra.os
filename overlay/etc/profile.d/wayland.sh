#!/bin/sh
# Wayland environment variables for ASTRA.OS

# XDG runtime directory (required by Wayland)
export XDG_RUNTIME_DIR=/run/user/0
mkdir -p "$XDG_RUNTIME_DIR"
chmod 0700 "$XDG_RUNTIME_DIR"

# Wayland settings
export XDG_SESSION_TYPE=wayland
export WAYLAND_DISPLAY=wayland-0

# Use Wayland for Qt apps
export QT_QPA_PLATFORM=wayland

# Use Wayland for GTK apps
export GDK_BACKEND=wayland

# Mesa driver selection (auto-detect)
# Uncomment to force specific driver:
# export MESA_LOADER_DRIVER_OVERRIDE=iris    # Intel
# export MESA_LOADER_DRIVER_OVERRIDE=radeonsi # AMD
# export MESA_LOADER_DRIVER_OVERRIDE=nouveau  # NVIDIA
# export MESA_LOADER_DRIVER_OVERRIDE=virgl    # QEMU

# Keyboard layout
export XKB_DEFAULT_LAYOUT=us
