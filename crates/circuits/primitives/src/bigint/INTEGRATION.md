# BigInt Primitives - Integration Guide

This document provides detailed guidance on integrating the BigInt Primitives component with other OpenVM components and external systems.

## Integration Architecture

### Component Dependencies

```
BigInt Primitives
├── Range Checker (for carry validation)
├── Variable Range Checker (bus-based range checking)
├── Field Arithmetic (underlying field operations)
└── Stark Backend (constraint system integration)
```

### Parent-Child AIR Pattern

BigInt Primitives follows the SubAir pattern where:
- **Parent AIRs** manage trace columns and witness generation
- **SubAirs** provide constraint logic and evaluation
- **Bus Integration** handles range checking interactions

## Range Checker Integration

### Setup and Configuration

```rust
use openvm_stark_backend::interaction::BusIndex;
use crate::bigint::check_carry_to_zero::CheckCarryToZeroSubAir;

pub struct YourBigIntAir {
    pub limb_bits: usize,
    pub range_checker_bus: BusIndex,
    pub decomp: usize, // Range checker decomposition parameter
    // ... other fields
}

impl YourBigIntAir {
    pub fn new(
        limb_bits: usize,
        range_checker_bus: BusIndex,
        decomp: usize,
    ) -> Self {
        Self {
            limb_bits,
            range_checker_bus,
            decomp,
        }
    }
}
```

### Range Check Bus Usage

```rust
use crate::bigint::utils::range_check;

// In your AIR evaluation
fn eval_range_checks<AB: InteractionBuilder>(
    builder: &mut AB,
    carries: &[AB::Var],
    carry_bits: usize,
    range_bus: BusIndex,
    decomp: usize,
    is_valid: AB::Expr,
) {
    let offset = AB::F::from_canonical_usize(1 << (carry_bits - 1));
    
    for &carry in carries {
        range_check(
            builder,
            range_bus,
            decomp,
            carry_bits,
            carry + offset, // Offset for signed range
            is_valid.clone(),
        );
    }
}
```

## Trace Column Management

### Column Layout Strategy

```rust
pub struct BigIntMultiplyColumns {
    // Input operands
    pub a_limbs: Vec<usize>,      // Start indices for operand A limbs
    pub b_limbs: Vec<usize>,      // Start indices for operand B limbs
    
    // Output
    pub result_limbs: Vec<usize>, // Start indices for result limbs
    
    // Modular arithmetic
    pub quotient_limbs: Vec<usize>, // Quotient for modular reduction
    
    // Carries
    pub carries: Vec<usize>,      // Carry columns for constraint checking
    
    // Control
    pub is_valid: usize,          // Boolean selector column
}

impl BigIntMultiplyColumns {
    pub fn new(base_col: usize, num_limbs: usize) -> Self {
        let mut col = base_col;
        
        let a_limbs = (0..num_limbs).map(|_| { let c = col; col += 1; c }).collect();
        let b_limbs = (0..num_limbs).map(|_| { let c = col; col += 1; c }).collect();
        let result_limbs = (0..num_limbs).map(|_| { let c = col; col += 1; c }).collect();
        let quotient_limbs = (0..num_limbs).map(|_| { let c = col; col += 1; c }).collect();
        
        // Carries for multiplication have 2*num_limbs - 1 elements
        let carries = (0..(2 * num_limbs - 1)).map(|_| { let c = col; col += 1; c }).collect();
        
        let is_valid = col;
        
        Self {
            a_limbs,
            b_limbs,
            result_limbs,
            quotient_limbs,
            carries,
            is_valid,
        }
    }
    
    pub fn total_columns(&self) -> usize {
        self.a_limbs.len() + self.b_limbs.len() + self.result_limbs.len() + 
        self.quotient_limbs.len() + self.carries.len() + 1
    }
}
```

### Witness Generation

```rust
use num_bigint::BigUint;
use crate::bigint::{OverflowInt, utils::big_uint_to_num_limbs};

impl YourBigIntAir {
    pub fn generate_trace_row(
        &self,
        a: &BigUint,
        b: &BigUint,
        modulus: &BigUint,
    ) -> Vec<F> {
        let num_limbs = self.num_limbs;
        let limb_bits = self.limb_bits;
        
        // Convert inputs to limb representation
        let a_limbs = big_uint_to_num_limbs(a, limb_bits, num_limbs);
        let b_limbs = big_uint_to_num_limbs(b, limb_bits, num_limbs);
        
        // Compute result and quotient
        let product = a * b;
        let result = &product % modulus;
        let quotient = &product / modulus;
        
        let result_limbs = big_uint_to_num_limbs(&result, limb_bits, num_limbs);
        let quotient_limbs = big_uint_to_num_limbs(&quotient, limb_bits, num_limbs);
        
        // Generate carries
        let a_overflow = OverflowInt::from_biguint(a, limb_bits, Some(num_limbs));
        let b_overflow = OverflowInt::from_biguint(b, limb_bits, Some(num_limbs));
        let r_overflow = OverflowInt::from_biguint(&result, limb_bits, Some(num_limbs));
        let q_overflow = OverflowInt::from_biguint(&quotient, limb_bits, Some(num_limbs));
        let m_overflow = OverflowInt::from_biguint(modulus, limb_bits, Some(num_limbs));
        
        let constraint_expr = a_overflow * b_overflow - r_overflow - q_overflow * m_overflow;
        let carries = constraint_expr.calculate_carries(limb_bits);
        
        // Pack into trace row
        let mut row = vec![F::ZERO; self.columns.total_columns()];
        
        // Fill columns
        for (i, &limb) in a_limbs.iter().enumerate() {
            row[self.columns.a_limbs[i]] = F::from_canonical_usize(limb);
        }
        for (i, &limb) in b_limbs.iter().enumerate() {
            row[self.columns.b_limbs[i]] = F::from_canonical_usize(limb);
        }
        for (i, &limb) in result_limbs.iter().enumerate() {
            row[self.columns.result_limbs[i]] = F::from_canonical_usize(limb);
        }
        for (i, &limb) in quotient_limbs.iter().enumerate() {
            row[self.columns.quotient_limbs[i]] = F::from_canonical_isize(limb as isize);
        }
        for (i, &carry) in carries.iter().enumerate() {
            row[self.columns.carries[i]] = F::from_canonical_isize(carry);
        }
        
        row[self.columns.is_valid] = F::ONE;
        
        row
    }
}
```

## SubAir Integration Patterns

### Single Operation Integration

```rust
use crate::bigint::check_carry_mod_to_zero::{CheckCarryModToZeroSubAir, CheckCarryModToZeroCols};

impl<AB: InteractionBuilder> Air<AB> for ModularMultiplyAir {
    fn eval(&self, builder: &mut AB) {
        // Extract trace columns
        let row = builder.main().row_slice(0);
        let is_valid = row[self.columns.is_valid];
        
        // Create OverflowInt expressions from trace columns
        let a_expr = self.create_overflow_expr(builder, &self.columns.a_limbs);
        let b_expr = self.create_overflow_expr(builder, &self.columns.b_limbs);
        let r_expr = self.create_overflow_expr(builder, &self.columns.result_limbs);
        
        // Expression to constrain: a*b - r ≡ 0 (mod m)
        let constraint_expr = a_expr * b_expr - r_expr;
        
        // Apply modular constraint
        let subair = CheckCarryModToZeroSubAir::new(
            self.modulus.clone(),
            self.limb_bits,
            self.range_checker_bus,
            self.decomp,
        );
        
        let quotient_vars = self.get_column_vars(builder, &self.columns.quotient_limbs);
        let carry_vars = self.get_column_vars(builder, &self.columns.carries);
        
        subair.eval(
            builder,
            (constraint_expr, CheckCarryModToZeroCols {
                quotient: quotient_vars,
                carries: carry_vars,
            }, is_valid)
        );
    }
    
    fn create_overflow_expr<AB: InteractionBuilder>(
        &self,
        builder: &AB,
        limb_cols: &[usize],
    ) -> OverflowInt<AB::Expr> {
        let row = builder.main().row_slice(0);
        let limb_exprs: Vec<AB::Expr> = limb_cols.iter()
            .map(|&col| row[col].into())
            .collect();
        
        OverflowInt::from_canonical_unsigned_limbs(limb_exprs, self.limb_bits)
    }
}
```

### Multi-Operation Integration

```rust
// For AIRs that perform multiple BigInt operations
impl<AB: InteractionBuilder> Air<AB> for ComplexBigIntAir {
    fn eval(&self, builder: &mut AB) {
        let row = builder.main().row_slice(0);
        let is_valid = row[self.columns.is_valid];
        
        // Create multiple SubAir instances
        let add_subair = CheckCarryToZeroSubAir::new(
            self.limb_bits, self.range_checker_bus, self.decomp
        );
        let mul_subair = CheckCarryModToZeroSubAir::new(
            self.modulus.clone(), self.limb_bits, 
            self.range_checker_bus, self.decomp
        );
        
        // First operation: a + b - sum = 0
        let a_expr = self.create_overflow_expr(builder, &self.columns.a_limbs);
        let b_expr = self.create_overflow_expr(builder, &self.columns.b_limbs);
        let sum_expr = self.create_overflow_expr(builder, &self.columns.sum_limbs);
        
        let add_constraint = a_expr + b_expr - sum_expr;
        let add_carries = self.get_column_vars(builder, &self.columns.add_carries);
        
        add_subair.eval(
            builder,
            (add_constraint, CheckCarryToZeroCols { carries: add_carries }, is_valid.clone())
        );
        
        // Second operation: sum * c ≡ result (mod m)
        let c_expr = self.create_overflow_expr(builder, &self.columns.c_limbs);
        let result_expr = self.create_overflow_expr(builder, &self.columns.result_limbs);
        
        let mul_constraint = sum_expr * c_expr - result_expr;
        let quotient_vars = self.get_column_vars(builder, &self.columns.quotient_limbs);
        let mul_carries = self.get_column_vars(builder, &self.columns.mul_carries);
        
        mul_subair.eval(
            builder,
            (mul_constraint, CheckCarryModToZeroCols {
                quotient: quotient_vars,
                carries: mul_carries,
            }, is_valid)
        );
    }
}
```

## Higher-Level Component Integration

### Elliptic Curve Integration

```rust
// Integration with elliptic curve point operations
use crate::bigint::{OverflowInt, check_carry_mod_to_zero::CheckCarryModToZeroSubAir};

pub struct EllipticCurvePointAddAir {
    pub field_modulus: BigUint,
    pub limb_bits: usize,
    pub num_limbs: usize,
    pub bigint_subair: CheckCarryModToZeroSubAir,
}

impl EllipticCurvePointAddAir {
    pub fn new(field_modulus: BigUint, limb_bits: usize, range_bus: BusIndex) -> Self {
        let num_limbs = (field_modulus.bits() as usize + limb_bits - 1) / limb_bits;
        let bigint_subair = CheckCarryModToZeroSubAir::new(
            field_modulus.clone(), limb_bits, range_bus, 8
        );
        
        Self {
            field_modulus,
            limb_bits,
            num_limbs,
            bigint_subair,
        }
    }
}

impl<AB: InteractionBuilder> Air<AB> for EllipticCurvePointAddAir {
    fn eval(&self, builder: &mut AB) {
        // Point addition formulas require multiple field multiplications
        // Each multiplication uses the BigInt SubAir
        
        // For point addition: (x1, y1) + (x2, y2) = (x3, y3)
        // Intermediate computations:
        // s = (y2 - y1) / (x2 - x1)  [slope]
        // x3 = s^2 - x1 - x2
        // y3 = s * (x1 - x3) - y1
        
        // Each division/multiplication needs modular constraint
        self.constrain_field_operation(builder, /* slope calculation */);
        self.constrain_field_operation(builder, /* x3 calculation */);
        self.constrain_field_operation(builder, /* y3 calculation */);
    }
    
    fn constrain_field_operation<AB: InteractionBuilder>(
        &self,
        builder: &mut AB,
        // operation-specific parameters
    ) {
        // Use BigInt SubAir for field arithmetic constraints
        // This delegates the complex carry propagation logic
    }
}
```

### Hash Function Integration

```rust
// Integration with hash functions that require large integer arithmetic
pub struct SHA256BigIntAir {
    pub bigint_subairs: Vec<CheckCarryToZeroSubAir>,
    pub round_constants: Vec<BigUint>,
}

impl SHA256BigIntAir {
    pub fn new(range_bus: BusIndex) -> Self {
        // SHA-256 operates on 32-bit words, but intermediate additions
        // can benefit from BigInt representation for constraint efficiency
        let subairs = (0..64) // 64 rounds
            .map(|_| CheckCarryToZeroSubAir::new(32, range_bus, 4))
            .collect();
            
        Self {
            bigint_subairs: subairs,
            round_constants: sha256_constants(),
        }
    }
}
```

## Error Handling and Debugging

### Common Integration Issues

```rust
// 1. Overflow Bound Mismanagement
pub fn check_overflow_bounds<F: Field>(
    max_overflow_bits: usize,
    limb_bits: usize,
) -> Result<(), String> {
    let field_bits = F::bits();
    let max_carry_bits = max_overflow_bits - limb_bits + 1;
    let max_constraint_bits = max_carry_bits + limb_bits;
    
    if max_constraint_bits >= field_bits - 1 {
        return Err(format!(
            "Constraint overflow: {} bits needed, {} available",
            max_constraint_bits, field_bits - 1
        ));
    }
    
    Ok(())
}

// 2. Range Check Configuration Validation
pub fn validate_range_config(
    carry_bits: usize,
    decomp: usize,
) -> Result<(), String> {
    if carry_bits % decomp != 0 {
        return Err(format!(
            "Range check decomposition mismatch: {} bits, {} decomp",
            carry_bits, decomp
        ));
    }
    
    Ok(())
}

// 3. Limb Count Consistency
pub fn validate_limb_consistency(
    value_bits: usize,
    limb_bits: usize,
    num_limbs: usize,
) -> Result<(), String> {
    let required_limbs = (value_bits + limb_bits - 1) / limb_bits;
    if num_limbs < required_limbs {
        return Err(format!(
            "Insufficient limbs: {} needed, {} provided",
            required_limbs, num_limbs
        ));
    }
    
    Ok(())
}
```

### Debugging Utilities

```rust
pub mod debug {
    use super::*;
    
    pub fn trace_overflow_progression<T: Clone + std::fmt::Debug>(
        operations: Vec<OverflowInt<T>>,
        operation_names: Vec<&str>,
    ) {
        println!("Overflow progression:");
        for (i, (op, name)) in operations.iter().zip(operation_names).enumerate() {
            println!("  {}: {} - max_abs: {}, overflow_bits: {}, limbs: {}", 
                i, name, op.limb_max_abs(), op.max_overflow_bits(), op.num_limbs());
        }
    }
    
    pub fn validate_carries(
        carries: &[isize],
        expected_final: isize,
        context: &str,
    ) -> Result<(), String> {
        if let Some(&final_carry) = carries.last() {
            if final_carry != expected_final {
                return Err(format!(
                    "{}: Final carry {} != expected {}",
                    context, final_carry, expected_final
                ));
            }
        }
        
        Ok(())
    }
}
```

## Performance Optimization

### Column Sharing Strategies

```rust
// Optimize trace width by sharing columns across operations
pub struct SharedColumnBigIntAir {
    pub shared_limbs: Vec<usize>,    // Reused for different operations
    pub shared_carries: Vec<usize>,  // Reused carry columns
    pub operation_selector: usize,   // Which operation is active
}

impl SharedColumnBigIntAir {
    fn eval_with_sharing<AB: InteractionBuilder>(&self, builder: &mut AB) {
        let row = builder.main().row_slice(0);
        let op_selector = row[self.operation_selector];
        
        // Use same columns for different operations based on selector
        let add_flag = self.compute_add_flag(builder, op_selector.clone());
        let mul_flag = self.compute_mul_flag(builder, op_selector.clone());
        
        // Conditional constraint application
        self.constrain_add_operation(builder, add_flag);
        self.constrain_mul_operation(builder, mul_flag);
    }
}
```

### Batch Processing

```rust
// Process multiple BigInt operations in parallel
pub fn batch_process_constraints<AB: InteractionBuilder>(
    builder: &mut AB,
    operations: Vec<(OverflowInt<AB::Expr>, /* constraint data */)>,
    subair: &CheckCarryModToZeroSubAir,
) {
    for (expr, cols, is_valid) in operations {
        subair.eval(builder, (expr, cols, is_valid));
    }
}
```

This integration guide covers the major patterns for incorporating BigInt Primitives into larger OpenVM components and external systems. The key is understanding the SubAir pattern and properly managing trace columns and witness generation.