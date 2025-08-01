# OpenVM Algebra Guest Implementation Guide

## Overview

This guide provides detailed implementation information for the OpenVM algebra guest component, including modular arithmetic strategies, field operation optimizations, and zkVM-specific considerations.

## Modular Arithmetic Implementation

### IntMod Trait Design

The `IntMod` trait provides a comprehensive interface for modular integers:

```rust
pub trait IntMod: 
    // Standard operations
    Sized + Eq + Clone + Debug +
    // Arithmetic with Output = Self
    Neg + Add + Sub + Mul + DivUnsafe +
    // Iterator support
    Sum + Product +
    // Reference operations
    for<'a> Add<&'a Self, Output = Self> +
    for<'a> Sub<&'a Self, Output = Self> +
    // Assignment operations
    AddAssign + SubAssign + MulAssign + DivAssignUnsafe
{
    type Repr: AsRef<[u8]> + AsMut<[u8]>;
    type SelfRef<'a>: /* binary ops with &'a Self */;
    
    const MODULUS: Self::Repr;
    const NUM_LIMBS: usize;
    const ZERO: Self;
    const ONE: Self;
}
```

Key design decisions:
1. **Generic over representation**: `Repr` allows different byte array sizes
2. **Reference arithmetic**: Avoids unnecessary clones in expressions
3. **Iterator traits**: Enables `sum()` and `product()` on collections
4. **Assignment variants**: For in-place operations

### Hardware Instruction Integration

Custom RISC-V instructions accelerate modular operations:

```rust
// Instruction encoding
pub const OPCODE: u8 = 0x2b;  // custom-1

// Modular arithmetic encoding
pub fn encode_mod_arith(mod_idx: u8, op: ModArithBaseFunct7) -> u8 {
    mod_idx * ModArithBaseFunct7::MODULAR_ARITHMETIC_MAX_KINDS + op as u8
}

// Usage in implementation (pseudo-code)
#[cfg(target_os = "zkvm")]
fn add_mod_impl(a: &Self, b: &Self, mod_idx: u8) -> Self {
    let funct7 = encode_mod_arith(mod_idx, ModArithBaseFunct7::AddMod);
    // Execute custom instruction with funct7, funct3=0b000
    custom_insn_r!(OPCODE, 0b000, funct7, a, b)
}
```

### Modulus Setup

Each modulus must be registered before use:

```rust
impl IntMod for MyModularType {
    fn set_up_once() {
        use once_cell::sync::OnceBool;
        static INIT: OnceBool = OnceBool::new();
        
        INIT.get_or_init(|| {
            #[cfg(target_os = "zkvm")]
            {
                // Register modulus with VM
                let funct7 = encode_mod_arith(
                    Self::MOD_IDX,
                    ModArithBaseFunct7::SetupMod
                );
                // Execute setup instruction
                setup_modulus(&Self::MODULUS, funct7);
            }
            true
        });
    }
}
```

### Representation and Canonicalization

The VM allows non-canonical representations for performance:

```rust
// Non-canonical allowed during computation
let a = MyMod::from_repr([0xFF; 32]); // May be > modulus

// Force canonical form when needed
a.assert_reduced(); // Panics if not reduced

// Check without panic
if a.is_reduced() {
    // a < modulus guaranteed
}

// Equality handles non-canonical
let b = MyMod::from_u32(5);
let c = b + MyMod::MODULUS; // c = 5 (mod p) but repr differs
assert!(b == c); // true - compares modulo p
```

## Field Operations Implementation

### Field Trait Simplification

The `Field` trait is simpler than `IntMod`, focusing on field properties:

```rust
impl<T: IntMod> Field for T 
where 
    T::Modulus: PrimeModulus, // Compile-time check
{
    fn invert(&self) -> Self {
        // For prime fields, Fermat's little theorem:
        // a^(p-1) ≡ 1 (mod p), so a^(p-2) ≡ a^(-1)
        self.exp_bytes(true, &(Self::MODULUS - 2).to_be_bytes())
    }
}
```

### Optimized Operations

Common operations have specialized implementations:

```rust
fn double_assign(&mut self) {
    // More efficient than self + self
    *self = self.add_ref::<false>(self);
}

fn square_assign(&mut self) {
    // Uses MulMod instruction but may have optimizations
    #[cfg(target_os = "zkvm")]
    {
        let funct7 = encode_mod_arith(Self::MOD_IDX, ModArithBaseFunct7::MulMod);
        *self = custom_insn_r!(OPCODE, 0b000, funct7, self, self);
    }
}
```

## Extension Field Implementation

### Complex Extension Fields

For quadratic extensions (e.g., Fp2 over Fp):

```rust
#[derive(Clone, Debug)]
struct Fp2<F: Field> {
    c0: F,  // Real part
    c1: F,  // Imaginary part
}

impl<F: Field> FieldExtension<F> for Fp2<F> {
    const D: usize = 2;
    type Coeffs = [F; 2];
    
    fn from_coeffs(coeffs: Self::Coeffs) -> Self {
        Fp2 { c0: coeffs[0], c1: coeffs[1] }
    }
    
    fn mul_base(&self, rhs: &F) -> Self {
        // Scalar multiplication
        Fp2 {
            c0: self.c0 * rhs,
            c1: self.c1 * rhs,
        }
    }
    
    fn frobenius_map(&self, power: usize) -> Self {
        // For Fp2, Frob(a + bi) = a - bi when power is odd
        if power % 2 == 0 {
            self.clone()
        } else {
            self.conjugate()
        }
    }
}
```

### Hardware Acceleration for Extensions

Complex field operations use separate instructions:

```rust
#[cfg(target_os = "zkvm")]
fn fp2_mul(a: &Fp2, b: &Fp2, fp2_idx: u8) -> Fp2 {
    let funct7 = fp2_idx * 8 + ComplexExtFieldBaseFunct7::Mul as u8;
    custom_insn_r!(OPCODE, 0b010, funct7, a, b)
}
```

## Exponentiation Implementation

### Window-Based Algorithm

The `exp_bytes` implementation uses 2-bit windows:

```rust
fn exp_bytes(&self, is_positive: bool, bytes_be: &[u8]) -> Self {
    let mut x = if is_positive { 
        self.clone() 
    } else { 
        self.invert() 
    };
    
    let mut res = Self::ONE;
    
    // Precompute x^1, x^2, x^3
    let x_sq = &x * &x;
    let ops = [x.clone(), x_sq.clone(), &x_sq * &x];
    
    // Process 2 bits at a time
    for &b in bytes_be.iter() {
        let mut mask = 0xc0; // 11000000
        for j in 0..4 {
            // Square 4 times (process 2 bits)
            res = &res * &res * &res * &res;
            
            // Extract 2-bit window
            let c = (b & mask) >> (6 - 2 * j);
            if c != 0 {
                res *= &ops[(c - 1) as usize];
            }
            mask >>= 2;
        }
    }
    res
}
```

Optimization rationale:
- 2-bit windows balance precomputation vs multiplications
- 4 squarings per window minimizes total operations
- Precomputed powers avoid redundant multiplications

## Performance Considerations

### Instruction Latency

Custom instructions have different latencies:
- AddMod/SubMod: ~3 cycles
- MulMod: ~10 cycles  
- DivMod: ~50 cycles (iterative)
- Setup: One-time cost

### Memory Layout

Modular integers are typically stored as:
```rust
#[repr(transparent)]
struct Fp([u8; 32]);  // For 256-bit primes

// Alignment matters for custom instructions
#[repr(C, align(4))]
struct AlignedFp([u8; 32]);
```

### Optimization Strategies

1. **Batch operations**: Group similar operations
2. **Avoid reductions**: Work with non-canonical values
3. **Reuse temporaries**: Assignment operations
4. **Precomputation**: For fixed bases/exponents

## zkVM-Specific Considerations

### Deterministic Execution

All operations must be deterministic:
```rust
// Bad: Non-deterministic based on representation
fn bad_is_zero(&self) -> bool {
    self.as_bytes() == &[0; 32]  // Fails for non-canonical zero
}

// Good: Handles non-canonical
fn good_is_zero(&self) -> bool {
    self == &Self::ZERO  // Uses modular equality
}
```

### Constraint Generation

Operations generate different constraints:
- Add/Sub: Linear constraints
- Mul: Quadratic constraints
- Div: Inverse + multiplication
- Setup: Registers modulus globally

### Memory Access Patterns

Custom instructions expect aligned access:
```rust
// Ensure alignment for performance
#[repr(align(4))]
struct ModularElement([u8; 32]);

// Stack allocations are automatically aligned
let a = ModularElement([0; 32]);  // OK

// Heap requires care
let vec: Vec<ModularElement> = Vec::with_capacity(100);
// Vec guarantees alignment based on type
```

## Common Patterns

### Multi-Exponentiation

```rust
fn multi_exp<F: Field>(bases: &[F], exps: &[Vec<u8>]) -> F {
    bases.iter()
        .zip(exps)
        .map(|(base, exp)| base.exp_bytes(true, exp))
        .product()
}
```

### Batch Inversion

```rust
fn batch_invert<F: Field>(elements: &[F]) -> Vec<F> {
    let mut products = vec![F::ONE; elements.len()];
    
    // Forward pass: compute cumulative products
    for i in 1..elements.len() {
        products[i] = products[i-1] * elements[i-1];
    }
    
    // Compute inverse of final product
    let mut inv = (products.last().unwrap() * elements.last().unwrap())
        .invert();
    
    // Backward pass: extract individual inverses
    let mut results = vec![F::ZERO; elements.len()];
    for i in (0..elements.len()).rev() {
        results[i] = products[i] * inv;
        inv *= elements[i];
    }
    
    results
}
```

## Testing Strategies

### Dual Compilation

Test on both targets:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_field_ops() {
        // Runs on host with software implementation
        test_add();
        test_mul();
    }
    
    #[cfg(target_os = "zkvm")]
    #[test] 
    fn test_custom_insns() {
        // Runs in zkVM with hardware acceleration
        test_hardware_add();
    }
}
```

### Property Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn field_axioms(a: MyField, b: MyField, c: MyField) {
        // Associativity
        assert_eq!((a + b) + c, a + (b + c));
        
        // Commutativity  
        assert_eq!(a + b, b + a);
        assert_eq!(a * b, b * a);
        
        // Distributivity
        assert_eq!(a * (b + c), a * b + a * c);
    }
}
```

## Error Handling

### Division Safety

```rust
// Safe division with Option
fn safe_div<F: Field>(a: F, b: F) -> Option<F> {
    if b == F::ZERO {
        None
    } else {
        Some(a.div_unsafe(b))
    }
}

// Or use Result for better errors
fn checked_div<F: Field>(a: F, b: F) -> Result<F, &'static str> {
    if b == F::ZERO {
        Err("division by zero")
    } else {
        Ok(a.div_unsafe(b))
    }
}
```

### Setup Verification

```rust
fn ensure_setup<F: IntMod>() {
    F::set_up_once(); // Idempotent
    
    // Verify setup worked
    let test = F::ONE + F::ONE;
    assert_eq!(test.double(), test + test);
}
```