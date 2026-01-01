# ASTRA.OS Architecture Overview

## System Stack

```
┌─────────────────────────────────────────────────────────────┐
│                      SERVO BROWSER ENGINE                    │
│  - HTML/CSS Parser                                          │
│  - Layout Engine (WebRender-lite)                           │
│  - JavaScript (SpiderMonkey - future)                       │
└─────────────────────────────────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────┐
│               RUST std LIBRARY (Our Implementation)          │
│  - std::fs (Hardcoded HTML → Ramdisk → FAT32)              │
│  - std::thread (Immediate → Cooperative → Preemptive)       │
│  - std::time (PIT ticks → RTC)                              │
│  - std::net (Stubs → smoltcp integration)                   │
│  - std::io, env, process, etc.                              │
└─────────────────────────────────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    ASTRA.OS KERNEL                          │
│  ┌─────────────────┬─────────────────┬──────────────────┐  │
│  │ Memory Manager  │  VGA Graphics   │  Interrupts      │  │
│  │ - Paging        │  - Mode 13h     │  - IDT/GDT       │  │
│  │ - Heap (256MB)  │  - 320x200x256  │  - Timer (PIT)   │  │
│  │ - Allocator     │  - Framebuffer  │  - Keyboard/Mouse│  │
│  └─────────────────┴─────────────────┴──────────────────┘  │
│  ┌─────────────────┬─────────────────┬──────────────────┐  │
│  │ Drivers         │  Network Stack  │  File System     │  │
│  │ - Serial        │  - smoltcp      │  - Ramdisk       │  │
│  │ - PS/2 Mouse    │  - TCP/IP       │  - (FAT32 future)│  │
│  │ - VGA           │  - Ethernet     │                  │  │
│  └─────────────────┴─────────────────┴──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    HARDWARE / QEMU                          │
│  - x86_64 CPU                                               │
│  - VGA Display                                              │
│  - PS/2 Keyboard/Mouse                                      │
│  - Serial Port                                              │
│  - Network Interface (future)                               │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow: Loading a Web Page

### Current Implementation (Week 3 Demo)

```
User Request
    │
    ▼
kernel_main() calls servo_main()
    │
    ▼
Servo::load_url("file:///index.html")
    │
    ▼
std::fs::File::open("index.html")  ← Our std implementation
    │
    ▼
sys::astra_os::fs::File::open()    ← Hardcoded HTML in memory
    │
    ▼
Returns: "<!DOCTYPE html><h1>ASTRA.OS</h1>..."
    │
    ▼
Servo HTML Parser
    │
    ▼
DOM Tree: html → body → h1("ASTRA.OS")
    │
    ▼
Servo Layout Engine
    │
    ▼
Layout Tree: Box(0,0,320,200) → Text(10,50,"ASTRA.OS")
    │
    ▼
Servo Renderer (WebRender-lite)
    │
    ▼
Pixel Buffer: [u8; 320*200]
    │
    ▼
ports::astra_os::Window::present()
    │
    ▼
framebuffer::fill_rect() / draw_pixel()
    │
    ▼
VGA Memory Write (0xA0000)
    │
    ▼
Screen Display! 🎉
```

### Future Implementation (Phase 6+)

```
User types "http://example.com" in address bar
    │
    ▼
Keyboard interrupt → Event queue
    │
    ▼
Servo processes input
    │
    ▼
DNS lookup via std::net::UdpSocket
    │
    ▼
sys::astra_os::net → smoltcp DNS query
    │
    ▼
Network packet via Ethernet driver
    │
    ▼
HTTP GET request via std::net::TcpStream
    │
    ▼
Response → HTML Parser → Render
    │
    ▼
Screen Display
```

## Memory Layout

```
Virtual Address Space (x86_64)

0x0000_0000_0000_0000  ┌─────────────────────────────┐
                       │  Null Guard Page            │
                       ├─────────────────────────────┤
0x0000_0000_0040_0000  │  Kernel Code (.text)        │
                       │  - main.rs                  │
                       │  - drivers/                 │
                       │  - memory/                  │
                       ├─────────────────────────────┤
                       │  Kernel Data (.data, .bss)  │
                       ├─────────────────────────────┤
0x0000_0000_00A0_0000  │  VGA Memory (mapped)        │ ← Identity mapped
0x0000_0000_00C0_0000  │  [128KB for VGA Mode 13h]   │
                       ├─────────────────────────────┤
0x0000_0000_0100_0000  │  Heap (256 MB)              │
                       │  - Kernel allocations       │
                       │  - Servo allocations        │
                       │  - std allocations          │
0x0000_0000_1000_0000  ├─────────────────────────────┤
                       │  Stack (grows down)         │
                       ├─────────────────────────────┤
                       │  Memory Map (bootloader)    │
                       ├─────────────────────────────┤
                       │  Page Tables                │
                       └─────────────────────────────┘
```

## Thread Model Evolution

### Phase 4 (Current): No Threads
```rust
// std::thread::spawn executes immediately
std::thread::spawn(|| {
    println!("This runs immediately, blocking the caller");
});
println!("This prints after thread completes");
```

### Phase 7: Cooperative Threading
```rust
// Task queue with manual yields
std::thread::spawn(|| {
    for i in 0..100 {
        println!("{}", i);
        std::thread::yield_now(); // Explicit yield
    }
});
```

### Phase 8+: Preemptive Multitasking
```rust
// Real threads with timer-based preemption
std::thread::spawn(|| {
    loop {
        println!("Background task");
        // Automatically preempted after time slice
    }
});
```

## Build Process

### Current (no_std kernel)
```
kernel/src/main.rs
    │
    ▼
rustc --target x86_64-browser_os.json
    │
    ▼
kernel.elf
    │
    ▼
bootimage (wraps with bootloader)
    │
    ▼
bootimage-kernel.bin
    │
    ▼
QEMU
```

### Future (with std + Servo)
```
Servo source code
    │
    ▼
Our custom Rust compiler (with x86_64-astra_os target)
    │
    ▼
libservo.rlib (static library)
    │                              kernel/src/main.rs
    │                                      │
    ▼                                      ▼
    └──────────────► Link ◄────────────────┘
                      │
                      ▼
                 kernel.elf (with Servo embedded!)
                      │
                      ▼
                 bootimage
                      │
                      ▼
              bootimage-kernel.bin
                      │
                      ▼
                   QEMU
```

## File System Evolution

### Week 3: Hardcoded Files
```rust
// sys/astra_os/fs.rs
match path {
    "index.html" => "<!DOCTYPE html>...",
    "style.css" => "body { color: red; }",
    _ => Err(NotFound),
}
```

### Week 4: Ramdisk
```rust
// In-memory file system
struct Ramdisk {
    files: HashMap<PathBuf, Vec<u8>>,
}

// Populated at boot from embedded resources
FILES.insert("index.html", include_bytes!("index.html"));
```

### Phase 6: Real File System (FAT32)
```rust
// Read from actual disk
pub fn open(path: &Path) -> Result<File> {
    let inode = fat32::lookup_path(path)?;
    let sectors = fat32::read_clusters(inode.cluster)?;
    Ok(File { data: sectors })
}
```

## Performance Expectations

### Week 3 Demo (Serial Execution)
- Page load: ~5-10 seconds (no parallelism)
- Rendering: ~10 FPS (VGA writes are slow)
- Memory: ~50 MB used by Servo

### Phase 7 (Cooperative Threading)
- Page load: ~2-3 seconds (parallel HTML/CSS parse)
- Rendering: ~30 FPS
- Memory: ~100 MB

### Phase 8 (Full Optimization)
- Page load: ~500ms
- Rendering: ~60 FPS (VGA limit)
- Memory: ~150 MB

## Key Design Decisions

### 1. Why Stub std Instead of Porting Servo?
**Pro:**
- All Rust programs work, not just Servo
- One-time effort, benefits entire ecosystem
- Standard interface, easier maintenance
- Future: Can run cargo, rustc, anything!

**Con:**
- More initial work (but pays off long-term)
- Need to fork Rust compiler

### 2. Why Start with VGA Mode 13h?
**Pro:**
- Simple, well-documented
- Works everywhere (QEMU, real hardware)
- No bootloader changes needed
- Quick iteration

**Con:**
- Low resolution (320x200)
- Limited colors (256)
- Slow memory writes

**Future:** UEFI GOP for 1920x1080x32bit

### 3. Why Immediate Thread Execution?
**Pro:**
- Zero complexity
- No scheduler needed
- Deterministic behavior
- Easy debugging

**Con:**
- No parallelism
- Slower page loads

**Future:** Implement real scheduler in Phase 7

## Testing Strategy

### Unit Tests (Phase 5+)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_file_read() {
        let content = std::fs::read_to_string("index.html").unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
    }
}
```

### Integration Tests
```rust
#[test]
fn test_servo_render() {
    let servo = Servo::new();
    servo.load_url("file:///test.html");
    servo.render_frame();
    assert_eq!(servo.get_pixels()[0], expected_color);
}
```

### Manual Testing Checklist
- [ ] Kernel boots without panic
- [ ] Serial output shows init messages
- [ ] VGA display shows graphics
- [ ] Servo initializes
- [ ] HTML parsing works
- [ ] First pixel rendered
- [ ] Full page rendered
- [ ] No memory leaks
- [ ] Stable for 60 seconds

## Development Timeline

```
Week 1-2: std Implementation ──────────┐
                                       ▼
Week 2-3: Servo Integration ──────────► [MILESTONE: First Render]
                                       ▼
Week 4: Debugging & Polish ────────────► Public Demo
                                       ▼
Month 2: High-Res Graphics (GOP) ──────► Better Display
                                       ▼
Month 3: File System (FAT32) ──────────► Real Web Pages
                                       ▼
Month 4: Threading ────────────────────► Performance
                                       ▼
Month 5-6: Network Stack ──────────────► Live Websites
                                       ▼
Month 7-12: Full std + Optimization ───► Production Ready
```

## Current Status

**What Works:**
- ✅ Kernel boots reliably
- ✅ Memory management (paging, heap)
- ✅ VGA graphics (320x200)
- ✅ Interrupts (timer, keyboard, mouse)
- ✅ Serial debugging
- ✅ Network stack (smoltcp initialized)

**In Progress:**
- 🔨 std library stubs (95% complete, ready to deploy)
- 🔨 Rust compiler fork (ready to start)
- 🔨 Servo integration (planned)

**Not Started:**
- ⏳ High-resolution graphics
- ⏳ File system
- ⏳ Multitasking
- ⏳ Full std implementation

## Next Command

To begin the Rust compiler fork:

```bash
cd ~/
git clone https://github.com/rust-lang/rust.git
cd rust
git checkout -b astra-os-target

# This will take 5-10 minutes...
# Meanwhile, we'll prepare the kernel changes!
```

**LET'S BUILD THIS! 🚀**
