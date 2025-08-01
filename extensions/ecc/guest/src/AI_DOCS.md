# AI Documentation: OpenVM ECC Guest Library

## Component Overview

The `openvm-ecc-guest` library provides zero-knowledge friendly elliptic curve cryptography primitives for OpenVM guest programs. It implements core elliptic curve operations, ECDSA signature verification, and multi-scalar multiplication (MSM) optimized for zkVM execution.

### Purpose

This library enables guest programs to:
- Perform elliptic curve operations on Weierstrass curves
- Verify ECDSA signatures
- Execute multi-scalar multiplications
- Support various curves including secp256k1, P-256, BN254, and BLS12-381

### Key Design Principles

1. **No-std compatibility**: Designed for embedded zkVM environments
2. **ZK-friendly**: Operations optimized for proof generation
3. **Modular architecture**: Extensible to support multiple curve types
4. **Type safety**: Strongly typed curve representations

## Architecture

### Core Components

1. **Affine Points** (`affine_point.rs`)
   - Basic 2D point representation
   - Handles point negation and infinity checks

2. **Group Traits** (`group.rs`)
   - Abstract group operations
   - Cyclic group support with generators

3. **Weierstrass Curves** (`weierstrass.rs`)
   - Short Weierstrass curve equation: y² = x³ + ax + b
   - Optimized addition and doubling formulas
   - Cached multiplication tables for MSM

4. **ECDSA** (`ecdsa.rs`)
   - Signature verification (no signing support)
   - Key recovery from signatures
   - Compatible with RustCrypto traits

5. **Multi-Scalar Multiplication** (`msm.rs`)
   - Pippenger's algorithm implementation
   - Windowed method optimization

### Integration Points

- **OpenVM Algebra**: Field arithmetic operations
- **OpenVM Custom Instructions**: Hardware-accelerated curve operations
- **RustCrypto Ecosystem**: ECDSA and elliptic curve traits

## Implementation Details

### Weierstrass Point Operations

The library implements efficient point arithmetic using:
- **Addition formula** (for P₁ ≠ ±P₂):
  ```
  λ = (y₂ - y₁) / (x₂ - x₁)
  x₃ = λ² - x₁ - x₂
  y₃ = λ(x₁ - x₃) - y₁
  ```

- **Doubling formula** (for a = 0):
  ```
  λ = 3x₁² / 2y₁
  x₃ = λ² - 2x₁
  y₃ = λ(x₁ - x₃) - y₁
  ```

### MSM Algorithm

Uses Pippenger's algorithm with:
- Dynamic window sizing based on input size
- Booth encoding for scalar representation
- Bucket accumulation for efficiency

### ECDSA Verification

Implements standard ECDSA verification:
1. Parse signature (r, s) and public key
2. Compute z = hash(message)
3. Calculate u₁ = z/s and u₂ = r/s
4. Verify R = u₁·G + u₂·Q has x-coordinate equal to r

## Usage Patterns

### Basic Point Operations

```rust
use openvm_ecc_guest::{AffinePoint, Group};

// Create points
let p1 = AffinePoint::new(x1, y1);
let p2 = AffinePoint::new(x2, y2);

// Point addition
let sum = p1 + p2;

// Point doubling
let doubled = p1.double();
```

### ECDSA Verification

```rust
use openvm_ecc_guest::ecdsa::{VerifyingKey, Signature};

// Parse public key
let vk = VerifyingKey::from_sec1_bytes(&pubkey_bytes)?;

// Verify signature
vk.verify(&message, &signature)?;
```

### Multi-Scalar Multiplication

```rust
use openvm_ecc_guest::msm;

// Compute Σ(scalar_i · point_i)
let result = msm(&scalars, &points);
```

## Performance Considerations

1. **Window size selection**: MSM performance depends on choosing appropriate window sizes
2. **Point representation**: Affine coordinates used for simplicity
3. **Memory layout**: Points stored contiguously for cache efficiency
4. **Booth encoding**: Reduces the number of point additions

## Security Notes

1. **No constant-time guarantees**: Operations may leak timing information
2. **Input validation**: Points must be on the curve
3. **Signature malleability**: Not prevented by default
4. **Side channels**: zkVM execution may reveal operation patterns

## Common Patterns and Best Practices

1. **Curve setup**: Call `set_up_once()` before operations
2. **Point validation**: Use `from_xy()` to ensure points are on curve
3. **Batch operations**: Use MSM for multiple scalar multiplications
4. **Error handling**: All operations return `Result` types

## Troubleshooting Guide

### Common Issues

1. **Point not on curve**: Ensure x,y coordinates satisfy curve equation
2. **Invalid signature**: Check signature encoding (big-endian)
3. **MSM mismatch**: Verify scalar and point array lengths match
4. **Setup not called**: Some curves require initialization

### Debug Strategies

1. Check point validity with `from_xy()`
2. Verify field element canonicality
3. Test with known test vectors
4. Enable debug logging for detailed traces

## Future Considerations

1. **Projective coordinates**: May improve performance
2. **Batch verification**: Could optimize multiple signatures
3. **Additional curves**: Easy to add new Weierstrass curves
4. **Constant-time operations**: For enhanced security