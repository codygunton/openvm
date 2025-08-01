# BLS12-381 Pairing Guest Library - Index

## Module Structure

### Root Files
- `mod.rs` - Main module definition, curve parameters, and type aliases
- `fp2.rs` - Quadratic extension field Fp2 implementation
- `fp12.rs` - 12-degree extension field Fp12 implementation
- `pairing.rs` - Miller loop and pairing check implementation
- `tests.rs` - Unit tests (conditional compilation)
- `utils.rs` - Conversion utilities for halo2curves (conditional)

## Key Types

### Field Types
- `Bls12_381Fp` - Base field modulo p (381-bit prime)
- `Bls12_381Scalar` - Scalar field modulo r (255-bit prime)
- `Bls12_381Fp2` - Quadratic extension Fp[i]/(i² + 1)
- `Fp` - Type alias for `Bls12_381Fp`
- `Scalar` - Type alias for `Bls12_381Scalar`
- `Fp2` - Type alias for `Bls12_381Fp2`
- `Fp12` - 12-degree extension field as `SexticExtField<Fp2>`

### Curve Types
- `Bls12_381G1Affine` - G1 affine points over Fp
- `G1Affine` - Type alias for `Bls12_381G1Affine`
- `G2Affine` - G2 affine points over Fp2 (defined in `g2` module)

### Pairing Type
- `Bls12_381` - Main pairing engine struct

## Constants

### Field Parameters
- `CURVE_B` - Curve parameter b = 4
- `XI` - Non-residue ξ = 1 + i in Fp2
- `FP2_TWO` - Constant 2 in Fp2
- `FP2_THREE` - Constant 3 in Fp2

### Curve Generators
- `G1Affine::GENERATOR` - Generator point for G1
- `G1Affine::NEG_GENERATOR` - Negated generator for G1

### Pairing Constants
- `PAIRING_IDX` - Curve index = 1
- `FINAL_EXPONENT` - 540-byte exponent for final exponentiation
- `FROBENIUS_COEFFS` - 12×5 array of Frobenius map coefficients

### Miller Loop Parameters  
- `BLS12_381_SEED_ABS` - Absolute value of curve seed
- `BLS12_381_PSEUDO_BINARY_ENCODING` - Optimized loop encoding

## Traits Implemented

### For `Bls12_381Fp2`
- `Field` - Field arithmetic operations
- `FieldExtension<Fp>` - Extension field interface
- Complex field operations via macros

### For `G1Affine`
- `CyclicGroup` - Cyclic group with generator
- `Group` - Group operations (via macro)
- `AffinePoint<Fp>` - Affine point interface
- `WeierstrassPoint<Fp>` - Weierstrass curve operations

### For `G2Affine`
- `Group` - Group operations
- `AffinePoint<Fp2>` - Affine point interface
- `WeierstrassPoint<Fp2>` - Weierstrass curve operations

### For `Bls12_381`
- `IntrinsicCurve` - MSM and curve operations
- `PairingIntrinsics` - Field tower configuration
- `PairingCheck` - Pairing verification
- `MultiMillerLoop` - Multi-pairing Miller loop
- `LineMulMType` - M-type line multiplication

### For `Fp12`
- `Field` - Field arithmetic
- `FieldExtension<Fp2>` - Extension over Fp2
- `ComplexConjugate` - Conjugation operation
- `FromLineMType<Fp2>` - Construction from line evaluation

## Functions

### Field Operations
- `Fp2::new(c0, c1)` - Create Fp2 element
- `Fp2::from_coeffs([c0, c1])` - Create from coefficient array
- `Fp2::from_bytes(bytes)` - Deserialize from bytes
- `Fp2::to_bytes()` - Serialize to bytes
- `Fp2::embed(base)` - Embed base field element
- `Fp2::frobenius_map(power)` - Apply Frobenius endomorphism
- `Fp12::new(coeffs)` - Create Fp12 element
- `Fp12::invert()` - Field inversion

### Curve Operations
- `G1Affine::from_xy(x, y)` - Create point from coordinates
- `G2Affine::from_xy(x, y)` - Create G2 point
- `Bls12_381::msm(coeffs, bases)` - Multi-scalar multiplication

### Pairing Operations
- `UnevaluatedLine::evaluate(xy_frac)` - Evaluate line at point
- `Fp12::from_evaluated_line_m_type(line)` - Create from M-type line
- `Bls12_381::mul_023_by_023(l0, l1)` - Multiply sparse lines
- `Bls12_381::mul_by_023(f, l)` - Multiply Fp12 by sparse line
- `Bls12_381::mul_by_02345(f, x)` - Multiply by 5-sparse element
- `Bls12_381::evaluate_lines_vec(f, lines)` - Evaluate multiple lines
- `Bls12_381::pre_loop(...)` - Miller loop preprocessing
- `Bls12_381::post_loop(...)` - Miller loop postprocessing
- `Bls12_381::pairing_check_hint(P, Q)` - Generate verification hint
- `Bls12_381::pairing_check(P, Q)` - Verify pairing equation
- `Bls12_381::try_honest_pairing_check(P, Q)` - Optimized verification
- `Bls12_381::multi_miller_loop(P, Q)` - Compute Miller loop
- `Bls12_381::multi_miller_loop_embedded_exp(P, Q, c)` - Miller loop with exponent

## Macros Used

### Field Generation
- `moduli_declare!` - Declare modular integer types
- `complex_declare!` - Declare complex field structure
- `complex_impl_field!` - Implement field operations

### Curve Generation
- `sw_declare!` - Declare short Weierstrass curve
- `impl_sw_affine!` - Implement affine point operations
- `impl_sw_group_ops!` - Implement group operations

## Module Organization

### Internal Modules
- `g2` - G2 curve implementation
  - Defines `G2Affine` type
  - Constants: `THREE`, `B`
  - Uses specialized macros for Fp2-based curves

### Conditional Modules
- `tests` - Unit tests (requires `test` and `halo2curves` features)
- `utils` - Conversion utilities (requires `halo2curves` feature)

## Type Relationships

```
Fp (base field)
├── Fp2 (quadratic extension)
│   ├── Fp12 (sextic tower over Fp2)
│   └── G2Affine (points over Fp2)
└── G1Affine (points over Fp)

Bls12_381 (pairing engine)
├── Uses G1Affine, G2Affine
├── Produces Fp12 results
└── Implements pairing protocols
```

## Feature Dependencies

- Core functionality: Always available
- `halo2curves`: Enables conversion utilities and cross-validation
- `test`: Enables unit tests
- `zkvm` target: Enables hardware acceleration