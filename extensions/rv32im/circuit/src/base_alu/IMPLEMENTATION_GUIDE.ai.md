# BaseAlu Implementation Guide

## Overview

This guide walks through implementing ALU operations in the OpenVM framework, using the BaseAlu component as a reference implementation.

## Core Components

### 1. Define the Column Structure

```rust
#[repr(C)]
#[derive(AlignedBorrow)]
pub struct BaseAluCoreCols<T, const NUM_LIMBS: usize, const LIMB_BITS: usize> {
    pub a: [T; NUM_LIMBS],    // Result
    pub b: [T; NUM_LIMBS],    // First operand
    pub c: [T; NUM_LIMBS],    // Second operand
    
    // Operation flags (exactly one should be true)
    pub opcode_add_flag: T,
    pub opcode_sub_flag: T,
    pub opcode_xor_flag: T,
    pub opcode_or_flag: T,
    pub opcode_and_flag: T,
}
```

### 2. Implement the AIR Constraints

```rust
impl<AB, I, const NUM_LIMBS: usize, const LIMB_BITS: usize> VmCoreAir<AB, I>
    for BaseAluCoreAir<NUM_LIMBS, LIMB_BITS>
{
    fn eval(&self, builder: &mut AB, local_core: &[AB::Var], _from_pc: AB::Var) 
        -> AdapterAirContext<AB::Expr, I> 
    {
        // 1. Validate operation flags
        let is_valid = validate_flags(builder, flags);
        
        // 2. Handle arithmetic operations with carry
        handle_arithmetic_ops(builder, cols, carry_divide);
        
        // 3. Validate through bitwise lookups
        validate_bitwise_ops(builder, cols, self.bus);
        
        // 4. Return adapter context
        AdapterAirContext { reads, writes, instruction }
    }
}
```

### 3. Implement Execution Logic

```rust
impl<F, I, const NUM_LIMBS: usize, const LIMB_BITS: usize> VmCoreChip<F, I>
    for BaseAluCoreChip<NUM_LIMBS, LIMB_BITS>
{
    fn execute_instruction(&self, instruction: &Instruction<F>, _from_pc: u32, reads: I::Reads) 
        -> Result<(AdapterRuntimeContext<F, I>, Self::Record)> 
    {
        // 1. Decode opcode
        let local_opcode = BaseAluOpcode::from_usize(opcode.local_opcode_idx(self.air.offset));
        
        // 2. Extract operands
        let data: [[F; NUM_LIMBS]; 2] = reads.into();
        
        // 3. Execute operation
        let a = run_alu::<NUM_LIMBS, LIMB_BITS>(local_opcode, &b, &c);
        
        // 4. Request bitwise lookups
        self.request_lookups(local_opcode, &a, &b, &c);
        
        // 5. Return result and record
        Ok((output, record))
    }
}
```

## Implementation Patterns

### Carry Propagation Pattern

```rust
// For addition with carry
for i in 0..NUM_LIMBS {
    carry[i] = carry_divide * (b[i] + c[i] - a[i] + prev_carry);
    builder.when(opcode_add_flag).assert_bool(carry[i]);
}
```

### Bitwise Operation Pattern

```rust
// For XOR/OR/AND operations
let x_xor_y = match opcode {
    XOR => a[i],
    OR => 2 * a[i] - b[i] - c[i],    // a = b | c
    AND => b[i] + c[i] - 2 * a[i],   // a = b & c
};
self.bus.send_xor(x, y, x_xor_y).eval(builder, is_valid);
```

### Range Checking Pattern

```rust
// For ADD/SUB - range check results
if is_arithmetic_op {
    for limb in result {
        bitwise_chip.request_xor(limb, limb); // Self-XOR for range check
    }
}
```

## Testing Strategy

### 1. Positive Tests
```rust
fn test_alu_operation(opcode: BaseAluOpcode) {
    // Generate random inputs
    // Execute operation
    // Verify constraints pass
}
```

### 2. Negative Tests
```rust
fn test_invalid_result() {
    // Create invalid trace
    // Modify result values
    // Verify constraints fail with expected error
}
```

### 3. Integration Tests
```rust
fn test_with_memory() {
    // Setup memory controller
    // Execute through full pipeline
    // Verify memory updates
}
```

## Common Extensions

### Adding New Operations

1. Add opcode to enum:
```rust
pub enum BaseAluOpcode {
    // Existing...
    NEW_OP,
}
```

2. Add flag to columns:
```rust
pub struct BaseAluCoreCols {
    // Existing...
    pub opcode_new_op_flag: T,
}
```

3. Implement constraints:
```rust
// In eval()
handle_new_op(builder, cols);
```

4. Implement execution:
```rust
// In run_alu()
BaseAluOpcode::NEW_OP => run_new_op(x, y),
```

### Optimizing for Different Limb Sizes

- Adjust `NUM_LIMBS` and `LIMB_BITS` constants
- Update carry computation logic
- Ensure bitwise lookup compatibility
- Maintain total bit width (NUM_LIMBS * LIMB_BITS)

## Debugging Tips

1. **Constraint Failures**: Check carry propagation first
2. **Interaction Errors**: Verify bitwise lookup requests
3. **Wrong Results**: Trace through limb-by-limb computation
4. **Range Issues**: Ensure all values < 2^LIMB_BITS

## Performance Considerations

1. **Minimize Constraint Degree**: Keep at 3 or below
2. **Batch Lookups**: Request all in single operation
3. **Optimize Column Layout**: Group related data
4. **Reuse Computations**: Share common subexpressions