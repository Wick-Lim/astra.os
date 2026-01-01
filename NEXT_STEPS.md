# ASTRA.OS - Next Steps for Servo Integration

## ✅ What We've Completed

### Phase 4: VGA Pixel Graphics
- ✅ VGA Mode 13h (320x200, 256 colors) fully working
- ✅ Fixed rendering crash issue (write_volatile for VGA memory)
- ✅ embedded-graphics DrawTarget implementation
- ✅ Full screen rendering confirmed working

### Rust std Stub Implementation (Ready to Deploy)
Created complete stub implementations for ASTRA.OS target:

**Files Created in `/rust-target-spec/`:**
- ✅ `x86_64_astra_os.rs` - Target specification
- ✅ `sys_astra_os/mod.rs` - Main sys module
- ✅ `sys_astra_os/fs.rs` - File system (hardcoded HTML)
- ✅ `sys_astra_os/thread.rs` - Threading (immediate execution)
- ✅ `sys_astra_os/time.rs` - Time & Duration (PIT timer)
- ✅ `sys_astra_os/stdio.rs` - stdin/stdout/stderr (serial port)
- ✅ `sys_astra_os/net.rs` - Network (stubs, ready for smoltcp)
- ✅ `sys_astra_os/env.rs` - Environment variables
- ✅ `sys_astra_os/args.rs` - Command-line arguments
- ✅ `sys_astra_os/process.rs` - Process management
- ✅ `sys_astra_os/io.rs` - IO error handling
- ✅ `sys_astra_os/alloc.rs` - Memory allocation (kernel heap)
- ✅ `REMAINING_STUBS.rs` - All other required modules

## 🎯 Next Steps (2-3 Weeks to Servo Demo)

### Step 1: Fork and Build Rust Toolchain (2-3 days)

#### 1.1 Fork Rust Repository
```bash
cd ~/
git clone https://github.com/rust-lang/rust.git
cd rust
git checkout stable  # or use specific version
git checkout -b astra-os-target
```

#### 1.2 Copy Our Stub Files
```bash
# Copy target spec
cp /Users/wick/Documents/workspaces/astra.os/rust-target-spec/x86_64_astra_os.rs \
   compiler/rustc_target/src/spec/

# Register target in mod.rs
# Edit: compiler/rustc_target/src/spec/mod.rs
# Add: ("x86_64-astra_os", x86_64_astra_os),

# Copy sys backend
cp -r /Users/wick/Documents/workspaces/astra.os/rust-target-spec/sys_astra_os \
      library/std/src/sys/

# Register sys backend in mod.rs
# Edit: library/std/src/sys/mod.rs
# Add: #[cfg(target_os = "astra_os")] mod astra_os;
```

#### 1.3 Build Rust Toolchain
```bash
# Configure build
./configure --enable-extended --tools=cargo

# Build (takes 1-2 hours)
./x.py build --stage 1 library/std --target x86_64-astra_os

# Install
./x.py install --stage 1

# Verify
rustc --version
rustc --print target-list | grep astra
```

**Expected Output:**
```
x86_64-astra_os
```

### Step 2: Update ASTRA.OS Kernel for std Support (1-2 days)

#### 2.1 Add Kernel Functions for std
Edit `kernel/src/lib.rs` (create if doesn't exist):

```rust
// Export kernel functions for std library

#[no_mangle]
pub unsafe extern "C" fn astra_os_serial_write(data: *const u8, len: usize) {
    let slice = core::slice::from_raw_parts(data, len);
    crate::serial::_print(core::str::from_utf8_unchecked(slice));
}

#[no_mangle]
pub unsafe extern "C" fn astra_os_serial_read(_data: *mut u8, _len: usize) -> usize {
    0 // No input yet
}

#[no_mangle]
pub unsafe extern "C" fn astra_os_alloc(size: usize, align: usize) -> *mut u8 {
    use core::alloc::{GlobalAlloc, Layout};
    let layout = Layout::from_size_align_unchecked(size, align);
    crate::memory::allocator::ALLOCATOR.alloc(layout)
}

#[no_mangle]
pub unsafe extern "C" fn astra_os_dealloc(ptr: *mut u8, size: usize, align: usize) {
    use core::alloc::{GlobalAlloc, Layout};
    let layout = Layout::from_size_align_unchecked(size, align);
    crate::memory::allocator::ALLOCATOR.dealloc(ptr, layout);
}

#[no_mangle]
pub unsafe extern "C" fn astra_os_realloc(ptr: *mut u8, old_size: usize, align: usize, new_size: usize) -> *mut u8 {
    use core::alloc::{GlobalAlloc, Layout};
    let old_layout = Layout::from_size_align_unchecked(old_size, align);
    crate::memory::allocator::ALLOCATOR.realloc(ptr, old_layout, new_size)
}

#[no_mangle]
pub unsafe extern "C" fn astra_os_exit(code: i32) -> ! {
    crate::serial_println!("Kernel exit: code {}", code);
    loop {
        x86_64::instructions::hlt();
    }
}

// Timer tick function (called by PIT interrupt)
#[no_mangle]
pub unsafe extern "C" fn astra_os_timer_tick() {
    // Increment tick counter in std::sys::astra_os::time
    extern "C" {
        fn astra_os_timer_tick_internal();
    }
    astra_os_timer_tick_internal();
}
```

#### 2.2 Update Kernel Config
Edit `kernel/.cargo/config.toml`:

```toml
[build]
target = "x86_64-astra_os"  # Changed from browser_os

[unstable]
build-std = ["core", "compiler_builtins", "alloc", "std"]  # Added std!
build-std-features = ["compiler-builtins-mem"]
```

#### 2.3 Test std in Kernel
Create `kernel/src/test_std.rs`:

```rust
// Test std functionality
pub fn test_std() {
    use std::fs::File;
    use std::io::Read;

    serial_println!("Testing std::fs...");

    match File::open("index.html") {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serial_println!("Read {} bytes from index.html", contents.len());
            serial_println!("First 100 chars: {}", &contents[..100]);
        }
        Err(e) => {
            serial_println!("Error opening file: {}", e);
        }
    }
}
```

Add to `main.rs`:
```rust
mod test_std;

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // ... existing init ...

    test_std::test_std();

    loop { x86_64::instructions::hlt(); }
}
```

### Step 3: Clone and Configure Servo (1 day)

```bash
cd ~/
git clone https://github.com/servo/servo.git
cd servo

# Create ASTRA.OS port
mkdir -p ports/astra-os/src

# Create minimal port
cat > ports/astra-os/Cargo.toml <<EOF
[package]
name = "servo-astra-os"
version = "0.1.0"
edition = "2021"

[dependencies]
servo = { path = "../../components/servo" }

[profile.release]
opt-level = "z"  # Optimize for size
lto = true
EOF
```

Create `ports/astra-os/src/main.rs`:

```rust
// Minimal Servo port for ASTRA.OS
#![no_std]
#![no_main]

extern crate alloc;

use servo::servo_url::ServoUrl;
use servo::Servo;

#[no_mangle]
pub extern "C" fn servo_main() {
    println!("Servo starting on ASTRA.OS...");

    // Create Servo instance
    let url = ServoUrl::parse("file:///index.html").unwrap();

    // TODO: Implement window/rendering backend
    println!("Loading: {}", url);

    // Render loop
    loop {
        // Servo render frame
        // Copy to framebuffer
    }
}
```

### Step 4: Build Servo for ASTRA.OS (2-3 days)

```bash
cd ~/servo

# Build with our custom Rust
RUSTC=~/rust/build/x86_64-unknown-linux-gnu/stage1/bin/rustc \
cargo build --release --target x86_64-astra_os -p servo-astra-os
```

**Expected:** Many errors initially. Fix incrementally:
1. Missing symbols → Add stubs
2. Unsupported features → Disable in Cargo.toml
3. Platform-specific code → Add astra_os cfg

### Step 5: Integrate Servo into Kernel (3-4 days)

#### 5.1 Link Servo Static Library
Edit `kernel/Cargo.toml`:

```toml
[dependencies]
servo-astra-os = { path = "../../servo/target/x86_64-astra_os/release" }
```

#### 5.2 Call Servo from Kernel
Edit `kernel/src/main.rs`:

```rust
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // ... init ...

    serial_println!("Starting Servo browser engine...");

    // Call Servo
    extern "C" {
        fn servo_main();
    }
    unsafe { servo_main(); }

    loop { x86_64::instructions::hlt(); }
}
```

### Step 6: First Rendering! (1-2 days)

**Goal:** See Servo render `<h1>ASTRA.OS</h1>` on screen

**Debugging Steps:**
1. Check serial output for Servo logs
2. Verify HTML parsing works
3. Confirm layout tree generation
4. Test pixel writing to framebuffer
5. Celebrate first render! 🎉

## Expected Timeline

| Week | Milestone | Status |
|------|-----------|--------|
| Week 1 | Rust toolchain built | ⏳ |
| Week 1-2 | Kernel updated for std | ⏳ |
| Week 2 | Servo builds for astra_os | ⏳ |
| Week 3 | Servo integrated in kernel | ⏳ |
| **Week 3** | **First Servo rendering** 🎉 | ⏳ |
| Week 4+ | Improvements & features | ⏳ |

## Potential Issues & Solutions

### Issue 1: Heap Size Too Small
**Problem:** Servo needs 200+ MB, we have 2MB
**Solution:** Increase heap in `memory/allocator.rs`:
```rust
pub const HEAP_SIZE: usize = 256 * 1024 * 1024; // 256MB
```

### Issue 2: Resolution Too Low
**Problem:** VGA 320x200 too small for Servo
**Solution:** Scale down or upgrade to UEFI GOP:
```rust
// Option A: Scale Servo output to 320x200
servo.set_viewport_size(320, 200);

// Option B: Use UEFI GOP (Phase 5)
// Requires UEFI bootloader update
```

### Issue 3: Missing std APIs
**Problem:** Servo uses std APIs we haven't implemented
**Solution:** Add on-demand:
```rust
// When linker says: undefined reference to `std::foo::bar`
// Add stub in sys_astra_os/foo.rs
pub fn bar() -> Result<T, Error> {
    unsupported!()
}
```

### Issue 4: Slow Performance (No Threads)
**Problem:** Serial execution is slow
**Solution:** Phase 7 - Add cooperative threading:
```rust
// thread.rs: Change from immediate execution to task queue
static TASK_QUEUE: Mutex<Vec<Task>> = Mutex::new(Vec::new());
```

## Success Criteria

**Minimum Viable Demo:**
- ✅ Kernel boots
- ✅ std functions work (println!, File::read, etc.)
- ✅ Servo initializes without panic
- ✅ HTML parsed successfully
- ✅ At least one pixel rendered by Servo
- ✅ Serial log shows "Servo rendered frame 1"

**Stretch Goals:**
- CSS styling works
- Multiple pages load
- Keyboard input processed
- 60 FPS rendering

## Let's Start!

**Immediate next command:**
```bash
cd ~/
git clone https://github.com/rust-lang/rust.git
```

This will take about 5-10 minutes. While it clones, we can prepare the kernel changes!

Ready to proceed? 🚀
