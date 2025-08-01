# JAL/LUI Implementation Guide

## Overview
This guide provides detailed implementation guidance for the JAL/LUI component in OpenVM's RV32IM extension.

## Architecture Decisions

### Why Combined Implementation?
The JAL and LUI instructions share:
- Similar register write patterns
- Common immediate handling infrastructure
- Overlapping constraint systems

Combining them reduces:
- Code duplication by ~60%
- Chip count in the VM
- Maintenance overhead

### Key Design Patterns

#### 1. Limb-Based Representation
```rust
// 32-bit value split into 4 x 8-bit limbs
pub rd_data: [T; RV32_REGISTER_NUM_LIMBS]  // RV32_REGISTER_NUM_LIMBS = 4
```

Benefits:
- Efficient field arithmetic
- Natural range checking boundaries
- Optimized for 8-bit lookup tables

#### 2. Boolean Instruction Selection
```rust
pub is_jal: T,
pub is_lui: T,
// Constraint: is_jal + is_lui = 1
```

This pattern allows shared constraints while maintaining distinct behaviors.

## Implementation Walkthrough

### Core Execution Logic

#### JAL Implementation
```rust
// 1. Calculate return address (PC + 4)
let rd_data = array::from_fn(|i| {
    ((pc + DEFAULT_PC_STEP) >> (8 * i)) & 0xFF
});

// 2. Calculate jump target
let next_pc = (pc as i32 + imm) as u32;  // Sign-extended immediate
```

Key considerations:
- Return address saved before jump
- Immediate is sign-extended 21-bit value
- PC wraparound handled naturally

#### LUI Implementation
```rust
// 1. Shift immediate left by 12 bits
let rd = (imm as u32) << 12;

// 2. Decompose into limbs
let rd_data = array::from_fn(|i| {
    (rd >> (8 * i)) & 0xFF
});

// 3. Increment PC
let next_pc = pc + 4;
```

Key considerations:
- Lower 12 bits always zero
- No sign extension needed
- Simple PC advancement

### Constraint System Design

#### Validity Constraints
```rust
builder.assert_bool(is_lui);
builder.assert_bool(is_jal);
builder.assert_bool(is_lui + is_jal);  // Exactly one
```

#### JAL-Specific Constraints
```rust
// 1. Return address composition
builder.when(is_jal).assert_eq(
    composed_rd_value,
    from_pc + DEFAULT_PC_STEP
);

// 2. PC bits limit (last limb special handling)
let last_limb_bits = PC_BITS - 8 * 3;  // Typically 24 - 24 = 0
```

#### LUI-Specific Constraints
```rust
// 1. First limb must be zero (12-bit shift)
builder.when(is_lui).assert_zero(rd[0]);

// 2. Upper limbs composition
builder.when(is_lui).assert_eq(
    composed_upper_limbs,
    imm * (1 << 12)
);
```

## Advanced Topics

### Immediate Encoding

#### JAL Format (J-type)
```
31                                 12 11      0
[imm[20|10:1|11|19:12]              ][rd     ][opcode]
```
- Bit 0 is implicit (always 0 for alignment)
- Bits rearranged for hardware efficiency
- Sign-extended to 32 bits

#### LUI Format (U-type)
```
31                   12 11      7 6      0
[imm[31:12]           ][rd      ][opcode]
```
- Upper 20 bits of immediate
- Lower 12 bits implicitly zero

### Range Checking Strategy

The component uses paired range checking:
```rust
for i in 0..RV32_REGISTER_NUM_LIMBS / 2 {
    self.bus.send_range(
        rd[i * 2], 
        rd[i * 2 + 1]
    ).eval(builder, is_valid);
}
```

This reduces lookup table interactions by 50%.

### PC Overflow Handling

For JAL with PC near maximum:
```rust
// Example: PC = 0xFFFFF0, imm = 0x20
// Result: 0x10 (wraparound)
let next_pc = pc.wrapping_add(imm as u32);
```

## Testing Strategies

### Essential Test Cases

1. **Boundary Tests**
   ```rust
   // Maximum positive JAL offset
   test_jal(pc: 0x1000, imm: 0xFFFFF);
   
   // Maximum negative JAL offset  
   test_jal(pc: 0x100000, imm: -0x100000);
   
   // LUI with all bits set
   test_lui(imm: 0xFFFFF);
   ```

2. **Edge Cases**
   ```rust
   // JAL to self (infinite loop)
   test_jal(pc: 0x1000, imm: 0);
   
   // LUI with zero (register clear)
   test_lui(imm: 0);
   ```

3. **Constraint Violations**
   ```rust
   // Both flags set
   test_negative(is_jal: true, is_lui: true);
   
   // Invalid limb values
   test_negative(rd_data: [256, 0, 0, 0]);
   ```

### Debugging Tips

1. **Immediate Decode Issues**
   - Print raw immediate value
   - Check sign extension logic
   - Verify bit shifting

2. **Constraint Failures**
   - Enable debug builder for detailed errors
   - Check limb decomposition
   - Verify range checking calls

3. **PC Calculation Errors**
   - Log before/after PC values
   - Check for overflow conditions
   - Verify immediate encoding

## Performance Optimization

### Current Optimizations
1. Shared bitwise lookup chip
2. Paired range checking
3. Combined instruction implementation

### Potential Improvements
1. Batch multiple instructions per row
2. Optimize carry propagation
3. Reduce boolean constraint count

### Benchmarking
Key metrics to track:
- Constraint count per instruction
- Lookup table interactions
- Trace generation time
- Proof generation time

## Integration Guide

### Adding to VM
```rust
// In VM builder
let jal_lui_chip = Rv32JalLuiChip::new(
    adapter,
    core_chip,
    offline_memory
);
executor.register_chip(jal_lui_chip);
```

### Transpiler Integration
Ensure transpiler maps:
- RISC-V JAL → `Rv32JalLuiOpcode::JAL`
- RISC-V LUI → `Rv32JalLuiOpcode::LUI`

### Memory System
The component writes through the adapter:
- Handles x0 special case
- Manages memory bridge interactions
- Coordinates with program bus

## Common Modifications

### Adding New Instruction
To add a new instruction to this component:

1. Add boolean flag to `Rv32JalLuiCoreCols`
2. Update validity constraint
3. Implement execution logic
4. Add specific constraints
5. Update trace generation
6. Add comprehensive tests

### Changing Limb Size
To modify from 8-bit to different limb size:

1. Update `RV32_CELL_BITS` constant
2. Adjust range checking logic
3. Update limb decomposition
4. Modify bitwise lookup configuration
5. Revalidate all constraints

## Troubleshooting

### Common Issues

1. **"Constraint evaluation failed"**
   - Check boolean flag combinations
   - Verify limb values < 256
   - Ensure proper range checking

2. **"PC overflow detected"**
   - Use wrapping arithmetic
   - Check PC_BITS limit
   - Verify immediate sign extension

3. **"Invalid opcode"**
   - Check transpiler mapping
   - Verify opcode offset
   - Ensure proper registration

### Debug Techniques

1. **Trace Inspection**
   ```rust
   println!("JAL trace: {:?}", core_cols);
   ```

2. **Constraint Verification**
   ```rust
   // Add assertions in execute_instruction
   assert!(rd_data.iter().all(|&x| x < 256));
   ```

3. **Step Debugging**
   - Set breakpoints in `execute_instruction`
   - Log intermediate calculations
   - Verify against RISC-V spec

## References

### Specifications
- RISC-V ISA Manual Vol 1, Ch 2.5 (JAL), 2.4 (LUI)
- OpenVM Architecture Documentation
- zkVM Circuit Primitives Guide

### Related Code
- `openvm_rv32im_transpiler::Rv32JalLuiOpcode`
- `openvm_circuit::arch::VmCoreChip`
- `openvm_circuit_primitives::bitwise_op_lookup`

### Examples
- Similar patterns in AUIPC implementation
- Branch instruction constraint systems
- Other RV32IM instruction implementations