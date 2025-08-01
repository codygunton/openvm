# LessThan Implementation Guide

## Overview

This guide walks through implementing comparison operations in the OpenVM framework, using the LessThan component as a reference implementation for SLT (signed less than) and SLTU (unsigned less than) operations.

## Core Components

### 1. Define the Column Structure

```rust
#[repr(C)]
#[derive(AlignedBorrow)]
pub struct LessThanCoreCols<T, const NUM_LIMBS: usize, const LIMB_BITS: usize> {
    pub b: [T; NUM_LIMBS],    // First operand
    pub c: [T; NUM_LIMBS],    // Second operand
    pub cmp_result: T,        // Comparison result (0 or 1)
    
    // Operation flags (exactly one should be true)
    pub opcode_slt_flag: T,   // Signed comparison
    pub opcode_sltu_flag: T,  // Unsigned comparison
    
    // MSB handling for sign
    pub b_msb_f: T,          // Field representation of b's MSB
    pub c_msb_f: T,          // Field representation of c's MSB
    
    // Difference tracking
    pub diff_marker: [T; NUM_LIMBS],  // Marks first differing limb
    pub diff_val: T,                  // Absolute difference value
}
```

### 2. Implement the AIR Constraints

```rust
impl<AB, I, const NUM_LIMBS: usize, const LIMB_BITS: usize> VmCoreAir<AB, I>
    for LessThanCoreAir<NUM_LIMBS, LIMB_BITS>
{
    fn eval(&self, builder: &mut AB, local_core: &[AB::Var], _from_pc: AB::Var) 
        -> AdapterAirContext<AB::Expr, I> 
    {
        // 1. Validate operation flags
        let is_valid = validate_opcode_flags(builder, flags);
        
        // 2. Validate MSB field representations
        validate_msb_values(builder, cols);
        
        // 3. Enforce difference marker constraints
        enforce_difference_markers(builder, cols);
        
        // 4. Range check MSB and difference values
        perform_range_checks(builder, cols, self.bus);
        
        // 5. Return adapter context with result
        AdapterAirContext { reads, writes, instruction }
    }
}
```

### 3. Implement Execution Logic

```rust
impl<F, I, const NUM_LIMBS: usize, const LIMB_BITS: usize> VmCoreChip<F, I>
    for LessThanCoreChip<NUM_LIMBS, LIMB_BITS>
{
    fn execute_instruction(&self, instruction: &Instruction<F>, _from_pc: u32, reads: I::Reads) 
        -> Result<(AdapterRuntimeContext<F, I>, Self::Record)> 
    {
        // 1. Decode opcode
        let less_than_opcode = LessThanOpcode::from_usize(opcode.local_opcode_idx(self.air.offset));
        
        // 2. Extract operands
        let data: [[F; NUM_LIMBS]; 2] = reads.into();
        let b = data[0].map(|x| x.as_canonical_u32());
        let c = data[1].map(|y| y.as_canonical_u32());
        
        // 3. Execute comparison
        let (cmp_result, diff_idx, b_sign, c_sign) = 
            run_less_than::<NUM_LIMBS, LIMB_BITS>(less_than_opcode, &b, &c);
        
        // 4. Handle MSB for signed/unsigned
        let (b_msb_f, c_msb_f) = compute_msb_fields(b, c, b_sign, c_sign);
        
        // 5. Request range checks
        self.request_range_checks(less_than_opcode, b_msb_f, c_msb_f, diff_val);
        
        // 6. Return result and record
        Ok((output, record))
    }
}
```

## Implementation Patterns

### Comparison Algorithm Pattern

```rust
pub fn run_less_than<const NUM_LIMBS: usize, const LIMB_BITS: usize>(
    opcode: LessThanOpcode,
    x: &[u32; NUM_LIMBS],
    y: &[u32; NUM_LIMBS],
) -> (bool, usize, bool, bool) {
    // Determine sign for MSB interpretation
    let x_sign = (x[NUM_LIMBS - 1] >> (LIMB_BITS - 1) == 1) && opcode == LessThanOpcode::SLT;
    let y_sign = (y[NUM_LIMBS - 1] >> (LIMB_BITS - 1) == 1) && opcode == LessThanOpcode::SLT;
    
    // Find first differing limb from MSB to LSB
    for i in (0..NUM_LIMBS).rev() {
        if x[i] != y[i] {
            // Compare with sign consideration
            return ((x[i] < y[i]) ^ x_sign ^ y_sign, i, x_sign, y_sign);
        }
    }
    
    // Equal values
    (false, NUM_LIMBS, x_sign, y_sign)
}
```

### MSB Field Representation Pattern

```rust
// For signed values (SLT)
if is_negative {
    msb_f = -F::from_canonical_u32((1 << LIMB_BITS) - msb_value);
    msb_range = msb_value - (1 << (LIMB_BITS - 1));
} else {
    msb_f = F::from_canonical_u32(msb_value);
    msb_range = msb_value + (slt_flag << (LIMB_BITS - 1));
}
```

### Difference Marker Constraints Pattern

```rust
// Ensure exactly one marker or none
let mut prefix_sum = AB::Expr::ZERO;
for i in (0..NUM_LIMBS).rev() {
    let diff = compute_diff_at_limb(i, cols);
    prefix_sum += marker[i].into();
    
    // Marker can only be 1 if no previous marker
    builder.assert_zero(not(prefix_sum.clone()) * diff.clone());
    
    // If marker is 1, difference must match diff_val
    builder.when(marker[i]).assert_eq(cols.diff_val, diff);
}
```

## Testing Strategy

### 1. Basic Comparison Tests
```rust
fn test_comparison(opcode: LessThanOpcode, x: u32, y: u32, expected: bool) {
    // Setup operands
    // Execute comparison
    // Verify result matches expected
}
```

### 2. Edge Case Tests
```rust
fn test_edge_cases() {
    // Equal values → result = 0
    // Maximum vs minimum values
    // Sign boundary cases (127 vs -128)
    // Single bit differences
}
```

### 3. Constraint Validation Tests
```rust
fn test_invalid_traces() {
    // Wrong difference marker placement
    // Invalid MSB range
    // Incorrect comparison result
    // Multiple markers set
}
```

## Common Extensions

### Adding Greater Than Operations

1. Add opcodes:
```rust
pub enum ComparisonOpcode {
    SLT, SLTU,
    SGT, SGTU,  // New greater than
}
```

2. Modify comparison logic:
```rust
// For greater than, swap operands
let (result, idx, _, _) = match opcode {
    SGT | SGTU => run_less_than(opcode.to_lt(), y, x),
    _ => run_less_than(opcode, x, y),
};
```

### Supporting Different Word Sizes

- Adjust `NUM_LIMBS` for different architectures
- Maintain sign bit position awareness
- Update range check bounds accordingly

## Debugging Tips

1. **Wrong Comparison Result**: 
   - Check sign interpretation for SLT
   - Verify limb ordering (MSB first)
   - Trace difference detection logic

2. **Constraint Failures**:
   - Verify exactly one diff_marker is set
   - Check MSB range for signed/unsigned
   - Ensure diff_val matches actual difference

3. **Range Check Errors**:
   - MSB + 128 for signed comparisons
   - diff_val - 1 must be valid
   - All values must fit in LIMB_BITS

## Performance Considerations

1. **Early Exit**: Stop at first difference
2. **Minimal Range Checks**: Only check what's needed
3. **Efficient Sign Handling**: Precompute sign bits
4. **Batch Lookups**: Request all range checks together