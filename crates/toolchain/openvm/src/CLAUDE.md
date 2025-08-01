# Claude Instructions for OpenVM Toolchain Component

## Component Overview
This is the OpenVM standard library (`openvm` crate) that provides essential runtime functionality for guest programs running in the zkVM. It serves as the primary interface between guest code and the OpenVM runtime.

## Key Responsibilities
1. **Runtime Setup**: Initialize stack, global pointer, and entry points
2. **I/O Operations**: Handle input/output between guest and host
3. **Serialization**: Provide word-aligned serde implementation
4. **Platform Abstraction**: Abstract zkVM-specific functionality

## Important Implementation Notes

### Target-Specific Code
- Always check for `#[cfg(target_os = "zkvm")]` when modifying code
- Provide host-side implementations in `#[cfg(not(target_os = "zkvm"))]` blocks
- Use the `host` module for non-zkVM implementations

### Memory Operations
- All I/O is word-aligned (32-bit boundaries)
- Use hint instructions for reading external data
- Memory operations have assembly implementations (memcpy.s, memset.s)

### Entry Point
- The `entry!` macro is critical for no-std guest programs
- Entry point setup happens in `__start` and `_start` (assembly)
- Stack and global pointer must be initialized before main

### Serialization
- The serde implementation is customized for word streams
- Always maintain word alignment in serialization
- See `serde/CLAUDE.md` for detailed serde instructions

## Code Patterns to Follow

### Adding I/O Functions
```rust
#[cfg(target_os = "zkvm")]
pub fn my_io_function() {
    // zkVM-specific implementation using hints
}

#[cfg(not(target_os = "zkvm"))]
pub fn my_io_function() {
    // Host-side mock implementation
}
```

### Error Handling
- Prefer returning `Result` types where possible
- Use the panic handler for unrecoverable errors
- Provide meaningful error messages for debugging

### Feature Gates
- Use `#[cfg(feature = "std")]` for std-only functionality
- Keep no-std compatibility as the default
- Document feature requirements in function docs

## Testing Guidelines
1. Write tests that work both on host and in zkVM
2. Use conditional compilation for environment-specific tests
3. Test word alignment and padding edge cases
4. Verify assembly routines match Rust implementations

## Security Considerations
- Never expose raw memory access to guest programs
- Validate all input through proper deserialization
- Control output revelation through designated functions
- Maintain memory safety invariants

## Common Pitfalls to Avoid
1. Forgetting word alignment in I/O operations
2. Missing host-side implementations for testing
3. Breaking no-std compatibility
4. Incorrect cfg attribute combinations
5. Unsafe memory operations without proper validation

## Performance Priorities
1. Minimize allocations in hot paths
2. Use assembly for critical operations
3. Optimize for word-sized operations
4. Avoid unnecessary copying

## Documentation Requirements
- Document zkVM vs host behavior differences
- Provide examples for common use cases
- Explain word alignment requirements
- Note any safety invariants

## Maintenance Tasks
- Keep assembly implementations in sync with Rust
- Update host mocks when adding zkVM features
- Maintain compatibility with platform changes
- Review and update re-exports as needed