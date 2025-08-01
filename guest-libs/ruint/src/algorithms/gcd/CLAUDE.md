# GCD Algorithms Component - AI Assistant Instructions

## Component Overview
You are working with the GCD (Greatest Common Divisor) algorithms component of the ruint library. This component implements high-performance Lehmer's GCD algorithm with matrix optimizations for arbitrary-precision unsigned integers.

## Key Implementation Details

### Architecture
- **Main Module**: `mod.rs` exports three public functions: `gcd`, `gcd_extended`, and `inv_mod`
- **Matrix Module**: `matrix.rs` contains the `LehmerMatrix` struct and its implementations
- **Alternative Implementation**: `gcd_old.rs` contains a U256-specific implementation (not part of public API)

### Core Algorithm
The implementation uses Lehmer's algorithm which:
1. Works with 64/128-bit prefixes of large numbers for efficiency
2. Computes 2x2 update matrices encoding multiple Euclidean steps
3. Falls back to single Euclidean steps when matrix computation fails
4. Uses SWAR (SIMD Within A Register) techniques for cofactor operations

### Critical Invariants
- In `gcd` and related functions, the first argument must be >= the second (swapped if needed)
- Matrix sign encoding: `.4 = true` means `[[+,-],[-,+]]`, `.4 = false` means `[[-,+],[+,-]]`
- Cofactors in prefix methods never exceed 32 bits (packed in u64)
- Extended GCD maintains exact Bezout coefficients throughout

## Common Tasks and Patterns

### When Adding New GCD Variants
1. Follow the pattern in existing functions (parameter order, swapping logic)
2. Use `LehmerMatrix::from()` for matrix computation
3. Check for `Matrix::IDENTITY` to detect when to fall back
4. Maintain invariant that first parameter >= second

### When Optimizing Performance
- Small values (<64 bits): Use `from_u64` directly
- Consider batching matrix operations when possible
- The identity matrix check is critical - don't skip it
- Prefix methods (64/128 bit) provide most of the speedup

### When Debugging Issues
- Check sign handling in extended GCD (the boolean flag is crucial)
- Verify input ordering (a >= b requirement)
- Matrix composition order matters: `m2.compose(m1)` means m2 * m1
- Test with known difficult cases (consecutive Fibonacci numbers)

## Code Style Guidelines
- Use `debug_assert!` for invariant checks (not regular `assert!`)
- Prefer `const fn` where possible for compile-time optimization
- Keep inline hints on performance-critical paths
- Document edge cases and preconditions clearly

## Safety and Correctness
- No unsafe code in this component
- All arithmetic operations are infallible (no overflow possible)
- Edge cases (zero inputs, equal inputs) are handled explicitly
- Sign tracking in extended GCD prevents precision loss

## Testing Approach
- Property-based testing with proptest is preferred
- Always verify mathematical properties (divisibility, Bezout identity)
- Include edge cases: zero, one, equal values, bit boundaries
- Compare against reference implementations when available

## Integration Notes
- This component is used by modular arithmetic operations
- Critical for cryptographic implementations (RSA, ECC)
- Performance-sensitive: changes can impact overall system performance
- Maintain backward compatibility - these are public APIs

## Common Pitfalls to Avoid
1. Don't forget to check modulus != 0 in `inv_mod`
2. Remember that extended GCD returns a sign flag - use it correctly
3. Matrix operations modify arguments in-place
4. The `gcd_old.rs` file is not part of the public API - don't use it in new code

## Performance Characteristics
- Lehmer's algorithm is O(n²) in bit length
- Matrix steps reduce constants significantly
- Best performance gain for numbers >128 bits
- Small number optimization paths are important

When working on this component, prioritize correctness over micro-optimizations. The mathematical properties must hold for all inputs.