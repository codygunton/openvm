# Implementation Guide: OpenVM ECC Guest Library

## Overview

This guide provides detailed implementation guidance for extending and modifying the OpenVM ECC guest library. It covers the internal architecture, extension points, and best practices for implementing new curves and operations.

## Architecture Deep Dive

### Layer Structure

```
┌─────────────────────────────────────┐
│         User Applications           │
├─────────────────────────────────────┤
│    High-Level APIs (ECDSA, MSM)    │
├─────────────────────────────────────┤
│   Curve Implementations (Macros)    │
├─────────────────────────────────────┤
│    Core Traits (Group, Weierstrass)│
├─────────────────────────────────────┤
│   Field Operations (algebra-guest)  │
└─────────────────────────────────────┘
```

### Trait Hierarchy

```rust
trait Group {
    // Basic group operations
    fn double(&self) -> Self;
    fn is_identity(&self) -> bool;
}

trait CyclicGroup: Group {
    // Adds generator constants
    const GENERATOR: Self;
}

trait WeierstrassPoint {
    // Curve-specific operations
    type Coordinate: Field;
    const CURVE_A: Self::Coordinate;
    const CURVE_B: Self::Coordinate;
}
```

## Implementing a New Curve

### Step 1: Define Curve Parameters

```rust
// In your curve module
use openvm_algebra_guest::{Field, IntMod};
use openvm_ecc_guest::{AffinePoint, Group, impl_sw_affine, impl_sw_group_ops};

// Define your field type
type Fq = YourFieldType;

// Define curve constants
const THREE: Fq = /* 3 in your field */;
const CURVE_B: Fq = /* b coefficient */;
```

### Step 2: Generate Curve Implementation

```rust
// Generate affine point type with curve operations
impl_sw_affine!(YourCurvePoint, Fq, THREE, CURVE_B);

// Generate group operation implementations
impl_sw_group_ops!(YourCurvePoint, Fq);
```

### Step 3: Implement Required Traits

```rust
impl CyclicGroup for YourCurvePoint {
    const GENERATOR: Self = Self(AffinePoint::new(GENERATOR_X, GENERATOR_Y));
    const NEG_GENERATOR: Self = Self(AffinePoint::new(GENERATOR_X, NEG_GENERATOR_Y));
}

impl IntrinsicCurve for YourCurve {
    type Scalar = YourScalarType;
    type Point = YourCurvePoint;
    
    fn msm(coeffs: &[Self::Scalar], bases: &[Self::Point]) -> Self::Point {
        crate::msm::msm(coeffs, bases)
    }
}
```

### Step 4: Add Decompression Support

```rust
impl FromCompressed<Fq> for YourCurvePoint {
    fn decompress(x: Fq, rec_id: &u8) -> Option<Self> {
        // Compute y² = x³ + ax + b
        let y_squared = x * x * x + CURVE_A * x + CURVE_B;
        
        // Find square root
        let y = y_squared.sqrt()?;
        
        // Select correct y based on parity
        let y = if (y.as_le_bytes()[0] & 1) == *rec_id {
            y
        } else {
            -y
        };
        
        Some(Self::from_xy_unchecked(x, y))
    }
}
```

## Optimizing Performance

### 1. Precomputed Tables

For fixed base scalar multiplication:

```rust
pub fn create_msm_table(base: &YourCurvePoint) -> CachedMulTable<YourCurve> {
    CachedMulTable::new_with_prime_order(&[base.clone()], OPTIMAL_WINDOW_BITS)
}
```

### 2. Unsafe Operations

When you can guarantee preconditions:

```rust
// Skip identity and equality checks
unsafe {
    point1.add_ne_nonidentity::<false>(&point2)
}
```

### 3. Batch Operations

Process multiple operations together:

```rust
pub fn batch_verify(pubkeys: &[PublicKey], messages: &[&[u8]], sigs: &[Signature]) -> bool {
    // Use MSM for batch verification
    let scalars: Vec<_> = /* compute challenge scalars */;
    let points: Vec<_> = /* collect public keys */;
    let result = msm(&scalars, &points);
    // Check single equation
}
```

## Custom Opcodes Integration

### 1. Define Opcode Constants

```rust
pub const CUSTOM_OP_FUNCT7: u8 = YOUR_CURVE_IDX * SwBaseFunct7::SHORT_WEIERSTRASS_MAX_KINDS + SwBaseFunct7::SwAddNe as u8;
```

### 2. Implement Hardware Acceleration

```rust
unsafe fn add_ne_nonidentity<const CHECK_SETUP: bool>(&self, p2: &Self) -> Self {
    if CHECK_SETUP {
        Self::set_up_once();
    }
    
    // Call custom instruction
    let result = custom_insn_rr(OPCODE, SW_FUNCT3, CUSTOM_OP_FUNCT7, self, p2);
    Self::from_xy_unchecked(result.0, result.1)
}
```

## Testing Strategies

### 1. Property-Based Tests

```rust
#[test]
fn test_group_laws() {
    // Associativity: (a + b) + c = a + (b + c)
    // Identity: a + 0 = a
    // Inverse: a + (-a) = 0
    // Commutativity: a + b = b + a
}
```

### 2. Known Answer Tests

```rust
#[test]
fn test_ecdsa_vectors() {
    // Test with known signatures from test vectors
    let test_cases = include!("test_vectors.rs");
    for case in test_cases {
        let vk = VerifyingKey::from_sec1_bytes(&case.pubkey).unwrap();
        assert!(vk.verify(&case.message, &case.signature).is_ok());
    }
}
```

### 3. Edge Cases

```rust
#[test]
fn test_edge_cases() {
    // Point at infinity
    // Points with same x-coordinate
    // Double of 2-torsion points
    // Maximum scalar values
}
```

## Common Pitfalls and Solutions

### 1. Coordinate System Mismatch

**Problem**: Different libraries use different coordinate representations.

**Solution**: Always validate coordinate encoding:
```rust
// Convert from external representation
let x = Coordinate::from_be_bytes(&external_x)?;
let y = Coordinate::from_be_bytes(&external_y)?;
let point = WeierstrassPoint::from_xy(x, y)?;
```

### 2. Field Element Reduction

**Problem**: Non-canonical field elements cause comparison failures.

**Solution**: Use `IntMod` trait which handles reduction:
```rust
// IntMod automatically reduces on operations
let sum = a + b; // Automatically reduced
```

### 3. Scalar/Field Size Mismatch

**Problem**: Scalar field and coordinate field have different sizes.

**Solution**: Use proper conversion:
```rust
// Convert coordinate to scalar
let x_mod_n = Scalar::reduce_le_bytes(x.as_le_bytes());
```

## Advanced Techniques

### 1. GLV Endomorphisms

For curves with efficient endomorphisms:

```rust
impl YourCurvePoint {
    fn endomorphism(&self) -> Self {
        // Apply efficient endomorphism
        Self::from_xy_unchecked(beta * self.x(), self.y().clone())
    }
}
```

### 2. Windowed NAF

For better scalar representation:

```rust
fn to_wnaf(scalar: &Scalar, window: usize) -> Vec<i8> {
    // Convert to width-w NAF representation
}
```

### 3. Parallel MSM

Split work across multiple operations:

```rust
fn parallel_msm(coeffs: &[Scalar], bases: &[Point]) -> Point {
    let chunk_size = coeffs.len() / NUM_CHUNKS;
    let chunks: Vec<_> = coeffs.chunks(chunk_size)
        .zip(bases.chunks(chunk_size))
        .map(|(c, b)| msm(c, b))
        .collect();
    
    chunks.into_iter().fold(Point::IDENTITY, |acc, p| acc + p)
}
```

## Debugging Tips

### 1. Assertion Helpers

```rust
#[cfg(debug_assertions)]
fn assert_on_curve(point: &YourCurvePoint) {
    let (x, y) = (point.x(), point.y());
    let lhs = y * y;
    let rhs = x * x * x + CURVE_A * x + CURVE_B;
    assert_eq!(lhs, rhs, "Point not on curve");
}
```

### 2. Trace Macros

```rust
macro_rules! trace_point {
    ($point:expr) => {
        #[cfg(feature = "trace")]
        eprintln!("Point: ({:?}, {:?})", $point.x(), $point.y());
    };
}
```

### 3. Invariant Checks

```rust
impl YourCurvePoint {
    fn check_invariants(&self) {
        debug_assert!(self.is_on_curve());
        debug_assert!(self.x().is_reduced());
        debug_assert!(self.y().is_reduced());
    }
}
```

## Integration Checklist

When adding a new curve:

- [ ] Define field and scalar types
- [ ] Implement curve parameters (a, b, generator)
- [ ] Use macros to generate implementations
- [ ] Add decompression support
- [ ] Implement `IntrinsicCurve` trait
- [ ] Add comprehensive tests
- [ ] Document curve-specific behavior
- [ ] Benchmark performance
- [ ] Add to feature flags if optional
- [ ] Update documentation

## Performance Profiling

### 1. Cycle Counting

```rust
#[cfg(feature = "profile")]
fn profile_operation() {
    let start = read_cycle_counter();
    // Perform operation
    let end = read_cycle_counter();
    println!("Cycles: {}", end - start);
}
```

### 2. Memory Usage

```rust
fn measure_memory() {
    let before = current_memory_usage();
    // Perform operation
    let after = current_memory_usage();
    println!("Memory delta: {} bytes", after - before);
}
```

## Future Extensibility

### 1. New Curve Models

To add non-Weierstrass curves:
- Create new trait hierarchy
- Implement specialized operations
- Provide conversion utilities

### 2. Optimization Opportunities

- Projective coordinates
- Montgomery ladder
- Batch inversion
- Straus-Shamir multi-exponentiation

### 3. Additional Features

- Pairing operations
- Hash-to-curve
- Cofactor clearing
- Subgroup checks