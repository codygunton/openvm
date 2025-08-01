# BN254 Pairing Implementation Guide

## Getting Started

### Basic Setup

Add the required dependencies to your `Cargo.toml`:

```toml
[dependencies]
openvm-pairing-guest = { version = "0.1", features = ["bn254"] }
openvm-algebra-guest = "0.1"
openvm-ecc-guest = "0.1"
```

### Import Essential Types

```rust
use openvm_pairing_guest::bn254::{Bn254, Fp, Fp2, Fp12, G1Affine, G2Affine};
use openvm_pairing_guest::PairingCheck;
use openvm_algebra_guest::Field;
use openvm_ecc_guest::{AffinePoint, Group};
```

## Common Implementation Patterns

### 1. Working with Field Elements

#### Creating Field Elements

```rust
// From integer
let a = Fp::from(42u64);

// From bytes (little-endian)
let b = Fp::from_le_bytes(&[1u8; 32]);

// From hex string (big-endian)
use hex_literal::hex;
let c = Fp::from_const_bytes(hex!(
    "0100000000000000000000000000000000000000000000000000000000000000"
));

// Field constants
let zero = Fp::ZERO;
let one = Fp::ONE;
```

#### Field Arithmetic

```rust
let x = Fp::from(10u64);
let y = Fp::from(20u64);

// Basic operations
let sum = &x + &y;
let product = &x * &y;
let square = x.square();
let inverse = x.invert().expect("x is non-zero");

// In-place operations
let mut z = x.clone();
z += &y;        // z = x + y
z.square_assign();  // z = (x + y)²
```

### 2. Working with Extension Fields

#### Fp2 Elements

```rust
// Create Fp2 element: a + b*u
let fp2_elem = Fp2::new(Fp::from(3u64), Fp::from(4u64));

// From coefficients
let coeffs = [Fp::from(1u64), Fp::from(2u64)];
let fp2_from_coeffs = Fp2::from_coeffs(coeffs);

// Complex conjugate
let conj = fp2_elem.conjugate(); // 3 - 4*u

// Frobenius map
let frob = fp2_elem.frobenius_map(1); // Same as conjugate for Fp2
```

#### Fp12 Elements

```rust
// Create from Fp2 coefficients
let fp12_one = Fp12::ONE;
let fp12_elem = Fp12::from_coeffs([
    Fp2::ONE,           // c0
    Fp2::ZERO,          // c1
    Fp2::ZERO,          // c2
    Fp2::ZERO,          // c3
    Fp2::ZERO,          // c4
    Fp2::ZERO,          // c5
]);

// Embedding Fp2 into Fp12
let fp2_val = Fp2::new(Fp::from(5u64), Fp::from(6u64));
let fp12_embedded = Fp12::embed(fp2_val);
```

### 3. Working with Elliptic Curve Points

#### G1 Points (over Fp)

```rust
// Generator point
let g1_gen = G1Affine::GENERATOR;

// Create point from coordinates
let x = Fp::from(1u64);
let y = Fp::from(2u64);
let point = G1Affine::new(x, y);

// Point operations
let double = g1_gen.double();
let sum = &g1_gen + &point;
let neg = -&g1_gen;

// Scalar multiplication
let scalar = Bn254Scalar::from(12345u64);
let result = g1_gen * &scalar;
```

#### G2 Points (over Fp2)

```rust
// G2 coordinates are Fp2 elements
let g2_x = Fp2::new(
    Fp::from_le_bytes(&[/* x0 bytes */]),
    Fp::from_le_bytes(&[/* x1 bytes */])
);
let g2_y = Fp2::new(
    Fp::from_le_bytes(&[/* y0 bytes */]),
    Fp::from_le_bytes(&[/* y1 bytes */])
);
let g2_point = G2Affine::new(g2_x, g2_y);
```

### 4. Pairing Operations

#### Single Pairing

```rust
// Compute e(P, Q)
let p = G1Affine::GENERATOR;
let q = /* some G2 point */;

// Using multi_miller_loop for single pairing
let miller_result = Bn254::multi_miller_loop(&[p], &[q]);
// Note: This gives f^{6x+2}, not the full pairing
```

#### Pairing Check

```rust
// Check if e(P1, Q1) * e(P2, Q2) * ... * e(Pn, Qn) = 1
let p_points = vec![p1, p2, p3];
let q_points = vec![q1, q2, q3];

match Bn254::pairing_check(&p_points, &q_points) {
    Ok(()) => println!("Pairing check passed"),
    Err(e) => println!("Pairing check failed: {:?}", e),
}
```

#### Multi-Pairing Product

```rust
// Compute product of pairings
fn compute_pairing_product(
    p_vec: &[G1Affine], 
    q_vec: &[G2Affine]
) -> Fp12 {
    // Get Miller loop result
    let f = Bn254::multi_miller_loop(p_vec, q_vec);
    
    // Apply final exponentiation manually if needed
    // (Usually not needed - use pairing_check instead)
    final_exponentiation(f)
}
```

### 5. Multi-Scalar Multiplication (MSM)

```rust
use openvm_ecc_guest::CyclicGroup;

// Prepare points and scalars
let points = vec![g1_1, g1_2, g1_3, g1_4];
let scalars = vec![s1, s2, s3, s4];

// Compute MSM: s1*g1_1 + s2*g1_2 + s3*g1_3 + s4*g1_4
let msm_result = Bn254::msm(&scalars, &points);
```

## Advanced Patterns

### 1. Optimized Pairing Checks with Hints

When running in ZKVM, the pairing check uses hints for optimization:

```rust
// This happens automatically in ZKVM
let (c, u) = Bn254::pairing_check_hint(&p_vec, &q_vec);

// The check verifies: f * u == c^λ
// where λ = 6x + 2 + q³ - q² + q
```

### 2. Custom Miller Loop Implementation

For special cases, you might need custom Miller loop logic:

```rust
use openvm_pairing_guest::pairing::{MillerStep, MultiMillerLoop};

// Example: Modified Miller loop with preprocessing
fn custom_miller_loop(
    p: &[G1Affine],
    q: &[G2Affine],
) -> Fp12 {
    // Precompute xy fractions for line evaluations
    let xy_fracs: Vec<(Fp, Fp)> = p.iter()
        .map(|point| {
            let y_inv = point.y.invert().unwrap();
            let x_over_y = &point.x * &y_inv;
            (x_over_y, y_inv)
        })
        .collect();
    
    // Run standard Miller loop
    Bn254::multi_miller_loop_embedded_exp(p, q, None)
}
```

### 3. Batch Operations

For efficiency, batch multiple operations:

```rust
// Batch pairing checks
fn batch_verify_signatures(
    messages: &[Message],
    signatures: &[Signature],
    public_keys: &[PublicKey],
) -> Result<(), Error> {
    let mut p_vec = Vec::new();
    let mut q_vec = Vec::new();
    
    for i in 0..messages.len() {
        // Add positive pairing
        p_vec.push(hash_to_g1(&messages[i]));
        q_vec.push(signatures[i].to_g2());
        
        // Add negative pairing
        p_vec.push(-G1Affine::GENERATOR);
        q_vec.push(public_keys[i].to_g2());
    }
    
    Bn254::pairing_check(&p_vec, &q_vec)
}
```

## Performance Tips

### 1. Reuse Computed Values

```rust
// Cache frequently used values
lazy_static! {
    static ref PRECOMPUTED_POWERS: Vec<G1Affine> = {
        let g = G1Affine::GENERATOR;
        (0..256).map(|i| g * Scalar::from(1u64 << i)).collect()
    };
}
```

### 2. Choose Appropriate MSM Strategy

```rust
fn smart_msm(scalars: &[Scalar], points: &[G1Affine]) -> G1Affine {
    if points.len() < 25 {
        // Use windowed multiplication for small sets
        Bn254::msm(scalars, points)
    } else {
        // For larger sets, the default MSM is used automatically
        Bn254::msm(scalars, points)
    }
}
```

### 3. Minimize Field Inversions

```rust
// Bad: Multiple inversions
let inv1 = a.invert().unwrap();
let inv2 = b.invert().unwrap();
let inv3 = c.invert().unwrap();

// Good: Batch inversion
fn batch_invert(elements: &[Fp]) -> Vec<Fp> {
    let mut products = vec![Fp::ONE];
    for elem in elements {
        products.push(products.last().unwrap() * elem);
    }
    
    let mut inv = products.last().unwrap().invert().unwrap();
    let mut result = vec![Fp::ZERO; elements.len()];
    
    for i in (0..elements.len()).rev() {
        result[i] = &inv * &products[i];
        inv *= &elements[i];
    }
    
    result
}
```

## Common Pitfalls and Solutions

### 1. Point Validation

Always validate points before use:

```rust
fn validate_g1_point(p: &G1Affine) -> Result<(), Error> {
    // Check point is on curve: y² = x³ + 3
    let y_squared = p.y.square();
    let x_cubed = p.x.square() * &p.x;
    let rhs = x_cubed + Fp::from(3u64);
    
    if y_squared != rhs {
        return Err(Error::PointNotOnCurve);
    }
    
    // BN254 G1 is prime order, so no subgroup check needed
    Ok(())
}
```

### 2. Handling Edge Cases

```rust
// Handle point at infinity
fn safe_point_add(p1: Option<G1Affine>, p2: Option<G1Affine>) -> Option<G1Affine> {
    match (p1, p2) {
        (None, None) => None,
        (Some(p), None) | (None, Some(p)) => Some(p),
        (Some(p1), Some(p2)) => Some(&p1 + &p2),
    }
}
```

### 3. ZKVM vs Host Differences

```rust
#[cfg(target_os = "zkvm")]
fn optimized_operation() {
    // ZKVM-specific implementation with hardware acceleration
}

#[cfg(not(target_os = "zkvm"))]
fn optimized_operation() {
    // Host implementation for testing
}
```

## Example: BLS Signature Verification

Complete example implementing BLS signature verification:

```rust
use openvm_pairing_guest::bn254::*;
use openvm_pairing_guest::PairingCheck;
use openvm_algebra_guest::Field;
use openvm_ecc_guest::{AffinePoint, Group};

struct BlsSignature {
    sigma: G2Affine,
}

struct BlsPublicKey {
    pk: G2Affine,
}

fn hash_to_g1(message: &[u8]) -> G1Affine {
    // Simplified - real implementation needs proper hash-to-curve
    let hash = sha256(message);
    let x = Fp::from_le_bytes(&hash);
    // Find y such that y² = x³ + 3
    // ... implementation details ...
    G1Affine::new(x, y)
}

fn verify_bls_signature(
    message: &[u8],
    signature: &BlsSignature,
    public_key: &BlsPublicKey,
) -> bool {
    let h = hash_to_g1(message);
    
    // Check e(H(m), pk) = e(g1, σ)
    let p_vec = vec![h, -G1Affine::GENERATOR];
    let q_vec = vec![public_key.pk.clone(), signature.sigma.clone()];
    
    Bn254::pairing_check(&p_vec, &q_vec).is_ok()
}
```

## Testing Your Implementation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pairing_bilinearity() {
        let a = Scalar::from(5u64);
        let b = Scalar::from(7u64);
        
        let p = G1Affine::GENERATOR;
        let q = /* G2 generator */;
        
        // e(aP, bQ) = e(P, Q)^(ab)
        let lhs = Bn254::multi_miller_loop(&[&p * &a], &[&q * &b]);
        let rhs = Bn254::multi_miller_loop(&[p], &[q]).pow(&(a * b));
        
        assert_eq!(lhs, rhs);
    }
}
```