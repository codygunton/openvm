# Modular Chip Implementation Guide

This guide provides detailed patterns and examples for implementing and extending modular arithmetic functionality in OpenVM.

## Chip Construction Patterns

### Basic Chip Instantiation
```rust
// Create range checker (shared across chips)
let range_checker = SharedVariableRangeCheckerChip::new(bus);

// Configure expression builder
let config = ExprBuilderConfig {
    modulus: your_modulus.clone(),
    num_limbs: 8,
    limb_bits: 32,
};

// Create adapter for memory interface
let adapter = Rv32VecHeapAdapterChip::new(
    memory_controller,
    mem_addr_offset,
);

// Instantiate modular arithmetic chip
let add_sub_chip = ModularAddSubChip::new(
    adapter,
    config,
    opcode_offset,
    range_checker,
    offline_memory,
);
```

### Multi-Modulus Configuration
```rust
// Support multiple moduli in single VM
for (idx, modulus) in moduli.iter().enumerate() {
    let offset = base_offset + idx * Rv32ModularArithmeticOpcode::COUNT;
    let config = ExprBuilderConfig {
        modulus: modulus.clone(),
        num_limbs: calculate_limbs(modulus),
        limb_bits: 32,
    };
    // Create chips with unique offsets
}
```

## Field Expression Patterns

### Building Arithmetic Expressions
```rust
pub fn create_modular_expression(
    config: ExprBuilderConfig,
    range_bus: VariableRangeCheckerBus,
) -> (FieldExpr, usize, usize) {
    let builder = ExprBuilder::new(config, range_bus.range_max_bits);
    let builder = Rc::new(RefCell::new(builder));
    
    // Create input variables
    let x = ExprBuilder::new_input(builder.clone());
    let y = ExprBuilder::new_input(builder.clone());
    
    // Build expression tree
    let sum = x.clone() + y.clone();
    let diff = x.clone() - y.clone();
    
    // Create selection flags
    let is_add = builder.borrow_mut().new_flag();
    let is_sub = builder.borrow_mut().new_flag();
    
    // Conditional selection
    let result = FieldVariable::select(is_add, &sum, &diff);
    result.save_output();
    
    let builder = builder.borrow().clone();
    (FieldExpr::new(builder, range_bus, true), is_add, is_sub)
}
```

### Constraint Implementation
```rust
// For division: constraint is x * y = z or z * y = x
let (z_idx, z) = builder.borrow_mut().new_var();
let z = FieldVariable::from_var(builder.clone(), z);

// Build constraint based on operation
let lvar = FieldVariable::select(is_mul_flag, &x, &z);
let rvar = FieldVariable::select(is_mul_flag, &z, &x);
let constraint = lvar * y.clone() - rvar;

// Set constraint and compute function
builder.borrow_mut().set_constraint(z_idx, constraint.expr);
builder.borrow_mut().set_compute(z_idx, compute_expr);
```

## Modular Equality Implementation

### Core Algorithm Structure
```rust
impl ModularIsEqualCoreAir {
    fn eval(&self, builder: &mut AB, local_core: &[AB::Var]) {
        let cols: &ModularIsEqualCoreCols<_, READ_LIMBS> = local_core.borrow();
        
        // 1. Verify equality/inequality based on cmp_result
        let eq_io = IsEqArrayIo {
            x: cols.b.map(Into::into),
            y: cols.c.map(Into::into),
            out: cols.cmp_result.into(),
            condition: cols.is_valid - cols.is_setup,
        };
        self.subair.eval(builder, (eq_io, cols.eq_marker));
        
        // 2. Constrain both values < modulus
        self.constrain_less_than_modulus(builder, cols);
        
        // 3. Range check difference values
        self.bus.send_range(
            cols.b_lt_diff - AB::Expr::ONE,
            cols.c_lt_diff - AB::Expr::ONE,
        ).eval(builder, cols.is_valid - cols.is_setup);
    }
}
```

### Lt_marker Pattern
```rust
// lt_marker[i] indicates relationship to modulus at limb i
// - 0: No special relationship
// - 1: b[i] < N[i] (b differs from N here)
// - 2: c[i] < N[i] (c differs from N here)

for i in (0..READ_LIMBS).rev() {
    prefix_sum += cols.lt_marker[i];
    
    // Ensure lt_marker values are valid
    builder.assert_zero(
        cols.lt_marker[i] 
        * (cols.lt_marker[i] - AB::F::ONE)
        * (cols.lt_marker[i] - cols.c_lt_mark)
    );
    
    // Constrain based on prefix sum
    builder
        .when_ne(prefix_sum.clone(), expected_value)
        .assert_eq(cols.b[i], modulus[i]);
}
```

## Execution Flow

### Setup Operations
```rust
// Setup operations initialize modulus without performing arithmetic
match opcode {
    SETUP_ADDSUB => {
        // Configure modulus for add/sub operations
        core.initialize_modulus(instruction.operands);
    }
    SETUP_MULDIV => {
        // Configure modulus for mul/div operations
        // May precompute modular inverse tables
    }
    SETUP_ISEQ => {
        // Configure modulus for equality checking
        // Validates modulus is properly formed
    }
}
```

### Runtime Execution
```rust
fn execute_instruction(&self, instruction: &Instruction<F>) -> Result<...> {
    let data: [[F; READ_LIMBS]; 2] = reads.into();
    
    match instruction.opcode {
        ADD => {
            let result = (data[0] + data[1]) % modulus;
            range_check_result(result);
        }
        MUL => {
            let result = (data[0] * data[1]) % modulus;
            // Handle overflow via witness hints
        }
        IS_EQ => {
            let (b_lt_n, b_idx) = check_less_than(data[0], modulus);
            let (c_lt_n, c_idx) = check_less_than(data[1], modulus);
            assert!(b_lt_n && c_lt_n);
            let equal = data[0] == data[1];
        }
    }
}
```

## Advanced Patterns

### Witness Generation for Division
```rust
// Division requires computing modular inverse
fn compute_division_witness(x: &[F], y: &[F], modulus: &BigUint) -> Vec<F> {
    let x_big = limbs_to_biguint(x);
    let y_big = limbs_to_biguint(y);
    
    // Compute y^(-1) mod N
    let y_inv = mod_inverse(&y_big, modulus);
    
    // Result = x * y^(-1) mod N
    let result = (&x_big * &y_inv) % modulus;
    
    biguint_to_limbs(result)
}
```

### Batched Operations
```rust
// Process multiple operations in single trace row
pub struct BatchedModularChip {
    configs: Vec<ExprBuilderConfig>,
    expressions: Vec<FieldExpr>,
}

impl BatchedModularChip {
    fn execute_batch(&mut self, ops: &[ModularOp]) {
        for (op, expr) in ops.iter().zip(&self.expressions) {
            // Execute with appropriate expression
        }
    }
}
```

## Testing Patterns

### Unit Test Structure
```rust
#[test]
fn test_modular_addition() {
    let modulus = BigUint::from_str("0xfffffffffffffffffffffffffffffffe").unwrap();
    let chip = create_test_chip(modulus);
    
    // Test cases near modulus boundary
    let a = modulus.clone() - 1u32;
    let b = BigUint::from(2u32);
    let expected = BigUint::from(1u32); // (N-1) + 2 = 1 mod N
    
    let result = chip.execute_add(a, b);
    assert_eq!(result, expected);
}
```

### Edge Case Testing
```rust
// Always test these cases:
// 1. Zero operands
// 2. Operands equal to modulus - 1  
// 3. Results that wrap around modulus
// 4. Division by 1 and by (modulus - 1)
// 5. Equality of identical values
// 6. Equality of different representations of same value mod N
```

## Performance Optimization

### Limb Configuration
```rust
// Choose limb size based on modulus
fn optimal_limb_config(modulus: &BigUint) -> (usize, usize) {
    let bits = modulus.bits();
    
    if bits <= 256 {
        (32, 8)  // 32-bit limbs, 8 limbs
    } else if bits <= 384 {
        (32, 12) // 32-bit limbs, 12 limbs  
    } else {
        (64, (bits + 63) / 64) // 64-bit limbs
    }
}
```

### Constraint Minimization
```rust
// Share constraints across similar operations
let common_expr = create_base_expression();

let add_expr = common_expr.clone().with_operation(ADD);
let sub_expr = common_expr.clone().with_operation(SUB);

// Reuse range checking logic
let shared_range_checker = SharedVariableRangeCheckerChip::new();
```