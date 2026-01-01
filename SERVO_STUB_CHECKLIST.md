# Servo to ASTRA.OS Porting Checklist

## Quick Reference: Dependencies to Stub/Replace

### TIER 1: Critical Blockers (Must Address First)

- [ ] **winit** (Window management)
  - Location: `ports/servoshell/` uses winit extensively
  - Action: Create `astra_windowing` abstraction layer
  - Files: `window.rs`, event loop code
  - Interfaces: `EventLoop`, `Window`, `WindowEvent`

- [ ] **surfman** (OpenGL context)
  - Location: `components/compositing/compositor.rs`
  - Action: Implement ASTRA.OS backend or fork
  - Approach: Map to framebuffer or custom GPU driver
  - Methods: `Connection::new()`, `Device::create_context()`

- [ ] **tokio** (Async runtime)
  - Location: `components/net/lib.rs`
  - Action: Custom reactor or alternative runtime
  - Options: Fork `mio`, use `smol`/`async-std`, or custom executor
  - Critical: Network I/O, timers, task spawning

### TIER 2: Platform Integration (Next Priority)

- [ ] **Fonts** (Text rendering)
  - Remove: CoreText (macOS), DirectWrite (Windows), fontconfig (Linux)
  - Keep: `harfbuzz-sys` (bundled), `read-fonts`, `skrifa`
  - Action: Bundle font files, pure Rust stack
  - Files: `components/fonts/platform/`

- [ ] **Memory allocator**
  - Current: `tikv-jemalloc-sys` (POSIX only)
  - Action: Enable `use-system-allocator` feature OR implement `GlobalAlloc`
  - File: `components/allocator/lib.rs`

- [ ] **IPC** (Multi-process)
  - Current: Unix sockets, shared memory
  - Action: Enable `force-inprocess` feature
  - Alternative: ASTRA.OS message passing
  - File: Cargo.toml feature flags

- [ ] **File I/O**
  - Current: `std::fs`, POSIX APIs
  - Action: Redirect to ASTRA.OS VFS (TAR filesystem)
  - Integration point: Kernel syscalls

### TIER 3: Optional (Can Disable)

- [ ] **GStreamer** (Media)
  - Action: Disable `media-gstreamer` feature
  - Keep: `servo-media-dummy` backend

- [ ] **WebGPU** (Advanced graphics)
  - Dependencies: `wgpu-core`, platform backends
  - Action: Disable `webgpu` feature initially

- [ ] **WebXR** (VR/AR)
  - Dependencies: `openxr`, platform APIs
  - Action: Disable `webxr` feature

- [ ] **Bluetooth/Gamepad**
  - Action: Disable `bluetooth`, `gamepad` features

- [ ] **Clipboard** (`arboard`)
  - Action: Stub with no-op implementation

---

## Build Configuration Changes

### Minimal Feature Set

```toml
# In ports/servoshell/Cargo.toml
[features]
default = []
astra-minimal = [
    "js_jit",              # If SpiderMonkey works on bare metal
    "max_log_level",
    "vello_cpu",           # CPU rendering fallback
]

# Disable these:
# gamepad = ...
# media-gstreamer = ...
# native-bluetooth = ...
# webxr = ...
# webgpu = ...
```

### Build Command

```bash
cargo build \
  --target x86_64-unknown-astra \
  --no-default-features \
  --features "astra-minimal" \
  --profile production-stripped
```

---

## Critical Source Files to Modify

### Window/Event Loop
- `ports/servoshell/window.rs` - Main window abstraction
- `ports/servoshell/lib.rs` - Event loop initialization
- `ports/servoshell/desktop/` - Desktop-specific code (remove/adapt)

### Graphics Integration
- `components/compositing/compositor.rs` - Rendering pipeline
- `components/compositing/windowing.rs` - Window integration
- Create: `components/surfman/backends/astra/` - Custom backend

### Fonts
- `components/fonts/platform/mod.rs` - Platform selection
- `components/fonts/platform/linux/` - Use as template
- Create: `components/fonts/platform/astra/` - Custom implementation

### Networking
- `components/net/lib.rs` - Network manager
- `components/net/connector.rs` - TCP connection
- May need: Custom DNS resolver

---

## SpiderMonkey (mozjs) Considerations

**Status:** HIGHEST RISK - extensive POSIX dependencies

### Option A: Port SpiderMonkey
- Requires: pthread API, mmap, signals
- Stub needed: ~20+ system calls
- Effort: 3-6 weeks

### Option B: Alternative JS Engine
- **QuickJS**: Smaller, C-based, POSIX-light
- **Boa**: Pure Rust, no POSIX, incomplete spec
- **Deno V8**: Too complex for bare metal

### Recommendation
1. Try minimal SpiderMonkey port first
2. If too complex, consider Boa for basic JS support
3. Worst case: No JavaScript (HTML/CSS only renderer)

---

## Testing Strategy

### Phase 1: Library Build
```bash
# Attempt to build libservo without platform deps
cargo build --lib --no-default-features
```
Expected: Compile errors in platform-specific code

### Phase 2: Stub Platform APIs
```rust
// Example stub
#[cfg(target_os = "astra")]
mod platform {
    pub fn create_window() -> Result<Window, Error> {
        // ASTRA.OS implementation
        todo!("Implement ASTRA.OS windowing")
    }
}
```

### Phase 3: Minimal Executable
```bash
# Build with minimal features
cargo build --bin servo --no-default-features --features "astra-minimal"
```

### Phase 4: Headless Test
```rust
// Test: Parse and layout HTML without display
use servo::Servo;

let html = "<html><body><h1>Hello ASTRA.OS</h1></body></html>";
let engine = Servo::new(HeadlessConfig);
engine.load_html(html);
// Verify: Layout computed, no crash
```

### Phase 5: Framebuffer Output
- Render to ASTRA.OS framebuffer
- Verify: Visual output of HTML

---

## Build Dependencies to Install

### Host System (Build Machine)
```bash
# Required
- Rust 1.86+ (workspace rust-version)
- Python 3.x (via uv: `uv run python`)
- C++ compiler (for SpiderMonkey, aws-lc-sys)
- Git

# Optional (if using system libraries)
- pkg-config
- cmake
```

### Avoid System Libraries
- Use bundled features where possible
- Static linking preferred
- No fontconfig, no X11, no GStreamer

---

## Size Optimization

### Profile Settings
```toml
[profile.astra-release]
inherits = "production"
opt-level = "z"        # Optimize for size
lto = "fat"            # Maximum link-time optimization
codegen-units = 1      # Single codegen unit
strip = true           # Remove symbols
panic = "abort"        # Smaller panic handler
```

### Feature Pruning
- Disable all optional crates
- Use `cargo-bloat` to identify large dependencies
- Consider: Removing image format support (keep PNG/JPEG only)

### Expected Size
- Minimal: 80-100 MB (stripped)
- With optimizations: 60-80 MB possible
- SpiderMonkey alone: 30-40 MB

---

## Known Issues & Workarounds

### Issue 1: Build Scripts Require Python
**Problem:** `script_bindings/build.rs` needs Python for codegen
**Workaround:** Pre-generate bindings on host, check into repo

### Issue 2: SpiderMonkey Requires POSIX Threads
**Problem:** Bare metal has no pthread
**Workaround:** Stub pthread API or use single-threaded mode

### Issue 3: Tokio Requires OS Event Loop
**Problem:** epoll/kqueue not available
**Workaround:** Custom reactor or polling-based executor

### Issue 4: Font Discovery Needs Filesystem
**Problem:** fontconfig, platform APIs expect filesystem
**Workaround:** Bundle fonts in binary, hardcode paths

---

## Success Criteria

### Milestone 1: Build Success
- [ ] `cargo build` completes without errors
- [ ] All stubbed functions compile
- [ ] Binary links successfully

### Milestone 2: Headless Rendering
- [ ] Parse HTML without crash
- [ ] Compute CSS layout
- [ ] Generate render tree

### Milestone 3: Visual Output
- [ ] Render to framebuffer
- [ ] Display simple HTML page
- [ ] Verify correctness visually

### Milestone 4: Interactivity
- [ ] JavaScript execution (if SpiderMonkey works)
- [ ] Handle input events
- [ ] Navigate between pages

### Milestone 5: Networking
- [ ] HTTP GET request
- [ ] Render remote page
- [ ] TLS connection

---

## Estimated Effort

| Phase | Task | Time Estimate |
|-------|------|---------------|
| 1 | Set up cross-compilation | 1-2 days |
| 2 | Stub windowing (winit replacement) | 1-2 weeks |
| 3 | Stub graphics (surfman backend) | 1-2 weeks |
| 4 | Port allocator/threading | 3-5 days |
| 5 | Integrate VFS/file I/O | 1 week |
| 6 | Port networking (tokio/custom) | 2-3 weeks |
| 7 | SpiderMonkey porting/stubbing | 3-6 weeks |
| 8 | Font subsystem | 1 week |
| 9 | Testing & debugging | 2-4 weeks |
| 10 | Optimization | 1-2 weeks |
| **TOTAL** | **Minimal browser** | **3-5 months** |
| **TOTAL** | **Full-featured** | **6-12 months** |

---

## Contact & Resources

### Servo Community
- GitHub: https://github.com/servo/servo
- Zulip Chat: https://servo.zulipchat.com/
- Contributing Guide: https://github.com/servo/servo/blob/main/CONTRIBUTING.md

### Useful Documentation
- Servo Book: https://book.servo.org/
- WebIDL Bindings: https://github.com/servo/servo/wiki/Bindings
- Build System: https://book.servo.org/hacking/building-servo.html

### Similar Ports
- Android port: `ports/servoshell/` (see Android conditionals)
- OHOS port: Search for `target_env = "ohos"`
- Embedded Servo: https://github.com/servo/servo/issues (search "embedded")

---

**IMPORTANT:** This is a complex porting effort. Start with the smallest possible build (library only, no features) and incrementally add functionality. Test each component in isolation before integration.
