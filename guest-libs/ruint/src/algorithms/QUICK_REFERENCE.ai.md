# ruint Algorithms - Quick Reference

## Core Types and Traits

```rust
// Double-word arithmetic trait
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

## Basic Arithmetic Operations

### Addition and Subtraction

```rust
// Add with carry propagation
pub fn adc_n(lhs: &mut [u64], rhs: &[u64], carry: u64) -> u64

// Subtract with borrow propagation  
pub fn sbb_n(lhs: &mut [u64], rhs: &[u64], borrow: u64) -> u64

// Single limb add with carry
pub fn adc(lhs: u64, rhs: u64, carry: u64) -> (u64, u64)

// Single limb subtract with borrow
pub fn sbb(lhs: u64, rhs: u64, borrow: u64) -> (u64, u64)

// Helper functions
pub const fn carrying_add(lhs: u64, rhs: u64, carry: bool) -> (u64, bool)
pub const fn borrowing_sub(lhs: u64, rhs: u64, borrow: bool) -> (u64, bool)
```

### Multiplication

```rust
// General multiply-accumulate with overflow detection
pub fn addmul(lhs: &mut [u64], a: &[u64], b: &[u64]) -> bool

// Fixed-size multiply-accumulate (n = array length)
pub fn addmul_n(lhs: &mut [u64], a: &[u64], b: &[u64])

// Multiply array by single limb
pub fn mul_nx1(lhs: &mut [u64], a: u64) -> u64

// Multiply-accumulate array by single limb
pub fn addmul_nx1(lhs: &mut [u64], a: &[u64], b: u64) -> u64

// Multiply-subtract array by single limb
pub fn submul_nx1(lhs: &mut [u64], a: &[u64], b: u64) -> u64

// Add single limb with carry propagation
pub fn add_nx1(lhs: &mut [u64], a: u64) -> u64
```

### Montgomery Arithmetic

```rust
// Montgomery multiplication: a * b * 2^(-BITS) mod modulus
pub fn mul_redc<const N: usize>(
    a: [u64; N], 
    b: [u64; N], 
    modulus: [u64; N], 
    inv: u64
) -> [u64; N]

// Montgomery squaring: a^2 * 2^(-BITS) mod modulus
pub fn square_redc<const N: usize>(
    a: [u64; N], 
    modulus: [u64; N], 
    inv: u64
) -> [u64; N]
```

### Bit Shifting

```rust
// Left shift by < 64 bits, returns overflow
pub fn shift_left_small(limbs: &mut [u64], amount: usize) -> u64

// Right shift by < 64 bits, returns underflow
pub fn shift_right_small(limbs: &mut [u64], amount: usize) -> u64
```

### Comparison

```rust
// Compare two u64 slices (big-endian semantics)
pub fn cmp(left: &[u64], right: &[u64]) -> Ordering
```

## Division Operations

```rust
// Main division interface (from div module)
pub fn div(dividend: &mut [u64], divisor: &mut [u64]) -> Option<()>

// Specialized division algorithms:
// - Knuth's Algorithm D for general division
// - Reciprocal-based division for repeated operations
// - Small divisor optimizations
```

## GCD Operations

```rust
// Greatest Common Divisor
pub fn gcd(a: &mut [u64], b: &mut [u64]) -> Vec<u64>

// Extended GCD (returns gcd and Bézout coefficients)
pub fn gcd_extended(
    a: &mut [u64], 
    b: &mut [u64]
) -> (Vec<u64>, Vec<u64>, Vec<u64>, bool, bool)

// Modular inverse
pub fn inv_mod(value: &mut [u64], modulus: &mut [u64]) -> Option<Vec<u64>>

// Lehmer matrix for GCD acceleration
pub struct LehmerMatrix {
    pub a00: u64,
    pub a01: u64, 
    pub a10: u64,
    pub a11: u64,
}
```

## Usage Examples

### Basic Addition
```rust
let mut result = [0u64; 4];
let carry = adc_n(&mut result, &[1, 2, 3, 4], &[5, 6, 7, 8], 0);
// result = [6, 8, 10, 12], carry = 0
```

### Multiplication
```rust
let mut accumulator = [0u64; 8];
let overflow = addmul(&mut accumulator, &[u64::MAX, u64::MAX], &[2, 0]);
// accumulator = [u64::MAX - 1, u64::MAX, 1, 0, ...], overflow = false
```

### Montgomery Multiplication
```rust
// Precompute Montgomery inverse
let modulus = [0x1234567890ABCDEF_u64, 0xFEDCBA0987654321];
let inv = (!modulus[0]).wrapping_add(1); // -modulus[0]^(-1) mod 2^64

// Perform Montgomery multiplication
let a_mont = [/* Montgomery form of a */];
let b_mont = [/* Montgomery form of b */];
let result = mul_redc(a_mont, b_mont, modulus, inv);
```

### Bit Shifting
```rust
let mut value = [0x8000000000000000_u64, 0x1];
let overflow = shift_left_small(&mut value, 1);
// value = [0, 3], overflow = 0
```

### Division with Remainder
```rust
let mut dividend = [100, 0, 0, 0];
let mut divisor = [7, 0, 0, 0];
div(&mut dividend, &mut divisor);
// dividend now contains remainder [2, 0, 0, 0]
// divisor unchanged
```

## Performance Notes

- **Fixed-size operations**: Use `addmul_n` for known sizes (1-4 limbs) for better performance
- **Montgomery arithmetic**: Prefer `square_redc` over `mul_redc` for squaring (approximately 2x faster)
- **Zero handling**: Functions automatically trim zeros for efficiency
- **Carry propagation**: All operations properly handle carry/borrow chains

## Common Patterns

### Checking for overflow
```rust
if addmul(&mut result, &a, &b) {
    // Handle overflow case
}
```

### Modular arithmetic setup
```rust
// Convert to Montgomery form
fn to_montgomery<const N: usize>(value: [u64; N], modulus: [u64; N], r2: [u64; N], inv: u64) -> [u64; N] {
    mul_redc(value, r2, modulus, inv)
}

// Convert from Montgomery form  
fn from_montgomery<const N: usize>(value: [u64; N], modulus: [u64; N], inv: u64) -> [u64; N] {
    mul_redc(value, [1, 0, /* ... */], modulus, inv)
}
```

### Working with variable-size integers
```rust
// Trim zeros before operations
fn trim_zeros(mut slice: &[u64]) -> &[u64] {
    while let [rest @ .., 0] = slice {
        slice = rest;
    }
    slice
}

// Process with trimmed inputs
let a = trim_zeros(&input_a);
let b = trim_zeros(&input_b);
let mut result = vec![0; a.len() + b.len()];
addmul(&mut result, a, b);
```

## Error Handling

Most functions in this module don't return errors but instead:
- Return overflow/underflow indicators (carry/borrow)
- Use debug assertions for preconditions
- Assume valid input in release mode for performance

Example precondition checks:
```rust
debug_assert_eq!(lhs.len(), rhs.len()); // Equal length required
debug_assert!(amount < 64);              // Shift amount bounds
debug_assert_eq!(cmp(&a, &modulus), Ordering::Less); // Montgomery input bounds
```