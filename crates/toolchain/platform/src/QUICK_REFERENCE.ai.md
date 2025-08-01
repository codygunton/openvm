# OpenVM Platform Quick Reference

## Memory Constants

```rust
use openvm_platform::*;

// Address space
const MEM_SIZE: usize = 536_870_912;  // 512MB (2^29)
const GUEST_MIN_MEM: usize = 0x400;    // 1KB reserved
const GUEST_MAX_MEM: usize = MEM_SIZE;

// Memory layout  
const STACK_TOP: u32 = 0x20400;    // Stack grows down
const TEXT_START: u32 = 0x20800;   // Program loaded here

// Platform
const WORD_SIZE: usize = 4;         // 32-bit words
const PAGE_SIZE: usize = 1024;      // 1KB pages
```

## File Descriptors

```rust
use openvm_platform::fileno::*;

STDIN   // 0 - Standard input
STDOUT  // 1 - Standard output  
STDERR  // 2 - Standard error
JOURNAL // 3 - Journal output
```

## Core Functions

```rust
// Memory alignment
let aligned = align_up(addr, WORD_SIZE);
let aligned_page = align_up(addr, PAGE_SIZE);

// Memory bounds checking
if memory::is_guest_memory(addr) {
    // Safe to access
}

// Debug output
print("Hello");           // No newline
println("Hello, world!"); // With newline

// Program termination
rust_rt::terminate::<0>();  // Success
rust_rt::terminate::<1>();  // Error
```

## Memory Allocation

```rust
// Low-level allocation (rarely used directly)
unsafe {
    let ptr = memory::sys_alloc_aligned(
        size,    // bytes to allocate
        align    // alignment requirement
    );
}
```

## Feature Flags

```toml
# Minimal
openvm-platform = "0.1"

# With runtime
openvm-platform = { version = "0.1", features = ["rust-runtime"] }

# With embedded allocator
openvm-platform = { version = "0.1", features = ["heap-embedded-alloc"] }

# All features
openvm-platform = { version = "0.1", features = [
    "rust-runtime",
    "heap-embedded-alloc", 
    "panic-handler",
    "std"
] }
```

## Platform Detection

```rust
#[cfg(target_os = "zkvm")]
{
    // zkVM-specific code
}

#[cfg(not(target_os = "zkvm"))]
{
    // Host-specific code
}
```

## Custom Instructions (zkVM only)

```rust
#[cfg(target_os = "zkvm")]
use openvm_platform::{custom_insn_i, custom_insn_r};

// Type I instruction
custom_insn_i!(
    opcode = 0x0b,
    funct3 = 0,
    rd = Const "x0",
    rs1 = Const "x0", 
    imm = Const 0
);

// Type R instruction
custom_insn_r!(
    opcode = 0x0b,
    funct3 = 0,
    funct7 = 0,
    rd = Const "x0",
    rs1 = Const "x1",
    rs2 = Const "x2"
);
```

## Common Patterns

### Safe Memory Read
```rust
fn read_u32(addr: u32) -> Option<u32> {
    if memory::is_guest_memory(addr) {
        unsafe { Some(*(addr as *const u32)) }
    } else {
        None
    }
}
```

### Aligned Allocation
```rust
fn alloc_aligned_buffer(size: usize) -> *mut u8 {
    unsafe {
        memory::sys_alloc_aligned(size, WORD_SIZE)
    }
}
```

### Debug Logging
```rust
fn debug_log(msg: &str) {
    #[cfg(target_os = "zkvm")]
    println(msg);
    
    #[cfg(all(not(target_os = "zkvm"), feature = "std"))]
    eprintln!("DEBUG: {}", msg);
}
```

### Memory Layout Check
```rust
fn check_heap_space(needed: usize) -> bool {
    // Get current heap position (implementation-specific)
    let heap_start = TEXT_START as usize + program_size();
    let heap_end = heap_start + needed;
    heap_end <= GUEST_MAX_MEM
}
```

## Allocator Selection

### Default (Bump Allocator)
- Never frees memory
- O(1) allocation
- Zero overhead
- Best for proofs

### Embedded Allocator
- Supports free/realloc
- Linked-list based
- Higher overhead
- Good for interactive

## Math Functions

When `export-libm` or `rust-runtime` enabled:

```rust
// Trigonometric
sinf(x: f32) -> f32
cosf(x: f32) -> f32
tanf(x: f32) -> f32

// Exponential
expf(x: f32) -> f32
logf(x: f32) -> f32
powf(x: f32, y: f32) -> f32

// And ~90 more functions...
```

## Memory Map Summary

```
0x00000000 ┌─────────────┐
           │   Reserved  │ 1KB
0x00000400 ├─────────────┤ GUEST_MIN_MEM
           │             │
           │   Unused    │
           │             │
0x00020400 ├─────────────┤ STACK_TOP
           │    Stack    │ (grows down)
           │      ↓      │
0x00020800 ├─────────────┤ TEXT_START
           │    Text     │
           ├─────────────┤
           │    Data     │
           ├─────────────┤
           │    BSS      │
           ├─────────────┤ _end (heap start)
           │    Heap     │ (grows up)
           │      ↑      │
           │             │
0x20000000 └─────────────┘ GUEST_MAX_MEM (512MB)
```

## Quick Troubleshooting

| Issue | Solution |
|-------|----------|
| Out of memory | Check heap usage, increase memory limit |
| Alignment fault | Use `align_up()` before access |
| Invalid address | Verify with `is_guest_memory()` |
| No output | Ensure using correct print functions |
| Allocation fails | Check against GUEST_MAX_MEM |

## Minimal Example

```rust
#![no_std]
#![no_main]

extern crate openvm_platform;
use openvm_platform::{println, rust_rt::terminate};

#[no_mangle]
pub extern "C" fn _start() {
    println("Hello from OpenVM!");
    terminate::<0>();
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    terminate::<1>();
}
```