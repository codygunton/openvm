# OpenVM Platform Component Index

## Component Location
`crates/toolchain/platform/src/`

## Purpose
Foundational platform definitions and runtime support for OpenVM guest programs, providing memory layout, allocators, runtime functions, and cross-platform abstractions.

## Key Files

### Core Platform
- **lib.rs** - Main module exports, constants, alignment utilities
- **memory.rs** - Memory layout, bounds checking, system allocation
- **rust_rt.rs** - Runtime support, program termination
- **print.rs** - Cross-platform debug output

### Heap Management
- **heap/mod.rs** - Heap allocator module selection
- **heap/bump.rs** - Simple bump allocator (default)
- **heap/embedded.rs** - Linked-list allocator with free

### Math Support
- **libm_extern.rs** - C-compatible math function exports

## Public API

### Constants
```rust
// Memory layout
pub const MEM_BITS: usize = 29;
pub const MEM_SIZE: usize = 1 << MEM_BITS;
pub const GUEST_MIN_MEM: usize = 0x0000_0400;
pub const GUEST_MAX_MEM: usize = MEM_SIZE;
pub const STACK_TOP: u32 = 0x0020_0400;
pub const TEXT_START: u32 = 0x0020_0800;

// Platform
pub const WORD_SIZE: usize = 4;
pub const PAGE_SIZE: usize = 1024;

// File descriptors
pub mod fileno {
    pub const STDIN: u32 = 0;
    pub const STDOUT: u32 = 1;
    pub const STDERR: u32 = 2;
    pub const JOURNAL: u32 = 3;
}
```

### Functions
- `align_up(addr: usize, align: usize) -> usize` - Align address upward
- `memory::is_guest_memory(addr: u32) -> bool` - Check guest bounds
- `memory::sys_alloc_aligned(bytes: usize, align: usize) -> *mut u8` - Allocate aligned memory
- `print::print<S: AsRef<str>>(s: S)` - Print to stdout
- `print::println<S: AsRef<str>>(s: S)` - Print with newline
- `rust_rt::terminate<const EXIT_CODE: u8>()` - Exit program

### Re-exports (when target_os = "zkvm")
- `custom_insn_i!` - Type I custom instructions
- `custom_insn_r!` - Type R custom instructions

## Key Features
- Dual-target support (zkVM and host)
- No-std by default
- Configurable heap allocators
- Complete libm math library
- Memory safety bounds checking
- Cross-platform debug output

## Configuration

### Features
- `default` = []
- `entrypoint` - Entry point generation
- `export-libm` - Export math functions
- `heap-embedded-alloc` - Use embedded allocator
- `panic-handler` - Include panic handler
- `rust-runtime` - Full runtime (includes export-libm)
- `std` - Standard library support

### Target Behavior
- `target_os = "zkvm"` - Full zkVM functionality
- Other targets - Host-side mock implementations

## Dependencies
- `openvm-custom-insn` - Custom instruction macros
- `openvm-rv32im-guest` - RISC-V guest support
- `critical-section` (optional) - For embedded allocator
- `embedded-alloc` (optional) - Alternative allocator
- `libm` (optional) - Math library

## Memory Map
```
0x0000_0000 - 0x0000_0400: Reserved
0x0020_0400: Stack top (grows down)
0x0020_0800: Program start (text, data, bss)
           : Heap start (grows up)
0x2000_0000: Memory limit (512MB)
```

## Usage Context
Foundation for all OpenVM guest programs:
- Memory management primitives
- Runtime initialization/termination
- Debug output capabilities
- Math function support
- Platform abstraction layer

## Testing
Supports dual compilation:
- zkVM target for production
- Host target for testing
- Mock implementations
- Consistent API

## Implementation Notes

### Allocator Selection
- Default: Bump allocator (never frees)
- Optional: Embedded allocator (supports free)
- Selected via feature flags

### Math Library
- Re-exports all libm functions
- C-compatible symbols
- No-std floating point

### Safety
- Guest memory bounds checking
- Alignment enforcement
- Panic on allocation failure
- Safe termination

## Related Components
- `openvm` - Standard library using this platform
- `openvm-rv32im-guest` - RISC-V instructions
- `openvm-custom-insn` - Custom opcodes
- All guest programs depend on this crate