# BigInt Primitives Implementation Guide

## Module Dependencies

### Internal Dependencies
```rust
use crate::{
    SubAir,
    var_range::{VariableRangeCheckerBus, BusIndex},
};
```

### External Dependencies
```rust
use num_bigint::{BigUint, BigInt};
use openvm_stark_backend::{
    interaction::InteractionBuilder,
    p3_field::{Field, FieldAlgebra, PrimeField64},
    p3_util::log2_ceil_usize,
};
```

## Implementation Patterns

### OverflowInt Construction

#### From Canonical Unsigned Limbs
```rust
// For non-negative inputs with known bit width
let limbs = vec![a0, a1, a2]; // Each in [0, 2^limb_bits)
let overflow_int = OverflowInt::from_canonical_unsigned_limbs(limbs, limb_bits);
```

#### From Canonical Signed Limbs
```rust
// For potentially negative limbs
let limbs = vec![a0, a1, a2]; // Each in [-2^limb_bits, 2^limb_bits)  
let overflow_int = OverflowInt::from_canonical_signed_limbs(limbs, limb_bits);
```

#### From BigUint
```rust
let big_num = BigUint::from_str("123456789...").unwrap();
let limb_bits = 10;
let min_limbs = Some(32); // Pad to 32 limbs if needed
let overflow_int = OverflowInt::from_biguint(&big_num, limb_bits, min_limbs);
```

### Arithmetic Operations

#### Addition Pattern
```rust
impl<T> Add for OverflowInt<T> 
where T: Add<Output = T> + Clone + Default
{
    fn add(self, other: OverflowInt<T>) -> OverflowInt<T> {
        // Extend to max length
        let len = max(self.limbs.len(), other.limbs.len());
        let mut limbs = Vec::with_capacity(len);
        
        // Add limb-wise
        for i in 0..len {
            let a = self.limbs.get(i).unwrap_or(&T::default());
            let b = other.limbs.get(i).unwrap_or(&T::default());
            limbs.push(a.clone() + b.clone());
        }
        
        // Update overflow tracking
        let new_max = self.limb_max_abs + other.limb_max_abs;
        let max_bits = log2_ceil_usize(new_max);
        
        OverflowInt {
            limbs,
            max_overflow_bits: max_bits,
            limb_max_abs: new_max,
        }
    }
}
```

#### Multiplication Pattern
```rust
// Convolution of limbs
let len = self.limbs.len() + other.limbs.len() - 1;
let mut limbs = vec![T::default(); len];

for i in 0..self.limbs.len() {
    for j in 0..other.limbs.len() {
        limbs[i + j] = limbs[i + j].clone() + 
                       self.limbs[i].clone() * other.limbs[j].clone();
    }
}

// Max value considers all partial products
let new_max = self.limb_max_abs * other.limb_max_abs * 
              min(self.limbs.len(), other.limbs.len());
```

### Carry Generation

#### Calculate Carries for Trace
```rust
impl OverflowInt<isize> {
    pub fn calculate_carries(&self, limb_bits: usize) -> Vec<isize> {
        let mut carries = Vec::with_capacity(self.limbs.len());
        let mut carry = 0;
        
        for i in 0..self.limbs.len() {
            // Arithmetic right shift for correct negative handling
            carry = (carry + self.limbs[i]) >> limb_bits;
            carries.push(carry);
        }
        carries
    }
}
```

### SubAir Integration

#### CheckCarryToZero Usage
```rust
// In parent AIR eval
pub fn eval_modular_op<AB: InteractionBuilder>(
    builder: &mut AB,
    overflow_expr: OverflowInt<AB::Expr>,
    carry_cols: &[AB::Var],
    is_valid: AB::Expr,
) {
    let carry_to_zero = CheckCarryToZeroSubAir::new(
        limb_bits,
        range_checker_bus,
        decomp_bits,
    );
    
    let cols = CheckCarryToZeroCols {
        carries: carry_cols.to_vec(),
    };
    
    carry_to_zero.eval(builder, (overflow_expr, cols, is_valid));
}
```

#### CheckCarryModToZero Usage
```rust
// For modular equality constraints
let carry_mod_to_zero = CheckCarryModToZeroSubAir::new(
    modulus,
    limb_bits, 
    range_checker_bus,
    decomp_bits,
);

let cols = CheckCarryModToZeroCols {
    carries: carry_cols.to_vec(),
    quotient: quotient_cols.to_vec(),
};

carry_mod_to_zero.eval(builder, (overflow_expr, cols, is_valid));
```

### Range Checking Pattern

```rust
pub fn range_check<AB: InteractionBuilder>(
    builder: &mut AB,
    range_bus: BusIndex,
    decomp: usize,
    bits: usize,
    expr: impl Into<AB::Expr>,
    count: impl Into<AB::Expr>,
) {
    assert!(bits <= decomp);
    let bus = VariableRangeCheckerBus::new(range_bus, decomp);
    bus.range_check(expr, bits).eval(builder, count);
}
```

### Common Modular Arithmetic Patterns

#### Modular Multiplication
```rust
// Prove: a * b ≡ r (mod p)
// Compute: a * b - r - q * p = 0

// Build overflow expression
let ab = overflow_a * overflow_b;
let qp = overflow_q * overflow_p;
let expr = ab - overflow_r - qp;

// Apply constraint
check_carry_to_zero.eval(builder, (expr, carry_cols, is_valid));
```

#### Modular Inverse
```rust
// Prove: a * a_inv ≡ 1 (mod p)
// Compute: a * a_inv - 1 - q * p = 0

let expr = overflow_a * overflow_a_inv
    .int_sub(1, |x| AB::Expr::from_canonical_usize(x))
    - overflow_q * overflow_p;
```

### Performance Optimizations

#### Batching Operations
```rust
// Instead of constraining each operation
let r1 = a1 * b1;
check_carry_to_zero(r1 - c1);
let r2 = a2 * b2;  
check_carry_to_zero(r2 - c2);

// Batch multiple operations
let expr1 = a1 * b1 - c1;
let expr2 = a2 * b2 - c2;
// Use same SubAir with different rows
```

#### Limb Size Selection
```rust
// For 256-bit integers
const LIMB_BITS: usize = 8;  // 32 limbs, more constraints
const LIMB_BITS: usize = 16; // 16 limbs, higher overflow

// Balance based on:
// - Number of operations before carry
// - Available range check decomposition
// - Trace width constraints
```

### Error Handling

#### Overflow Detection
```rust
// Check if operation will exceed field size
let max_bits = overflow_int.max_overflow_bits();
assert!(max_bits < F::bits() - 1, "Overflow exceeds field size");
```

#### Debug Helpers
```rust
#[cfg(debug_assertions)]
fn validate_carries(limbs: &[isize], carries: &[isize], limb_bits: usize) {
    let mut carry = 0;
    for i in 0..limbs.len() {
        let expected = (carry + limbs[i]) >> limb_bits;
        assert_eq!(carries[i], expected, "Carry mismatch at {}", i);
        carry = carries[i];
    }
}
```

## Complete Example: Modular Multiplication AIR

```rust
pub struct ModMulAir {
    pub limb_bits: usize,
    pub check_carry: CheckCarryToZeroSubAir,
}

impl<AB: InteractionBuilder> Air<AB> for ModMulAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        
        // Parse columns
        let a_limbs = &local[0..32];
        let b_limbs = &local[32..64];
        let r_limbs = &local[64..96];
        let q_limbs = &local[96..128];
        let carries = &local[128..191]; // 2*32-1 carries
        
        // Build overflow integers
        let a = OverflowInt::from_canonical_unsigned_limbs(
            a_limbs.to_vec(), self.limb_bits
        );
        let b = OverflowInt::from_canonical_unsigned_limbs(
            b_limbs.to_vec(), self.limb_bits
        );
        
        // Compute a*b - r - q*p
        let ab = a * b;
        let r = OverflowInt::from_canonical_unsigned_limbs(
            r_limbs.to_vec(), self.limb_bits
        );
        let q = OverflowInt::from_canonical_signed_limbs(
            q_limbs.to_vec(), self.limb_bits
        );
        let p = self.modulus_overflow();
        
        let expr = ab - r - q * p;
        
        // Apply constraint
        let carry_cols = CheckCarryToZeroCols {
            carries: carries.to_vec(),
        };
        
        self.check_carry.eval(
            builder,
            (expr, carry_cols, AB::Expr::one())
        );
    }
}
```