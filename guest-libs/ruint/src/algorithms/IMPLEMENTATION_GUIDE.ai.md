# ruint Algorithms - Implementation Guide

## Overview

This guide provides implementation guidance for working with and extending the ruint algorithms module. It covers best practices, common patterns, and detailed examples for implementing new algorithms or modifying existing ones.

## Core Concepts

### Limb Representation

All algorithms operate on arrays of u64 limbs in little-endian order:
- Least significant limb at index 0
- Most significant limb at highest index
- Empty arrays represent zero

Example:
```rust
// Number: 0x1234567890ABCDEF_FEDCBA0987654321
// Limbs: [0xFEDCBA0987654321, 0x1234567890ABCDEF]
```

### Carry/Borrow Propagation

When implementing arithmetic operations, always propagate carries/borrows:

```rust
pub fn add_with_carry(a: &mut [u64], b: &[u64]) -> u64 {
    let mut carry = 0u64;
    for i in 0..a.len() {
        let (sum, new_carry) = carrying_add(a[i], b[i], carry != 0);
        a[i] = sum;
        carry = new_carry as u64;
    }
    carry
}
```

### Using DoubleWord Operations

For multiplication and mixed operations, use the DoubleWord trait:

```rust
use super::DoubleWord;

fn multiply_and_add(a: u64, b: u64, c: u64) -> (u64, u64) {
    let result = u128::muladd(a, b, c);
    result.split() // Returns (low, high)
}
```

## Implementation Patterns

### Pattern 1: Fixed-Size Specialization

For performance-critical operations, provide specialized implementations for common sizes:

```rust
pub fn operation_n(a: &mut [u64], b: &[u64]) {
    match a.len() {
        0 => {},
        1 => operation_1(a, b),
        2 => operation_2(a, b),
        3 => operation_3(a, b),
        4 => operation_4(a, b),
        _ => operation_general(a, b),
    }
}

#[inline(always)]
fn operation_1(a: &mut [u64], b: &[u64]) {
    assume!(a.len() == 1);
    assume!(b.len() == 1);
    // Optimized implementation for single limb
}
```

### Pattern 2: Zero Trimming

Remove leading/trailing zeros before processing:

```rust
pub fn process(mut a: &[u64], mut b: &[u64]) {
    // Trim leading zeros
    while let [0, rest @ ..] = a {
        a = rest;
    }
    while let [0, rest @ ..] = b {
        b = rest;
    }
    
    // Trim trailing zeros
    while let [rest @ .., 0] = a {
        a = rest;
    }
    while let [rest @ .., 0] = b {
        b = rest;
    }
    
    if a.is_empty() || b.is_empty() {
        return; // Handle zero case
    }
}
```

### Pattern 3: Overflow Detection

Always handle and report overflow conditions:

```rust
pub fn checked_operation(result: &mut [u64], a: &[u64], b: &[u64]) -> bool {
    let mut overflow = false;
    
    // Perform operation
    for i in 0..result.len() {
        // ... operation logic ...
        if carry != 0 && i == result.len() - 1 {
            overflow = true;
        }
    }
    
    overflow
}
```

## Algorithm Implementation Examples

### Example 1: Implementing Addition

```rust
pub fn add_assign(lhs: &mut [u64], rhs: &[u64]) -> bool {
    let mut carry = 0u64;
    let mut overflow = false;
    
    for i in 0..lhs.len() {
        if i < rhs.len() {
            (lhs[i], carry) = adc(lhs[i], rhs[i], carry);
        } else if carry != 0 {
            (lhs[i], carry) = adc(lhs[i], 0, carry);
        } else {
            break;
        }
    }
    
    // Check for overflow
    if carry != 0 {
        overflow = true;
    }
    
    // Check if there are remaining limbs in rhs
    if rhs.len() > lhs.len() {
        for &limb in &rhs[lhs.len()..] {
            if limb != 0 {
                overflow = true;
                break;
            }
        }
    }
    
    overflow
}
```

### Example 2: Implementing Modular Reduction

```rust
pub fn reduce_barrett(value: &mut [u64], modulus: &[u64], mu: &[u64]) {
    // Barrett reduction: value = value mod modulus
    // mu = floor(2^(2k) / modulus) where k = bit length of modulus
    
    let k = modulus.len();
    if value.len() <= k {
        return; // Already reduced
    }
    
    // Step 1: q = floor(value / 2^(k-1))
    let q_start = k.saturating_sub(1);
    let q = &value[q_start..];
    
    // Step 2: q = q * mu
    let mut q_mu = vec![0u64; q.len() + mu.len()];
    addmul(&mut q_mu, q, mu);
    
    // Step 3: q = floor(q / 2^(k+1))
    let q = &q_mu[k + 1..];
    
    // Step 4: r = value - q * modulus
    let mut q_mod = vec![0u64; k + 1];
    addmul(&mut q_mod, q, modulus);
    
    // Subtract from value
    let borrow = sbb_n(&mut value[..q_mod.len()], &q_mod, 0);
    
    // At most two subtractions of modulus needed
    while cmp(&value[..k], modulus) >= Ordering::Equal {
        sbb_n(&mut value[..k], modulus, 0);
    }
}
```

### Example 3: Implementing Karatsuba Multiplication

```rust
pub fn mul_karatsuba(result: &mut [u64], a: &[u64], b: &[u64]) {
    const KARATSUBA_THRESHOLD: usize = 32;
    
    if a.len() < KARATSUBA_THRESHOLD || b.len() < KARATSUBA_THRESHOLD {
        // Use schoolbook multiplication for small inputs
        addmul(result, a, b);
        return;
    }
    
    // Split inputs
    let split = a.len().min(b.len()) / 2;
    let (a_lo, a_hi) = a.split_at(split);
    let (b_lo, b_hi) = b.split_at(split);
    
    // Allocate temporary storage
    let mut z0 = vec![0u64; 2 * split];
    let mut z1 = vec![0u64; 2 * split + 2];
    let mut z2 = vec![0u64; 2 * (a.len() - split).max(b.len() - split)];
    
    // z0 = a_lo * b_lo
    mul_karatsuba(&mut z0, a_lo, b_lo);
    
    // z2 = a_hi * b_hi
    mul_karatsuba(&mut z2, a_hi, b_hi);
    
    // z1 = (a_lo + a_hi) * (b_lo + b_hi) - z0 - z2
    let mut a_sum = vec![0u64; split + 1];
    let mut b_sum = vec![0u64; split + 1];
    a_sum[..a_lo.len()].copy_from_slice(a_lo);
    b_sum[..b_lo.len()].copy_from_slice(b_lo);
    let a_carry = add_nx1(&mut a_sum[..a_hi.len()], a_hi);
    let b_carry = add_nx1(&mut b_sum[..b_hi.len()], b_hi);
    
    mul_karatsuba(&mut z1, &a_sum, &b_sum);
    sbb_n(&mut z1[..z0.len()], &z0, 0);
    sbb_n(&mut z1[..z2.len()], &z2, 0);
    
    // Combine results
    add_nx1(&mut result[..z0.len()], &z0);
    add_nx1(&mut result[split..][..z1.len()], &z1);
    add_nx1(&mut result[2 * split..][..z2.len()], &z2);
}
```

## Performance Optimization Guidelines

### 1. Use Compiler Intrinsics

Leverage intrinsics for better performance:

```rust
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{_addcarry_u64, _subborrow_u64};

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn adc_intrinsic(a: u64, b: u64, carry: u8) -> (u64, u8) {
    unsafe {
        let mut result = 0;
        let carry_out = _addcarry_u64(carry, a, b, &mut result);
        (result, carry_out)
    }
}
```

### 2. Minimize Allocations

Pre-allocate buffers and reuse them:

```rust
pub struct MulContext {
    temp_buffer: Vec<u64>,
}

impl MulContext {
    pub fn multiply(&mut self, result: &mut [u64], a: &[u64], b: &[u64]) {
        // Reuse temp_buffer instead of allocating
        self.temp_buffer.clear();
        self.temp_buffer.resize(a.len() + b.len(), 0);
        // ... use temp_buffer ...
    }
}
```

### 3. Branch Prediction

Structure code to help branch prediction:

```rust
// Good: Predictable branch
for i in 0..n {
    if i < threshold {
        // Fast path
    } else {
        // Slow path
    }
}

// Better: Branchless when possible
for i in 0..n {
    let mask = ((i < threshold) as u64).wrapping_neg();
    result[i] = (fast_value & mask) | (slow_value & !mask);
}
```

## Testing Guidelines

### 1. Property-Based Testing

Use proptest for thorough testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{proptest, prop_assert_eq};
    
    proptest! {
        #[test]
        fn test_addition_commutative(a: Vec<u64>, b: Vec<u64>) {
            let mut result1 = vec![0; a.len().max(b.len()) + 1];
            let mut result2 = vec![0; a.len().max(b.len()) + 1];
            
            add(&mut result1, &a, &b);
            add(&mut result2, &b, &a);
            
            prop_assert_eq!(result1, result2);
        }
    }
}
```

### 2. Edge Case Testing

Always test edge cases:

```rust
#[test]
fn test_edge_cases() {
    // Empty arrays
    test_operation(&[], &[], &[]);
    
    // Single limb
    test_operation(&[u64::MAX], &[1], &[0, 1]);
    
    // Overflow cases
    test_operation(&[u64::MAX; 4], &[1], expected_with_overflow);
    
    // Power of two boundaries
    for i in 0..64 {
        test_operation(&[1u64 << i], &[1u64 << i], expected);
    }
}
```

### 3. Fuzzing

Consider fuzzing for security-critical algorithms:

```rust
#[cfg(fuzzing)]
pub fn fuzz_target(data: &[u8]) {
    if data.len() < 16 {
        return;
    }
    
    let (a_len, rest) = data.split_at(8);
    let (b_len, rest) = rest.split_at(8);
    
    let a_len = usize::from_le_bytes(a_len.try_into().unwrap()) % 100;
    let b_len = usize::from_le_bytes(b_len.try_into().unwrap()) % 100;
    
    // Fuzz the operation
    // ...
}
```

## Common Pitfalls

### 1. Incorrect Carry Handling

Always propagate carries through the entire operation:

```rust
// Wrong: Stops at first zero carry
for i in 0..n {
    (result[i], carry) = adc(a[i], b[i], carry);
    if carry == 0 { break; } // Bug!
}

// Correct: Continues until no carry
for i in 0..n {
    if carry == 0 && i >= b.len() { break; }
    let b_limb = if i < b.len() { b[i] } else { 0 };
    (result[i], carry) = adc(a[i], b_limb, carry);
}
```

### 2. Bounds Checking

Use slicing to enable bounds check elimination:

```rust
// Suboptimal: Bounds checks in loop
for i in 0..n {
    result[i] = a[i] + b[i];
}

// Better: Bounds checks eliminated
let n = a.len().min(b.len()).min(result.len());
let (a, b, result) = (&a[..n], &b[..n], &mut result[..n]);
for i in 0..n {
    result[i] = a[i] + b[i];
}
```

### 3. Alignment Issues

Ensure proper alignment for SIMD operations:

```rust
#[repr(align(32))]
struct AlignedBuffer([u64; 64]);

// Use aligned buffers for SIMD
let mut buffer = AlignedBuffer([0; 64]);
```

## Integration with ruint

When implementing new algorithms, ensure compatibility with the ruint API:

```rust
use crate::Uint;

impl<const BITS: usize, const LIMBS: usize> Uint<BITS, LIMBS> {
    pub fn custom_operation(&self, other: &Self) -> Self {
        let mut result = Self::ZERO;
        
        // Use the algorithm module
        custom_algorithm(result.as_limbs_mut(), self.as_limbs(), other.as_limbs());
        
        result
    }
}
```