# Servo Integration Analysis
**Date**: 2026-01-01
**Status**: Initial Analysis Complete

## Executive Summary

Servo has been cloned (131,313 files) and analyzed for minimal port to ASTRA.OS. The codebase is modular with clear component separation, making it feasible to extract core rendering functionality.

## Servo Component Structure

### Key Components (in /tmp/servo-analysis/components/)

#### Core Rendering Pipeline
1. **layout** (~38,500 lines)
   - Box model calculation
   - Flexbox/Grid layout
   - Positioning system
   - Dependencies: stylo, taffy, webrender_api, fonts

2. **compositing**
   - Layer management
   - Display list generation
   - Rendering coordination

3. **canvas**
   - 2D canvas API
   - Graphics primitives

4. **fonts**
   - Font loading/rendering
   - Text shaping (HarfBuzz)
   - Dependencies: freetype-sys, dwrote

#### DOM & Scripting (Defer for Phase 1)
5. **script** - JavaScript engine integration
6. **script_bindings** - WebIDL bindings
7. **dom_struct** - DOM infrastructure

#### Supporting Infrastructure
8. **geometry** - Geometric primitives (euclid-based)
9. **pixels** - Pixel format handling
10. **shared/** - Shared trait definitions

## External Dependencies (Key)

### Already Available as Crates
- `html5ever` - HTML5 parser
- `cssparser` - CSS parsing
- `stylo` - Style system (from Firefox Stylo)
- `webrender_api` - WebRender integration
- `euclid` - 2D/3D math
- `app_units` - Device-independent units
- `taffy` - Layout algorithm

### Heavy Dependencies (Need Porting/Stubbing)
- `tokio` - Async runtime → **Stub initially**
- `hyper` - HTTP client → **Stub initially**
- `gstreamer` - Media → **Remove for Phase 1**
- `webgpu/webgl` - 3D graphics → **Remove for Phase 1**
- `servo-tracing` - Telemetry → **Remove for Phase 1**

## Minimal Viable Browser (Phase 1) - Component Selection

### INCLUDE (Essential):
```
✅ html5ever        - HTML parsing
✅ cssparser        - CSS parsing
✅ stylo            - Style computation
✅ euclid           - Math primitives
✅ app_units        - Units
⚠️  layout          - Core layout engine (needs porting)
⚠️  fonts           - Font rendering (needs OS integration)
⚠️  geometry        - Geometric types
⚠️  pixels          - Pixel handling
```

### STUB (Replace with Minimal Implementation):
```
⚠️  compositing     - Simple single-layer compositing
⚠️  webrender_api  - Direct framebuffer rendering instead
⚠️  net             - Hardcoded HTML for demo
⚠️  tokio/async    - Synchronous execution only
```

### REMOVE (Phase 2+):
```
❌ script          - JavaScript (add later)
❌ canvas          - Canvas API (add later)
❌ media           - Audio/video
❌ webgpu/webgl    - 3D graphics
❌ bluetooth/webxr - Advanced APIs
❌ devtools        - Developer tools
```

## Estimated Code Sizes

| Component | Lines of Code | Complexity | Port Effort |
|-----------|--------------|------------|-------------|
| layout    | ~38,500     | High       | 2-3 weeks   |
| fonts     | ~15,000 est | Medium     | 1-2 weeks   |
| geometry  | ~5,000 est  | Low        | 3 days      |
| pixels    | ~2,000 est  | Low        | 2 days      |
| **Glue code** | N/A     | Medium     | 1 week      |
| **TOTAL** | ~60,000     | -          | **4-6 weeks** |

## Critical Path Dependencies

### For ASTRA.OS Integration:
1. ✅ **Syscall Interface** (Track B - DONE)
   - sys_write for debug output
   - sys_brk for heap allocation

2. ⏳ **Memory Allocator**
   - Global allocator working
   - Need to fix heap expansion issues

3. ⏳ **Graphics Output**
   - VGA Mode 13h (320x200) working
   - Need VESA for higher resolution (Track C)

4. ❌ **Font Loading** (CRITICAL BLOCKER)
   - Need filesystem or embedded fonts
   - Options:
     - Embed single bitmap font in kernel
     - Add simple FAT32 filesystem
     - Use initrd with font files

5. ❌ **Input Handling**
   - Keyboard driver exists but not integrated
   - Mouse driver exists but not integrated

## Proposed Implementation Strategy

### Milestone 1: "Hello HTML" (Week 1-2)
- Extract html5ever + cssparser
- Hardcode simple HTML: `<html><body><h1>Hello</h1></body></html>`
- Parse to DOM tree
- Print DOM structure to serial console
- **Deliverable**: Parse and display HTML structure

### Milestone 2: "Static Layout" (Week 3-4)
- Port layout engine core
- Implement basic box model
- Calculate positions for simple HTML
- Render text using embedded bitmap font
- **Deliverable**: Render "Hello World" heading visually

### Milestone 3: "Styled Content" (Week 5-6)
- Add CSS parsing
- Implement style computation
- Apply colors and sizing
- Render simple styled page
- **Deliverable**: Colorful formatted HTML page

### Milestone 4: "Real HTML" (Week 7-8)
- Add filesystem support (FAT32 or initrd)
- Load external HTML files
- Load external fonts
- Improve layout quality
- **Deliverable**: Render real HTML pages from disk

## Immediate Next Steps (Priority Order)

1. **Fix Allocator Issues** (Track B continuation)
   - Debug heap expansion triple fault
   - Essential for any Servo code

2. **Upgrade to VESA** (Track C)
   - Need 640x480 minimum for readable text
   - Mode 13h (320x200) too low-res for browser

3. **Embed Test Font** (New task)
   - Find/create minimal bitmap font
   - Embed as static array
   - Create font rendering primitives

4. **Extract Servo Core** (Track A next phase)
   - Create `/kernel/src/browser/` directory
   - Copy html5ever, cssparser as dependencies
   - Create minimal DOM representation

5. **Build Simple Renderer** (Integration)
   - Framebuffer abstraction
   - Text rendering with embedded font
   - Color/rectangle primitives

## Risk Assessment

### HIGH RISK:
- **Memory pressure**: Servo is memory-heavy, allocator must be solid
- **Font rendering**: Complex dependency chain (FreeType, HarfBuzz)
- **Style computation**: Stylo needs significant Gecko infrastructure

### MEDIUM RISK:
- **Layout complexity**: ~38K lines, many edge cases
- **Graphics performance**: Software rendering may be slow

### LOW RISK:
- **HTML parsing**: html5ever is standalone
- **CSS parsing**: cssparser is standalone
- **Basic rendering**: Can start with simple text

## Success Criteria (Phase 1)

- [ ] Parse simple HTML document
- [ ] Compute basic layout (box positions)
- [ ] Render text to framebuffer
- [ ] Display styled headings and paragraphs
- [ ] Handle basic CSS (colors, fonts, margins)

**Target**: Simple but visually correct HTML rendering by Week 6

## Conclusion

Servo integration is **feasible** with focused scope. By excluding JavaScript, media, and complex APIs, we can achieve a minimal but functional browser rendering engine in 4-6 weeks. The modular architecture makes this extraction practical.

**Key Enablers**:
1. Servo's component modularity
2. Availability of html5ever/cssparser as standalone crates
3. Clear separation between layout and scripting

**Key Blockers (Must Fix First)**:
1. Heap allocator stability
2. Higher resolution graphics mode
3. Font rendering infrastructure
