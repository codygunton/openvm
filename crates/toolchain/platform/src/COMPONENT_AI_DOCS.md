# OpenVM Platform Component AI Documentation

## Overview

The OpenVM platform component (`openvm-platform`) provides foundational platform definitions and runtime support for the OpenVM zkVM framework. This is a core no-std crate that other components depend on for basic platform functionality.

## Key Architecture

### Memory Management
- **Memory Size**: 512MB (2^29 bytes) total addressable space
- **Guest Memory Range**: 0x0000_0400 to 0x20000000 (512MB)
- **Stack**: Grows down from 0x0020_0400
- **Text/Data**: Loaded starting at 0x0020_0800
- **Heap**: Bump allocator starting after program sections

### Core Constants
- `WORD_SIZE`: 4 bytes (32-bit architecture)
- `PAGE_SIZE`: 1024 bytes
- `MEM_BITS`: 29 (for 512MB memory space)

### File Descriptors
- `STDIN`: 0
- `STDOUT`: 1  
- `STDERR`: 2
- `JOURNAL`: 3

## Key Components

### 1. Memory Layout (`memory.rs`)
Defines the zkVM memory layout and bounds checking:
- Memory size constants and bounds
- Guest memory validation
- Aligned allocation function for heap management

### 2. Heap Management (`heap/`)
Two allocation strategies:
- **Bump Allocator** (default): Simple, never deallocates, O(1) allocation
- **Embedded Allocator**: Full allocator with deallocation support

### 3. Runtime Support (`rust_rt.rs`)
Provides Rust runtime necessities:
- Program entry points
- Panic handlers
- Process termination

### 4. Math Functions (`libm_extern.rs`)
Re-exports libm math functions for no-std environments:
- Trigonometric functions (sin, cos, tan, etc.)
- Exponential and logarithmic functions
- Floating-point utilities

### 5. Print Utilities (`print.rs`)
Basic output functionality for debugging and logging.

## Features

### Core Features
- `default`: Minimal platform definitions
- `std`: Standard library compatibility
- `entrypoint`: Program entry point support

### Runtime Features
- `rust-runtime`: Full Rust runtime with libm
- `panic-handler`: Custom panic handling
- `export-libm`: Export math functions
- `heap-embedded-alloc`: Use embedded-alloc instead of bump allocator

## Dual-Target Support

All code supports both:
- `target_os = "zkvm"`: Actual zkVM execution
- Host targets: For testing and development

Platform-specific code uses conditional compilation:
```rust
#[cfg(target_os = "zkvm")]
// zkVM-specific implementation

#[cfg(not(target_os = "zkvm"))]
// Host implementation for testing
```

## Memory Safety

### Allocation Safety
- Always panics on allocation failure (never returns NULL)
- Maintains alignment requirements automatically
- Bounds checking prevents memory corruption
- Single-threaded safety assumptions

### Alignment Utilities
- `align_up()`: Aligns addresses to power-of-2 boundaries
- Automatic alignment in allocation functions
- Word-aligned access optimization for RISC-V

## Integration Points

### Dependencies
Minimal dependencies by design:
- `openvm-custom-insn`: Custom instruction support
- `openvm-rv32im-guest`: RISC-V guest support
- Optional: `critical-section`, `embedded-alloc`, `libm`

### Usage Pattern
Most crates should use the higher-level `openvm` crate instead of depending on `openvm-platform` directly. This crate provides the foundational layer.

## Security Considerations

### Memory Security
- No exposure of internal heap pointers
- Automatic bounds checking on all allocations
- Panic-on-fail allocation model prevents undefined behavior
- Single-threaded execution model (no race conditions)

### Cryptographic Safety
- No cryptographic operations in this crate
- Provides secure foundation for higher-level crypto components
- Memory zeroing guarantees from zkVM environment

## Performance Characteristics

### Allocation Performance
- O(1) allocation time (bump allocator)
- No deallocation overhead in default configuration
- Minimal metadata overhead
- Word-aligned access patterns

### Memory Usage
- Linear memory growth (no fragmentation)
- Predictable memory consumption
- Zero-initialized memory from zkVM

## Common Use Cases

### Basic Platform Setup
```rust
use openvm_platform::{WORD_SIZE, PAGE_SIZE};
use openvm_platform::memory::{GUEST_MIN_MEM, GUEST_MAX_MEM};
```

### Memory Allocation
```rust
// Allocation happens automatically through global allocator
let data = vec![0u8; 1024]; // Uses bump allocator
```

### Math Operations
```rust
// With export-libm feature
use openvm_platform::libm_extern::*;
let result = sin(3.14159);
```

## Development Guidelines

### Adding New Platform Constants
1. Document memory layout implications
2. Ensure no conflicts with existing constants
3. Maintain alignment requirements
4. Test on both zkVM and host targets

### Modifying Memory Layout
1. Check all existing constants for conflicts
2. Update memory map documentation
3. Preserve alignment guarantees
4. Test allocation patterns thoroughly

### Feature Additions
- Keep features additive only
- Maintain minimal dependency policy
- Ensure no-std compatibility
- Test all feature combinations

## Testing Strategy

### Test Coverage
- Dual-target testing (zkVM + host)
- Memory bounds testing
- Alignment verification
- Feature combination testing

### Performance Testing
- Allocation benchmarks
- Memory usage profiling
- Alignment overhead measurement

## File Structure

```
src/
├── lib.rs              # Main module exports and constants
├── memory.rs           # Memory layout and allocation
├── heap/               # Heap allocator implementations
│   ├── mod.rs         # Allocator selection
│   ├── bump.rs        # Bump allocator (default)
│   └── embedded.rs    # Embedded allocator (optional)
├── libm_extern.rs     # Math function exports
├── print.rs           # Basic output utilities
└── rust_rt.rs         # Rust runtime support
```

## AI Assistant Guidelines

When working with this component:

1. **Maintain Minimal Dependencies**: This crate must remain lightweight
2. **Dual-Target Testing**: Always test both zkVM and host targets
3. **Memory Safety**: Verify all memory operations include bounds checking
4. **Alignment Requirements**: Ensure all operations maintain proper alignment
5. **Feature Compatibility**: Test with different feature combinations
6. **Documentation**: Update memory layout docs for any changes
7. **Performance**: Consider allocation patterns and memory usage
8. **Security**: Never expose internal pointers or break safety guarantees

This component is foundational to the entire OpenVM ecosystem, so changes must be carefully considered for their impact on dependent crates.