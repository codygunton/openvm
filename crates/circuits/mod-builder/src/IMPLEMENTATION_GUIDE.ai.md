# OpenVM Mod-Builder Implementation Guide

## Core Concepts

### Limb-Based Field Representation

The mod-builder represents large field elements as vectors of smaller "limbs":

```rust
// Example: 256-bit field element with 8-bit limbs
// Field element: 0x123456789ABCDEF0...
// Limbs: [0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12, ...]
```

Key properties:
- Each limb fits in the native field (BabyBear)
- Limb size configurable (typically 8 bits)
- Little-endian representation
- Supports both signed and unsigned operations

### Overflow Integer Representation

During computation, limbs can overflow their normal bounds:

```rust
pub struct OverflowInt<T> {
    pub limbs: Vec<T>,  // Can be negative or > 2^limb_bits
}
```

The system tracks:
- `limb_max_abs`: Maximum absolute value per limb
- `max_overflow_bits`: Bits needed to represent max value
- Carry propagation requirements

### Expression Building Pattern

The builder follows a three-phase pattern:

1. **Build Phase**: Construct symbolic expressions
2. **Save Phase**: Convert expressions to variables with constraints
3. **Constrain Phase**: Add final equality constraints

```rust
// Build
let expr = &a * &b + &c;

// Save (creates intermediate constraint)
let d = expr.save();

// Constrain
builder.constrain_eq(&d, &expected);
```

## Implementation Details

### ExprBuilder Internals

The `ExprBuilder` maintains several critical data structures:

```rust
pub struct ExprBuilder {
    // Field configuration
    pub prime: BigUint,
    pub num_limbs: usize,
    pub limb_bits: usize,
    
    // Variable tracking
    vars: Vec<FieldVariable>,
    num_input: usize,
    
    // Constraint management
    constraints: Vec<(FieldVariable, BigInt)>,
    
    // Range checking
    range_checker: SharedRangeCheckerReference,
}
```

Key methods:

1. **Variable Creation**
   ```rust
   pub fn new_var(&mut self) -> (usize, FieldVariable) {
       let idx = self.vars.len();
       let var = FieldVariable {
           expr: SymbolicExpr::Var(idx),
           limb_max_abs: 0,
           max_overflow_bits: 0,
           expr_limbs: self.num_limbs,
           builder: Rc::clone(&self_rc),
       };
       self.vars.push(var.clone());
       (idx, var)
   }
   ```

2. **Constraint Addition**
   ```rust
   pub fn constrain_eq(&mut self, lhs: &FieldVariable, rhs: &FieldVariable) {
       let diff = lhs - rhs;
       let saved = diff.save();
       // Constraint: saved expression must equal zero
   }
   ```

### FieldVariable Operations

Arithmetic operations create new symbolic expressions:

```rust
impl Add for &FieldVariable {
    type Output = FieldVariable;
    
    fn add(self, rhs: Self) -> Self::Output {
        FieldVariable {
            expr: SymbolicExpr::Add(
                Box::new(self.expr.clone()),
                Box::new(rhs.expr.clone())
            ),
            limb_max_abs: self.limb_max_abs + rhs.limb_max_abs,
            max_overflow_bits: calculate_new_overflow_bits(...),
            ...
        }
    }
}
```

### Constraint Generation

When saving a variable, the builder:

1. **Computes Quotient and Remainder**
   ```rust
   // For expression E and prime P:
   // E = Q * P + R, where R < P
   let (quotient, remainder) = compute_division(&expr_value, &prime);
   ```

2. **Generates Carry Constraints**
   ```rust
   // Propagate carries to ensure each limb fits in field
   let carries = compute_carries(&overflow_limbs, limb_bits);
   add_carry_constraints(&carries);
   ```

3. **Creates Check-to-Zero Constraint**
   ```rust
   // Verify: E - saved_var - Q * P = 0
   let constraint = expr - var - quotient * prime;
   check_carry_to_zero(constraint);
   ```

### AIR Integration

The `FieldExpressionCoreAir` implements the AIR trait:

```rust
impl<AB: AirBuilder> Air<AB> for FieldExpressionCoreAir {
    fn eval(&self, builder: &mut AB) {
        // Load inputs and variables
        let inputs = load_inputs(builder);
        let vars = load_variables(builder);
        
        // Evaluate constraints
        for (expr, expected) in &self.constraints {
            let computed = expr.evaluate(&inputs, &vars);
            builder.assert_eq(computed, expected);
        }
        
        // Range check interactions
        send_range_checks(builder);
    }
}
```

## Advanced Patterns

### Conditional Execution

The `Select` expression enables conditional logic:

```rust
let result = builder.select(
    flag_idx,
    &true_branch,   // When flag = 1
    &false_branch   // When flag = 0
);
```

Implementation uses arithmetic selection:
```
result = flag * true_branch + (1 - flag) * false_branch
```

### Multi-Opcode Chips

Support for multiple operations in one chip:

```rust
pub struct MultiOpChip {
    ops: Vec<Operation>,
    opcode_flags: Vec<usize>,
}

// During execution
let active_op = determine_active_operation(opcode);
let result = execute_operation(active_op, inputs);
```

### Optimizations

1. **Expression Flattening**
   - Flatten nested additions/multiplications
   - Combine integer operations
   - Reduce expression depth

2. **Lazy Evaluation**
   - Build symbolic expressions without computing values
   - Evaluate only when needed for constraints
   - Share common subexpressions

3. **Carry Optimization**
   - Batch carry computations
   - Minimize range check calls
   - Reuse carry columns

## Error Handling

Common error conditions:

1. **Overflow Errors**
   ```rust
   assert!(
       limb_value.bits() <= max_field_bits,
       "Limb overflow: value too large for field"
   );
   ```

2. **Division by Zero**
   ```rust
   if divisor.is_zero() {
       panic!("Division by zero in field operation");
   }
   ```

3. **Invalid Configuration**
   ```rust
   assert!(
       modulus.bits() <= num_limbs * limb_bits,
       "Modulus too large for limb configuration"
   );
   ```

## Performance Tuning

### Limb Size Selection

Trade-offs:
- Smaller limbs (e.g., 8 bits):
  - ✓ Simpler range checks
  - ✓ Fits multiplication in native field
  - ✗ More limbs needed
  - ✗ More carry propagation

- Larger limbs (e.g., 16 bits):
  - ✓ Fewer limbs
  - ✓ Less carry propagation
  - ✗ Complex range checking
  - ✗ Risk of field overflow

### Memory Layout

Optimize trace layout:
```rust
// Group related values
[input_limbs][var_limbs][carry_values][range_checks]

// Minimize padding
align_to_power_of_two(total_columns)
```

### Constraint Reduction

Strategies:
1. Share common subexpressions
2. Combine similar constraints
3. Optimize carry patterns
4. Batch range checks

## Testing Strategies

### Unit Testing

Test individual components:
```rust
#[test]
fn test_field_addition() {
    let (a, b) = generate_random_field_elements();
    let c = &a + &b;
    assert_eq!(c.evaluate(), (a.evaluate() + b.evaluate()) % prime);
}
```

### Integration Testing

Test full constraint system:
```rust
#[test]
fn test_constraint_satisfaction() {
    let builder = setup_builder();
    add_constraints(&mut builder);
    let trace = generate_trace(&builder);
    verify_constraints(&trace);
}
```

### Fuzzing

Properties to check:
- Field arithmetic laws (associativity, commutativity)
- Overflow handling correctness
- Constraint satisfaction
- Range bound compliance