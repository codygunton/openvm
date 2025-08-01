# BLS12-381 Pairing Guest Library

## Overview

The BLS12-381 pairing guest library provides pairing-based cryptographic operations for the BLS12-381 curve within the OpenVM zkVM framework. This library enables efficient pairing computations for zero-knowledge proof systems and other pairing-based protocols like BLS signatures and identity-based encryption.

## Architecture

### Core Components

1. **Field Extensions** (`fp2.rs`, `fp12.rs`)
   - `Fp2`: Quadratic extension field over base field Fp
   - `Fp12`: 12-degree extension field implemented as tower extension
   - Optimized field arithmetic using complex number representation

2. **Curve Definition** (`mod.rs`)
   - Base field `Fp`: 381-bit prime field
   - Scalar field: 255-bit prime field
   - G1: Points over base field with curve equation y² = x³ + 4
   - G2: Points over quadratic extension field Fp2

3. **Pairing Operations** (`pairing.rs`)
   - Miller loop implementation with pseudo-binary encoding
   - Line function evaluation for M-type pairings
   - Final exponentiation with hint-based optimization
   - Multi-pairing support for batch verification

### Key Features

1. **Optimized Field Tower**
   - Fp12 = Fp6[w]/(w² - v) where Fp6 = Fp2[v]/(v³ - ξ)
   - ξ = 1 + i in Fp2 (non-residue)
   - Efficient sparse multiplication algorithms

2. **Miller Loop Optimization**
   - Pseudo-binary encoding of loop parameter
   - Combined double-and-add steps
   - Specialized line multiplication for M-type structure

3. **Hardware Acceleration**
   - Intrinsic operations for field arithmetic
   - Custom RISC-V instructions for pairing operations
   - Phantom execution for final exponentiation

## Key Types

### Field Elements
- `Fp` (alias for `Bls12_381Fp`): Base field element
- `Fp2` (alias for `Bls12_381Fp2`): Quadratic extension element
- `Fp12`: Full extension field element for pairing results
- `Scalar` (alias for `Bls12_381Scalar`): Scalar field element

### Curve Points
- `G1Affine`: Affine points on G1 (base field)
- `G2Affine`: Affine points on G2 (extension field)

### Pairing Structure
- `Bls12_381`: Main pairing engine implementing `PairingIntrinsics` and `PairingCheck`

## Implementation Details

### Field Parameters
- **Base field modulus**: 0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab
- **Scalar field modulus**: 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001
- **Curve parameter b**: 4

### Frobenius Map
The implementation includes precomputed Frobenius coefficients for efficient field extension operations. These are stored in `FROBENIUS_COEFFS` as a 12×5 array for powers 0 through 11.

### Miller Loop Constants
- **Seed**: x = -0xd201000000010000 (negative)
- **Pseudo-binary encoding**: Optimized loop iteration pattern
- **Line evaluation**: M-type structure with coefficients at positions 0, 2, 3

## Security Considerations

1. **Subgroup Membership**
   - G1 and G2 have cofactors, requiring subgroup checks
   - Points constructed via `from_xy` may not be in prime-order subgroup

2. **Field Arithmetic**
   - All operations maintain canonical field representations
   - Overflow prevention through proper modular reduction

3. **Pairing Verification**
   - Honest verifier optimization using hints
   - Fallback to standard verification if hint fails

## Performance Characteristics

1. **Memory Layout**
   - Fp: 48 bytes (384 bits)
   - Fp2: 96 bytes (2 × 48)
   - Fp12: 576 bytes (12 × 48)
   - Optimized for zkVM memory model

2. **Computational Complexity**
   - Miller loop: O(log p) field operations
   - Final exponentiation: Dominated by Frobenius maps
   - Multi-pairing: Amortized line evaluations

## Integration with OpenVM

The library integrates with:
- `openvm-algebra-guest`: Field arithmetic primitives
- `openvm-ecc-guest`: Elliptic curve operations
- `openvm-pairing-guest`: Common pairing interfaces
- Hardware acceleration through custom instructions

## Usage Notes

1. **Feature Flags**
   - Enable with `features = ["bls12_381"]`
   - Optional `halo2curves` for cross-validation

2. **No-std Environment**
   - Designed for embedded zkVM execution
   - Uses `alloc` for dynamic allocations

3. **Testing**
   - Conditional compilation for zkVM vs host
   - Cross-validation with halo2curves implementation