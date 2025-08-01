# OpenVM Platform Component Documentation

## Overview

The OpenVM platform component (`openvm-platform`) provides the foundational platform definitions and runtime support for OpenVM guest programs. It defines memory regions, low-level runtime functions, custom instruction constants, and essential platform abstractions that enable programs to run within the OpenVM zkVM environment.

This crate is designed to be lightweight and can be imported both when targeting OpenVM (zkVM) and when targeting normal host machines, making it a critical foundation for cross-platform development and testing.

## Architecture

### Core Components

1. **Memory Management** (`memory.rs`)
   - Defines zkVM memory layout and constants
   - Provides guest memory bounds checking
   - Implements system-level memory allocation (`sys_alloc_aligned`)
   - Manages heap position tracking for bump allocation
   - Stack and text segment addresses

2. **Heap Allocators** (`heap/`)
   - **Bump Allocator** (`bump.rs`) - Default simple allocator that never frees
   - **Embedded Allocator** (`embedded.rs`) - Optional linked-list allocator with deallocation support
   - Configurable via feature flags

3. **Runtime Support** (`rust_rt.rs`)
   - Program termination with exit codes
   - System opcode definitions
   - Core runtime functionality for Rust programs

4. **Printing Support** (`print.rs`)
   - Cross-platform print/println functions
   - Conditional compilation for zkVM vs host
   - Debug output capabilities

5. **Math Library Exports** (`libm_extern.rs`)
   - Exports all libm math functions to global namespace
   - Provides C-compatible math function symbols
   - Enables floating-point operations in no-std environment

### Memory Layout

```
0x0000_0000 - 0x0000_0400: Reserved (GUEST_MIN_MEM)
0x0020_0400: Stack top (stack grows down)
0x0020_0800: Text start (program code, data, bss)
            : Heap start (grows up after program sections)
0x2000_0000: Maximum memory (GUEST_MAX_MEM, 512MB)
```

### Key Design Principles

1. **Dual-Target Support**
   - Seamless compilation for both `target_os = "zkvm"` and host environments
   - Enables testing and development outside zkVM
   - Conditional features based on target

2. **Minimal Dependencies**
   - Designed to have as few dependencies as possible
   - Core functionality with optional features
   - No-std by default

3. **Platform Abstraction**
   - Provides consistent interface across environments
   - Hides zkVM-specific implementation details
   - Enables portable guest program development

## Features

### Core Features

- **Memory Management**: Guest memory bounds, allocation, alignment utilities
- **Runtime Support**: Program initialization and termination
- **Debug Output**: Cross-platform printing capabilities
- **Math Support**: Complete libm function exports

### Optional Features

- `entrypoint`: Enable entry point generation
- `export-libm`: Export libm math functions (included in rust-runtime)
- `heap-embedded-alloc`: Use embedded-alloc instead of bump allocator
- `panic-handler`: Include panic handler implementation
- `rust-runtime`: Full Rust runtime support (default includes export-libm)
- `std`: Standard library support for host-side usage

## API Reference

### Constants

```rust
// Memory layout
pub const MEM_BITS: usize = 29;              // 29-bit address space
pub const MEM_SIZE: usize = 1 << MEM_BITS;   // 512MB total memory
pub const GUEST_MIN_MEM: usize = 0x0000_0400; // Minimum guest address
pub const GUEST_MAX_MEM: usize = MEM_SIZE;    // Maximum guest address
pub const STACK_TOP: u32 = 0x0020_0400;       // Stack start address
pub const TEXT_START: u32 = 0x0020_0800;      // Program load address

// Platform constants  
pub const WORD_SIZE: usize = 4;    // 32-bit words
pub const PAGE_SIZE: usize = 1024;  // 1KB pages

// File descriptors
pub mod fileno {
    pub const STDIN: u32 = 0;
    pub const STDOUT: u32 = 1;
    pub const STDERR: u32 = 2;
    pub const JOURNAL: u32 = 3;
}
```

### Functions

```rust
// Memory utilities
pub fn is_guest_memory(addr: u32) -> bool;
pub unsafe fn sys_alloc_aligned(bytes: usize, align: usize) -> *mut u8;
pub const fn align_up(addr: usize, align: usize) -> usize;

// Runtime control
pub fn terminate<const EXIT_CODE: u8>();

// Debug output
pub fn print<S: AsRef<str>>(s: S);
pub fn println<S: AsRef<str>>(s: S);
```

### Custom Instructions

When targeting zkVM, provides access to custom instruction macros via `openvm_custom_insn`:
- `custom_insn_i!` - Type I custom instructions
- `custom_insn_r!` - Type R custom instructions

## Usage Patterns

### Basic Platform Usage

```rust
use openvm_platform::{print, println, WORD_SIZE, align_up};

// Debug output
println("Starting program...");

// Memory alignment
let aligned = align_up(ptr as usize, WORD_SIZE);

// Check guest memory bounds
if openvm_platform::memory::is_guest_memory(addr) {
    // Safe to access
}
```

### Memory Allocation

```rust
// Low-level allocation (usually not called directly)
let ptr = unsafe {
    openvm_platform::memory::sys_alloc_aligned(size, align)
};
```

### Program Termination

```rust
use openvm_platform::rust_rt::terminate;

// Exit with success
terminate::<0>();

// Exit with error
terminate::<1>();
```

## Implementation Details

### Heap Allocation Strategy

1. **Bump Allocator (Default)**
   - Simple pointer increment
   - Never deallocates memory
   - Zero-initialized (zkVM guarantees)
   - Most efficient for single-run programs

2. **Embedded Allocator (Optional)**
   - Linked-list based with free support
   - Higher overhead but supports deallocation
   - Useful for long-running programs

### Target-Specific Behavior

- **zkVM Target**:
  - Uses custom instructions for system calls
  - Accesses `_end` symbol for heap start
  - Prints via `openvm_rv32im_guest`
  
- **Host Target**:
  - Uses standard library when available
  - Provides mock implementations
  - Enables testing outside zkVM

### Math Library Integration

The platform re-exports all libm functions with C-compatible symbols, enabling:
- Floating-point operations in no-std
- C library compatibility
- Complete math function coverage

## Security Considerations

1. **Memory Bounds**: Always check addresses with `is_guest_memory`
2. **Alignment**: Use provided alignment utilities
3. **Allocation Limits**: Enforced maximum memory with panic on overflow
4. **Exit Codes**: Consistent error reporting via terminate

## Performance Notes

- Bump allocator has O(1) allocation
- No deallocation overhead by default  
- Word-aligned operations for efficiency
- Minimal runtime overhead

## Testing

Platform functions can be tested on host:
- Conditional compilation handles platform differences
- Mock implementations for host testing
- Same API across environments

## Related Components

- `openvm-rv32im-guest`: RISC-V guest instructions
- `openvm-custom-insn`: Custom instruction macros
- `openvm`: Standard library built on this platform
- Guest programs: All use this as foundational dependency