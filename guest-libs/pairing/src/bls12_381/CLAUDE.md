# CLAUDE Instructions: BLS12-381 Pairing Guest Library

## Component Overview

You are working with the BLS12-381 pairing implementation for OpenVM's zero-knowledge virtual machine. This library provides pairing-based cryptography for BLS signatures, Groth16 verification, and other pairing-based protocols.

## Key Implementation Details

### Architecture
- **No-std library**: Designed for zkVM execution without standard library
- **Tower-based field extension**: Fp12 = Fp6[w]/(w² - v) where Fp6 = Fp2[v]/(v³ - ξ)
- **M-type pairing**: Line functions have sparse structure (0,2,3)
- **Hardware acceleration**: Custom RISC-V instructions for pairing operations

### Critical Files
1. **mod.rs** (623 lines): Core type definitions, constants, and trait implementations
2. **pairing.rs** (353 lines): Miller loop and pairing check implementation
3. **fp2.rs** (59 lines): Quadratic extension field implementation
4. **fp12.rs** (91 lines): 12-degree extension field implementation

### Important Patterns
- **Macro-driven types**: Uses `moduli_declare!`, `complex_declare!`, `sw_declare!`
- **Conditional compilation**: Different code paths for zkVM vs host
- **Sparse multiplication**: Optimized algorithms for M-type line structures
- **Hint-based verification**: Phantom execution provides final exponentiation hints

## Common Tasks

### Adding New Pairing Functions
1. Define trait in `openvm-pairing-guest`
2. Implement for `Bls12_381` struct
3. Add zkVM intrinsic path with `custom_insn_r!`
4. Add host fallback implementation
5. Test with halo2curves cross-validation

### Optimizing Field Operations
- Identify sparse patterns in Fp12 multiplication
- Implement specialized multiplication functions
- Use Frobenius map for efficient exponentiation
- Leverage tower structure for inversions

### Debugging Pairing Issues
- Check field element canonicity first
- Verify points are in correct subgroup (cofactor!)
- Use halo2curves for cross-validation
- Test with known pairing identities

## Important Constraints

### Field Parameters
- **Base field**: 381-bit prime (48 bytes)
- **Scalar field**: 255-bit prime (32 bytes)
- **Non-residue ξ**: (1, 1) in Fp2
- **Curve b**: 4

### Memory Layout
- Fp: 48 bytes (little-endian internally)
- Fp2: 96 bytes (c0, c1)
- Fp12: 576 bytes (12 × 48)
- Points stored as affine coordinates

### Security Requirements
- Points must be subgroup-checked
- Field elements must be canonical
- Pairing equations must be verified
- No timing side-channels in zkVM

## Best Practices

### When Implementing
1. Use existing macros for consistency
2. Follow tower field structure
3. Add both zkVM and host paths
4. Include cross-validation tests
5. Document sparse element positions

### When Modifying
1. Preserve M-type line structure
2. Maintain Frobenius coefficient accuracy
3. Keep hint generation deterministic
4. Update both guest and circuit sides
5. Test with edge cases (infinity, cofactor)

### When Debugging
1. Enable halo2curves feature for validation
2. Check intermediate Miller loop values
3. Verify Frobenius map coefficients
4. Test individual field operations
5. Use known test vectors

## Common Pitfalls

1. **Cofactor confusion**: G1/G2 have non-trivial cofactors
   - Always validate subgroup membership
   - Use cleared cofactor generators

2. **Field representation**: Internal little-endian vs external big-endian
   - Convert at API boundaries
   - Document byte order clearly

3. **Sparse multiplication**: Wrong coefficient positions break pairing
   - M-type: positions 0, 2, 3
   - D-type: positions 0, 1, 3 (not used here)

4. **Negative seed**: BLS12-381 has negative x
   - Conjugate after Miller loop
   - Handle in pre/post processing

## Performance Considerations

- Miller loop dominates computation (~70%)
- Final exponentiation is expensive (~25%)
- Multi-pairing amortizes line evaluations
- Batch verification saves ~40% vs individual
- Hint-based verification avoids final exp

## Testing Strategy

1. **Unit tests**: Field arithmetic properties
2. **Pairing tests**: Bilinearity, non-degeneracy
3. **Cross-validation**: Against halo2curves
4. **Edge cases**: Infinity, same x-coordinate
5. **Known vectors**: BLS test vectors, Groth16 proofs

## Integration Notes

### With Pairing Circuit Extension
- Guest library defines types and algorithms
- Circuit extension implements constraints
- Phantom execution bridges the gap
- Same constants must be used

### With Other Extensions
- Depends on algebra-guest for fields
- Depends on ecc-guest for curves
- Provides types for pairing-guest trait
- No circular dependencies

Remember: This is security-critical cryptographic code. Every change must maintain correctness, determinism, and proof soundness. When in doubt, add more tests and cross-validation.