# ruint Algorithms - Detailed Documentation

## Architecture Overview

The ruint algorithms module implements low-level arithmetic operations on arrays of 64-bit unsigned integers (limbs). These algorithms form the computational foundation for arbitrary-precision arithmetic in the ruint library.

### Design Principles

1. **Limb-Based Arithmetic**: All operations work on arrays of u64 limbs in little-endian order
2. **Carry/Borrow Propagation**: Explicit handling of carries and borrows between limbs
3. **Double-Word Operations**: Uses u128 type for intermediate calculations to handle overflow
4. **Performance Optimization**: Specialized implementations for common small sizes
5. **No Dynamic Allocation**: All algorithms work with pre-allocated arrays

## Core Components

### DoubleWord Trait

The `DoubleWord<T>` trait abstracts 128-bit arithmetic operations for u64 limbs:

```rust
trait DoubleWord<T>: Sized + Copy {
    fn join(high: T, low: T) -> Self;
    fn add(a: T, b: T) -> Self;
    fn mul(a: T, b: T) -> Self;
    fn muladd(a: T, b: T, c: T) -> Self;
    fn muladd2(a: T, b: T, c: T, d: T) -> Self;
    fn high(self) -> T;
    fn low(self) -> T;
    fn split(self) -> (T, T);
}
```

This trait is implemented for u128, providing efficient double-precision arithmetic.

### Addition Operations

#### `adc_n` - Add with Carry
- Adds two equal-length arrays with carry propagation
- Returns final carry out
- Time complexity: O(n) where n is array length

#### `sbb_n` - Subtract with Borrow
- Subtracts two equal-length arrays with borrow propagation
- Returns final borrow out
- Time complexity: O(n)

### Multiplication Operations

#### `addmul` - General Multiply-Accumulate
- Computes `result += a * b` with overflow detection
- Handles arbitrary-sized operands
- Uses schoolbook multiplication algorithm
- Optimizes by using shorter operand as outer loop
- Time complexity: O(n*m) where n, m are operand lengths

#### Specialized Multiplication Functions
- `mul_nx1`: Multiply array by single limb
- `addmul_nx1`: Multiply-accumulate array by single limb
- `submul_nx1`: Multiply-subtract array by single limb
- `addmul_n`: Fixed-size multiply-accumulate with specialized implementations for n=1,2,3,4

### Montgomery Multiplication

#### `mul_redc` - Montgomery Multiplication
Implements CIOS (Coarsely Integrated Operand Scanning) algorithm:
1. Computes `a * b * 2^(-BITS) mod modulus`
2. Requires inverse of `-modulus[0]` modulo 2^64
3. Inputs must be less than modulus
4. Constant-time when compiled with appropriate flags

#### `square_redc` - Montgomery Squaring
- Optimized version of `mul_redc` for squaring
- Exploits symmetry to reduce operations
- Approximately 2x faster than general multiplication

### Shift Operations

#### `shift_left_small` / `shift_right_small`
- Shifts array by less than 64 bits
- Returns overflow/underflow bits
- Efficient for bit-level adjustments

### Division Algorithms

The division submodule provides multiple algorithms:

1. **Knuth's Algorithm D**: General-purpose division for large operands
2. **Reciprocal-based**: Uses precomputed reciprocals for repeated divisions
3. **Small divisor**: Optimized for single-limb divisors

### GCD Algorithms

The GCD submodule implements:

1. **Binary GCD**: Efficient for similar-sized operands
2. **Extended GCD**: Computes Bézout coefficients
3. **Lehmer's Algorithm**: Uses matrix reduction for acceleration
4. **Modular Inverse**: Built on extended GCD

## Implementation Details

### Carry/Borrow Handling

The module provides helper functions for carry/borrow arithmetic:

```rust
pub const fn carrying_add(lhs: u64, rhs: u64, carry: bool) -> (u64, bool);
pub const fn borrowing_sub(lhs: u64, rhs: u64, borrow: bool) -> (u64, bool);
```

These compile to efficient CPU instructions on modern architectures.

### Comparison Function

The `cmp` function compares two u64 slices:
- Compares in reverse order (most significant limb first)
- Handles different-length slices correctly
- Optimized to eliminate bounds checks

### Performance Optimizations

1. **Loop Unrolling**: Fixed-size operations (1-4 limbs) have unrolled implementations
2. **Trimming**: Zero limbs are trimmed before multiplication
3. **Compiler Hints**: Uses `assume!` and `unreachable_unchecked` for optimization
4. **Intrinsics**: Leverages LLVM intrinsics for carry operations

## Usage Patterns

### Basic Arithmetic
```rust
// Addition with carry
let mut result = [0u64; 4];
let carry = adc_n(&mut result, &operand_a, &operand_b, 0);

// Multiplication
let overflow = addmul(&mut accumulator, &multiplicand, &multiplier);
```

### Montgomery Arithmetic
```rust
// Precompute inverse
let inv = compute_montgomery_inverse(modulus[0]);

// Montgomery multiplication
let product = mul_redc(a_mont, b_mont, modulus, inv);

// Montgomery squaring (more efficient)
let square = square_redc(a_mont, modulus, inv);
```

### Multi-Precision Division
```rust
// Divide with remainder
let quotient = div(&mut dividend, &mut divisor);
// dividend now contains remainder
```

## Testing Strategy

The module uses property-based testing with proptest:
- Verifies arithmetic properties (commutativity, associativity)
- Tests edge cases (zero operands, maximum values)
- Compares optimized implementations against reference implementations
- Ensures carry/borrow propagation correctness

## Security Considerations

1. **Timing Attacks**: Montgomery multiplication can be constant-time with proper compilation
2. **Side Channels**: Avoid data-dependent branches in cryptographic contexts
3. **Overflow Safety**: All operations explicitly handle overflow conditions
4. **Input Validation**: Functions assert preconditions in debug mode

## Future Improvements

Potential optimizations and enhancements:
1. SIMD implementations for parallel limb operations
2. Karatsuba multiplication for large operands
3. Barrett reduction as alternative to Montgomery
4. Hardware-specific optimizations (BMI2, ADX instructions)
5. Formal verification of critical algorithms