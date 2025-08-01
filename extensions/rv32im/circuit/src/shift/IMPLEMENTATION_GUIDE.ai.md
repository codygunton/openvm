# Shift Component - Implementation Guide

## Adding a New Shift Operation

### 1. Define the Opcode
```rust
// In openvm_rv32im_transpiler
pub enum ShiftOpcode {
    SLL = 0,
    SRL = 1, 
    SRA = 2,
    // Add new opcode here
    NEW_SHIFT = 3,
}
```

### 2. Update Core Implementation
```rust
// In core.rs - Update run_shift dispatch
pub(super) fn run_shift<const NUM_LIMBS: usize, const LIMB_BITS: usize>(
    opcode: ShiftOpcode,
    x: &[u32; NUM_LIMBS],
    y: &[u32; NUM_LIMBS],
) -> ([u32; NUM_LIMBS], usize, usize) {
    match opcode {
        ShiftOpcode::SLL => run_shift_left::<NUM_LIMBS, LIMB_BITS>(x, y),
        ShiftOpcode::SRL => run_shift_right::<NUM_LIMBS, LIMB_BITS>(x, y, true),
        ShiftOpcode::SRA => run_shift_right::<NUM_LIMBS, LIMB_BITS>(x, y, false),
        ShiftOpcode::NEW_SHIFT => run_new_shift::<NUM_LIMBS, LIMB_BITS>(x, y),
    }
}
```

### 3. Add Constraint Flags
```rust
// In ShiftCoreCols - Add new flag
pub struct ShiftCoreCols<T, const NUM_LIMBS: usize, const LIMB_BITS: usize> {
    // ... existing fields ...
    pub opcode_new_shift_flag: T,
}
```

### 4. Update AIR Constraints
```rust
// In eval() method
let flags = [
    cols.opcode_sll_flag,
    cols.opcode_srl_flag,
    cols.opcode_sra_flag,
    cols.opcode_new_shift_flag, // Add new flag
];

// Add operation-specific constraints
let mut when_new_shift = builder.when(cols.opcode_new_shift_flag);
// Define constraints for new operation
```

## Implementing Custom Shift Logic

### Basic Pattern
```rust
fn run_new_shift<const NUM_LIMBS: usize, const LIMB_BITS: usize>(
    x: &[u32; NUM_LIMBS],
    y: &[u32; NUM_LIMBS],
) -> ([u32; NUM_LIMBS], usize, usize) {
    let mut result = [0u32; NUM_LIMBS];
    
    // 1. Extract shift amount
    let (limb_shift, bit_shift) = get_shift::<NUM_LIMBS, LIMB_BITS>(y);
    
    // 2. Implement shift logic
    for i in 0..NUM_LIMBS {
        // Custom shift computation
        result[i] = compute_shifted_limb(x, i, limb_shift, bit_shift);
    }
    
    (result, limb_shift, bit_shift)
}
```

## Testing Pattern

### 1. Positive Test
```rust
#[test]
fn rv32_shift_new_rand_test() {
    run_rv32_shift_rand_test(ShiftOpcode::NEW_SHIFT, 100);
}
```

### 2. Negative Test Template
```rust
#[test]
fn rv32_new_shift_wrong_result_negative_test() {
    let a = [/* expected wrong result */];
    let b = [/* input x */];
    let c = [/* input y (shift amount) */];
    let prank_vals = ShiftPrankValues {
        // Prank specific values
        ..Default::default()
    };
    run_rv32_shift_negative_test(ShiftOpcode::NEW_SHIFT, a, b, c, prank_vals, false);
}
```

## Constraint Patterns

### Range Checking Pattern
```rust
// Check value is in valid range
self.range_bus
    .send(value, max_bits)
    .eval(builder, is_valid.clone());
```

### Bitwise Operation Pattern
```rust
// Use bitwise lookup for XOR/AND/OR
self.bitwise_lookup_bus
    .send_xor(a, b, a_xor_b)
    .eval(builder, condition);
```

### Conditional Constraint Pattern
```rust
// Apply constraints only when condition is true
let mut when_condition = builder.when(condition_flag);
when_condition.assert_eq(computed_value, expected_value);
```

## Common Implementation Patterns

### 1. Bit Extraction
```rust
// Extract specific bit from value
let bit_n = (value >> n) & 1;
```

### 2. Sign Extension
```rust
// Extend sign bit to fill value
let sign_bit = value >> (LIMB_BITS - 1);
let sign_extended = if sign_bit == 1 {
    ((1 << LIMB_BITS) - 1)
} else {
    0
};
```

### 3. Modular Arithmetic
```rust
// Keep values within limb bounds
let result = (computed_value) % (1 << LIMB_BITS);
```

## Performance Optimization Tips

1. **Batch Lookups**: Request all bitwise operations before constraints
2. **Reuse Computations**: Store intermediate values to avoid recomputation
3. **Minimize Range Checks**: Only check values that could exceed bounds
4. **Efficient Markers**: Use one-hot encoding for shift markers