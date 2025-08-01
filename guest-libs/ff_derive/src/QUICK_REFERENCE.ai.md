# ff_derive Quick Reference

## Basic Usage

```rust
use openvm_ff_derive::openvm_prime_field;

#[openvm_prime_field]
#[PrimeFieldModulus = "52435875175126190479447740508185965837690552500527637822603658699938581184513"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
struct Scalar([u64; 4]);
```

## Required Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `PrimeFieldModulus` | Prime modulus as decimal string | `"65537"` |
| `PrimeFieldGenerator` | Multiplicative generator | `"3"` |
| `PrimeFieldReprEndianness` | Byte order (`"little"` or `"big"`) | `"little"` |

## Struct Requirements

- Must be a tuple struct with single field
- Field must be `[u64; N]` array
- Field must not be public
- Array size auto-computed from modulus

## Generated Traits

### From `ff` crate:
- `Field` - Basic field operations
- `PrimeField` - Prime field specific operations
- `PrimeFieldBits` (with `bits` feature)

### Standard Operations:
- `Copy`, `Clone`, `Debug`, `Default`
- `PartialEq`, `Eq`, `PartialOrd`, `Ord`
- `Add`, `Sub`, `Mul`, `Neg`
- `AddAssign`, `SubAssign`, `MulAssign`
- `Sum`, `Product`
- `From<u64>`
- `ConditionallySelectable`
- `ConstantTimeEq`

## Generated Constants

```rust
// Field characteristics
const MODULUS: &'static str
const NUM_BITS: u32
const CAPACITY: u32
const S: u32  // 2^S * t = p - 1

// Field elements
const ZERO: Self
const ONE: Self
const TWO_INV: Self
const GENERATOR: Self
const ROOT_OF_UNITY: Self
const ROOT_OF_UNITY_INV: Self
const DELTA: Self
```

## Memory Layout

| Modulus Size | zkVM Layout | Standard Layout |
|--------------|-------------|-----------------|
| ≤ 256 bits   | 32 bytes    | 4 × u64         |
| ≤ 384 bits   | 48 bytes    | 6 × u64         |
| > 384 bits   | Not supported | Not supported |

## Key Methods

### Field Operations
```rust
// Arithmetic
fn add(&self, other: &Self) -> Self
fn sub(&self, other: &Self) -> Self
fn mul(&self, other: &Self) -> Self
fn square(&self) -> Self
fn double(&self) -> Self
fn neg(self) -> Self

// Advanced
fn invert(&self) -> CtOption<Self>
fn sqrt(&self) -> CtOption<Self>
fn pow(&self, exp: [u64; N]) -> Self
```

### Conversions
```rust
// From/to bytes
fn from_repr(repr: Repr) -> CtOption<Self>
fn to_repr(&self) -> Repr

// From integers
fn from(val: u64) -> Self
```

## Platform Differences

### Standard Rust
- Uses Montgomery form internally
- All operations in Montgomery space
- Optimized 64-bit arithmetic

### zkVM (`target_os = "zkvm"`)
- Direct modular arithmetic
- Uses `openvm_algebra_guest` operations
- Byte-array representation

## Common Patterns

### Creating Field Elements
```rust
use ff::Field;

// Zero and one
let zero = Scalar::ZERO;
let one = Scalar::ONE;

// From u64
let two = Scalar::from(2u64);

// From bytes (little-endian example)
let bytes = [1u8; 32];
let element = Scalar::from_repr(ScalarRepr(bytes)).unwrap();
```

### Field Arithmetic
```rust
let a = Scalar::from(5);
let b = Scalar::from(7);

let sum = a + b;
let product = a * b;
let square = a.square();
let inverse = a.invert().unwrap();
```

### Checking Properties
```rust
// Check if zero
if element.is_zero_vartime() {
    // handle zero case
}

// Check if odd
if element.is_odd().into() {
    // handle odd case
}
```

## Performance Notes

1. **Inversion**: Most expensive operation
2. **Squaring**: Optimized vs general multiplication
3. **Addition/Subtraction**: Cheap operations
4. **Constants**: Precomputed at compile time

## Common Field Moduli

```rust
// BLS12-381 scalar field
"52435875175126190479447740508185965837690552500527637822603658699938581184513"

// BN254 scalar field
"21888242871839275222246405745257275088548364400416034343698204186575808495617"

// Small Fermat prime
"65537"
```

## Debugging Tips

1. Check modulus size fits in supported range
2. Verify generator is quadratic non-residue
3. Use `Debug` impl to inspect values
4. Test both standard and zkVM targets
5. Validate constants match expected values