# Ruint Division Algorithms - Quick Reference

## Main Entry Point
```rust
pub fn div(numerator: &mut [u64], divisor: &mut [u64])
// Quotient stored in numerator, remainder in divisor
// Panics if divisor is zero
```

## Small Division Functions

### Single Limb Division
```rust
pub fn div_nx1(limbs: &mut [u64], divisor: u64) -> u64
// Returns remainder, quotient in limbs

pub fn div_nx1_normalized(u: &mut [u64], d: u64) -> u64  
// Requires d >= 2^63
```

### Double Limb Division
```rust
pub fn div_nx2(limbs: &mut [u64], divisor: u128) -> u128
// Requires divisor >= 2^64

pub fn div_nx2_normalized(u: &mut [u64], d: u128) -> u128
// Requires d >= 2^127
```

### 2x1 Division (128÷64)
```rust
pub fn div_2x1(u: u128, d: u64, v: u64) -> (u64, u64)
// Requires: d >= 2^63, u < d * 2^64, v = reciprocal(d)
// Returns: (quotient, remainder)
```

### 3x2 Division (192÷128)
```rust
pub fn div_3x2(u21: u128, u0: u64, d: u128, v: u64) -> (u64, u128)
// Requires: d >= 2^127, u21 < d, v = reciprocal_2(d)
// Returns: (quotient, remainder)
```

## Knuth Division
```rust
pub fn div_nxm(numerator: &mut [u64], divisor: &mut [u64])
// General n×m division, requires len >= 3
// Remainder in divisor, quotient in numerator

pub fn div_nxm_normalized(numerator: &mut [u64], divisor: &[u64])
// Requires highest bit of divisor set
```

## Reciprocal Functions
```rust
pub fn reciprocal(d: u64) -> u64
// Computes ⌊(2^128 - 1) / d⌋ - 2^64
// Requires d >= 2^63

pub fn reciprocal_2(d: u128) -> u64  
// Computes ⌊(2^192 - 1) / d⌋ - 2^64
// Requires d >= 2^127
```

## Key Requirements
- Divisor must be non-zero
- For normalized functions: highest bit must be set
- For div_2x1: numerator < divisor * 2^64
- For div_3x2: high numerator < divisor
- Results are in-place to avoid allocations

## Performance Guidelines
- Use div_nx1 for single-limb divisors
- Use div_nx2 for double-limb divisors  
- Normalized variants are faster if divisor is pre-normalized
- Reciprocals should be computed once and reused
- div_2x1 is ~7x faster than naive division

## Common Patterns
```rust
// Basic division
let mut n = vec![/* numerator */];
let mut d = vec![/* divisor */];
div(&mut n, &mut d);
// n = quotient, d = remainder

// With known small divisor
let remainder = div_nx1(&mut numerator, small_divisor);

// Pre-normalized fast path
let shift = divisor.leading_zeros();
let normalized = divisor << shift;
let remainder = div_nx1_normalized(&mut numerator, normalized) >> shift;
```