# OpenVM Mod-Builder Quick Reference

## Setup and Configuration

### Basic Setup
```rust
use openvm_mod_circuit_builder::{ExprBuilder, ExprBuilderConfig};
use openvm_circuit_primitives::var_range::{VariableRangeCheckerChip, VariableRangeCheckerBus};

// Configure range checker
let range_bus = VariableRangeCheckerBus::new(bus_idx, decomp_bits);
let range_checker = Arc::new(VariableRangeCheckerChip::new(range_bus));

// Configure builder
let config = ExprBuilderConfig {
    modulus: BigUint::from_str("21888242871..."),  // Field prime
    num_limbs: 32,                                  // Number of limbs
    limb_bits: 8,                                   // Bits per limb
};

// Create builder
let builder = ExprBuilder::new(config, range_checker.range_max_bits());
```

### With Test Utils
```rust
use openvm_mod_circuit_builder::test_utils::{setup, BN254_MODULUS};

let (range_checker, builder) = setup(&BN254_MODULUS);
```

## Variable Creation

### New Variables
```rust
// Single variable
let (idx, var) = builder.borrow_mut().new_var();

// Input variables (for trace generation)
let inputs = (0..4).map(|_| builder.borrow_mut().new_var().1).collect();

// Constant
let one = builder.borrow_mut().one();
let constant = builder.borrow_mut().constant(BigUint::from(42u32));
```

### From BigUint
```rust
let value = BigUint::from_str("12345678901234567890").unwrap();
let var = builder.borrow_mut().constant(value);
```

## Arithmetic Operations

### Basic Operations
```rust
// Addition
let sum = &a + &b;

// Subtraction  
let diff = &a - &b;

// Multiplication
let product = &a * &b;

// Division (expensive, use sparingly)
let quotient = &a / &b;

// Integer operations
let scaled = a.int_mul(5);        // Multiply by constant
let shifted = a.int_add(-3);      // Add constant
```

### Complex Expressions
```rust
// Polynomial evaluation: ax² + bx + c
let result = &(&a * &x * &x) + &(&b * &x) + &c;

// Conditional: flag ? expr1 : expr2
let selected = builder.borrow_mut().select(flag_idx, &expr1, &expr2);
```

## Constraint Management

### Save Variables
```rust
// Save expression as new variable (creates constraint)
let saved_idx = result.save();

// Check if already saved
if let SymbolicExpr::Var(idx) = expr.expr {
    // Already saved, idx is the variable index
}
```

### Add Constraints
```rust
// Equality constraint: a = b
builder.borrow_mut().constrain_eq(&a, &b);

// Zero constraint: expr = 0
let zero = builder.borrow_mut().zero();
builder.borrow_mut().constrain_eq(&expr, &zero);

// Constraint with computation
let lhs = &a * &b + &c;
let rhs = &d * &e;
builder.borrow_mut().constrain_eq(&lhs, &rhs);
```

## AIR Generation

### Generate Field Expression
```rust
let field_expr = builder.borrow_mut().generate_field_expr();

// Access components
let num_inputs = field_expr.builder.num_input;
let constraints = field_expr.builder.constraints.clone();
```

### Create Core AIR
```rust
use openvm_mod_circuit_builder::FieldExpressionCoreAir;

let air = FieldExpressionCoreAir {
    expr: field_expr,
    offset: 0,                          // Global opcode offset
    local_opcode_idx: vec![0, 1],       // Supported opcodes
    opcode_flag_idx: vec![],            // Multi-op flags
};
```

## Common Patterns

### Elliptic Curve Addition
```rust
// P + Q = R on curve y² = x³ + ax + b
let (x1, y1) = (&inputs[0], &inputs[1]);
let (x2, y2) = (&inputs[2], &inputs[3]);

// Compute slope
let lambda = if p_equals_q {
    // Point doubling: λ = (3x₁² + a) / 2y₁
    let numerator = &x1 * &x1 * 3 + &a;
    let denominator = &y1 * 2;
    &numerator / &denominator
} else {
    // Point addition: λ = (y₂ - y₁) / (x₂ - x₁)
    &(&y2 - &y1) / &(&x2 - &x1)
};

// Compute result
let x3 = &(&lambda * &lambda) - &x1 - &x2;
let y3 = &(&lambda * &(&x1 - &x3)) - &y1;
```

### Field Inversion
```rust
// Compute a⁻¹ mod p using Fermat's little theorem
// a⁻¹ = a^(p-2) mod p
let inv = field_exp(&a, &(prime - 2u32));

// Verify: a * a⁻¹ = 1
let one = builder.borrow_mut().one();
builder.borrow_mut().constrain_eq(&(&a * &inv), &one);
```

### Modular Reduction
```rust
// Reduce large value modulo prime
let (quotient, remainder) = div_rem(&large_value, &prime);

// Verify: large_value = quotient * prime + remainder
let reconstructed = &(&quotient * &prime) + &remainder;
builder.borrow_mut().constrain_eq(&large_value, &reconstructed);
```

## Testing Utilities

### Generate Test Data
```rust
use openvm_mod_circuit_builder::test_utils::*;

// Random field element
let random = generate_random_biguint(&modulus);

// Convert to limbs
let limbs = biguint_to_limbs::<32>(&random, 8);

// Evaluate limbs back to BigUint
let value = evaluate_biguint(&limbs, 8);
```

### Curve-Specific Utils
```rust
// BN254
use openvm_mod_circuit_builder::test_utils::{BN254_MODULUS, bn254_point_add};

// BLS12-381
use openvm_mod_circuit_builder::test_utils::{BLS12_381_MODULUS, bls12_381_double};
```

## Performance Tips

1. **Minimize Divisions**: Division is expensive, precompute inverses when possible
2. **Save Strategically**: Save intermediate results that are used multiple times
3. **Batch Operations**: Group similar operations together
4. **Reuse Variables**: Don't create new variables for the same value
5. **Optimize Expressions**: Flatten nested operations, combine constants

## Common Errors

### Overflow Detection
```rust
// Check if operation might overflow
if var.limb_max_abs > (1 << 30) {
    // Save variable to reset overflow bounds
    var.save();
}
```

### Division by Zero
```rust
// Always check divisor
if divisor.is_zero() {
    panic!("Division by zero");
}
```

### Configuration Mismatch
```rust
// Ensure modulus fits in limb configuration
assert!(modulus.bits() <= (num_limbs * limb_bits) as u64);
```

## Integration Example

```rust
// Complete example: Field multiplication chip
pub struct FieldMulChip {
    air: FieldExpressionCoreAir,
}

impl FieldMulChip {
    pub fn new(modulus: BigUint) -> Self {
        let (range_checker, builder_ref) = setup(&modulus);
        let builder = builder_ref.borrow_mut();
        
        // Create inputs: a, b
        let a = builder.new_var().1;
        let b = builder.new_var().1;
        
        // Compute: c = a * b
        let c = &a * &b;
        c.save();
        
        // Generate AIR
        let expr = builder.generate_field_expr();
        let air = FieldExpressionCoreAir {
            expr,
            offset: 0,
            local_opcode_idx: vec![0],
            opcode_flag_idx: vec![],
        };
        
        Self { air }
    }
}
```