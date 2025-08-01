# GCD Algorithms - Quick Reference

## Core Functions

### `gcd`
```rust
pub fn gcd<const BITS: usize, const LIMBS: usize>(
    a: Uint<BITS, LIMBS>,
    b: Uint<BITS, LIMBS>
) -> Uint<BITS, LIMBS>
```
- **Purpose**: Compute greatest common divisor
- **Returns**: GCD of a and b
- **Algorithm**: Lehmer's algorithm with matrix acceleration
- **Time Complexity**: O(n²) where n is bit length

### `gcd_extended`
```rust
pub fn gcd_extended<const BITS: usize, const LIMBS: usize>(
    a: Uint<BITS, LIMBS>,
    b: Uint<BITS, LIMBS>
) -> (Uint<BITS, LIMBS>, Uint<BITS, LIMBS>, Uint<BITS, LIMBS>, bool)
```
- **Returns**: `(gcd, x, y, sign)`
- **Bezout Identity**: 
  - If sign=true: `gcd = a*x - b*y`
  - If sign=false: `gcd = b*y - a*x`
- **Use Case**: When you need coefficients for linear combinations

### `inv_mod`
```rust
pub fn inv_mod<const BITS: usize, const LIMBS: usize>(
    num: Uint<BITS, LIMBS>,
    modulus: Uint<BITS, LIMBS>
) -> Option<Uint<BITS, LIMBS>>
```
- **Purpose**: Compute modular multiplicative inverse
- **Returns**: `Some(inverse)` if exists, `None` if not coprime
- **Property**: `num * inverse ≡ 1 (mod modulus)`
- **Use Case**: Cryptography, modular arithmetic

## LehmerMatrix Type

### Structure
```rust
pub struct Matrix(pub u64, pub u64, pub u64, pub u64, pub bool);
```
- **Fields**: 2x2 matrix entries + sign pattern flag
- **Sign Patterns**:
  ```
  true:  [[ .0, -.1],    false: [[-.0,  .1],
          [-.2,  .3]]            [ .2, -.3]]
  ```

### Key Methods

#### `Matrix::from`
```rust
pub fn from<const BITS: usize, const LIMBS: usize>(
    a: Uint<BITS, LIMBS>,
    b: Uint<BITS, LIMBS>
) -> Self
```
- **Purpose**: Create matrix from two integers
- **Precondition**: `a >= b`
- **Algorithm**: Chooses optimal path based on bit size

#### `Matrix::from_u64`
```rust
pub fn from_u64(r0: u64, r1: u64) -> Self
```
- **Purpose**: Optimized for 64-bit values
- **Use**: When both inputs fit in u64
- **Performance**: Fastest path for small numbers

#### `Matrix::from_u64_prefix`
```rust
pub fn from_u64_prefix(a0: u64, a1: u64) -> Self
```
- **Purpose**: Compute matrix from high 64 bits
- **Precondition**: `a0` has highest bit set
- **Use**: Internal optimization for large numbers

#### `Matrix::apply`
```rust
pub fn apply<const BITS: usize, const LIMBS: usize>(
    &self,
    a: &mut Uint<BITS, LIMBS>,
    b: &mut Uint<BITS, LIMBS>
)
```
- **Purpose**: Apply matrix transformation in-place
- **Effect**: Updates (a,b) according to matrix

#### `Matrix::compose`
```rust
pub const fn compose(self, other: Self) -> Self
```
- **Purpose**: Matrix multiplication
- **Returns**: `self * other`
- **Use**: Combining multiple transformation steps

## Constants and Special Values

### `Matrix::IDENTITY`
```rust
pub const IDENTITY: Self = Self(1, 0, 0, 1, true);
```
- **Purpose**: Identity matrix (no-op transformation)
- **Use**: Detecting when Lehmer step fails

## Common Patterns

### Basic GCD
```rust
let g = gcd(a, b);
```

### Check Coprimality
```rust
let are_coprime = gcd(a, b) == Uint::ONE;
```

### Modular Inverse with Error Handling
```rust
match inv_mod(x, m) {
    Some(x_inv) => { /* use x_inv */ },
    None => { /* x and m not coprime */ }
}
```

### Extended GCD Usage
```rust
let (g, x, y, sign) = gcd_extended(a, b);
let combination = if sign {
    a * x - b * y
} else {
    b * y - a * x
};
assert_eq!(combination, g);
```

### Matrix Application
```rust
let m = LehmerMatrix::from(a, b);
if m != LehmerMatrix::IDENTITY {
    m.apply(&mut a, &mut b);
}
```

## Performance Notes

### When to Use What
- **Small numbers (<64 bits)**: Direct `from_u64`
- **Medium numbers (64-128 bits)**: `from_u128_prefix`
- **Large numbers (>128 bits)**: Standard `from` method

### Optimization Flags
- Matrix operations vectorize well
- Identity check prevents unnecessary work
- Prefix methods reduce precision requirements

## Error Conditions

### `inv_mod` Returns None When:
- Modulus is zero
- `gcd(num, modulus) != 1`
- Numbers are not coprime

### Panics
- `Matrix::from` panics if `b > a`
- Most operations handle edge cases gracefully

## Type Constraints
- All functions work with `Uint<BITS, LIMBS>`
- `BITS`: Total bit size of the integer
- `LIMBS`: Number of 64-bit limbs (usually `BITS/64`)
- Functions are generic over these parameters