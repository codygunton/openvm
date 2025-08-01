# Quick Reference: OpenVM ECC Guest Library

## Essential Imports

```rust
use openvm_ecc_guest::{
    AffinePoint, Group, CyclicGroup,
    weierstrass::{WeierstrassPoint, IntrinsicCurve, FromCompressed},
    ecdsa::{VerifyingKey, verify_prehashed},
    msm,
    impl_sw_affine, impl_sw_group_ops,
};
```

## Common Operations

### Point Operations

```rust
// Create points
let p1 = AffinePoint::new(x1, y1);
let p2 = AffinePoint::new(x2, y2);

// Basic operations
let sum = p1 + p2;              // Point addition
let diff = p1 - p2;              // Point subtraction
let neg = -p1;                   // Point negation
let doubled = p1.double();       // Point doubling

// Check identity
if p1.is_infinity() { /* ... */ }

// In-place operations
let mut p = p1.clone();
p += &p2;                        // Add assign
p -= &p2;                        // Sub assign
p.double_assign();               // Double assign
```

### ECDSA Verification

```rust
// Parse public key from SEC1 bytes
let vk = VerifyingKey::from_sec1_bytes(&pubkey_bytes)?;

// Verify signature on message
vk.verify(&message, &signature)?;

// Verify with prehashed message
vk.verify_prehash(&hash, &signature)?;

// Recover public key from signature
let vk = VerifyingKey::recover_from_prehash(
    &hash, &signature, recovery_id
)?;
```

### Multi-Scalar Multiplication

```rust
// Basic MSM
let result = msm(&scalars, &points);

// With precomputed table
let table = CachedMulTable::new_with_prime_order(&bases, window_bits);
let result = table.windowed_mul(&scalars);
```

## Implementing a New Curve

### Quick Template

```rust
use openvm_algebra_guest::Field;
use openvm_ecc_guest::*;

// 1. Define field type
type Fq = YourFieldType;

// 2. Define constants
const THREE: Fq = /* 3 in field */;
const CURVE_B: Fq = /* b coefficient */;

// 3. Generate implementation
impl_sw_affine!(YourPoint, Fq, THREE, CURVE_B);
impl_sw_group_ops!(YourPoint, Fq);

// 4. Add cyclic group
impl CyclicGroup for YourPoint {
    const GENERATOR: Self = Self(AffinePoint::new(GEN_X, GEN_Y));
    const NEG_GENERATOR: Self = Self(AffinePoint::new(GEN_X, NEG_GEN_Y));
}
```

## Key Traits

### Group
```rust
trait Group {
    type SelfRef<'a>;
    const IDENTITY: Self;
    fn is_identity(&self) -> bool;
    fn double(&self) -> Self;
    fn double_assign(&mut self);
}
```

### WeierstrassPoint
```rust
trait WeierstrassPoint {
    type Coordinate: Field;
    const CURVE_A: Self::Coordinate;
    const CURVE_B: Self::Coordinate;
    const IDENTITY: Self;
    
    fn from_xy(x: Self::Coordinate, y: Self::Coordinate) -> Option<Self>;
    fn as_le_bytes(&self) -> &[u8];
    fn set_up_once();
}
```

### IntrinsicCurve
```rust
trait IntrinsicCurve {
    type Scalar: Clone;
    type Point: Clone;
    fn msm(coeffs: &[Self::Scalar], bases: &[Self::Point]) -> Self::Point;
}
```

## Performance Tips

### Use Unsafe When Appropriate
```rust
// When you know points are distinct and non-identity
unsafe {
    p1.add_ne_nonidentity::<false>(&p2)
}

// When you know point is non-identity
unsafe {
    p.double_nonidentity::<false>()
}
```

### MSM Window Sizes
- < 4 points: 1 bit
- < 32 points: 3 bits  
- ≥ 32 points: log₂(n) bits

### Batch Operations
```rust
// Process multiple operations together
let results: Vec<_> = points.iter()
    .map(|p| scalar * p)
    .collect();
```

## Common Patterns

### Point Validation
```rust
// Safe construction with validation
let point = YourPoint::from_xy(x, y)?;

// Unsafe construction (no validation)
let point = YourPoint::from_xy_unchecked(x, y);
```

### Coordinate Access
```rust
// Read coordinates
let x = point.x();
let y = point.y();

// Mutable access
let x_mut = point.x_mut();
let y_mut = point.y_mut();

// Extract coordinates
let (x, y) = point.into_coords();
```

### Serialization
```rust
// To SEC1 bytes
let compressed = vk.to_sec1_bytes(true);
let uncompressed = vk.to_sec1_bytes(false);

// From SEC1 bytes
let vk = VerifyingKey::from_sec1_bytes(&bytes)?;
```

## Error Handling

### Common Errors
```rust
// Point not on curve
WeierstrassPoint::from_xy(x, y) // Returns None

// Invalid public key (identity)
PublicKey::from_affine(identity) // Returns Err

// Invalid signature
verify_prehashed(pubkey, hash, sig) // Returns Err
```

### Recovery Patterns
```rust
// With default
let point = WeierstrassPoint::from_xy(x, y)
    .unwrap_or(WeierstrassPoint::IDENTITY);

// With error propagation
let point = WeierstrassPoint::from_xy(x, y)?;
```

## Constants and Configuration

### Opcodes
```rust
pub const OPCODE: u8 = 0x2b;  // custom-1
pub const SW_FUNCT3: u8 = 0b001;
```

### Curve Configuration
```rust
SwBaseFunct7::SwAddNe    // Addition
SwBaseFunct7::SwDouble   // Doubling
SwBaseFunct7::SwSetup    // Setup
```

## Debugging

### Assertions
```rust
#[cfg(debug_assertions)]
{
    assert!(point.is_on_curve());
    assert!(scalar != Scalar::ZERO);
}
```

### Printing (with std)
```rust
#[cfg(feature = "std")]
println!("Point: ({:?}, {:?})", point.x(), point.y());
```

## Feature Flags

```toml
[dependencies]
openvm-ecc-guest = { version = "...", features = ["halo2curves"] }
```

- `default`: No features
- `halo2curves`: Halo2 curve support
- `std`: Standard library
- `alloc`: Allocation support

## Quick Troubleshooting

| Problem | Solution |
|---------|----------|
| Point not on curve | Use `from_xy()` instead of `from_xy_unchecked()` |
| Signature verification fails | Check byte encoding (big-endian) |
| MSM wrong result | Ensure array lengths match |
| Setup not called | Call `YourPoint::set_up_once()` |
| Field elements not canonical | Use `IntMod` types |

## Useful Snippets

### ECDSA Recovery
```rust
let vk = VerifyingKey::recover_from_prehash(
    &hash, &signature, recovery_id
)?;
let address = /* derive from vk */;
```

### Batch MSM
```rust
let chunks: Vec<_> = (0..n).step_by(chunk_size)
    .map(|i| {
        let end = (i + chunk_size).min(n);
        msm(&scalars[i..end], &points[i..end])
    })
    .collect();
let result = chunks.into_iter().sum();
```

### Custom Curve Validation
```rust
fn validate_curve_point(x: Fq, y: Fq) -> bool {
    y * y == x * x * x + CURVE_A * x + CURVE_B
}
```