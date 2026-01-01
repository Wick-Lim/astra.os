# Servo Integration Roadmap for ASTRA.OS

## Current Status (Phase 1 - COMPLETE ✅)

### What We Have:
- ✅ **Ring 0 Kernel** (no_std)
  - Process management
  - System call interface (int 0x80)
  - GDT with Ring 3 segments
  - Memory management

- ✅ **Ring 3 Userspace** (no_std)
  - Embedded in kernel binary
  - Simple HTML parser
  - DOM construction
  - Syscall wrappers
  - Working browser loop

### Architecture:
```
Kernel (Ring 0, no_std)
   ↕ iretq / int 0x80
Userspace (Ring 3, no_std + alloc)
   ↕ Simple HTML Parser
Basic Browser Functionality
```

## Phase 2: Add std to Userspace

### Goal:
Enable full std support in Ring 3 so Servo can run.

### Steps:

1. **Custom std build for userspace**
   - Build std with our astra_os backend
   - Link std with userspace code
   - Test std features (String, Vec, HashMap, etc.)

2. **Update userspace build**
   ```toml
   # userspace/browser/Cargo.toml
   [dependencies]
   # Enable std
   ```

3. **Test std in userspace**
   ```rust
   use std::println;
   use std::collections::HashMap;

   fn main() {
       println!("std works!");
       let mut map = HashMap::new();
       map.insert("key", "value");
   }
   ```

## Phase 3: Integrate Servo

### Prerequisites:
- ✅ Userspace with std
- ✅ System calls working
- ⏳ File system (optional - can use embedded HTML)
- ⏳ Network stack (optional initially)

### Steps:

1. **Add Servo dependencies**
   ```toml
   [dependencies]
   html5ever = { path = "../../html5ever-local" }
   markup5ever = { path = "../../markup5ever-local" }
   tendril = { path = "../../tendril-local" }
   web_atoms = { path = "../../web_atoms-local" }
   ```

2. **Replace simple_html with Servo**
   ```rust
   use html5ever::parse_document;
   use html5ever::tendril::TendrilSink;

   fn parse_html(html: &str) {
       let dom = parse_document(RcDom::default(), Default::default())
           .from_utf8()
           .read_from(&mut html.as_bytes())
           .unwrap();
   }
   ```

3. **Add rendering**
   - Use syscalls to draw to framebuffer
   - Implement basic layout engine
   - Render DOM to screen

## Phase 4: Add JavaScript Engine

### Options:
1. **SpiderMonkey** (Mozilla's JS engine - used by Servo)
2. **QuickJS** (Lightweight alternative)
3. **Boa** (Rust-based JS engine)

### Integration:
```rust
use servo_script::ScriptEngine;

fn run_javascript(code: &str) {
    let engine = ScriptEngine::new();
    engine.eval(code);
}
```

## Current Blockers & Solutions

### Blocker 1: std Dependencies
**Problem**: Servo needs std, userspace is no_std
**Solution**: ✅ Build std for userspace (we already have astra_os backend!)

### Blocker 2: File System
**Problem**: Servo may need file I/O
**Solution**:
- Option A: Embed HTML/CSS/JS in binary
- Option B: Implement simple VFS

### Blocker 3: Network Stack
**Problem**: Fetching web pages
**Solution**:
- Phase 1: Use embedded HTML
- Phase 2: Implement HTTP client via syscalls
- Phase 3: Full network stack in kernel

## Timeline Estimate

| Phase | Task | Time | Status |
|-------|------|------|--------|
| 1 | Userspace infrastructure | 1 day | ✅ DONE |
| 2 | std in userspace | 1-2 days | ⏳ NEXT |
| 3 | Servo integration | 2-3 days | ⏳ TODO |
| 4 | JS engine | 3-5 days | ⏳ TODO |

## Success Criteria

### Phase 1 (Current):
- [x] Ring 0 → Ring 3 transition works
- [x] Syscalls working
- [x] HTML parsing in userspace
- [x] Browser loop running

### Phase 2:
- [ ] std::println! works in userspace
- [ ] std::collections work
- [ ] std::string::String works

### Phase 3:
- [ ] html5ever parses HTML
- [ ] DOM tree constructed
- [ ] Basic rendering to framebuffer

### Phase 4:
- [ ] JavaScript executes
- [ ] DOM manipulation from JS
- [ ] Event handling

## Final Goal

A working browser that can:
1. Parse HTML with Servo's html5ever
2. Execute JavaScript
3. Render to screen
4. Handle user input

All running in **Ring 3 userspace** on a custom OS!
