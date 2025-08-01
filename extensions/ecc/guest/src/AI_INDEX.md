# AI Index: OpenVM ECC Guest Library

## Component Structure

### Core Files

1. **lib.rs** (40 lines)
   - Entry point and module declarations
   - Re-exports of public APIs
   - Opcode and function code definitions
   - No-std configuration

2. **affine_point.rs** (46 lines)
   - `AffinePoint<F>` struct: 2D point representation
   - Point negation and infinity checking
   - Serialization support

3. **group.rs** (37 lines)
   - `Group` trait: Abstract group operations
   - `CyclicGroup` trait: Groups with generators
   - Required operations: add, sub, neg, double

4. **weierstrass.rs** (615 lines)
   - `WeierstrassPoint` trait: Core curve operations
   - `FromCompressed` trait: Point decompression
   - `IntrinsicCurve` trait: Bridge to external curves
   - `CachedMulTable`: Precomputed scalar multiples
   - `impl_sw_affine!` macro: Generate curve implementations
   - `impl_sw_group_ops!` macro: Generate group operations

5. **msm.rs** (162 lines)
   - `msm()` function: Multi-scalar multiplication
   - Pippenger's algorithm implementation
   - Booth encoding for scalars
   - Dynamic window sizing

6. **ecdsa.rs** (563 lines)
   - `SigningKey`: Placeholder for signing (not implemented)
   - `VerifyingKey`: ECDSA verification key
   - `PublicKey`: Elliptic curve public key
   - `verify_prehashed()`: Core verification logic
   - `recover_from_prehash_noverify()`: Key recovery
   - RustCrypto trait implementations

## Key Data Structures

### AffinePoint<F>
- **Purpose**: Represent elliptic curve points in affine coordinates
- **Fields**: x, y coordinates of generic field type F
- **Methods**: new(), neg_borrow(), is_infinity()

### Group Trait Hierarchy
- **Group**: Base trait for group operations
- **CyclicGroup**: Extends Group with generator constants
- **WeierstrassPoint**: Specialized for Weierstrass curves

### VerifyingKey<C>
- **Purpose**: ECDSA signature verification
- **Contains**: PublicKey with curve point
- **Methods**: verify(), recover_from_prehash(), from_sec1_bytes()

### CachedMulTable<C>
- **Purpose**: Optimize scalar multiplication via precomputation
- **Fields**: window_bits, bases, table of multiples
- **Usage**: MSM with fixed base points

## Important Functions

### Core Operations
- `msm<EcPoint, Scalar>()`: Multi-scalar multiplication
- `verify_prehashed<C>()`: ECDSA signature verification
- `get_booth_index()`: Booth encoding helper

### Trait Methods
- `WeierstrassPoint::add_ne_nonidentity()`: Fast addition
- `WeierstrassPoint::double_nonidentity()`: Fast doubling
- `Group::double_assign()`: In-place doubling

### Macros
- `impl_sw_affine!`: Generate Weierstrass curve type
- `impl_sw_group_ops!`: Generate group operation impls

## Dependencies

### Internal Dependencies
- `openvm-algebra-guest`: Field arithmetic
- `openvm-ecc-sw-macros`: Code generation
- `openvm-custom-insn`: Custom instructions
- `openvm-rv32im-guest`: RISC-V support

### External Dependencies
- `ecdsa-core`: ECDSA traits and types
- `elliptic-curve`: Curve arithmetic traits
- `serde`: Serialization
- `strum_macros`: Enum utilities
- `once_cell`: Lazy initialization

## Configuration

### Features
- `default`: No features enabled
- `halo2curves`: Support for halo2 curves
- `std`: Standard library support
- `alloc`: Allocation support

### Constants
- `OPCODE`: 0x2b (custom-1 in RISC-V)
- `SW_FUNCT3`: 0b001 (short Weierstrass function)
- `SHORT_WEIERSTRASS_MAX_KINDS`: 8 curve types

## API Patterns

### Point Creation
```rust
AffinePoint::new(x, y)
WeierstrassPoint::from_xy(x, y)
WeierstrassPoint::from_xy_unchecked(x, y)
```

### Verification Flow
```rust
VerifyingKey::from_sec1_bytes(&bytes)?
verifying_key.verify(&message, &signature)?
```

### MSM Usage
```rust
msm(&scalars, &points)
CachedMulTable::new_with_prime_order(&bases, window_bits)
table.windowed_mul(&scalars)
```

## Performance Hints

1. **MSM Window Sizes**:
   - 1 bit for <4 points
   - 3 bits for <32 points
   - log2(n) bits for larger sets

2. **Cached Tables**: Precompute for fixed base points

3. **Booth Encoding**: Reduces point additions by ~50%

4. **Safety Annotations**: `unsafe` blocks skip redundant checks

## Integration Points

### With OpenVM
- Custom opcodes for hardware acceleration
- Integration with memory model
- Compatibility with proof system

### With RustCrypto
- Implements standard ECDSA traits
- SEC1 encoding compatibility
- Digest trait integration

## Error Handling

### Common Errors
- `Error::new()`: Generic ECDSA error
- Point not on curve
- Invalid signature format
- Zero scalars in signature

### Validation Points
- Public key non-identity check
- Signature (r,s) non-zero check
- Point on curve validation
- Coordinate canonicality

## Macro Expansion Example

```rust
impl_sw_affine!(Secp256k1Point, Fq, THREE, CURVE_B);
// Generates: struct with WeierstrassPoint impl

impl_sw_group_ops!(Secp256k1Point, Fq);
// Generates: Add, Sub, Neg trait impls
```