# Ruint Division Algorithms - AI Assistant Guidelines

## Component Context
You are working with high-performance division algorithms for arbitrary-precision unsigned integers in the ruint library. These algorithms are critical for cryptographic operations and must be handled with extreme care.

## Key Safety Considerations

### Algorithm Invariants
- **Divisor Non-Zero**: Always check/assert divisor ≠ 0
- **Normalization**: Highest bit set requirements are strict
- **Bounds**: Many functions have specific input range requirements
- **In-Place**: Results overwrite inputs - track carefully

### Common Issues to Watch For
1. **Off-by-one errors** in quotient estimation
2. **Overflow** in intermediate calculations
3. **Incorrect normalization** shift amounts
4. **Array indexing** errors in loops
5. **Borrow/carry propagation** bugs

## Code Patterns to Recognize

### Reciprocal Usage
```rust
let v = reciprocal(d);      // Compute once
div_2x1(n, d, v)           // Use many times
```

### Normalization Pattern
```rust
let shift = d.leading_zeros();
let d_norm = d << shift;
// ... compute with d_norm ...
remainder >> shift          // Don't forget denormalization!
```

### Result Storage
```rust
// IMPORTANT: Results stored in-place
div(&mut numerator, &mut divisor);
// Now: numerator = quotient, divisor = remainder
```

## Implementation Guidelines

### When Modifying Algorithms
1. **Preserve normalization requirements** - they're essential for correctness
2. **Maintain debug assertions** - they catch critical errors
3. **Keep proptest coverage** - property tests find edge cases
4. **Document preconditions** clearly in function docs
5. **Benchmark changes** - performance is critical

### When Adding Features
1. Start with **small cases** (div_2x1, div_3x2)
2. Build up to **general case** (div_nxm)
3. Add **specialized paths** for common sizes
4. Include **comprehensive tests** with known vectors
5. Consider **reciprocal caching** opportunities

### Testing Requirements
- Fixed test vectors from reference implementations
- Property-based tests for all size combinations
- Edge cases: overflow, max values, normalization boundaries
- Performance benchmarks for regression detection

## Common Tasks

### Adding a New Division Variant
1. Study existing patterns in `small.rs`
2. Implement with clear preconditions
3. Add to dispatch logic in `mod.rs`
4. Include specialized tests
5. Benchmark against general case

### Optimizing Existing Code
1. Profile first - identify actual bottlenecks
2. Check for unnecessary allocations
3. Consider reciprocal reuse
4. Look for vectorization opportunities
5. Maintain correctness invariants

### Debugging Division Issues
1. Check normalization requirements
2. Verify reciprocal computation
3. Test correction branch coverage
4. Compare against reference implementation
5. Use property tests to find minimal case

## Algorithm References
- **MG10**: "Modern Computer Arithmetic" - Primary reference
- **K97**: Knuth's "Art of Computer Programming" Vol 2
- **intx**: C++ implementation for test vectors

## Performance Targets
- div_2x1: < 3ns
- div_3x2: < 10ns  
- div_nx1: Linear in numerator size
- div_nxm: Quadratic but optimized

## Security Considerations
- No timing variations based on input values
- No secret-dependent branches
- Careful with overflow conditions
- Consider side-channel resistance for crypto use

## Questions to Ask
1. Is the divisor normalized as required?
2. Are all preconditions documented?
3. Is the correction branch tested?
4. Are intermediate overflows handled?
5. Is the algorithm constant-time if needed?