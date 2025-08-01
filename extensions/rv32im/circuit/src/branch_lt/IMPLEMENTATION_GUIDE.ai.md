# Branch Less Than Implementation Guide

## Quick Start

### Creating a Branch Less Than Chip
```rust
use openvm_circuit_primitives::bitwise_op_lookup::SharedBitwiseOperationLookupChip;
use openvm_rv32im_circuit::{Rv32BranchLessThanChip, BranchLessThanCoreChip};

// Create bitwise lookup chip (shared across components)
let bitwise_chip = SharedBitwiseOperationLookupChip::<8>::new(bus);

// Create core chip with opcode offset
let core_chip = BranchLessThanCoreChip::<8, 8>::new(bitwise_chip.clone(), opcode_offset);

// Wrap with RV32 adapter
let branch_lt_chip = Rv32BranchLessThanChip::new(adapter, core_chip);
```

## Understanding the Comparison Algorithm

### Signed vs Unsigned Comparison
The key difference is in MSB interpretation:
- **Signed**: MSB bit indicates sign (1 = negative in two's complement)
- **Unsigned**: MSB is just another magnitude bit

### Comparison Process
1. **Find First Difference**: Scan from MSB to LSB for first differing limb
2. **Apply Sign Logic**: For signed ops, consider sign bits
3. **Determine Result**: Compare at difference position considering signedness
4. **Handle Equality**: If all limbs equal, result depends on >= vs < operation

## Implementing Custom Branch Operations

### Step 1: Define Execution Logic
```rust
fn execute_instruction(
    &self,
    instruction: &Instruction<F>,
    from_pc: u32,
    reads: I::Reads,
) -> Result<(AdapterRuntimeContext<F, I>, Self::Record)> {
    // Extract operands
    let [a, b] = reads.into();
    
    // Perform comparison
    let (cmp_result, diff_idx, a_sign, b_sign) = 
        run_cmp(opcode, &a, &b);
    
    // Calculate branch target
    let to_pc = if cmp_result {
        from_pc + instruction.c // Branch taken
    } else {
        from_pc + 4 // Next instruction
    };
    
    // Return context and record
}
```

### Step 2: Define AIR Constraints
Key constraints to implement:
1. **Opcode Validation**: Exactly one opcode flag set
2. **Comparison Logic**: Result matches opcode semantics
3. **MSB Range Checks**: Values in valid signed/unsigned range
4. **Difference Marker**: Exactly one position marked if values differ
5. **PC Update**: Correct branching based on result

### Step 3: Handle Edge Cases
- **Equal Values**: Ensure correct behavior for >= operations
- **Sign Overflow**: Proper two's complement handling
- **All Limbs Equal**: Difference marker should be unmarked

## Constraint Implementation Details

### Critical Constraints
```rust
// 1. Validate single opcode
let is_valid = opcode_flags.sum();
builder.assert_bool(is_valid);

// 2. MSB sign representation
let msb_diff = limb_value - field_value;
builder.assert_zero(msb_diff * (2^LIMB_BITS - msb_diff));

// 3. Difference marker consistency
for i in 0..NUM_LIMBS {
    builder.assert_bool(marker[i]);
    // If prefix_sum = 0, this is first difference
    builder.when_not(prefix_sum).assert_eq(diff_val, b[i] - a[i]);
    prefix_sum += marker[i];
}

// 4. Comparison result
let cmp_lt = /* comparison logic */;
builder.assert_eq(
    cmp_lt,
    cmp_result * is_lt_op + not(cmp_result) * is_ge_op
);
```

## Integration Tips

### With Bitwise Lookup
- Request range checks for MSB values
- Request non-zero check for difference values
- Batch requests for efficiency

### With Instruction Decoder
- Map local opcodes to instruction opcodes
- Handle immediate value extraction
- Validate instruction format

### With Program Counter
- Calculate branch target: `pc + immediate`
- Default to `pc + 4` if not branching
- Handle PC overflow edge cases

## Common Pitfalls

1. **Sign Extension**: Ensure MSB properly converted to field element
2. **Opcode Mapping**: Verify offset calculations are correct
3. **Range Boundaries**: Check [-128, 127] for signed, [0, 255] for unsigned
4. **Difference Direction**: Remember comparison direction affects diff_val sign

## Testing Strategies

### Unit Tests
- Test each opcode with known values
- Include boundary cases (0, MAX, MIN)
- Test equal values for all opcodes

### Property Tests
- Random value generation
- Verify against reference implementation
- Check constraint satisfaction

### Integration Tests
- Full instruction execution
- PC update verification
- Trace generation validation

## Performance Optimization

1. **Shared Lookups**: Reuse bitwise chip across operations
2. **Batch Processing**: Group range check requests
3. **Trace Layout**: Optimize column ordering for cache efficiency
4. **Constraint Ordering**: Evaluate cheap constraints first