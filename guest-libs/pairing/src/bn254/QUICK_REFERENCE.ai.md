# BN254 Pairing - Quick Reference

## Essential Imports

```rust
use openvm_pairing_guest::bn254::{Bn254, Fp, Fp2, Fp12, G1Affine, G2Affine, Scalar};
use openvm_pairing_guest::PairingCheck;
use openvm_algebra_guest::Field;
use openvm_ecc_guest::{AffinePoint, Group, CyclicGroup};
```

## Type Definitions

| Type | Description | Size |
|------|-------------|------|
| `Fp` | Base field element | 32 bytes |
| `Fp2` | Quadratic extension field | 64 bytes |
| `Fp12` | 12th degree extension (target field) | 384 bytes |
| `Scalar` | Scalar field element | 32 bytes |
| `G1Affine` | Point on G1 (E(Fp)) | 64 bytes |
| `G2Affine` | Point on G2 (E'(Fp2)) | 128 bytes |

## Field Operations

### Basic Arithmetic
```rust
let a = Fp::from(42u64);
let b = Fp::from(17u64);

let sum = &a + &b;           // Addition
let diff = &a - &b;          // Subtraction  
let prod = &a * &b;          // Multiplication
let quot = a.div_unsafe(&b); // Division (b ≠ 0)
let sq = a.square();         // Squaring
let inv = a.invert();        // Inversion (returns Option)
```

### Extension Field Operations
```rust
// Fp2
let z = Fp2::new(a, b);      // a + b*u
let conj = z.conjugate();    // a - b*u
let frob = z.frobenius_map(1); // Same as conjugate

// Fp12
let w = Fp12::embed(z);      // Embed Fp2 into Fp12
let frob12 = w.frobenius_map(3); // Frobenius power
```

## Curve Points

### Creating Points
```rust
// Generators
let g1 = G1Affine::GENERATOR;
let g1_neg = G1Affine::NEG_GENERATOR;

// From coordinates
let p = G1Affine::new(x, y);
let q = G2Affine::new(x_fp2, y_fp2);
```

### Point Operations
```rust
// Basic operations
let sum = &p1 + &p2;         // Addition
let double = p.double();     // Doubling
let neg = -&p;              // Negation

// Scalar multiplication
let scalar = Scalar::from(12345u64);
let result = &g1 * &scalar;
```

## Pairing Operations

### Pairing Check
```rust
// Check e(P1,Q1) * e(P2,Q2) * ... = 1
let p_vec = vec![p1, p2, p3];
let q_vec = vec![q1, q2, q3];

match Bn254::pairing_check(&p_vec, &q_vec) {
    Ok(()) => { /* success */ },
    Err(_) => { /* failure */ }
}
```

### Miller Loop
```rust
// Compute f^{6x+2} (not full pairing)
let f = Bn254::multi_miller_loop(&p_vec, &q_vec);

// With embedded exponent
let f_c = Bn254::multi_miller_loop_embedded_exp(&p_vec, &q_vec, Some(c));
```

### Multi-Scalar Multiplication
```rust
let points = vec![p1, p2, p3];
let scalars = vec![s1, s2, s3];

// Compute s1*p1 + s2*p2 + s3*p3
let result = Bn254::msm(&scalars, &points);
```

## Constants

### Field Moduli
```
p = 21888242871839275222246405745257275088696311157297823662689037894645226208583
r = 21888242871839275222246405745257275088548364400416034343698204186575808495617
```

### Curve Parameters
- **Curve**: y² = x³ + 3
- **Twisted Curve**: y² = x³ + 3/(9+u)
- **Embedding Degree**: k = 12
- **Seed**: x = 4965661367192848881 (63 bits)

### Key Constants in Code
```rust
Bn254::XI           // ξ = 9 + u (non-residue for Fp12)
Bn254::FINAL_EXPONENT    // (p^12 - 1)/r
Bn254::FROBENIUS_COEFFS  // Precomputed Frobenius constants
```

## Common Patterns

### BLS Signature Verification
```rust
fn verify_bls(msg: &[u8], sig: &G2Affine, pk: &G2Affine) -> bool {
    let h = hash_to_g1(msg);
    let p_vec = vec![h, -G1Affine::GENERATOR];
    let q_vec = vec![pk.clone(), sig.clone()];
    Bn254::pairing_check(&p_vec, &q_vec).is_ok()
}
```

### Groth16 Verification
```rust
fn verify_groth16(
    vk_alpha_g1: &G1Affine,
    vk_beta_g2: &G2Affine,
    vk_gamma_g2: &G2Affine,
    vk_delta_g2: &G2Affine,
    public_inputs: &[Scalar],
    proof_a: &G1Affine,
    proof_b: &G2Affine,
    proof_c: &G1Affine,
) -> bool {
    // Compute vk_x
    let vk_x = compute_vk_x(public_inputs);
    
    // Pairing check: e(A,B) = e(α,β) * e(vk_x,γ) * e(C,δ)
    let p_vec = vec![
        proof_a.clone(),
        -vk_alpha_g1.clone(),
        -vk_x,
        -proof_c.clone(),
    ];
    let q_vec = vec![
        proof_b.clone(),
        vk_beta_g2.clone(),
        vk_gamma_g2.clone(),
        vk_delta_g2.clone(),
    ];
    
    Bn254::pairing_check(&p_vec, &q_vec).is_ok()
}
```

### Batch Operations
```rust
// Batch inversion
fn batch_invert(elements: &[Fp]) -> Vec<Option<Fp>> {
    // Montgomery batch inversion trick
    // ... implementation ...
}

// Batch MSM
fn batch_msm(
    scalars_list: &[Vec<Scalar>],
    points_list: &[Vec<G1Affine>],
) -> Vec<G1Affine> {
    scalars_list.iter()
        .zip(points_list.iter())
        .map(|(s, p)| Bn254::msm(s, p))
        .collect()
}
```

## Performance Guidelines

### Optimization Thresholds
- MSM: Use windowed method for <25 points
- Line evaluation: Batch in pairs when possible
- Field ops: Minimize inversions, use batch inversion

### Memory Alignment
- Fp elements: 32-byte aligned
- Points: Natural alignment (64/128 bytes)
- Arrays: Consider cache line alignment

### ZKVM Specific
```rust
#[cfg(target_os = "zkvm")]
{
    // Hardware accelerated operations available
    // Pairing hints generated via custom instructions
}

#[cfg(not(target_os = "zkvm"))]
{
    // Software implementation
    // Enable halo2curves feature for testing
}
```

## Error Handling

### Common Errors
```rust
use openvm_pairing_guest::pairing::PairingCheckError;

match result {
    Ok(()) => { /* success */ },
    Err(PairingCheckError::InvalidPairing) => { 
        // Pairing equation doesn't hold
    },
    Err(PairingCheckError::InvalidInput) => {
        // Invalid points or mismatched array lengths
    },
}
```

### Validation Functions
```rust
fn is_on_curve_g1(p: &G1Affine) -> bool {
    let y2 = p.y.square();
    let x3_plus_3 = p.x.square() * &p.x + Fp::from(3u64);
    y2 == x3_plus_3
}

fn is_valid_scalar(s: &Scalar) -> bool {
    // Scalars are always valid in the field
    true
}
```

## Quick Formulas

### Pairing Properties
- **Bilinearity**: e(aP, bQ) = e(P, Q)^(ab)
- **Non-degeneracy**: e(G, H) ≠ 1 for generators
- **Inverse**: e(P, -Q) = e(P, Q)^(-1)

### Miller Loop Output
- `multi_miller_loop` returns f^(6x+2), not e(P,Q)
- Full pairing requires final exponentiation
- Use `pairing_check` for equality testing

### Efficiency Tips
1. Prefer `pairing_check` over computing full pairings
2. Combine multiple checks into single multi-pairing
3. Reuse point additions when possible
4. Cache frequently used scalar multiplications