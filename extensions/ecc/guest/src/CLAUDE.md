# CLAUDE Instructions: OpenVM ECC Guest Library

## Component Overview

You are working with the OpenVM ECC guest library, which provides elliptic curve cryptography primitives for zero-knowledge virtual machine execution. This library focuses on Weierstrass curves and ECDSA signature verification.

## Key Implementation Details

### Architecture
- **No-std library**: Designed for embedded zkVM environments without standard library
- **Trait-based design**: Core functionality defined through `Group`, `WeierstrassPoint`, and `IntrinsicCurve` traits
- **Macro-driven**: Uses `impl_sw_affine!` and `impl_sw_group_ops!` macros to generate curve implementations
- **ECDSA-focused**: Verification only, no signing support

### Critical Files
1. **weierstrass.rs** (615 lines): Core trait definitions and macro implementations
2. **ecdsa.rs** (563 lines): ECDSA verification logic and key recovery
3. **msm.rs** (162 lines): Multi-scalar multiplication using Pippenger's algorithm

### Important Patterns
- **Unsafe operations**: Functions like `add_ne_nonidentity` skip safety checks for performance
- **Generic over fields**: All operations are generic over field types from `openvm-algebra-guest`
- **Big-endian encoding**: Signatures and keys use big-endian byte encoding
- **Affine coordinates**: Points stored as (x,y) pairs, not projective

## Common Tasks

### Adding a New Curve
1. Define field type and constants (THREE, CURVE_B)
2. Use `impl_sw_affine!` macro to generate point type
3. Use `impl_sw_group_ops!` macro to generate operations
4. Implement `CyclicGroup` with generator constants
5. Implement `IntrinsicCurve` for MSM support

### Modifying ECDSA
- Core verification in `verify_prehashed()` function
- Key recovery in `recover_from_prehash_noverify()`
- Custom hooks via `VerifyCustomHook` trait

### Optimizing Performance
- Use `CachedMulTable` for fixed-base scalar multiplication
- Choose appropriate window sizes for MSM (see `msm.rs` lines 18-25)
- Use unsafe variants when preconditions are guaranteed

## Important Constraints

### Memory Layout
- `AffinePoint<F>` must have contiguous x,y coordinates
- Points serialized as little-endian internally
- SEC1 encoding uses big-endian for external compatibility

### Field Requirements
- Must implement `Field` trait from `openvm-algebra-guest`
- Must support `IntMod` for canonical representation
- Scalar field size ≤ Coordinate field size

### Safety Requirements
- Points must be validated before unsafe operations
- Setup must be called before hardware-accelerated operations
- Identity point has special handling in all operations

## Best Practices

### When Implementing
1. Always validate points with `from_xy()` not `from_xy_unchecked()`
2. Use `CHECK_SETUP` parameter in trait methods
3. Handle identity point explicitly in operations
4. Maintain big-endian external encoding

### When Modifying
1. Preserve trait boundaries - don't leak implementation details
2. Maintain compatibility with RustCrypto traits
3. Keep no-std compatibility
4. Document unsafe preconditions clearly

### When Debugging
1. Check field element canonicality first
2. Verify point is on curve
3. Ensure correct byte encoding (BE vs LE)
4. Test with known vectors from standards

## Common Pitfalls

1. **Coordinate system mismatch**: External libs may use different representations
2. **Field reduction**: Non-canonical elements break comparisons
3. **Scalar size mismatch**: Scalar and coordinate fields may differ
4. **Setup not called**: Hardware acceleration requires initialization

## Performance Considerations

- MSM dominates ECDSA verification cost
- Window size selection critical for MSM performance
- Precomputation amortizes over multiple operations
- Booth encoding reduces point additions by ~50%

## Testing Strategy

1. Test group laws (associativity, identity, inverse)
2. Use known ECDSA test vectors
3. Test edge cases (infinity, same x-coordinate)
4. Verify against reference implementations

Remember: This library prioritizes correctness and zkVM compatibility over raw performance. Always maintain proof-soundness when making changes.