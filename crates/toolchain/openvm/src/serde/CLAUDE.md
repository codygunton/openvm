# OpenVM Serde Component - Claude Instructions

## Component Overview
This is the OpenVM serde component, a custom serialization library optimized for word-aligned (32-bit) data streams in zkVM environments. It provides full compatibility with Rust's serde framework while optimizing for the specific needs of zero-knowledge virtual machines.

## Key Responsibilities
When working with this component:

1. **Maintain Word Alignment**: All data must be properly aligned to 32-bit boundaries. This is critical for zkVM performance.

2. **Preserve Serde Compatibility**: Changes must maintain full compatibility with standard serde traits and derives.

3. **Optimize for zkVM**: Consider cycle count and memory usage in zkVM contexts when making changes.

4. **Handle Errors Gracefully**: Proper error handling is essential for security and debugging.

## Code Standards

### Serialization Format
- Always use little-endian encoding
- Pad all sub-word data to 32-bit boundaries with zeros
- Include length prefixes for variable-length types
- Maintain consistent variant indices for enums

### Performance Guidelines
- Minimize allocations in hot paths
- Pre-allocate buffers when sizes are known
- Use zero-copy operations where possible
- Batch operations to reduce overhead

### Testing Requirements
- Every new feature needs round-trip tests
- Test edge cases (empty collections, max sizes)
- Verify compatibility with standard derives
- Include benchmarks for performance-critical changes

## Common Tasks

### Adding New Type Support
1. Implement serialization in `serialize_*` method
2. Implement deserialization in `deserialize_*` method
3. Add round-trip tests
4. Update documentation

### Optimizing Existing Types
1. Profile current implementation
2. Identify bottlenecks
3. Implement optimization
4. Verify with benchmarks
5. Ensure tests still pass

### Debugging Serialization Issues
1. Check word alignment
2. Verify length prefixes
3. Examine padding
4. Test round-trip behavior

## Architecture Decisions

### Why Word-Based?
- zkVM operates on 32-bit words natively
- Simplifies memory operations in circuits
- Reduces proof generation complexity
- Aligns with platform architecture

### Why Custom Implementation?
- Standard serde formats not optimized for zkVM
- Need precise control over alignment
- Performance critical for proof generation
- Specific security requirements

## Integration Points
- Used by guest/host communication layer
- Critical for proof serialization
- State management depends on this
- All OpenVM components use this for data exchange

## Security Considerations
- Never trust input lengths without validation
- Prevent allocation-based DoS attacks
- Validate UTF-8 strings properly
- Handle malformed data gracefully

## Future Considerations
When extending this component:
- Consider backward compatibility
- Think about proof size impact
- Evaluate cycle count in zkVM
- Consider memory usage patterns

Remember: This component is fundamental to OpenVM's operation. Changes here affect the entire system.