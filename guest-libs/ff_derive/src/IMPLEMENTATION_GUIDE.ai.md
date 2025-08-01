# ff_derive Implementation Guide

## Overview

The `openvm-ff-derive` crate implements a procedural macro that generates finite field arithmetic implementations with dual-target support (standard Rust and zkVM). This guide explains the implementation details and how to extend or modify the crate.

## Core Implementation Flow

### 1. Macro Processing (`openvm_prime_field`)

The macro processes a struct definition with attributes:

```rust
#[proc_macro_attribute]
pub fn openvm_prime_field(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // 1. Parse struct
    // 2. Extract attributes (modulus, generator, endianness)
    // 3. Validate struct format
    // 4. Generate implementations
    // 5. Return modified token stream
}
```

### 2. Struct Validation

The macro expects:
- Single unnamed field: `struct Name([u64; N])`
- Field must not be public
- Array size must match computed limb count

```rust
fn validate_struct(ast: &syn::ItemStruct, limbs: usize) -> Option<proc_macro2::TokenStream>
```

### 3. Limb Calculation

Determines required 64-bit limbs:
```rust
let mut limbs = 1;
let mod2 = (&modulus) << 1; // modulus * 2
let mut cur = BigUint::one() << 64;
while cur < mod2 {
    limbs += 1;
    cur <<= 64;
}
```

### 4. zkVM Memory Layout

Selects appropriate byte representation:
```rust
let zkvm_limbs = if bytes <= 32 {
    32
} else if bytes <= 48 {
    48
} else {
    // Error: modulus too large
};
```

## Code Generation Details

### 1. Dual Implementation Pattern

The macro generates conditional compilation:

```rust
#[cfg(target_os = "zkvm")]
    // zkVM implementation using openvm_algebra_guest
#[cfg(not(target_os = "zkvm"))]
    // Standard implementation using Montgomery form
```

### 2. Constant Generation

Computes field constants at compile time:
- `R = 2^(64*limbs) mod p` (Montgomery parameter)
- `R2 = R^2 mod p` (Montgomery squaring constant)
- `INV = -p^(-1) mod 2^64` (Montgomery reduction constant)
- `TWO_INV = 2^(-1) mod p`
- `ROOT_OF_UNITY` (2^s root of unity)
- Generator and related constants

### 3. Montgomery Arithmetic

Standard environment uses Montgomery form:
```rust
fn mont_reduce(&mut self, r0: u64, mut r1: u64, ...) {
    // Algorithm 14.32 from Handbook of Applied Cryptography
    for i in 0..limbs {
        let k = r[i].wrapping_mul(INV);
        // Reduce using modulus
    }
}
```

### 4. zkVM Integration

zkVM target delegates to `openvm_algebra_guest`:
```rust
#[cfg(target_os = "zkvm")]
{
    <Self as ::openvm_algebra_guest::IntMod>::add(self, other)
}
```

## Key Algorithms

### 1. Square Root (Tonelli-Shanks)

For primes where `p ≡ 1 (mod 4)`:
```rust
// Find w = self^((t-1)/2)
// Iterate to find square root
// Uses precomputed 2^S root of unity
```

For `p ≡ 3 (mod 4)`, uses simpler formula:
```rust
sqrt = self^((p+1)/4)
```

### 2. Addition Chains (pow_fixed.rs)

Generates optimal exponentiation sequences:
```rust
pub fn generate(base: &TokenStream, exponent: BigUint) -> TokenStream {
    let steps = build_addition_chain(exponent);
    // Generate doubling and addition operations
}
```

### 3. Field Element Ordering

Implements lexicographic ordering:
- zkVM: Direct byte comparison after reduction
- Standard: Montgomery reduction before comparison

## Extension Points

### 1. Adding New Field Sizes

To support larger moduli:
1. Update zkVM limb selection logic
2. Adjust memory layout constants
3. Update validation logic

### 2. Custom Operations

To add new operations:
1. Add method to generated impl block
2. Provide dual implementations (zkVM/standard)
3. Ensure constant-time properties if needed

### 3. Optimization Opportunities

- Implement specialized squaring for specific moduli
- Add SIMD optimizations for standard target
- Optimize addition chains for common exponents

## Common Patterns

### 1. Conditional Selection

```rust
fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
    #[cfg(target_os = "zkvm")]
    {
        // Byte-wise selection
    }
    #[cfg(not(target_os = "zkvm"))]
    {
        // Limb-wise selection
    }
}
```

### 2. Reduction Checking

zkVM uses constant-time reduction check:
```rust
fn constant_time_is_reduced(&self) -> Choice {
    // Compare against modulus limb by limb
}
```

### 3. Conversion Between Representations

```rust
fn from_repr(r: Repr) -> CtOption<Self> {
    #[cfg(target_os = "zkvm")]
        // Direct byte interpretation
    #[cfg(not(target_os = "zkvm"))]
        // Montgomery conversion
}
```

## Testing Considerations

1. Test both compilation targets
2. Verify constant generation
3. Check edge cases (zero, one, p-1)
4. Validate against known test vectors
5. Ensure zkVM compatibility