# GCD Algorithms - Detailed Documentation

## Module Structure

### Core Module (`mod.rs`)
The main entry point providing three primary functions:

```rust
pub fn gcd<const BITS: usize, const LIMBS: usize>(a: Uint<BITS, LIMBS>, b: Uint<BITS, LIMBS>) -> Uint<BITS, LIMBS>
pub fn gcd_extended<const BITS: usize, const LIMBS: usize>(a: Uint<BITS, LIMBS>, b: Uint<BITS, LIMBS>) -> (Uint<BITS, LIMBS>, Uint<BITS, LIMBS>, Uint<BITS, LIMBS>, bool)
pub fn inv_mod<const BITS: usize, const LIMBS: usize>(num: Uint<BITS, LIMBS>, modulus: Uint<BITS, LIMBS>) -> Option<Uint<BITS, LIMBS>>
```

### Lehmer Matrix (`matrix.rs`)

#### Matrix Structure
```rust
pub struct Matrix(pub u64, pub u64, pub u64, pub u64, pub bool);
```

The matrix represents transformations with implicit signs:
- When `.4` is true: `[[.0, -.1], [-.2, .3]]`
- When `.4` is false: `[[-.0, .1], [.2, -.3]]`

#### Key Methods

**Matrix Construction:**
- `from<const BITS, const LIMBS>(a: Uint, b: Uint)` - Main entry point
- `from_u64(r0: u64, r1: u64)` - Small value optimization
- `from_u64_prefix(a0: u64, a1: u64)` - Prefix-based computation
- `from_u128_prefix(r0: u128, r1: u128)` - Double precision variant

**Matrix Operations:**
- `compose(self, other: Self)` - Matrix multiplication
- `apply(a: &mut Uint, b: &mut Uint)` - Apply to arbitrary precision integers
- `apply_u128(a: u128, b: u128)` - Apply to 128-bit values

## Algorithm Details

### Lehmer's Algorithm Overview
Lehmer's algorithm accelerates the Euclidean GCD algorithm by:

1. **Prefix Extraction**: Extract the most significant 64 or 128 bits
2. **Matrix Computation**: Compute update matrix using only the prefix
3. **Bulk Application**: Apply matrix to update full-precision values
4. **Fallback**: When precision insufficient, perform single Euclidean step

### Matrix Computation Process

#### Small Values (`from_u64`)
For values fitting in 64 bits, performs extended Euclidean algorithm:
```rust
loop {
    q = r0 / r1;
    r0 -= q * r1;
    // Update cofactors
    q00 += q * q10;
    q01 += q * q11;
    if r0 == 0 { return Matrix(...); }
    // Symmetric step for r1/r0
}
```

#### Prefix-Based (`from_u64_prefix`)
For larger values, works with 64-bit prefixes:
1. Maintains quotient limit of 2^32 to prevent overflow
2. Tracks last 3-4 values and cofactors
3. Uses Jebelean's exact conditions to determine correct stopping point
4. SWAR optimization packs two 32-bit cofactors in single u64

### Extended GCD Implementation

Maintains state variables throughout:
- `s0, s1`: First Bezout coefficient sequence
- `t0, t1`: Second Bezout coefficient sequence  
- `even`: Parity tracker for sign determination

Sign correction at end:
```rust
if even {
    t0 = Uint::ZERO - t0;  // t negative
} else {
    s0 = Uint::ZERO - s0;  // s negative
}
```

### Modular Inverse Optimization

Specialized for computing only the required cofactor:
- Early termination when GCD ≠ 1
- Only tracks `t0, t1` coefficients
- Handles modular reduction of negative results

## Performance Optimizations

### SWAR Technique
Cofactors packed in single u64 for SIMD-like operations:
```rust
let mut k0 = 1_u64 << 32;  // u0 = 1, v0 = 0
let mut k1 = 1_u64;         // u1 = 0, v1 = 1
```

### Early Termination
Multiple termination conditions optimize for different cases:
- Identity matrix return when no progress possible
- Immediate return for small remainders
- Jebelean's exact conditions for optimal stopping

### Division Optimization
Special handling when Lehmer step fails:
```rust
if m == LehmerMatrix::IDENTITY {
    // Large quotient expected, do full precision step
    a %= b;
    swap(&mut a, &mut b);
}
```

## Implementation Notes

### Edge Cases
- Zero inputs handled explicitly
- Equal inputs supported
- Proper handling of bit size boundaries

### Numerical Stability
- All intermediate calculations stay within bounds
- Overflow impossible in matrix operations due to quotient limits
- Sign tracking prevents precision loss

### Testing Strategy
- Property-based testing with proptest
- Comparison against reference implementations
- Known difficult cases (e.g., Fibonacci numbers)
- Edge cases at bit boundaries