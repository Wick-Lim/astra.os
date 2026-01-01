# Servo Integration Plan - Option B (Fast Path)

## Goal
Get Servo browser engine rendering on ASTRA.OS within 3-4 weeks using std stub implementation.

## Timeline

### Week 1-2: Rust std Stub Implementation
**Approach:** Fork Rust and create x86_64-astra_os target with minimal std implementation

#### Step 1: Setup Rust Development Environment (1-2 days)
- [ ] Fork rust-lang/rust repository
- [ ] Create `x86_64-astra_os.json` target specification
- [ ] Add target to rustc_target/src/spec/mod.rs
- [ ] Configure build system for new target

#### Step 2: Implement sys Backend (3-5 days)
Create `library/std/src/sys/astra_os/` with stub implementations:

**sys/astra_os/mod.rs** - Main module
```rust
pub mod args;
pub mod cmath;
pub mod env;
pub mod fs;
pub mod io;
pub mod net;
pub mod os;
pub mod path;
pub mod pipe;
pub mod process;
pub mod stdio;
pub mod thread;
pub mod time;
```

**Priority Modules for Servo:**

1. **fs.rs** (HIGH PRIORITY)
   - `File::open()` - Return hardcoded HTML content
   - `read_to_string()` - Return "`<!DOCTYPE html><h1>ASTRA.OS</h1>`"
   - Other ops return `NotSupported` error

2. **thread.rs** (HIGH PRIORITY)
   - `Thread::spawn()` - Execute immediately (no real threads yet)
   - `Thread::current()` - Return dummy thread ID
   - `sleep()` - Call `hlt` instruction

3. **time.rs** (HIGH PRIORITY)
   - `Instant::now()` - Return tick count from PIT timer
   - `SystemTime::now()` - Return fixed epoch (2026-01-01)

4. **net.rs** (MEDIUM PRIORITY)
   - Integrate with existing smoltcp stack
   - `TcpStream::connect()` - Use smoltcp sockets
   - Or stub out for first demo

5. **stdio.rs** (MEDIUM PRIORITY)
   - `stdout()` - Map to serial port
   - `stdin()` - Return empty (no input yet)

6. **process.rs** (LOW PRIORITY)
   - `exit()` - Halt kernel
   - `Command::spawn()` - Return error (no processes)

7. **env.rs** (LOW PRIORITY)
   - `vars()` - Return empty iterator
   - `current_dir()` - Return "/"

#### Step 3: Build Custom Rust Toolchain (1 day)
```bash
cd rust
./x.py build --stage 1 library/std --target x86_64-astra_os
```

### Week 3: Servo Integration (5-7 days)

#### Step 4: Minimal Servo Port (2-3 days)
- [ ] Clone servo repository
- [ ] Create `ports/astra-os/` directory
- [ ] Implement minimal Window/Rendering backend
- [ ] Configure Cargo.toml for x86_64-astra_os target

**ports/astra-os/window.rs** - Window implementation
```rust
pub struct Window {
    framebuffer: &'static mut Framebuffer,
}

impl Window {
    pub fn new() -> Self {
        // Use our VGA framebuffer (or future GOP framebuffer)
        Window {
            framebuffer: unsafe { get_framebuffer() }
        }
    }

    pub fn present(&mut self, pixels: &[u8]) {
        // Copy Servo's rendered output to VGA memory
        self.framebuffer.blit(pixels);
    }
}
```

**ports/astra-os/main.rs** - Entry point
```rust
pub fn servo_main() {
    let window = Window::new();
    let servo = Servo::new(window);

    // Load hardcoded HTML
    servo.load_url("file:///index.html");

    // Render loop
    loop {
        servo.render_frame();
        window.present(servo.get_pixels());
    }
}
```

#### Step 5: Build Servo (1 day)
```bash
cd servo
cargo build --release --target x86_64-astra_os
```
Expected: Linker errors → fix missing symbols → repeat

#### Step 6: Kernel Integration (2-3 days)
- [ ] Link Servo as static library into kernel
- [ ] Export kernel functions to Servo (framebuffer access, etc.)
- [ ] Handle Servo's memory allocation through our allocator

**kernel/Cargo.toml**
```toml
[dependencies]
servo = { path = "../../servo", default-features = false }

[profile.release]
lto = true  # Link-time optimization for static linking
```

**kernel/src/main.rs** - Call Servo
```rust
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // ... existing initialization ...

    serial_println!("Starting Servo browser engine...");

    // Call into Servo
    servo::ports::astra_os::servo_main();

    loop { x86_64::instructions::hlt(); }
}
```

### Week 4: First Demo & Iteration

#### Step 7: First Rendering (1-2 days)
Target: Display "Hello ASTRA.OS" rendered by Servo on VGA screen

**Expected Output:**
```
[Serial Log]
ASTRA.OS v0.3.0 - Servo Integration
Kernel starting...
Memory initialized
Interrupts initialized
Framebuffer initialized
Starting Servo browser engine...
Servo: Loading file:///index.html
Servo: Parsing HTML...
Servo: Building layout tree...
Servo: Rendering frame 1...

[VGA Display]
+----------------------------------+
|   ASTRA.OS                       |
|                                  |
|   Hello from Servo!              |
|                                  |
+----------------------------------+
```

#### Step 8: Debug & Optimize (2-3 days)
- [ ] Fix memory issues (Servo needs ~200MB heap)
- [ ] Optimize rendering pipeline
- [ ] Add basic event handling

#### Step 9: Enhanced Demo (2-3 days)
- [ ] Add keyboard input → Servo events
- [ ] Implement ramdisk with multiple HTML files
- [ ] Add CSS styling support
- [ ] Mouse cursor integration

## Technical Challenges & Solutions

### Challenge 1: Memory Requirements
**Problem:** Servo needs 200+ MB, we have 2MB heap
**Solution:**
1. Increase heap size to 256MB in allocator.rs
2. Test memory allocation with large allocations first
3. Profile Servo's actual minimum requirement

### Challenge 2: Resolution Limitation
**Problem:** VGA Mode 13h is 320x200, too small for Servo
**Solution:**
1. First demo: Force Servo viewport to 320x200 (proof of concept)
2. Week 4+: Upgrade to UEFI GOP (1024x768 or higher)

### Challenge 3: No Threading
**Problem:** Servo uses parallel layout/rendering
**Solution:**
1. Thread::spawn() stub executes immediately (serial execution)
2. Slower but functional for proof of concept
3. Week 5+: Implement cooperative threading

### Challenge 4: Missing std APIs
**Problem:** Servo may use std APIs we haven't implemented
**Solution:**
1. Implement on-demand as link errors appear
2. Start with panic stubs that log to serial
3. Replace with real implementations incrementally

## Milestones

- [ ] **Milestone 1** (Day 5): Rust toolchain builds for x86_64-astra_os
- [ ] **Milestone 2** (Day 10): std stubs compile successfully
- [ ] **Milestone 3** (Day 15): Servo builds for x86_64-astra_os
- [ ] **Milestone 4** (Day 20): Servo linked into kernel, no panics
- [ ] **Milestone 5** (Day 21): First pixel rendered by Servo 🎉
- [ ] **Milestone 6** (Day 25): Full "Hello World" HTML page rendered
- [ ] **Milestone 7** (Day 30): Multiple pages, CSS, basic interactivity

## Success Criteria

**Minimum Viable Demo (Week 3):**
- Servo engine integrated into kernel
- Renders hardcoded HTML: `<h1>ASTRA.OS</h1><p>Powered by Servo</p>`
- Displays on VGA framebuffer (even if scaled down)
- Serial output shows Servo logs

**Enhanced Demo (Week 4):**
- Renders formatted HTML with CSS
- Multiple HTML pages in ramdisk
- Basic keyboard navigation
- No crashes, stable rendering loop

## Next Actions (Start Now)

1. Clone Rust repository: `git clone https://github.com/rust-lang/rust.git ~/rust`
2. Create feature branch: `git checkout -b astra-os-target`
3. Create target spec file: `compiler/rustc_target/src/spec/x86_64_astra_os.rs`
4. Create sys backend: `library/std/src/sys/astra_os/`
5. Test build: `./x.py build --stage 1`

Let's start! 🚀
