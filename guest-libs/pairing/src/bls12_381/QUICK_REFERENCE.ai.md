# BLS12-381 Pairing Guest Library - Quick Reference

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
openvm-pairing-guest-libs = { workspace = true, features = ["bls12_381"] }
```

## Basic Usage

### Import Types

```rust
use openvm_pairing_guest_libs::bls12_381::{
    Bls12_381, Fp, Fp2, Fp12, Scalar, G1Affine, G2Affine
};
use openvm_pairing_guest_libs::PairingCheck;
```

### Creating Field Elements

```rust
// Base field element
let a = Fp::from_u32(42);
let b = Fp::from_bytes(&[/* 48 bytes */]);

// Quadratic extension
let c = Fp2::new(a, b);
let d = Fp2::from_coeffs([a, b]);

// Full extension
let e = Fp12::from_coeffs([/* 6 Fp2 elements */]);
```

### Working with Curve Points

```rust
// G1 points (over base field)
let g1_gen = G1Affine::GENERATOR;
let p1 = G1Affine::from_xy(x, y)?; // Returns None if not on curve

// G2 points (over extension field)  
let p2 = G2Affine::from_xy(x2, y2)?; // x2, y2 are Fp2 elements

// Scalar multiplication
let result = Bls12_381::msm(&[scalar], &[g1_gen]);
```

### Pairing Operations

```rust
// Single pairing check: e(P, Q) = 1
let ok = Bls12_381::pairing_check(&[p1], &[p2])?;

// Multi-pairing check: ∏ e(Pi, Qi) = 1
let ok = Bls12_381::pairing_check(&[p1, p2], &[q1, q2])?;

// Get pairing result (expensive - includes final exponentiation)
let f = Bls12_381::multi_miller_loop(&[p1], &[q1]);
// Apply final exponentiation manually if needed
```

## Field Element Operations

### Arithmetic Operations

```rust
let a = Fp::from_u32(10);
let b = Fp::from_u32(20);

// Basic arithmetic
let sum = a + b;
let diff = a - b;
let prod = a * b;
let quot = a / b;  // Panics if b = 0
let inv = a.invert().unwrap();

// Field operations
let sq = a.square();
let dbl = a.double();
let neg = -a;
```

### Extension Field Operations

```rust
let a = Fp2::new(x0, x1);
let b = Fp2::new(y0, y1);

// Arithmetic works the same
let c = a * b;

// Extension-specific operations
let conj = a.conjugate();  // (x0, x1) -> (x0, -x1)
let frob = a.frobenius_map(1);  // Frobenius endomorphism
```

## Curve Point Operations

### Point Creation and Validation

```rust
// Safe construction (validates point is on curve)
let p = G1Affine::from_xy(x, y)?;

// Identity/infinity point
let identity = G1Affine::identity();

// Check if point is identity
if p.is_identity() {
    // Handle infinity
}
```

### Group Operations

```rust
let p = G1Affine::GENERATOR;
let q = /* another point */;

// Point addition
let sum = p.add(&q);

// Point doubling  
let doubled = p.double();

// Scalar multiplication
let s = Scalar::from_u32(42);
let result = p.mul(&s);

// Multi-scalar multiplication (MSM)
let scalars = vec![s1, s2, s3];
let points = vec![p1, p2, p3];
let msm_result = Bls12_381::msm(&scalars, &points);
```

## Common Patterns

### BLS Signature Verification

```rust
fn verify_bls_signature(
    pubkey: &G1Affine,
    message_hash: &G2Affine,
    signature: &G2Affine,
) -> bool {
    // e(pubkey, message_hash) = e(G1, signature)
    Bls12_381::pairing_check(
        &[*pubkey, G1Affine::GENERATOR],
        &[*message_hash, signature.neg()]
    ).is_ok()
}
```

### Groth16 Verification

```rust
fn verify_groth16(
    vk_alpha_g1: &G1Affine,
    vk_beta_g2: &G2Affine,
    proof_a: &G1Affine,
    proof_b: &G2Affine,
    proof_c: &G1Affine,
    public_inputs_g1: &G1Affine,
) -> bool {
    // e(A, B) = e(alpha, beta) * e(public_inputs, gamma) * e(C, delta)
    Bls12_381::pairing_check(
        &[*proof_a, vk_alpha_g1.neg(), public_inputs_g1.neg(), proof_c.neg()],
        &[*proof_b, *vk_beta_g2, /* gamma_g2 */, /* delta_g2 */]
    ).is_ok()
}
```

## Memory Layout

### Field Elements
```
Fp: 48 bytes (384 bits, little-endian)
Fp2: 96 bytes (2 × 48)
Fp12: 576 bytes (12 × 48)
Scalar: 32 bytes (255 bits)
```

### Curve Points
```
G1Affine: 96 bytes (x: 48, y: 48)
G2Affine: 192 bytes (x: 96, y: 96)
```

## Constants

### Field Moduli
```rust
// Base field modulus p
Fp::MODULUS = 0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab

// Scalar field modulus r  
Scalar::MODULUS = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001
```

### Curve Parameters
```rust
// Curve equation: y² = x³ + b
const B: Fp = Fp::from_u32(4);

// Non-residue for Fp2
const XI: Fp2 = Fp2::new(Fp::ONE, Fp::ONE);  // 1 + i
```

## Performance Tips

1. **Batch Operations**
   ```rust
   // Good: Single MSM call
   let result = Bls12_381::msm(&all_scalars, &all_points);
   
   // Bad: Multiple individual multiplications
   for (s, p) in scalars.iter().zip(points) {
       results.push(p.mul(s));
   }
   ```

2. **Reuse Pairing Results**
   ```rust
   // If checking multiple equations with same pairing
   let miller_result = Bls12_381::multi_miller_loop(&[p], &[q]);
   // Reuse miller_result instead of recomputing
   ```

3. **Avoid Unnecessary Conversions**
   ```rust
   // Work in affine coordinates when possible
   // Projective coordinates not exposed in this API
   ```

## Error Handling

### Common Errors

```rust
use openvm_pairing_guest::pairing::PairingCheckError;

match Bls12_381::pairing_check(&points_g1, &points_g2) {
    Ok(()) => println!("Pairing verified"),
    Err(PairingCheckError) => println!("Pairing check failed"),
}
```

### Point Construction
```rust
// from_xy returns Option
let p = G1Affine::from_xy(x, y);
match p {
    Some(point) => { /* use point */ },
    None => { /* x, y don't satisfy curve equation */ },
}
```

## Testing Utilities

### Generate Test Points
```rust
#[cfg(test)]
fn test_point() -> G1Affine {
    let x = Fp::from_bytes(&hex!("..."));
    let y = Fp::from_bytes(&hex!("..."));
    G1Affine::from_xy(x, y).unwrap()
}
```

### Known Test Vectors
```rust
// Generator points
let g1 = G1Affine::GENERATOR;
let g2 = G2Affine::GENERATOR;  // Implement via trait

// Identity points
let id1 = G1Affine::identity();
let id2 = G2Affine::identity();
```

## Feature Flags

```toml
[features]
# Enable BLS12-381 support (included by default)
bls12_381 = []

# Enable cross-validation with halo2curves
halo2curves = ["openvm-pairing-guest/halo2curves"]

# Required for tests
test = []
```

## Debug Helpers

```rust
// Field element debugging
println!("Fp: {:?}", fp_element);
println!("Fp bytes: {:?}", fp_element.to_bytes());

// Point debugging  
if !point.is_identity() {
    println!("Point: ({:?}, {:?})", point.x, point.y);
}
```