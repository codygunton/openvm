# OpenVM Platform Component Instructions

## Overview

The OpenVM platform component provides low-level platform definitions and runtime support. This is a foundational crate that other components depend on.

## Key Principles

1. **Maintain Minimal Dependencies**: This crate should have as few dependencies as possible to maximize its usability
2. **Dual-Target Support**: All code must work correctly for both `target_os = "zkvm"` and host targets
3. **No Allocation in Core**: Core platform definitions should not allocate memory
4. **Safety First**: Memory operations must include proper bounds checking

## Code Guidelines

### When Adding New Platform Constants

Always document memory layout implications:
```rust
/// New memory region for X functionality.
/// Located at Y to avoid conflicts with Z.
pub const NEW_REGION: u32 = 0x30000;
```

### When Modifying Memory Layout

1. Check all existing constants for conflicts
2. Update the memory map documentation
3. Ensure alignment requirements are met
4. Test on both zkVM and host targets

### Platform-Specific Code

Always provide both zkVM and host implementations:
```rust
#[cfg(target_os = "zkvm")]
pub fn platform_function() {
    // zkVM implementation
}

#[cfg(not(target_os = "zkvm"))]
pub fn platform_function() {
    // Host implementation for testing
}
```

### Allocator Modifications

The allocator is critical infrastructure:
1. Never break existing allocation patterns
2. Maintain alignment guarantees
3. Preserve the "never fails" property (panic on OOM)
4. Test extensively with different allocation patterns

## Common Tasks

### Adding a New Platform Constant

1. Add to appropriate module (lib.rs or memory.rs)
2. Document the purpose and any constraints
3. Update AI_DOCS.md and AI_INDEX.md
4. Add to QUICK_REFERENCE.ai.md

### Adding Math Function Exports

1. Add to libm_extern.rs following existing pattern
2. Use exact C library signatures
3. Include both f32 and f64 variants
4. Test compilation with export-libm feature

### Modifying the Allocator

1. Consider impact on existing programs
2. Maintain bump allocator simplicity
3. Update documentation for any behavior changes
4. Benchmark allocation performance

## Architecture Decisions

### Why Bump Allocator by Default?

- Proof generation is single-run
- Deallocation adds unnecessary overhead
- Simplicity improves security
- Zero-initialized memory is guaranteed

### Why Re-export libm?

- No-std environment needs math functions
- C compatibility required for some tools
- Single source of math implementations
- Consistent behavior across platforms

### Why 512MB Memory Limit?

- Sufficient for most programs
- Fits within zkVM constraints
- Allows efficient memory proofs
- Simple bit manipulation (2^29)

## Testing Requirements

1. **Dual-Target Testing**: Test on both zkVM and host
2. **Memory Bounds**: Test allocation near limits
3. **Alignment**: Verify all alignment requirements
4. **Feature Combinations**: Test with different features

## Performance Considerations

- Alignment operations use bit manipulation
- Allocation is O(1) always
- No memory zeroing needed (zkVM guarantee)
- Word-aligned access for RISC-V efficiency

## Security Notes

- Never expose internal heap pointer
- Always check memory bounds
- Panic on allocation failure (no NULL returns)
- Maintain single-threaded safety assumptions

## Integration Notes

- Most crates should use `openvm` instead of this directly
- This is a `no_std` crate by design
- Features should be additive only
- Breaking changes require major version bump