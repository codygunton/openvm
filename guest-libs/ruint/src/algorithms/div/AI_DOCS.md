# Ruint Division Algorithms - Comprehensive Documentation

## Table of Contents
1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Algorithm Details](#algorithm-details)
4. [Implementation Patterns](#implementation-patterns)
5. [Performance Optimization](#performance-optimization)
6. [Testing Strategy](#testing-strategy)

## Overview

The ruint division algorithms module provides high-performance division operations for arbitrary-precision unsigned integers. It implements both classical Knuth's Algorithm D and modern reciprocal-based methods from the MG10 paper, with careful optimization for different input sizes.

### Key Design Principles
- **Performance-first**: Optimized paths for common cases
- **In-place operations**: Minimize memory allocations
- **Reciprocal-based**: Use multiplication instead of division where possible
- **Normalization**: Ensure optimal bit alignment for algorithms
- **Safety**: Comprehensive debug assertions and overflow handling

## Architecture

### Module Structure
```
div/
├── mod.rs        # Main entry point and dispatch
├── small.rs      # Small divisor optimizations (1-3 limbs)
├── reciprocal.rs # Reciprocal computation algorithms
└── knuth.rs      # General n×m Knuth division
```

### Division Flow
1. **Entry**: `div()` function in `mod.rs`
2. **Trimming**: Remove leading zeros from inputs
3. **Dispatch**: Route to optimal algorithm based on size
4. **Computation**: Execute appropriate division algorithm
5. **Result**: Store quotient in numerator, remainder in divisor

## Algorithm Details

### Small Division Cases

#### div_2x1 (128-bit ÷ 64-bit)
```rust
// MG10 Algorithm 4
// Input: u (128-bit), d (64-bit, normalized), v (reciprocal)
// Output: (quotient, remainder)
```
- Uses precomputed reciprocal for multiplication-based division
- Requires divisor ≥ 2^63 (normalized)
- ~2.7ns performance vs 18ns for naive approach

#### div_3x2 (192-bit ÷ 128-bit)
```rust
// MG10 Algorithm 5
// Input: u21 (128-bit high), u0 (64-bit low), d (128-bit), v (reciprocal)
// Output: (quotient, remainder)
```
- Extension of div_2x1 for larger inputs
- Critical for Knuth algorithm's inner loop
- Handles special overflow cases

### Reciprocal Computation

#### reciprocal (64-bit)
```rust
// MG10 Algorithm 3
// Computes ⌊(2^128 - 1) / d⌋ - 2^64
```
- Uses lookup table for initial approximation
- Newton-Raphson refinement steps
- Optimized for normalized divisors

#### reciprocal_2 (128-bit)
```rust
// MG10 Algorithm 6
// Computes ⌊(2^192 - 1) / d⌋ - 2^64
```
- Builds on 64-bit reciprocal
- Used for div_3x2 operations

### Knuth Division (div_nxm)

Classical Algorithm D with modern optimizations:
1. **Normalization**: Shift inputs for optimal bit alignment
2. **Reciprocal**: Compute divisor reciprocal
3. **Loop**: Process quotient limbs from high to low
4. **3x2 Division**: Use div_3x2 for quotient estimation
5. **Correction**: Handle rare overestimation cases
6. **Denormalization**: Shift remainder back

## Implementation Patterns

### In-Place Operations
```rust
// Results stored in input arrays to avoid allocation
pub fn div(numerator: &mut [u64], divisor: &mut [u64]) {
    // Quotient → numerator
    // Remainder → divisor
}
```

### Normalization Handling
```rust
// Ensure highest bit set for optimal algorithms
let shift = divisor.leading_zeros();
if shift == 0 {
    div_nx1_normalized(limbs, divisor)
} else {
    // Shift and process with reciprocal
}
```

### Overflow Detection
```rust
// Special handling for quotient overflow
if unlikely(n21 == d) {
    let q = u64::MAX;
    // Special overflow path
}
```

## Performance Optimization

### Branch Prediction
- `likely()` and `unlikely()` hints for hot paths
- Overflow cases marked as unlikely
- Common paths optimized for prediction

### Loop Unrolling
- Manual bounds checking elimination with unsafe
- Careful indexing to avoid redundant checks
- Optimal memory access patterns

### Reciprocal Caching
- Compute reciprocal once per division
- Reuse across all quotient limb calculations
- Significant speedup for large divisions

### Algorithm Selection
```rust
match divisor.len() {
    1 => div_nx1(numerator, divisor[0]),
    2 => div_nx2(numerator, u128::from_limbs(divisor)),
    _ => div_nxm(numerator, divisor),
}
```

## Testing Strategy

### Unit Tests
- Fixed test vectors from intx library
- Edge cases (overflow, normalization)
- Rollback scenarios for correction paths

### Property-Based Testing
```rust
proptest! {
    // Verify: n = q * d + r, where r < d
    |(quotient, divisor, remainder)| {
        let mut numerator = compute_product(quotient, divisor);
        add_remainder(&mut numerator, remainder);
        div(&mut numerator, &mut divisor);
        assert_eq!(numerator, quotient);
        assert_eq!(divisor, remainder);
    }
}
```

### Coverage Focus
- All algorithm paths (normalized/unnormalized)
- Overflow conditions
- Size variations (1-10 limbs)
- Correction branch coverage

## Usage Examples

### Basic Division
```rust
let mut numerator = [/* 256-bit value */];
let mut divisor = [/* 128-bit value */];
div(&mut numerator, &mut divisor);
// numerator now contains quotient
// divisor now contains remainder
```

### With Normalization
```rust
// For optimal performance with known normalized divisor
div_nx1_normalized(&mut numerator, normalized_divisor);
```

### Small Divisor Optimization
```rust
// Single limb divisor - uses optimized path
let remainder = div_nx1(&mut numerator, single_limb_divisor);
```

## References
- [MG10]: Modern Computer Arithmetic (Granlund & Montgomery, 2010)
- [K97]: The Art of Computer Programming Vol. 2 (Knuth, 1997)
- [intx]: C++ implementation reference