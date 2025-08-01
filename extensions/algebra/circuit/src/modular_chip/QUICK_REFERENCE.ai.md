# Modular Chip Quick Reference

## Chip Instantiation

### Basic Setup
```rust
use openvm_algebra_circuit::modular_chip::*;
use openvm_circuit_primitives::var_range::SharedVariableRangeCheckerChip;

// Create shared range checker
let range_checker = SharedVariableRangeCheckerChip::new(bus);

// Create modular add/sub chip
let modular_addsub = ModularAddSubChip::new(
    adapter,
    config,
    offset,
    range_checker.clone(),
    offline_memory.clone(),
);
```

## Operation Opcodes

### Arithmetic Operations
```rust
// From openvm_algebra_transpiler::Rv32ModularArithmeticOpcode
ADD         // Modular addition
SUB         // Modular subtraction  
MUL         // Modular multiplication
DIV         // Modular division
IS_EQ       // Modular equality check

// Setup operations (must be called first)
SETUP_ADDSUB  // Initialize for add/sub
SETUP_MULDIV  // Initialize for mul/div
SETUP_ISEQ    // Initialize for equality
```

## Expression Builder Patterns

### Addition/Subtraction
```rust
let x1 = ExprBuilder::new_input(builder.clone());
let x2 = ExprBuilder::new_input(builder.clone());
let sum = x1.clone() + x2.clone();
let diff = x1.clone() - x2.clone();
let result = FieldVariable::select(is_add_flag, &sum, &diff);
result.save_output();
```

### Multiplication/Division  
```rust
let x = ExprBuilder::new_input(builder.clone());
let y = ExprBuilder::new_input(builder.clone());
let (z_idx, z) = builder.borrow_mut().new_var();

// Constraint: x * y = z (mul) or z * y = x (div)
let lvar = FieldVariable::select(is_mul_flag, &x, &z);
let rvar = FieldVariable::select(is_mul_flag, &z, &x);
let constraint = lvar * y - rvar;
```

## Common Patterns

### Multi-Modulus Support
```rust
// Each modulus gets unique offset
let mod_idx = 2; // Third modulus
let offset = base_offset + mod_idx * Rv32ModularArithmeticOpcode::COUNT;
```

### Setup Before Use
```rust
// Always setup before arithmetic
vm.execute(SETUP_ADDSUB, modulus_addr);
vm.execute(ADD, result_addr, a_addr, b_addr);
```

### Range Checking
```rust
// Automatic for all operations
self.bitwise_lookup_chip.request_range(
    modulus[idx] - value[idx] - 1,
    expected_range,
);
```

## Constraint Helpers

### Less Than Modulus Check
```rust
// Used in IS_EQ operation
let (is_less, diff_idx) = run_unsigned_less_than(&value, &modulus_limbs);
assert!(is_less, "Value must be < modulus");
```

### Lt_marker Values
```rust
// Marker array for modulus comparison
lt_marker[i] = match i {
    i if i == b_diff_idx => 1,
    i if i == c_diff_idx => c_lt_mark,
    _ => 0,
};
```

## Type Aliases

### Standard Configurations
```rust
// From mod.rs
type ModularIsEqualChip<F, const NUM_LANES, const LANE_SIZE, const TOTAL_LIMBS> = 
    VmChipWrapper<
        F,
        Rv32IsEqualModAdapterChip<F, 2, NUM_LANES, LANE_SIZE, TOTAL_LIMBS>,
        ModularIsEqualCoreChip<TOTAL_LIMBS, RV32_REGISTER_NUM_LIMBS, RV32_CELL_BITS>,
    >;
```

## Error Handling

### Common Assertions
```rust
// Value must be less than modulus
assert!(b_cmp, "{:?} >= {:?}", b, self.air.modulus_limbs);

// Invalid setup configuration  
panic!("SETUP_ISEQ is not valid for rd = x0");

// Division by zero handled by guest
```

## Memory Layout

### Register Mapping
```rust
// RV32 register to memory offset
let offset = RV32_REGISTER_NUM_LIMBS * register_idx;
```

### Limb Organization
```rust
// Little-endian limb storage
// limbs[0] = least significant
// limbs[N-1] = most significant
```

## Testing Snippets

### Basic Test
```rust
#[test]
fn test_modular_add() {
    let p = BigUint::from_str("0xffffffff00000001").unwrap();
    let chip = setup_test_chip(p);
    
    // Test: (p-1) + 2 = 1 mod p
    let result = chip.add(p - 1u32, 2u32);
    assert_eq!(result, BigUint::from(1u32));
}
```

### Edge Cases
```rust
// Always test:
// - Zero operands
// - Modulus - 1
// - Overflow cases  
// - Identity operations (x + 0, x * 1, x / 1)
```

## Performance Tips

- Use shared range checkers across chips
- Batch operations when possible
- Choose appropriate limb sizes for modulus
- Reuse expression builders for similar operations
- Minimize constraint complexity in hot paths