# OpenVM Modular Arithmetic Circuit Builder - Examples

This document provides practical examples of using the `openvm-mod-circuit-builder` for various cryptographic and arithmetic operations.

## Basic Setup

```rust
use openvm_mod_circuit_builder::*;
use num_bigint::BigUint;
use std::{cell::RefCell, rc::Rc, sync::Arc};

// Setup with secp256k1 prime field
let prime = secp256k1_coord_prime();
let config = ExprBuilderConfig {
    modulus: prime.clone(),
    limb_bits: 8,
    num_limbs: 32,
};

let range_checker = Arc::new(VariableRangeCheckerChip::new(
    VariableRangeCheckerBus::new(1, 17)
));
let builder = Rc::new(RefCell::new(
    ExprBuilder::new(config, range_checker.range_max_bits())
));
```

## Example 1: Basic Field Arithmetic

```rust
#[test]
fn example_basic_arithmetic() {
    let prime = secp256k1_coord_prime();
    let (range_checker, builder) = setup(&prime);

    // Create input variables
    let a = ExprBuilder::new_input(builder.clone());
    let b = ExprBuilder::new_input(builder.clone());
    let c = ExprBuilder::new_input(builder.clone());

    // Perform arithmetic: result = (a + b) * c
    let sum = a + b;
    let mut result = sum * c;
    result.save(); // Convert to constrained variable

    // Setup the expression for constraint generation
    let builder_ref = builder.borrow().clone();
    let expr = FieldExpr::new(builder_ref, range_checker.bus(), false);

    // Test with concrete values
    let x = BigUint::from(12345u64);
    let y = BigUint::from(67890u64);
    let z = BigUint::from(24680u64);
    let expected = ((&x + &y) * &z) % &prime;
    
    let inputs = vec![x, y, z];
    let mut row = BabyBear::zero_vec(BaseAir::<BabyBear>::width(&expr));
    expr.generate_subrow((&range_checker, inputs, vec![]), &mut row);
    
    let FieldExprCols { vars, .. } = expr.load_vars(&row);
    let computed = evaluate_biguint(&vars[0], 8);
    assert_eq!(computed, expected);
}
```

## Example 2: Modular Division

```rust
#[test]
fn example_modular_division() {
    let prime = secp256k1_coord_prime();
    let (range_checker, builder) = setup(&prime);

    let numerator = ExprBuilder::new_input(builder.clone());
    let denominator = ExprBuilder::new_input(builder.clone());
    
    // Division automatically handles modular inverse
    let quotient = numerator / denominator; // Auto-saved due to division
    
    let builder_ref = builder.borrow().clone();
    let expr = FieldExpr::new(builder_ref, range_checker.bus(), false);

    // Test with concrete values
    let num = BigUint::from(98765u64);
    let den = BigUint::from(43210u64);
    let den_inv = den.modinv(&prime).unwrap();
    let expected = (&num * &den_inv) % &prime;
    
    let inputs = vec![num, den];
    let mut row = BabyBear::zero_vec(BaseAir::<BabyBear>::width(&expr));
    expr.generate_subrow((&range_checker, inputs, vec![]), &mut row);
    
    let FieldExprCols { vars, .. } = expr.load_vars(&row);
    let computed = evaluate_biguint(&vars[0], 8);
    assert_eq!(computed, expected);
}
```

## Example 3: Conditional Selection

```rust
#[test]
fn example_conditional_selection() {
    let prime = secp256k1_coord_prime();
    let (range_checker, builder) = setup(&prime);

    let a = ExprBuilder::new_input(builder.clone());
    let b = ExprBuilder::new_input(builder.clone());
    
    // Create a flag for conditional selection
    let flag_id = builder.borrow_mut().new_flag();
    
    // Select between a and b based on flag
    let selected = FieldVariable::select(flag_id, a.clone(), b.clone());
    let mut result = selected;
    result.save();
    
    let builder_ref = builder.borrow().clone();
    let expr = FieldExpr::new(builder_ref, range_checker.bus(), false);

    // Test with flag = 1 (select first option)
    let x = BigUint::from(11111u64);
    let y = BigUint::from(22222u64);
    let inputs = vec![x.clone(), y];
    let flags = vec![BabyBear::one()]; // Select first option
    
    let mut row = BabyBear::zero_vec(BaseAir::<BabyBear>::width(&expr));
    expr.generate_subrow((&range_checker, inputs, flags), &mut row);
    
    let FieldExprCols { vars, .. } = expr.load_vars(&row);
    let computed = evaluate_biguint(&vars[0], 8);
    assert_eq!(computed, x); // Should equal first input
}
```

## Example 4: Elliptic Curve Point Addition (Partial)

```rust
#[test]
fn example_ec_point_addition_slope() {
    let prime = secp256k1_coord_prime();
    let (range_checker, builder) = setup(&prime);

    // Point P = (x1, y1) and Q = (x2, y2)
    let x1 = ExprBuilder::new_input(builder.clone());
    let y1 = ExprBuilder::new_input(builder.clone());
    let x2 = ExprBuilder::new_input(builder.clone());
    let y2 = ExprBuilder::new_input(builder.clone());

    // Calculate slope: λ = (y2 - y1) / (x2 - x1)
    let y_diff = y2 - y1;
    let x_diff = x2 - x1;
    let mut lambda = y_diff / x_diff; // Auto-saved due to division
    
    // Point addition: x3 = λ² - x1 - x2
    let lambda_squared = &mut lambda * &mut lambda;
    let mut x3 = lambda_squared - x1 - x2;
    x3.save();

    let builder_ref = builder.borrow().clone();
    let expr = FieldExpr::new(builder_ref, range_checker.bus(), false);

    // Test with concrete curve points
    let x1_val = BigUint::from(55066263022277343669578718895168534326250603453777594175500187360389116729240u128);
    let y1_val = BigUint::from(32670510020758816978083085130507043184471273380659243275938904335757337482424u128);
    let x2_val = BigUint::from(89565891926547004231252920425935692360644145829622209833684329913297188986597u128);
    let y2_val = BigUint::from(12158399299693830322967808612713398636155367887041628176798871954788371653930u128);
    
    let inputs = vec![x1_val, y1_val, x2_val, y2_val];
    let mut row = BabyBear::zero_vec(BaseAir::<BabyBear>::width(&expr));
    expr.generate_subrow((&range_checker, inputs, vec![]), &mut row);
    
    // Verify constraint satisfaction
    let trace = RowMajorMatrix::new(row, BaseAir::<BabyBear>::width(&expr));
    let range_trace = range_checker.generate_trace();
    
    BabyBearBlake3Engine::run_simple_test_no_pis_fast(
        any_rap_arc_vec![expr, range_checker.air],
        vec![trace, range_trace],
    ).expect("Verification failed");
}
```

## Example 5: Efficient Expression Reuse

```rust
#[test]
fn example_expression_reuse() {
    let prime = secp256k1_coord_prime();
    let (range_checker, builder) = setup(&prime);

    let a = ExprBuilder::new_input(builder.clone());
    let b = ExprBuilder::new_input(builder.clone());
    
    // Compute a² + b² efficiently by reusing squares
    let mut a_squared = &a * &a;
    a_squared.save(); // Save for reuse
    
    let mut b_squared = &b * &b;
    b_squared.save(); // Save for reuse
    
    // Reuse the saved squares
    let mut sum_of_squares = a_squared + b_squared;
    sum_of_squares.save();
    
    // Use squares again in another computation: (a² + b²) * a²
    let mut final_result = sum_of_squares * a_squared;
    final_result.save();

    println!("Number of variables created: {}", builder.borrow().num_vars);
    // Only 4 variables created instead of recomputing squares
}
```

## Example 6: Working with Constants

```rust
#[test]
fn example_constants() {
    let prime = secp256k1_coord_prime();
    let (range_checker, builder) = setup(&prime);

    let x = ExprBuilder::new_input(builder.clone());
    
    // Create constants
    let two = ExprBuilder::new_const(builder.clone(), BigUint::from(2u32));
    let three = ExprBuilder::new_const(builder.clone(), BigUint::from(3u32));
    
    // Quadratic expression: 2x² + 3x + 1
    let x_squared = &x * &x;
    let two_x_squared = two * x_squared;
    let three_x = three * x;
    let mut result = two_x_squared + three_x + BigUint::from(1u32);
    result.save();
    
    let builder_ref = builder.borrow().clone();
    let expr = FieldExpr::new(builder_ref, range_checker.bus(), false);

    // Test with x = 5: 2(25) + 3(5) + 1 = 66
    let x_val = BigUint::from(5u32);
    let expected = BigUint::from(66u32);
    
    let inputs = vec![x_val];
    let mut row = BabyBear::zero_vec(BaseAir::<BabyBear>::width(&expr));
    expr.generate_subrow((&range_checker, inputs, vec![]), &mut row);
    
    let FieldExprCols { vars, .. } = expr.load_vars(&row);
    let computed = evaluate_biguint(&vars[0], 8);
    assert_eq!(computed, expected);
}
```

## Example 7: Multi-Operation Chip

```rust
#[test]
fn example_multi_operation_chip() {
    let prime = secp256k1_coord_prime();
    let (range_checker, builder) = setup(&prime);

    let a = ExprBuilder::new_input(builder.clone());
    let b = ExprBuilder::new_input(builder.clone());
    
    // Create flags for different operations
    let add_flag = builder.borrow_mut().new_flag();
    let mul_flag = builder.borrow_mut().new_flag();
    
    // Define operations
    let sum = a + b;
    let product = a * b;
    
    // Select operation based on flags
    let mut result = FieldVariable::select(add_flag, sum, product);
    result.save();
    
    let builder_ref = builder.borrow().clone();
    let expr = FieldExpr::new(builder_ref, range_checker.bus(), false);

    // Test addition operation (add_flag = 1, mul_flag = 0)
    let x = BigUint::from(15u32);
    let y = BigUint::from(25u32);
    let expected_sum = BigUint::from(40u32);
    
    let inputs = vec![x, y];
    let flags = vec![BabyBear::one(), BabyBear::zero()];
    
    let mut row = BabyBear::zero_vec(BaseAir::<BabyBear>::width(&expr));
    expr.generate_subrow((&range_checker, inputs, flags), &mut row);
    
    let FieldExprCols { vars, .. } = expr.load_vars(&row);
    let computed = evaluate_biguint(&vars[0], 8);
    assert_eq!(computed, expected_sum);
}
```

## Example 8: Error Handling

```rust
#[test]
#[should_panic(expected = "Division by zero")]
fn example_division_by_zero() {
    let prime = secp256k1_coord_prime();
    let (range_checker, builder) = setup(&prime);

    let numerator = ExprBuilder::new_input(builder.clone());
    let zero = ExprBuilder::new_const(builder.clone(), BigUint::zero());
    
    // This will panic during trace generation
    let _quotient = numerator / zero;
    
    let builder_ref = builder.borrow().clone();
    let expr = FieldExpr::new(builder_ref, range_checker.bus(), false);
    
    let inputs = vec![BigUint::from(42u32)];
    let mut row = BabyBear::zero_vec(BaseAir::<BabyBear>::width(&expr));
    
    // This will panic
    expr.generate_subrow((&range_checker, inputs, vec![]), &mut row);
}
```

## Example 9: BN254 Curve Operations

```rust
#[cfg(feature = "test-utils")]
#[test]
fn example_bn254_field_operations() {
    use crate::test_utils::*;
    use halo2curves_axiom::{bn256::Fq, ff::Field};
    
    let prime = bn254_fq_prime();
    let (range_checker, builder) = setup(&prime);

    let a = ExprBuilder::new_input(builder.clone());
    let b = ExprBuilder::new_input(builder.clone());
    
    // Compute a³ + 3 (part of elliptic curve equation y² = x³ + 3)
    let a_squared = &a * &a;
    let a_cubed = a_squared * a;
    let three = ExprBuilder::new_const(builder.clone(), BigUint::from(3u32));
    let mut result = a_cubed + three;
    result.save();
    
    let builder_ref = builder.borrow().clone();
    let expr = FieldExpr::new(builder_ref, range_checker.bus(), false);

    // Test with random BN254 field element
    let x_fq = Fq::random(&mut create_seeded_rng());
    let x_biguint = bn254_fq_to_biguint(x_fq);
    let expected_fq = x_fq * x_fq * x_fq + Fq::from(3);
    let expected = bn254_fq_to_biguint(expected_fq);
    
    let inputs = vec![x_biguint];
    let mut row = BabyBear::zero_vec(BaseAir::<BabyBear>::width(&expr));
    expr.generate_subrow((&range_checker, inputs, vec![]), &mut row);
    
    let FieldExprCols { vars, .. } = expr.load_vars(&row);
    let computed = evaluate_biguint(&vars[0], 8);
    assert_eq!(computed, expected);
}
```

## Key Takeaways

1. **Always call `.save()`** on expressions that will be reused or are final outputs
2. **Division automatically saves** the result to handle modular inverse constraints
3. **Use flags for conditional operations** with `FieldVariable::select()`
4. **Reuse saved expressions** to minimize constraint count
5. **Constants are created with `new_const()`** and can be used in arithmetic
6. **Range checker integration** is automatic but must be properly configured
7. **Error handling** includes division by zero detection at trace generation time
8. **Multi-operation chips** use flags to select between different computations

These examples cover the most common usage patterns for the modular arithmetic circuit builder, from basic field operations to complex cryptographic computations.