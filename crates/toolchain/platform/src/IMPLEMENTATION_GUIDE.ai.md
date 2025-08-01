# OpenVM Platform Implementation Guide

## Overview

This guide provides detailed implementation information for the OpenVM platform component, including memory management strategies, allocator implementations, and platform-specific considerations.

## Memory Management Implementation

### Memory Layout Design

The zkVM uses a 29-bit address space (512MB) with the following layout:

```rust
// Memory constants in memory.rs
pub const MEM_BITS: usize = 29;           // 2^29 = 512MB address space
pub const MEM_SIZE: usize = 1 << MEM_BITS;
pub const GUEST_MIN_MEM: usize = 0x0000_0400;  // First 1KB reserved
pub const GUEST_MAX_MEM: usize = MEM_SIZE;     // Full 512MB available
```

Memory regions:
1. **Reserved** (0x0-0x400): System use, not accessible to guest
2. **Stack** (0x20400 downward): Fixed starting position
3. **Program** (0x20800 upward): Text, data, BSS sections
4. **Heap** (after program): Dynamic allocation area

### System Allocator Implementation

The `sys_alloc_aligned` function provides low-level allocation:

```rust
pub unsafe extern "C" fn sys_alloc_aligned(bytes: usize, align: usize) -> *mut u8 {
    static mut HEAP_POS: usize = 0;
    
    // Initialize heap position from _end symbol on first use
    #[cfg(target_os = "zkvm")]
    if heap_pos == 0 {
        heap_pos = unsafe { (&_end) as *const u8 as usize };
    }
    
    // Align allocation
    let align = usize::max(align, WORD_SIZE);
    let offset = heap_pos & (align - 1);
    if offset != 0 {
        heap_pos += align - offset;
    }
    
    // Check bounds and update position
    match heap_pos.checked_add(bytes) {
        Some(new_heap_pos) if new_heap_pos <= GUEST_MAX_MEM => {
            unsafe { HEAP_POS = new_heap_pos };
        }
        _ => {
            println("ERROR: Maximum memory exceeded");
            terminate::<1>();
        }
    }
    heap_pos as *mut u8
}
```

Key aspects:
- Single static heap pointer (safe due to single-threaded execution)
- Automatic alignment to word boundaries minimum
- Overflow checking with panic on failure
- `_end` symbol provided by linker for heap start

## Allocator Implementations

### Bump Allocator (Default)

Located in `heap/bump.rs`, provides O(1) allocation:

```rust
pub struct BumpPointerAlloc;

unsafe impl GlobalAlloc for BumpPointerAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        sys_alloc_aligned(layout.size(), layout.align())
    }
    
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // Never deallocates - suitable for single-run programs
    }
    
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // zkVM memory is pre-zeroed, so no explicit zeroing needed
        self.alloc(layout)
    }
}
```

Advantages:
- Extremely fast allocation
- No fragmentation
- Minimal code size
- Perfect for proof generation

### Embedded Allocator (Optional)

When `heap-embedded-alloc` feature is enabled:

```rust
// In heap/embedded.rs
use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// Initialization with critical section support
fn init_heap() {
    critical_section::with(|cs| {
        HEAP.init(get_heap_start(), get_heap_size())
    });
}
```

Features:
- Linked-list based free list
- Supports deallocation and reuse
- Higher overhead but more flexible
- Good for interactive/long-running programs

## Runtime Implementation

### Program Termination

The `terminate` function uses custom RISC-V instructions:

```rust
pub fn terminate<const EXIT_CODE: u8>() {
    #[cfg(target_os = "zkvm")]
    crate::custom_insn_i!(
        opcode = SYSTEM_OPCODE,  // 0x0b (custom-0)
        funct3 = 0,              // terminate function
        rd = Const "x0",         // no destination
        rs1 = Const "x0",        // no source
        imm = Const EXIT_CODE    // exit code in immediate
    );
    
    #[cfg(not(target_os = "zkvm"))]
    unimplemented!()  // Host testing typically uses panic
}
```

### Cross-Platform Printing

Print functions adapt to the target:

```rust
pub fn print<S: AsRef<str>>(s: S) {
    #[cfg(all(not(target_os = "zkvm"), feature = "std"))]
    print!("{}", s.as_ref());
    
    #[cfg(target_os = "zkvm")]
    openvm_rv32im_guest::print_str_from_bytes(s.as_ref().as_bytes());
}
```

This enables:
- Standard output on host for testing
- Guest-host communication in zkVM
- Consistent API across platforms

## Math Library Integration

The `libm_extern.rs` file exports ~100 math functions:

```rust
#[no_mangle]
pub extern "C" fn sinf(x: f32) -> f32 {
    libm::sinf(x)
}
```

Implementation strategy:
1. Re-export all libm functions with C ABI
2. Use `#[no_mangle]` for symbol visibility
3. Maintain exact C library signatures
4. Enable floating-point in no-std environment

## Feature Flag Combinations

### Minimal Guest Program
```toml
[dependencies]
openvm-platform = { version = "*" }
```
- Bump allocator only
- No panic handler
- Basic functionality

### Full Runtime Support
```toml
[dependencies]
openvm-platform = { version = "*", features = ["rust-runtime"] }
```
- Includes libm exports
- Panic handling
- Complete runtime

### With Deallocation Support
```toml
[dependencies]
openvm-platform = { version = "*", features = ["heap-embedded-alloc"] }
```
- Embedded allocator
- Free/realloc support
- Critical section handling

## Platform-Specific Considerations

### zkVM Target

When compiling for `target_os = "zkvm"`:
- Custom instructions available
- `_end` symbol from linker
- Special print mechanism
- No system calls

### Host Target

For testing and development:
- Mock implementations
- Standard library when available
- Panic instead of terminate
- Direct printing

## Safety and Security

### Memory Safety
1. Bounds checking in `is_guest_memory`
2. Alignment enforcement
3. Overflow detection
4. No unsafe memory access

### Allocation Safety
```rust
// Always check allocation success
let ptr = sys_alloc_aligned(size, align);
if ptr.is_null() {
    // Program terminates automatically on failure
}
```

### Thread Safety
- Single-threaded execution model
- No synchronization needed
- Static mut variables are safe

## Performance Optimization

### Alignment Strategy
```rust
pub const fn align_up(addr: usize, align: usize) -> usize {
    let mask = align - 1;
    (addr + mask) & !mask  // Branchless alignment
}
```

### Allocation Performance
- Bump allocator: O(1) always
- No allocation metadata overhead
- Pre-zeroed memory (no memset needed)
- Word-aligned for RISC-V efficiency

## Common Patterns

### Custom Allocator Integration
```rust
#[cfg(feature = "custom-alloc")]
#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator::new();

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        openvm_platform::memory::sys_alloc_aligned(
            layout.size(),
            layout.align()
        )
    }
    // ...
}
```

### Platform Detection
```rust
#[cfg(target_os = "zkvm")]
fn platform_specific() {
    // zkVM-specific code
}

#[cfg(not(target_os = "zkvm"))]
fn platform_specific() {
    // Host fallback
}
```

### Safe Memory Access
```rust
fn read_memory(addr: u32) -> Option<u32> {
    if openvm_platform::memory::is_guest_memory(addr) {
        unsafe { Some(*(addr as *const u32)) }
    } else {
        None
    }
}
```

## Debugging Tips

1. **Memory Debugging**: Use `println!` to track heap position
2. **Allocation Tracking**: Log all allocations in debug builds
3. **Bounds Checking**: Always verify addresses before access
4. **Host Testing**: Test allocator behavior on host first

## Integration Guidelines

1. **As Dependency**: Most crates should depend on `openvm` instead
2. **Direct Usage**: Only for very low-level system programming
3. **Feature Selection**: Choose minimal features needed
4. **Platform Testing**: Always test on both zkVM and host