# GCD Algorithms Implementation Guide

## Overview
This guide provides patterns and examples for implementing and extending the GCD algorithms in the ruint library.

## Core Implementation Patterns

### 1. Basic GCD Usage
```rust
use ruint::{Uint, algorithms::gcd};

// Simple GCD computation
let a = Uint::<256, 4>::from(48u64);
let b = Uint::<256, 4>::from(18u64);
let result = gcd(a, b);
assert_eq!(result, Uint::from(6u64));

// GCD handles any input order
let result2 = gcd(b, a);  // Same result
assert_eq!(result, result2);
```

### 2. Extended GCD for Bezout Coefficients
```rust
use ruint::{Uint, algorithms::gcd_extended};

let a = Uint::<256, 4>::from(240u64);
let b = Uint::<256, 4>::from(46u64);
let (gcd, x, y, sign) = gcd_extended(a, b);

// Verify Bezout identity
if sign {
    assert_eq!(gcd, a * x - b * y);
} else {
    assert_eq!(gcd, b * y - a * x);
}
```

### 3. Modular Inverse
```rust
use ruint::{Uint, algorithms::inv_mod};

let modulus = Uint::<256, 4>::from(17u64);
let num = Uint::<256, 4>::from(3u64);

match inv_mod(num, modulus) {
    Some(inverse) => {
        // Verify: num * inverse ≡ 1 (mod modulus)
        assert_eq!((num * inverse) % modulus, Uint::ONE);
    }
    None => {
        // Numbers are not coprime
        panic!("No modular inverse exists");
    }
}
```

## Advanced Patterns

### 4. Custom Matrix Operations
```rust
use ruint::algorithms::LehmerMatrix;

// Create matrix from values
let a = Uint::<256, 4>::from(1000u64);
let b = Uint::<256, 4>::from(600u64);
let matrix = LehmerMatrix::from(a, b);

// Apply matrix transformation
let mut c = a;
let mut d = b;
matrix.apply(&mut c, &mut d);

// Compose multiple matrices
let matrix2 = LehmerMatrix::from(c, d);
let combined = matrix2.compose(matrix);
```

### 5. Implementing Custom GCD Variants
```rust
/// GCD with step counting
pub fn gcd_with_steps<const BITS: usize, const LIMBS: usize>(
    mut a: Uint<BITS, LIMBS>,
    mut b: Uint<BITS, LIMBS>,
) -> (Uint<BITS, LIMBS>, usize) {
    use core::mem::swap;
    let mut steps = 0;
    
    if b > a {
        swap(&mut a, &mut b);
    }
    
    while b != Uint::ZERO {
        let m = LehmerMatrix::from(a, b);
        steps += 1;
        
        if m == LehmerMatrix::IDENTITY {
            a %= b;
            swap(&mut a, &mut b);
        } else {
            m.apply(&mut a, &mut b);
        }
    }
    
    (a, steps)
}
```

### 6. Binary GCD Alternative
```rust
/// Binary GCD for comparison
pub fn binary_gcd<const BITS: usize, const LIMBS: usize>(
    mut a: Uint<BITS, LIMBS>,
    mut b: Uint<BITS, LIMBS>,
) -> Uint<BITS, LIMBS> {
    use core::mem::swap;
    
    if a.is_zero() { return b; }
    if b.is_zero() { return a; }
    
    // Factor out common powers of 2
    let shift = (a | b).trailing_zeros();
    a >>= a.trailing_zeros();
    b >>= b.trailing_zeros();
    
    while a != b {
        if a > b {
            swap(&mut a, &mut b);
        }
        b -= a;
        b >>= b.trailing_zeros();
    }
    
    a << shift
}
```

## Performance Optimization Tips

### 7. Batch GCD Operations
```rust
/// Compute GCD of multiple pairs efficiently
pub fn batch_gcd<const BITS: usize, const LIMBS: usize>(
    pairs: &[(Uint<BITS, LIMBS>, Uint<BITS, LIMBS>)],
) -> Vec<Uint<BITS, LIMBS>> {
    pairs.iter()
        .map(|(a, b)| gcd(*a, *b))
        .collect()
}
```

### 8. Specialized Small Value Handling
```rust
/// Optimized GCD for values known to fit in 64 bits
pub fn gcd_small(a: u64, b: u64) -> u64 {
    use ruint::Uint;
    
    if a.leading_zeros() + b.leading_zeros() >= 128 {
        // Use specialized u64 path
        let matrix = LehmerMatrix::from_u64(a.max(b), a.min(b));
        let (c, d) = matrix.apply_u128(a as u128, b as u128);
        c as u64
    } else {
        // Fall back to full implementation
        gcd(Uint::<128, 2>::from(a), Uint::<128, 2>::from(b))
            .try_into()
            .unwrap()
    }
}
```

## Error Handling Patterns

### 9. Safe Modular Inverse
```rust
/// Wrapper with custom error type
#[derive(Debug)]
pub enum ModInvError {
    ZeroModulus,
    NotCoprime,
}

pub fn safe_inv_mod<const BITS: usize, const LIMBS: usize>(
    num: Uint<BITS, LIMBS>,
    modulus: Uint<BITS, LIMBS>,
) -> Result<Uint<BITS, LIMBS>, ModInvError> {
    if modulus.is_zero() {
        return Err(ModInvError::ZeroModulus);
    }
    
    inv_mod(num, modulus)
        .ok_or(ModInvError::NotCoprime)
}
```

## Integration Examples

### 10. RSA Key Generation Helper
```rust
/// Compute private exponent for RSA
pub fn compute_rsa_d<const BITS: usize, const LIMBS: usize>(
    e: Uint<BITS, LIMBS>,
    p: Uint<BITS, LIMBS>,
    q: Uint<BITS, LIMBS>,
) -> Option<Uint<BITS, LIMBS>> {
    // Compute Euler's totient
    let phi = (p - Uint::ONE) * (q - Uint::ONE);
    
    // Compute modular inverse
    inv_mod(e, phi)
}
```

### 11. Rational Number Reduction
```rust
/// Reduce a fraction to lowest terms
pub struct Rational<const BITS: usize, const LIMBS: usize> {
    numerator: Uint<BITS, LIMBS>,
    denominator: Uint<BITS, LIMBS>,
}

impl<const BITS: usize, const LIMBS: usize> Rational<BITS, LIMBS> {
    pub fn reduce(&mut self) {
        let g = gcd(self.numerator, self.denominator);
        if g > Uint::ONE {
            self.numerator /= g;
            self.denominator /= g;
        }
    }
}
```

## Testing Patterns

### 12. Property-Based Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_gcd_properties(
            a: u128,
            b: u128,
        ) {
            let a = Uint::<256, 4>::from(a);
            let b = Uint::<256, 4>::from(b);
            let g = gcd(a, b);
            
            // GCD divides both inputs
            if !a.is_zero() {
                assert_eq!(a % g, Uint::ZERO);
            }
            if !b.is_zero() {
                assert_eq!(b % g, Uint::ZERO);
            }
            
            // GCD is commutative
            assert_eq!(g, gcd(b, a));
        }
    }
}
```

## Common Pitfalls and Solutions

### 13. Handling Edge Cases
```rust
// Always check for zero modulus in inv_mod
if modulus.is_zero() {
    return None;
}

// Handle equal inputs in extended GCD
if a == b {
    // Special case: gcd(a,a) = a
    return (a, Uint::ZERO, Uint::ONE, false);
}

// Be careful with sign handling in extended GCD
let (gcd, x, y, sign) = gcd_extended(a, b);
// The sign flag is critical for correct interpretation
```

### 14. Performance Considerations
- Lehmer's algorithm excels for large numbers (>128 bits)
- For small numbers, consider specialized paths
- Matrix operations have overhead; batch when possible
- The identity matrix check prevents unnecessary work

### 15. Numerical Stability
- All operations stay within type bounds
- No risk of overflow in standard operations
- Sign tracking in extended GCD is exact
- Modular reduction handles negative cofactors correctly