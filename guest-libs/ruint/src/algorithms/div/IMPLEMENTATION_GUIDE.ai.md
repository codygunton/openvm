# Ruint Division Algorithms - Implementation Guide

## Core Concepts

### Limb Representation
- Numbers stored as arrays of 64-bit limbs (little-endian)
- Example: 256-bit number = [u64; 4]
- Empty/zero handling via trimming operations

### Normalization
- Highest bit of divisor should be set for optimal algorithms
- Shift amount tracked for denormalization
- Critical for reciprocal accuracy

### Reciprocal-Based Division
- Replace division with multiplication by reciprocal
- Reciprocal: `v ≈ 2^k / d` for appropriate k
- Much faster than hardware division

## Implementation Details

### Entry Point Strategy
```rust
pub fn div(numerator: &mut [u64], divisor: &mut [u64]) {
    // 1. Trim zeros
    let divisor = trim_zeros(divisor);
    let numerator = trim_zeros(numerator);
    
    // 2. Handle trivial cases
    if numerator.len() < divisor.len() {
        // quotient = 0, remainder = numerator
        return;
    }
    
    // 3. Dispatch by size
    match divisor.len() {
        1 => single_limb_path(),
        2 => double_limb_path(),
        _ => knuth_division(),
    }
}
```

### Reciprocal Computation

#### 64-bit Reciprocal (MG10 Algorithm 3)
```rust
// Key steps:
1. Extract key bits: d0, d9, d40, d63
2. Table lookup for initial approximation
3. Newton-Raphson iterations:
   v1 = 2^11 * v0 - v0² * d40 / 2^40 - 1
   v2 = 2^13 * v1 + v1 * (2^60 - v1 * d40) / 2^47
4. Final correction step
```

#### 128-bit Reciprocal (MG10 Algorithm 6)
```rust
// Build on 64-bit reciprocal:
1. v = reciprocal(d_high)
2. Adjust for low limb contribution
3. Multiple correction steps for accuracy
```

### Small Division Algorithms

#### div_2x1 Implementation
```rust
pub fn div_2x1_mg10(u: u128, d: u64, v: u64) -> (u64, u64) {
    // Approximate quotient
    let q = u + (u >> 64) * u128::from(v);
    let q1 = ((q >> 64) as u64).wrapping_add(1);
    
    // Compute remainder
    let r = (u as u64).wrapping_sub(q1.wrapping_mul(d));
    
    // Correction steps (at most 2)
    // Handle cases where q is off by 1
}
```

#### div_nx1 with Normalization
```rust
pub fn div_nx1(limbs: &mut [u64], divisor: u64) -> u64 {
    let shift = divisor.leading_zeros();
    if shift == 0 {
        return div_nx1_normalized(limbs, divisor);
    }
    
    // Shift divisor and numerator
    let divisor = divisor << shift;
    let reciprocal = reciprocal(divisor);
    
    // Process limbs from high to low
    let mut remainder = limbs[n-1] >> (64 - shift);
    for i in (1..n).rev() {
        let u = shift_combine(limbs[i], limbs[i-1], shift);
        let (q, r) = div_2x1(u128::join(remainder, u), divisor, reciprocal);
        limbs[i] = q;
        remainder = r;
    }
    
    // Denormalize remainder
    remainder >> shift
}
```

### Knuth Division Algorithm

#### Key Innovation: 3x2 Division
```rust
// Instead of guessing quotient digit:
// Use div_3x2 to get accurate quotient from 3 limbs ÷ 2 limbs
let (q, r) = div_3x2(n21, n0, d, v);
```

#### Main Loop Structure
```rust
for j in (0..=m).rev() {
    // 1. Extract 3 limbs from numerator
    let n21 = u128::join(num[j+n], num[j+n-1]);
    let n0 = num[j+n-2];
    
    // 2. Compute quotient digit
    let (mut q, r) = div_3x2(n21, n0, d, v);
    
    // 3. Multiply and subtract
    let borrow = submul_nx1(&mut num[j..j+n-2], &div[..n-2], q);
    
    // 4. Handle correction (rare)
    if unlikely(borrow) {
        q -= 1;
        add_back_divisor();
    }
    
    // 5. Store quotient
    num[j+n] = q;
}
```

### Optimization Techniques

#### Unsafe Indexing
```rust
// Avoid bounds checks in hot loops
let upper = unsafe { limbs.get_unchecked(i) };
let lower = unsafe { limbs.get_unchecked(i - 1) };
```

#### Branch Prediction
```rust
if unlikely(overflow_condition) {
    // Rare path
} else {
    // Common path
}
```

#### In-Place Shifting
```rust
// Combine shift with division loop
let u = (upper << shift) | (lower >> (64 - shift));
```

## Common Pitfalls

### Normalization Errors
- Always check divisor highest bit
- Track shift amounts carefully
- Denormalize remainder correctly

### Overflow Handling
- Check for n21 == d case in div_3x2
- Handle quotient overflow (q = 2^64 - 1)
- Proper borrow propagation

### Index Management
- Careful with loop bounds
- Handle edge cases (n = m, etc.)
- Array aliasing considerations

## Testing Strategies

### Unit Test Pattern
```rust
#[test]
fn test_division_case() {
    // Setup
    let mut numerator = create_test_numerator();
    let mut divisor = create_test_divisor();
    
    // Execute
    div(&mut numerator, &mut divisor);
    
    // Verify invariant: n = q * d + r
    let mut check = multiply(numerator, original_divisor);
    add(&mut check, divisor); // remainder
    assert_eq!(check, original_numerator);
}
```

### Property Testing
```rust
proptest! {
    |(q: Vec<u64>, d: Vec<u64>, r: Vec<u64>)| {
        // Ensure r < d
        let r = ensure_valid_remainder(r, &d);
        
        // Compute n = q * d + r
        let n = compute_numerator(&q, &d, &r);
        
        // Test division
        test_division(n, d, q, r);
    }
}
```

## Performance Tuning

### Profile-Guided Optimizations
1. Measure hot paths with perf
2. Focus on inner loops
3. Minimize memory traffic
4. Optimize for common sizes

### Cache Considerations
- Keep working set in L1/L2
- Align data structures
- Minimize pointer chasing
- Consider prefetching

### SIMD Opportunities
- Parallel limb operations
- Vectorized multiply-subtract
- Batch remainder calculations
- Platform-specific intrinsics