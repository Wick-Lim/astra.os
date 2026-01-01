# mozjs (SpiderMonkey JavaScript Engine) Requirements Analysis for ASTRA.OS

**Analysis Date:** 2026-01-01
**Target:** mozjs 0.14.4 (Rust bindings to SpiderMonkey ESR 140)
**Purpose:** Evaluate feasibility of integrating SpiderMonkey into ASTRA.OS browser kernel

---

## Executive Summary

mozjs 0.14.4 is a Rust wrapper around Mozilla's SpiderMonkey JavaScript engine (ESR 140.5). Integration into ASTRA.OS presents significant challenges due to extensive POSIX dependencies, C++ runtime requirements, and complex build toolchain needs. **JIT compilation can be disabled**, reducing binary size by ~9MB and simplifying requirements, but the engine still demands substantial OS infrastructure.

**Current Status in Servo:** Active dependency used for DOM/JavaScript execution
**Package Type:** Git-only (not published on crates.io)
**Binary Size:** 89 MB (with JIT), 80 MB (without JIT) for linux production-stripped builds

---

## 1. mozjs Package Information

### Version and Source
- **Version:** 0.14.4 (exact version specified in Servo's Cargo.toml)
- **SpiderMonkey Version:** ESR 140.5 (mozilla-esr140 branch)
- **Repository:** https://github.com/servo/mozjs
- **Availability:** Git-only, not published to crates.io
- **License:** MPL-2.0

### Crate Structure
mozjs consists of two primary crates:

1. **mozjs-sys** - Low-level C++ API bindings to SpiderMonkey
   - Direct FFI bindings to SpiderMonkey C++ API
   - Handles building the actual SpiderMonkey engine
   - Contains build.rs with complex build logic

2. **mozjs** - Higher-level Rust API wrapper
   - Safe Rust abstractions over mozjs-sys
   - Provides ergonomic JavaScript runtime interface
   - Used by Servo's script component

### Servo Integration
```toml
# From servo/Cargo.toml
js = { package = "mozjs", version = "=0.14.4", default-features = false,
       features = ["libz-sys", "intl"] }
```

Features used:
- `libz-sys`: Compression support
- `intl`: Internationalization (ICU integration)
- `js_jit`: JIT compilation (optional, enabled by default)
- `debugmozjs`: Debug symbols
- `profilemozjs`: Profiling support

---

## 2. System Library Dependencies

### Core C/C++ Dependencies

#### 2.1 C++ Standard Library
- **Requirement:** Full C++ standard library (libstdc++ or libc++)
- **Version:** Modern C++17/C++20 features used
- **Critical:** SpiderMonkey is written in C++, Rust, and JavaScript
- **ABI Compatibility:** Must use same C++ stdlib SpiderMonkey was built with

#### 2.2 libc Functions (Partial List)

**Memory Management:**
```c
malloc, free, realloc, calloc
mmap, munmap, mprotect
posix_memalign
```

**Threading (pthread):**
```c
pthread_create, pthread_join, pthread_detach
pthread_mutex_init, pthread_mutex_lock, pthread_mutex_unlock
pthread_cond_init, pthread_cond_wait, pthread_cond_signal
pthread_key_create, pthread_setspecific, pthread_getspecific
pthread_once
pthread_rwlock_*
```

**Thread-Local Storage:**
- POSIX TLS support required
- Minimum _POSIX_THREAD_KEYS_MAX (128 keys)
- Many implementations use 1024+ keys

**File I/O:**
```c
open, close, read, write, lseek
fopen, fclose, fread, fwrite
stat, fstat, access
```

**String/Memory Operations:**
```c
memcpy, memset, memmove, memcmp
strlen, strcmp, strcpy, strncpy
snprintf, sprintf
```

**Math Functions:**
```c
sin, cos, tan, sqrt, pow, log, exp
floor, ceil, round, fabs
```

**Time Functions:**
```c
clock_gettime, gettimeofday
time, localtime, gmtime
```

**Dynamic Loading:**
```c
dlopen, dlsym, dlclose, dlerror
```

**Signal Handling:**
```c
signal, sigaction, sigprocmask
```

**Process/Environment:**
```c
getenv, setenv
getpid, fork (potentially)
```

### 2.3 System Libraries

**Required:**
- `libm` - Math library
- `libpthread` - POSIX threads
- `libdl` - Dynamic loading
- `libz` / `libz-sys` - Compression (via feature flag)
- `libstdc++` or `libc++` - C++ standard library

**Optional:**
- `libicu` - Internationalization (via `intl` feature)
- `libjemalloc` - Alternative allocator (SpiderMonkey can use jemalloc)

---

## 3. Required Syscalls

While SpiderMonkey doesn't publish an official syscall list, analysis indicates the following Linux syscalls are likely required:

### 3.1 Memory Management Syscalls
```
mmap, munmap, mprotect, madvise
brk, sbrk (for malloc implementations)
```

### 3.2 Threading Syscalls
```
clone (for pthread_create)
futex (for mutex/condvar implementations)
set_robust_list, get_robust_list
set_tid_address
```

### 3.3 File/IO Syscalls
```
open, close, read, write, lseek
openat, readv, writev
stat, fstat, lstat
ioctl, fcntl
```

### 3.4 Process/Signal Syscalls
```
getpid, gettid
rt_sigaction, rt_sigprocmask, sigaltstack
exit, exit_group
```

### 3.5 Time Syscalls
```
clock_gettime, gettimeofday
nanosleep
```

### 3.6 Synchronization
```
futex (heavily used for userspace synchronization)
```

### 3.7 Other
```
getcwd, uname
getrandom (for PRNG)
```

**Note:** This is an estimated list. A complete analysis would require running SpiderMonkey under `strace` to capture actual runtime syscall usage.

---

## 4. Build Requirements

### 4.1 Toolchain Dependencies

**C++ Compiler:**
- Clang 14+ (recommended) or GCC with modern C++ support
- C++17/C++20 standard support
- Must support same ABI as SpiderMonkey build

**Build Tools:**
- Autoconf 2.13 (SPECIFIC VERSION REQUIRED - newer versions won't work)
- Automake
- Make
- Python 3.11+
- Rust toolchain (for the Rust bindings)

**Rust Build Dependencies (mozjs-sys):**
```toml
[build-dependencies]
bindgen = "^0.51.1"  # FFI binding generation
cc = "^1.0"           # C/C++ compiler wrapper
walkdir = "^2"        # Filesystem traversal
```

### 4.2 Platform-Specific Build Requirements

**Linux:**
- Python 3.x
- Clang 3.9+ (with LIBCLANG_PATH if multiple versions)
- build-essential (gcc, make, etc.)
- Development headers for system libraries

**Windows:**
- MozTools 4.0
- Clang (LLVM 14+)
- Visual Studio
- Python 3.11
- MOZTOOLS_PATH environment variable

**macOS:**
- Xcode Command Line Tools
- Clang (from Xcode)
- Python 3.x

### 4.3 Build Process

SpiderMonkey uses a complex autoconf-based build system:

1. Run `autoconf-2.13` in `js/src` directory
2. Create separate build directory
3. Run `configure` with options (e.g., `--disable-jit`)
4. Build with `make`

The mozjs-sys crate handles this automatically via `build.rs`, but requires:
- All build tools available in PATH
- Proper environment variables set
- Can use pre-built archives via `MOZJS_ARCHIVE` env var
- Can force source build via `MOZJS_FROM_SOURCE=1`

### 4.4 Build Time and Resources

- **Compilation Time:** 10-30 minutes on modern hardware (full build)
- **Disk Space:** ~500MB+ for build artifacts
- **RAM:** Recommend 4GB+ for parallel builds

---

## 5. JIT vs Interpreter Options

### 5.1 JIT Compilation (Default)

**Tiers:**
1. **Baseline Interpreter** - Hybrid interpreter/JIT with Inline Caches
2. **Baseline Compiler** - Fast 1-to-1 bytecode-to-machine-code translation
3. **IonMonkey** - Optimizing JIT with aggressive optimizations

**Pros:**
- Significantly faster JavaScript execution
- Industry-standard performance

**Cons:**
- Larger binary size (+9MB)
- Requires runtime code generation capabilities
- More complex security implications
- May not work on all platforms (needs JIT codegen support)

**Architecture Support:**
- Full support: x86, x86_64, ARM
- Partial support: SPARC, MIPS (JIT provided but not fully supported)

### 5.2 Disabling JIT

**Build Option:** `--disable-jit` configure flag
**Rust Feature:** Disable `js_jit` default feature (added in PR #37972)

**Configuration:**
```toml
# Disable JIT in Cargo.toml
mozjs = { version = "=0.14.4", default-features = false,
          features = ["libz-sys", "intl"] }
```

**Effects:**
- Binary size reduction: 89 MB → 80 MB (9MB savings)
- No runtime code generation required
- More portable across platforms
- Significantly slower JavaScript execution
- Suitable for environments where JIT is prohibited

### 5.3 Portable Baseline Interpreter (PBL)

**New Feature (SpiderMonkey Bug 1855321):**
- Alternative execution tier that doesn't require runtime codegen
- Suitable for WebAssembly/WASI environments
- Faster than pure C++ interpreter
- No JIT compilation needed

**Status:** Available in recent SpiderMonkey versions (ESR 140 likely includes)

**Use Cases:**
- Security-sensitive environments
- Platforms without JIT support
- Custom OS environments (like ASTRA.OS)

### 5.4 Recommendation for ASTRA.OS

**Use Interpreter-Only Mode** with JIT disabled:
- Configure with `--disable-jit`
- Use PBL if available
- Accept performance tradeoff for reduced complexity
- Simpler security model (no runtime code generation)
- Easier to port to custom OS

---

## 6. Memory Requirements

### 6.1 Heap Structure

**Arena-Based Allocation:**
- Basic allocation unit: 4KB arenas
- Arenas managed by SpiderMonkey's GC
- No hard minimum heap size documented

**Garbage Collection:**
- Generational GC with "nursery" for new objects
- Nursery is small and collected frequently
- Can allocate tens of GB if system allows

### 6.2 Memory Footprint Estimates

**Minimum Runtime (Estimated):**
- SpiderMonkey core: ~20-40 MB
- Per-context overhead: ~1-5 MB
- Per-script overhead: Varies by code size
- GC heap: Dynamic, starts small (~1-2 MB)

**Typical Browser Usage:**
- Base runtime: 50-100 MB
- Per-tab JavaScript heap: 10-50 MB average
- Can grow to 500MB+ for complex web apps

**Memory Configuration:**
- No documented minimum heap requirement
- GC is dynamic and adjusts to available memory
- Can set limits via JSAPI if needed

### 6.3 Virtual Memory

**User Report:**
- One deployment reported +2GB virtual memory usage
- Likely due to memory mapping strategies
- Actual RSS (resident set) much lower

### 6.4 Stack Requirements

**Per-Thread Stack:**
- JavaScript execution stack size configurable
- Default varies by platform
- Can be tuned via JSContext configuration

### 6.5 Recommendations for ASTRA.OS

**Minimum System Requirements:**
- Physical RAM: 128 MB minimum, 256 MB+ recommended
- JavaScript heap: Start with 16-32 MB default limit
- Per-process VM space: 1-2 GB address space
- Stack: 1-2 MB per thread

---

## 7. ASTRA.OS Compatibility Assessment

### 7.1 Current ASTRA.OS Capabilities

**From libc-fork analysis:**
```rust
// /Users/wick/Documents/workspaces/astra.os/libc-fork/src/new/astra_os/mod.rs
pub mod unistd;  // Minimal libc implementation
```

**Current Implementation:**
- Minimal libc stub (very limited functionality)
- No pthread support evident
- No full C++ stdlib
- Basic syscall interface exists

### 7.2 Gap Analysis

#### Critical Missing Components

**1. Threading Infrastructure:**
- ❌ No pthread implementation
- ❌ No TLS (Thread-Local Storage)
- ❌ No futex syscall for synchronization
- ❌ No thread creation/management

**2. C++ Runtime:**
- ❌ No C++ standard library
- ❌ No C++ exception handling
- ❌ No C++ RTTI (Run-Time Type Information)
- ❌ No C++ global constructors/destructors

**3. Memory Management:**
- ⚠️ Basic memory allocation exists (kernel allocator)
- ❌ No mmap/munmap
- ❌ No mprotect for permission changes
- ❌ No memory mapping subsystem

**4. File System:**
- ✅ TAR filesystem implemented (Phase 2)
- ⚠️ VFS layer exists
- ❌ Full POSIX file operations incomplete

**5. Dynamic Linking:**
- ❌ No dlopen/dlsym
- ❌ No dynamic library loading
- ❌ Static linking only

**6. POSIX APIs:**
- ❌ Minimal signal handling
- ❌ No environment variables
- ❌ Limited process management
- ❌ No fork/exec

### 7.3 Integration Challenges

**High Complexity:**
1. Requires implementing substantial POSIX infrastructure
2. Need full threading subsystem (pthreads)
3. Must port or implement C++ standard library
4. Complex build toolchain requirements

**Medium Complexity:**
5. Memory mapping subsystem (mmap/munmap)
6. Signal handling infrastructure
7. Dynamic linker/loader (if not static linking)

**Lower Complexity:**
8. Additional syscalls (time, random, etc.)
9. Math library (could use libm port)
10. Compression library (libz)

### 7.4 Estimated Development Effort

**Full SpiderMonkey Integration:**
- **Estimated Time:** 6-12 months (for experienced team)
- **Components:**
  - pthread implementation: 2-3 months
  - C++ stdlib port: 2-3 months
  - Memory management enhancements: 1-2 months
  - Syscall implementation: 1-2 months
  - Integration & testing: 2-3 months

**Alternative: Minimal JS Engine**
- Consider lighter alternatives (QuickJS, Duktape)
- Significantly smaller footprint
- Fewer dependencies
- Trade-off: Less spec compliance

---

## 8. Alternative Approaches

### 8.1 Alternative JavaScript Engines

#### QuickJS
- **Size:** ~600 KB (vs 80-89 MB for SpiderMonkey)
- **Dependencies:** Minimal (basic libc)
- **Performance:** Slower, but adequate for many uses
- **Spec Compliance:** ES2020 with some ES2021+
- **Threading:** Limited/optional
- **Pros:** Much easier to port, tiny footprint
- **Cons:** Slower, less battle-tested at scale

#### Duktape
- **Size:** ~200 KB compiled
- **Dependencies:** Minimal (C89 compatible)
- **Performance:** Slower than modern engines
- **Spec Compliance:** ES5/ES5.1
- **Threading:** No multi-threading
- **Pros:** Very portable, minimal deps
- **Cons:** Older ES version, single-threaded

#### Boa (Rust-native)
- **Language:** Pure Rust
- **Dependencies:** Rust stdlib only
- **Performance:** Developing
- **Spec Compliance:** Improving ES6+ support
- **Pros:** Memory-safe, Rust-native
- **Cons:** Still experimental, incomplete spec

### 8.2 Servo Components Without Full JS

**Option:** Use Servo's HTML/CSS engines without SpiderMonkey
- Layout engine (Taffy)
- CSS parser (stylo)
- HTML parser (html5ever)
- Rendering (WebRender)

**Trade-offs:**
- No JavaScript support
- Static HTML/CSS only
- Much simpler integration
- Still useful for document rendering

### 8.3 Phased Approach

**Phase 1:** Static rendering (no JS)
- Integrate HTML/CSS parsers
- Layout engine
- Rendering pipeline

**Phase 2:** Minimal JS engine
- Port QuickJS or Duktape
- Basic DOM manipulation
- Limited API surface

**Phase 3:** Full SpiderMonkey (if needed)
- Complete POSIX infrastructure
- Full threading support
- Production-grade JS execution

---

## 9. Recommendations

### 9.1 Short-Term (Current Phase 4)

**Do NOT attempt full SpiderMonkey integration yet:**

1. **Continue with static HTML/CSS rendering**
   - Focus on completing layout engine
   - Integrate stylo CSS engine
   - Build robust rendering pipeline

2. **Evaluate minimal JS engine**
   - Prototype QuickJS integration
   - Assess effort vs. benefit
   - Consider if JS is critical for MVP

3. **Document JS requirements**
   - Identify which JS features are essential
   - Determine minimum spec compliance needed
   - Evaluate security implications

### 9.2 Medium-Term (Next 6-12 months)

**If JavaScript is required:**

1. **Implement basic POSIX infrastructure**
   - Start with pthread basics
   - Add TLS support
   - Implement critical syscalls

2. **Port lightweight JS engine (QuickJS)**
   - Much more achievable than SpiderMonkey
   - Validate infrastructure with simpler engine
   - Gain experience with JS engine integration

3. **Build up gradually**
   - Add threading incrementally
   - Implement syscalls as needed
   - Test thoroughly at each stage

### 9.3 Long-Term (12+ months)

**Only if full browser compatibility required:**

1. **Complete POSIX implementation**
   - Full pthread support
   - All required syscalls
   - Memory management subsystem

2. **Port/integrate C++ stdlib**
   - Consider lightweight alternatives (musl++)
   - Or static link against existing implementation
   - Ensure ABI compatibility

3. **Integrate SpiderMonkey**
   - Start with JIT disabled
   - Use static linking
   - Extensive testing required

4. **Performance optimization**
   - Consider enabling JIT later
   - Tune memory management
   - Optimize syscall paths

### 9.4 Decision Matrix

**Use SpiderMonkey if:**
- Full web compatibility required
- Large, complex web applications needed
- Have resources for 6-12 month integration effort
- Team has OS development expertise

**Use QuickJS/Duktape if:**
- Basic JavaScript sufficient
- Tight resource constraints
- Faster time-to-market
- Can accept ES5/ES2020 limitations

**Use no JavaScript if:**
- Static content only
- Simplicity prioritized
- Very limited resources
- Security-critical environment

---

## 10. Technical Risks

### 10.1 Integration Risks

**High Risk:**
- Threading bugs (race conditions, deadlocks)
- C++ ABI compatibility issues
- Memory corruption from incomplete syscall implementations
- Build system complexity

**Medium Risk:**
- Performance issues without JIT
- Memory leaks in GC integration
- Signal handling conflicts
- TLS implementation bugs

**Low Risk:**
- Binary size growth
- Build time increases
- Documentation gaps

### 10.2 Mitigation Strategies

1. **Extensive testing**
   - Use existing JS test suites (test262)
   - Stress testing with threading
   - Memory leak detection

2. **Incremental integration**
   - Don't attempt full integration at once
   - Validate each subsystem independently
   - Use feature flags extensively

3. **Static linking**
   - Avoid dynamic linking complications
   - Embed SpiderMonkey directly
   - Simplifies deployment

4. **Upstream collaboration**
   - Engage with Servo/SpiderMonkey communities
   - Report integration issues
   - Contribute fixes upstream

---

## 11. Conclusion

### Summary

mozjs 0.14.4 (SpiderMonkey ESR 140) is a powerful, production-grade JavaScript engine, but requires extensive POSIX infrastructure that ASTRA.OS currently lacks. The minimal libc implementation is insufficient for SpiderMonkey's needs.

### Key Findings

1. **Dependencies:** Extensive - requires full libc, pthread, C++ stdlib
2. **Syscalls:** 20+ syscalls needed, including threading primitives
3. **Build:** Complex toolchain (Autoconf 2.13, Clang, Python, Rust)
4. **JIT:** Optional - can disable for 9MB savings and simpler porting
5. **Memory:** ~80MB binary, 50-100MB runtime, dynamic heap
6. **Effort:** 6-12 months for full integration

### Final Recommendation

**For current ASTRA.OS state:**
- ✅ **Continue Phase 4** HTML/CSS rendering without JavaScript
- ✅ **Evaluate QuickJS** as lighter alternative if JS needed
- ❌ **Defer SpiderMonkey** until core OS infrastructure mature
- ✅ **Build incrementally** - POSIX infrastructure first, then JS

**JavaScript is not required for MVP browser OS.** Static HTML/CSS rendering provides significant value without the complexity overhead of a full JS engine.

### Next Steps

1. Complete Phase 4 layout engine
2. Assess whether JavaScript is truly needed
3. If yes, prototype QuickJS integration
4. If no, focus on rendering performance and features
5. Document long-term path to full web compatibility

---

## Appendix A: Reference Links

- **mozjs Repository:** https://github.com/servo/mozjs
- **SpiderMonkey Docs:** https://spidermonkey.dev/
- **Servo Repository:** https://github.com/servo/servo
- **Build Documentation:** https://firefox-source-docs.mozilla.org/js/build.html
- **JIT Disable PR:** https://github.com/servo/servo/pull/37972
- **QuickJS:** https://bellard.org/quickjs/
- **Duktape:** https://duktape.org/

## Appendix B: Feature Flags Reference

```toml
# mozjs default features
[features]
default = ["js_jit"]  # JIT enabled by default

js_jit = []           # Enable JIT compilation
debugmozjs = []       # Debug symbols
profilemozjs = []     # Profiling support
jitspew = []          # JIT debug output
crown = []            # Servo-specific features
intl = ["icu"]        # Internationalization

# To disable JIT:
# mozjs = { version = "=0.14.4", default-features = false }
```

## Appendix C: Build Environment Variables

```bash
# mozjs-sys build configuration
MOZJS_ARCHIVE=<path>        # Use pre-built archive
MOZJS_FROM_SOURCE=1         # Force source build
LIBCLANG_PATH=<path>        # Clang library path
MOZTOOLS_PATH=<path>        # Windows: Mozilla build tools
```

---

**Document Version:** 1.0
**Author:** Claude (Anthropic)
**For:** ASTRA.OS Browser Operating System Project
**Last Updated:** 2026-01-01
