# Servo Build Dependency Analysis for ASTRA.OS

**Date:** 2026-01-01
**Target:** Bare-metal ASTRA.OS (no standard OS)
**Purpose:** Identify dependencies to stub/replace for cross-compilation

---

## Executive Summary

Servo is a complex browser engine with **215+ workspace dependencies** and extensive platform-specific code. For ASTRA.OS bare-metal execution, we need to:

1. **Stub windowing layer** (winit, egui, surfman platform backends)
2. **Replace JavaScript engine** or stub mozjs system dependencies
3. **Stub native graphics** (OpenGL/Vulkan context creation)
4. **Remove/stub multimedia** (GStreamer)
5. **Adapt networking** (tokio runtime needs custom executor)
6. **Replace file I/O** (integrate with ASTRA.OS VFS)

**Estimated minimum binary size:** 80-150 MB (stripped, minimal features)

---

## 1. Full Dependency Tree Analysis

### 1.1 Core Workspace Members

```
servo/
├── ports/servoshell/          # Main executable - MUST REPLACE
├── components/
│   ├── servo/                 # Core libservo library
│   ├── script/                # JavaScript/DOM bindings (CRITICAL)
│   ├── layout/                # CSS layout engine (KEEP)
│   ├── net/                   # Networking (ADAPT)
│   ├── compositing/           # Rendering pipeline (ADAPT)
│   ├── webrender/             # GPU rendering (ADAPT)
│   ├── webgl/                 # WebGL support (STUB)
│   ├── webgpu/                # WebGPU support (STUB)
│   ├── fonts/                 # Font rendering (ADAPT)
│   ├── script_bindings/       # WebIDL bindings (KEEP)
│   └── [40+ other components]
```

### 1.2 Critical External Dependencies

#### JavaScript Engine
- **mozjs (SpiderMonkey) 0.14.4**
  - Requires: libz-sys, C++ compiler, Python for codegen
  - Platform: POSIX APIs (pthread, mmap, etc.)
  - **Status:** CRITICAL - needs extensive stubbing for bare metal

#### Graphics & Rendering
- **webrender** (Git: servo/webrender)
  - GPU-accelerated 2D rendering
  - Dependencies: gleam, euclid, surfman

- **surfman 0.11.0**
  - Cross-platform OpenGL context management
  - Features: sm-x11 (Linux), sm-angle (Windows), chains
  - **Platform-specific:** MUST STUB platform backends

- **mozangle 0.5.3**
  - ANGLE (OpenGL ES on D3D) - Windows only
  - **Status:** REMOVE for ASTRA.OS

- **glow 0.16.0**
  - OpenGL bindings (safe wrapper)
  - **Status:** KEEP - provides GL function pointers

#### Windowing (MUST REPLACE)
- **winit 0.30.12**
  - Cross-platform windowing and input
  - **Platform:** Linux (X11/Wayland), Windows (Win32), macOS (Cocoa)
  - **Replacement needed:** ASTRA.OS window system integration

- **egui 0.33.3** + egui-winit + egui_glow
  - Immediate mode GUI for browser UI
  - **Status:** REPLACE with ASTRA.OS native UI or minimal stub

- **raw-window-handle 0.6**
  - Safe abstraction for window handles
  - **Status:** KEEP API, implement ASTRA.OS backend

#### Fonts
- **Platform-specific font APIs:**
  - **Linux:** freetype-sys 0.20, fontconfig_sys
  - **macOS:** core-text 20.1, core-foundation, core-graphics
  - **Windows:** dwrote 0.11.5
  - **Android:** XML-based font configs

- **harfbuzz-sys 0.6.1** (bundled feature)
  - Text shaping
  - **Status:** KEEP - use bundled static build

- **Modern font stack:**
  - read-fonts 0.35.0, skrifa 0.37.0
  - **Status:** KEEP - pure Rust

#### Networking
- **hyper 1.8** + hyper-rustls 0.27 + hyper-util 0.1
  - HTTP/1.1 and HTTP/2 client
  - **Status:** KEEP

- **tokio 1.x** (multi-thread runtime)
  - Async runtime with epoll/kqueue/IOCP
  - **Platform:** POSIX or Windows APIs
  - **Replacement needed:** Custom executor for ASTRA.OS

- **rustls 0.23** + rustls-platform-verifier 0.6.2
  - TLS implementation
  - Platform verifier uses OS certificate stores
  - **Status:** STUB platform verifier

- **async-tungstenite 0.32** + tungstenite 0.28
  - WebSocket support
  - **Status:** KEEP

#### Multimedia
- **gstreamer 0.24** (optional: media-gstreamer feature)
  - Video/audio playback
  - Multiple -sys crates (gstreamer-sys, gstreamer-gl-sys, etc.)
  - **Status:** DISABLE - not needed for minimal build

- **servo-media** + servo-media-dummy + servo-media-gstreamer
  - Media playback abstraction
  - **Status:** Use dummy backend

#### Memory Allocation
- **tikv-jemalloc-sys 0.6.1** + tikv-jemallocator 0.6.1
  - High-performance allocator
  - **Platform:** POSIX (not Windows/OHOS)
  - **Status:** REPLACE with ASTRA.OS allocator or use system allocator

#### Image Formats
- **image 0.25**
  - Features: avif, bmp, gif, ico, jpeg, png, webp, rayon
  - **Status:** KEEP - pure Rust decoders

- **resvg 0.45.0** + vello 0.6 + vello_cpu 0.0.4
  - SVG rendering (CPU and GPU)
  - **Status:** KEEP for SVG support

#### CSS & Layout
- **stylo** (Git: servo/stylo, branch: 2025-11-01)
  - CSS engine (from Firefox)
  - Multiple related crates: selectors, servo_arc, etc.
  - **Status:** KEEP - core layout dependency

- **taffy 0.9.2**
  - Flexbox/Grid layout (partial?)
  - Features: grid, calc, std
  - **Status:** KEEP

#### WebGL/WebGPU
- **wgpu-core 26** + wgpu-types 26
  - WebGPU implementation
  - Platform-specific backends:
    - macOS/iOS: Metal
    - Linux: Vulkan + GLES
    - Windows: DX12 + Vulkan
  - **Status:** STUB or disable - complex GPU dependencies

#### IPC & Serialization
- **ipc-channel 0.20.2**
  - Cross-process communication
  - Features: force-inprocess (for single-process mode)
  - **Status:** KEEP with force-inprocess

- **serde 1.0.228** ecosystem
  - serde_json, serde_bytes, bincode
  - **Status:** KEEP

#### Parsing
- **html5ever 0.36.1** + markup5ever + xml5ever
  - HTML/XML parsers
  - **Status:** KEEP

- **cssparser 0.36**
  - CSS parser
  - **Status:** KEEP

#### Crypto
- **aws-lc-rs 1.15** (default-features = false)
  - Cryptographic primitives
  - Includes aws-lc-sys (native build)
  - **Status:** KEEP but may need build.rs adjustments

- **rustls 0.23**, **aes-gcm 0.10.3**, **chacha20poly1305 0.10**, etc.
  - TLS and Web Crypto API support
  - **Status:** KEEP - mostly pure Rust

#### Bluetooth/Gamepad/XR (Optional)
- **bluetooth** (native-bluetooth feature)
  - Linux: BlueZ, Android/macOS: platform APIs
  - **Status:** DISABLE

- **gilrs 0.11.0** (gamepad feature)
  - Gamepad input
  - **Status:** DISABLE

- **openxr 0.20** (webxr feature)
  - VR/AR support
  - **Status:** DISABLE

---

## 2. Platform-Specific Dependencies to Stub/Replace

### 2.1 Critical OS-Specific Code

| Component | Platform Code | ASTRA.OS Action |
|-----------|--------------|-----------------|
| **winit** | X11/Wayland/Win32/Cocoa APIs | REPLACE with ASTRA.OS window manager |
| **surfman** | GLX/EGL/WGL/CGL context creation | STUB platform backends, use framebuffer |
| **egui/egui_glow** | winit integration | REPLACE or MINIMAL UI stub |
| **tokio** | epoll/kqueue/IOCP event loop | CUSTOM executor for ASTRA.OS |
| **rustls-platform-verifier** | OS cert stores (Security framework/WinCrypt) | STUB or disable cert verification |
| **fonts** | CoreText/DirectWrite/fontconfig | USE bundled fonts + freetype static |
| **mozjs** | POSIX threads, mmap, signals | EXTENSIVE stubbing needed |
| **ipc-channel** | Unix domain sockets/shared memory | USE force-inprocess feature |
| **tikv-jemalloc** | POSIX memory APIs | USE system allocator or custom |

### 2.2 Conditional Compilation Patterns

Found in `Cargo.toml` files:

```toml
# Linux-specific
[target.'cfg(target_os = "linux")'.dependencies]
freetype-sys = { workspace = true }
fontconfig_sys = { package = "yeslogic-fontconfig-sys", version = "6" }

# macOS-specific
[target.'cfg(target_os = "macos")'.dependencies]
core-text = "20.1"
core-foundation = "0.9"
mach2 = { workspace = true }

# Windows-specific
[target.'cfg(target_os = "windows")'.dependencies]
dwrote = { workspace = true }
windows-sys = { workspace = true }

# Android/OHOS-specific
[target.'cfg(any(target_os = "android", target_env = "ohos"))'.dependencies]
surfman = { workspace = true, features = ["sm-angle-default"] }
```

**For ASTRA.OS:** Add new target configuration:
```toml
[target.'cfg(target_os = "astra")'.dependencies]
# ASTRA.OS-specific dependencies
```

---

## 3. Build Script Requirements

### 3.1 Build.rs Scripts Found

1. **components/script_bindings/build.rs**
   - **Critical:** Generates WebIDL bindings
   - Requires: Python (via `uv run python`)
   - Runs: `codegen/run.py` to generate Rust from WebIDL
   - Output: Interface bindings, DOM types, PHF maps
   - **ASTRA.OS impact:** Python dependency for build-time only

2. **components/script/build.rs**
   - Copies generated files from script_bindings
   - No platform-specific logic

3. **components/servo/build.rs**
   - Conditional: GStreamer plugin generation (Python script)
   - Only runs if `media-gstreamer` feature enabled
   - **ASTRA.OS impact:** Disable feature

4. **ports/servoshell/build.rs**
   - Platform-specific:
     - Windows: Embed icon/manifest with winresource
     - macOS: Compile C code for thread counting
     - Android: Create libgcc.a workaround
   - Sets git SHA environment variable
   - **ASTRA.OS impact:** Add custom build logic

5. **components/shared/embedder/build.rs**
   - Not analyzed in detail, likely resource embedding

### 3.2 Build-Time Dependencies

- **Python 3.x** (via uv package manager)
  - WebIDL codegen
  - GStreamer plugin detection (if enabled)

- **C/C++ Compiler**
  - SpiderMonkey (mozjs) - extensive C++ build
  - Native font libraries (harfbuzz, freetype if not bundled)
  - aws-lc-sys (crypto)
  - Potentially surfman platform code

- **pkg-config** (Linux)
  - Find system libraries (fontconfig, gstreamer, etc.)

---

## 4. Minimum Feature Set for ASTRA.OS

### 4.1 Recommended Feature Flags

**DISABLE:**
```toml
# In servoshell Cargo.toml
default = []  # Start with nothing

# Explicitly disable:
# - gamepad
# - media-gstreamer
# - native-bluetooth
# - webxr
# - webgpu (initially)
```

**ENABLE:**
```toml
# Minimal working browser:
minimal = [
    "libservo/clipboard",      # Text operations
    "js_jit",                  # JavaScript JIT (if SpiderMonkey works)
    "max_log_level",           # Logging
    "vello_cpu",               # CPU-based rendering fallback
]
```

**For debugging:**
```toml
debug = [
    "debugmozjs",              # SpiderMonkey debugging
    "refcell_backtrace",       # Runtime borrow checking
    "js_backtrace",            # JS stack traces
]
```

### 4.2 Component-Level Features to Adjust

**libservo features:**
- KEEP: `clipboard`, `vello_cpu`
- MAYBE: `js_jit` (if bare metal supports JIT pages)
- DISABLE: `media-gstreamer`, `gamepad`, `webxr`, `webgpu`, `native-bluetooth`

**script features:**
- KEEP: Core JS/DOM
- DISABLE: `bluetooth`, `gamepad`, `webgpu`, `webxr`

**net features:**
- KEEP: HTTP client
- MAY NEED: Custom DNS resolver

**compositing features:**
- DISABLE: `webgpu`, `webxr`
- KEEP: Basic rendering

---

## 5. Estimated Binary Size

### 5.1 Component Size Estimates

| Component | Estimated Size (Release) | Notes |
|-----------|-------------------------|-------|
| **SpiderMonkey (mozjs)** | 30-50 MB | Largest dependency |
| **WebRender** | 5-10 MB | GPU rendering |
| **Stylo (CSS)** | 10-15 MB | CSS engine from Firefox |
| **Script/DOM** | 15-25 MB | DOM implementation + bindings |
| **Layout** | 5-8 MB | Layout engine |
| **Networking** | 3-5 MB | HTTP/TLS stack |
| **Image codecs** | 2-4 MB | PNG, JPEG, WebP, etc. |
| **Fonts** | 3-5 MB | Font rendering + shaping |
| **WebGL/WebGPU** | 10-15 MB | If enabled |
| **Base libraries** | 5-10 MB | IPC, utils, etc. |

**Total (minimal config):** 80-120 MB
**Total (with WebGL/GPU):** 100-150 MB
**Stripped binary:** -20-30% size reduction

### 5.2 Size Optimization Strategies

1. **Profile: production-stripped**
   ```toml
   [profile.production-stripped]
   inherits = "production"
   strip = true
   lto = true
   codegen-units = 1
   opt-level = "s"  # Optimize for size
   ```

2. **Disable unused features:**
   - No media playback: -5-10 MB
   - No WebGPU: -10-15 MB
   - No WebXR: -5 MB
   - JIT disabled: -10-15 MB (but much slower JS)

3. **Link-time optimization (LTO):**
   - Reduces code duplication
   - Already in production profile

4. **Remove debug info:**
   - `strip = true` removes symbols
   - Keep minimal debug for crash reports

---

## 6. Critical Dependencies Requiring Bare-Metal Adaptation

### 6.1 Tier 1: MUST STUB (Blockers)

#### **winit** - Window Management
**Current use:** Event loop, window creation, input handling
**ASTRA.OS stub location:** Create `astra_windowing` crate
**Stub interfaces:**
```rust
// Minimal trait to replace winit::event_loop::EventLoop
trait AstraEventLoop {
    fn run<F>(self, event_handler: F) where F: FnMut(Event);
    fn create_window(&self, attrs: WindowAttributes) -> Window;
}

// Replace winit::window::Window
struct AstraWindow {
    // Implement with ASTRA.OS window system
}
```
**Files to modify:** `ports/servoshell/window.rs`, event loop code

#### **surfman** - OpenGL Context Management
**Current use:** Create GL contexts for rendering
**ASTRA.OS stub location:** Fork surfman or create `surfman_astra` backend
**Stub approach:**
- Implement `Connection` trait for ASTRA.OS
- Return dummy GL context that maps to framebuffer
- Or integrate with ASTRA.OS GPU driver if available

**Files to modify:** `components/compositing/compositor.rs`

#### **tokio** - Async Runtime
**Current use:** Network I/O, timers, task spawning
**ASTRA.OS integration:**
- Port tokio's reactor to use ASTRA.OS event system
- OR: Use `tokio` with custom `mio` backend
- OR: Replace with simpler async runtime (async-std, smol)

**Files to modify:** `components/net/lib.rs`, network manager

### 6.2 Tier 2: SHOULD ADAPT

#### **fonts** - Font Rendering
**Current state:** Platform-specific (CoreText/DirectWrite/fontconfig)
**ASTRA.OS approach:**
- Use pure-Rust font stack: `read-fonts` + `skrifa`
- Bundle font files in binary or VFS
- Keep `harfbuzz-sys` with bundled feature for shaping

**Files to modify:** `components/fonts/platform/`

#### **ipc-channel** - Multi-Process Communication
**Current state:** Unix sockets / shared memory
**ASTRA.OS approach:**
- Use `force-inprocess` feature for single-process mode
- OR: Implement custom IPC with ASTRA.OS message passing

**Files to modify:** Enable feature in Cargo.toml

#### **allocator** - Memory Management
**Current state:** jemalloc (POSIX) or system allocator
**ASTRA.OS approach:**
- Use `use-system-allocator` feature
- Implement `GlobalAlloc` trait for ASTRA.OS heap

**Files to modify:** `components/allocator/lib.rs`

### 6.3 Tier 3: OPTIONAL (Can Disable)

- **gstreamer** - Disable `media-gstreamer` feature
- **bluetooth** - Disable feature
- **gamepad** - Disable feature
- **webxr** - Disable feature
- **arboard** (clipboard) - Stub with no-op initially

---

## 7. Recommended Build Strategy for ASTRA.OS

### Phase 1: Core Engine (No Graphics)
1. **Target:** Build `libservo` with headless mode
2. **Features:**
   ```toml
   [features]
   default = []
   astra-minimal = ["force-inprocess"]
   ```
3. **Stubs needed:**
   - Disable all windowing (conditional compile out)
   - Use dummy rendering context
   - Stub input events
4. **Goal:** Successfully compile core DOM/CSS/JS engine

### Phase 2: Graphics Integration
1. **Add framebuffer rendering**
   - Implement minimal `surfman` backend
   - Map WebRender output to ASTRA.OS framebuffer
2. **Add basic windowing**
   - Replace winit with ASTRA.OS window system
3. **Goal:** Display static HTML page

### Phase 3: Networking
1. **Port tokio or replace**
   - Integrate with ASTRA.OS network stack
2. **Implement custom DNS**
   - Use ASTRA.OS network syscalls
3. **Goal:** Load remote web pages

### Phase 4: Input & Interactivity
1. **Input events**
   - Map ASTRA.OS keyboard/mouse to Servo events
2. **JavaScript**
   - Ensure SpiderMonkey works on bare metal
   - May need custom memory mapping
3. **Goal:** Interactive web pages

### Phase 5: Optimization
1. **Reduce binary size**
   - Strip unused features
   - LTO and size optimization
2. **Performance tuning**
   - Profile and optimize hot paths
3. **Goal:** <100 MB stripped binary

---

## 8. Build Command Reference

### Current Servo Build
```bash
# Standard desktop build
cargo build --release -p servoshell

# Minimal features
cargo build --release -p servoshell --no-default-features --features "js_jit,max_log_level"

# Production optimized
cargo build --profile production -p servoshell

# Stripped binary
cargo build --profile production-stripped -p servoshell
```

### ASTRA.OS Cross-Compilation (Future)
```bash
# Set target
export ASTRA_TARGET=x86_64-unknown-astra

# Configure cross-compilation
cargo build --target $ASTRA_TARGET \
    --no-default-features \
    --features "astra-minimal" \
    --profile production-stripped

# Estimated binary: 80-120 MB
```

### Dependency Analysis Commands
```bash
# Full dependency tree
cargo tree --depth 3

# Platform-specific deps
cargo tree --target x86_64-unknown-linux-gnu -e normal

# Feature-gated deps
cargo tree --no-default-features --features "minimal"

# Build script overview
find . -name "build.rs" -exec wc -l {} +
```

---

## 9. Critical Files to Monitor

### Build Configuration
- `/servo/Cargo.toml` - Workspace dependencies
- `/servo/ports/servoshell/Cargo.toml` - Binary features
- `/servo/components/servo/Cargo.toml` - Library features
- `/servo/.cargo/config.toml` - Build configuration

### Platform Integration Points
- `/servo/ports/servoshell/window.rs` - Windowing abstraction
- `/servo/ports/servoshell/desktop/` - Desktop-specific code
- `/servo/components/compositing/compositor.rs` - Rendering pipeline
- `/servo/components/fonts/platform/` - Font backends
- `/servo/components/net/` - Networking stack

### Build Scripts (Python required)
- `/servo/components/script_bindings/build.rs` - WebIDL codegen
- `/servo/components/script_bindings/codegen/run.py` - Binding generator
- `/servo/ports/servoshell/build.rs` - Platform-specific builds

---

## 10. Conclusion & Next Steps

### Feasibility Assessment
**MEDIUM-HIGH COMPLEXITY** - Servo can be ported to ASTRA.OS, but requires significant work:

**Blockers:**
1. SpiderMonkey (mozjs) has deep POSIX dependencies
2. Windowing/input layer tightly coupled to OS
3. Graphics stack assumes OpenGL context
4. Networking assumes standard event loops (epoll/kqueue)

**Advantages:**
1. Most dependencies are pure Rust
2. Feature flags allow disabling complex subsystems
3. Clear separation between core engine and platform layer
4. WebRender can use CPU fallback (vello_cpu)

### Recommended Approach

**Option A: Minimal Integration**
- Use Servo as library, not full browser
- Headless rendering to framebuffer
- Single-process mode (no IPC)
- Disable multimedia, WebGL, WebGPU
- **Estimated effort:** 2-3 months

**Option B: Full Browser Port**
- Complete windowing integration
- OpenGL/Vulkan context from ASTRA.OS
- Full feature set
- **Estimated effort:** 6-12 months

### Immediate Action Items

1. **Set up cross-compilation toolchain** for ASTRA.OS target
2. **Fork critical dependencies:** winit, surfman, tokio
3. **Disable all optional features** and attempt minimal build
4. **Stub platform APIs** in order of priority:
   - Memory allocation
   - Threading (if SpiderMonkey needs it)
   - File I/O (integrate with ASTRA.OS VFS)
   - Window management
   - Graphics context
5. **Test core engine** in headless mode first

### Risk Mitigation

- **Alternative JavaScript engine:** Consider QuickJS or Boa (pure Rust) if SpiderMonkey proves too difficult
- **Simplified rendering:** Start with CPU-only rendering (no GPU)
- **Incremental porting:** Get basic HTML rendering working before adding JS/CSS complexity
- **Upstream collaboration:** Engage with Servo community for portability improvements

---

## Appendix A: Key Dependencies by Category

### Must Keep (Core Functionality)
- html5ever, xml5ever - HTML/XML parsing
- cssparser, stylo - CSS parsing and engine
- script, script_bindings - DOM and JavaScript
- layout - Layout engine
- webrender - Rendering backend
- hyper, rustls - HTTP/TLS
- image - Image decoding
- serde - Serialization

### Must Stub (OS-Specific)
- winit - Windowing
- surfman - OpenGL contexts
- tokio - Async runtime
- egui/egui_glow - UI toolkit
- ipc-channel - IPC (use force-inprocess)
- tikv-jemalloc - Allocator

### Can Disable (Optional)
- gstreamer - Media playback
- webgpu, wgpu-core - WebGPU
- openxr - VR/AR
- gilrs - Gamepad
- bluetooth - Bluetooth support
- servo-media-gstreamer - Media backend

### Uncertain (Needs Testing)
- mozjs (SpiderMonkey) - May require extensive stubbing
- aws-lc-sys - Crypto library build
- freetype-sys, harfbuzz-sys - Font rendering (use bundled versions)

---

**END OF ANALYSIS**
