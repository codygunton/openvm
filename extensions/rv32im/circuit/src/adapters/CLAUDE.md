# RV32IM Circuit Adapters - Claude AI Assistant Instructions

## Overview
When working with the RV32IM Circuit Adapters component, you are implementing the RISC-V 32-bit instruction set within OpenVM's zero-knowledge proof framework. These adapters must correctly implement RISC-V semantics while generating sound constraints for proof generation.

## Key Principles

### 1. RISC-V Specification Compliance
- **ALWAYS** follow the official RISC-V ISA specification
- x0 register must always read as zero and ignore writes
- Sign extension must be correct for all immediate types
- Memory operations must handle alignment properly

### 2. Register Safety
- **NEVER** access registers directly - use `read_rv32_register()`
- Register pointers are in address space 1 (`RV32_REGISTER_AS`)
- Registers are 4 limbs of 8 bits each
- Use `compose()` and `decompose()` for conversions

### 3. Memory Operation Patterns
- LoadStore adapter handles ALL memory operations
- Always batch to 4-byte aligned accesses internally
- Calculate shift amounts for sub-word operations
- Handle sign/zero extension correctly for loads

### 4. Execution State Management
- Always propagate `from_state` to `to_state` correctly
- PC increment is usually `DEFAULT_PC_STEP` (4)
- Branches add immediate offset to current PC
- JALR sets PC to (rs1 + immediate) & ~1

### 5. Constraint Completeness
- Every computation must be fully constrained
- No implicit assumptions about values
- Range check all offsets and shifts
- Verify all memory operations

## Common Implementation Patterns

### When Adding a New RISC-V Instruction

1. **Determine the Adapter Type**:
   - ALU: Register arithmetic/logic operations
   - Branch: Conditional PC modifications
   - JALR: Register-based jumps
   - LoadStore: Memory access
   - Mul: Multiplication/division
   - RdWrite: Immediate loads

2. **Understand Operand Encoding**:
   ```rust
   // Standard operand layout:
   // operands[0:2] - rd (destination register)
   // operands[2:4] - rs1 (source register 1)
   // operands[4:6] - rs2 (source register 2)
   // operands[6:8] - immediate value
   ```

3. **Handle Immediate Values**:
   - I-type: 12-bit sign-extended
   - S-type: 12-bit from two fields
   - B-type: 13-bit (bit 0 always 0)
   - J-type: 21-bit (bit 0 always 0)

## Critical Safety Rules

### Register Operations
```rust
// CORRECT: Use proper read function
let (record, value) = read_rv32_register(memory, F::ONE, rs1_ptr);

// WRONG: Direct memory access
let value = memory.read_raw(rs1_ptr); // NEVER DO THIS
```

### x0 Register Handling
```rust
// CORRECT: Check and skip write for x0
if rd_ptr != F::ZERO {
    memory.write(F::ONE, rd_ptr, decompose(result));
}

// WRONG: Write without checking
memory.write(F::ONE, rd_ptr, decompose(result)); // x0 corruption!
```

### Sign Extension
```rust
// CORRECT: Proper sign extension
let imm_signed = ((imm as i32) << 20) >> 20; // 12-bit sign extend

// WRONG: No sign extension
let imm_signed = imm as i32; // Incorrect for negative immediates
```

## Adapter-Specific Guidelines

### ALU Adapter
- Support both register-register and register-immediate
- Use bitwise lookup tables for AND, OR, XOR
- Check `rs2_as` to determine immediate vs register
- Always write to rd (hardware handles x0)

### Branch Adapter
- Compare rs1 and rs2 based on opcode
- Modify PC only if condition is true
- No register writes ever
- Immediate offset is PC-relative

### JALR Adapter
- Read base address from rs1
- Add immediate offset
- Clear LSB (set to 0)
- Optionally write PC+4 to rd

### LoadStore Adapter
- Most complex adapter - be extra careful
- Calculate effective address: rs1 + immediate
- Determine operation size from opcode
- Handle alignment with shift operations
- Separate read and write paths

### Mul Adapter
- Implement both multiplication and division
- Handle signed/unsigned variants
- Check for division by zero
- Upper 32 bits for MULH variants

### RdWrite Adapter
- Simplest adapter - no reads
- LUI: immediate << 12
- AUIPC: PC + (immediate << 12)
- Direct immediate writes

## Common Pitfalls to Avoid

### 1. Incorrect Immediate Decoding
```rust
// WRONG: Forgetting immediate is split in S-type
let imm = instruction.operands[6];

// CORRECT: Reconstruct from pieces
let imm = reconstruct_s_type_immediate(instruction);
```

### 2. Missing Alignment Handling
```rust
// WRONG: Direct byte access
memory.write_byte(addr, value);

// CORRECT: Use LoadStore adapter's shift logic
let shift = addr & 0x3;
let aligned_addr = addr & !0x3;
```

### 3. Forgetting Sign Extension
```rust
// WRONG: Zero extension for LH
let value = half_word as u32;

// CORRECT: Sign extension
let value = (half_word as i16) as i32 as u32;
```

### 4. Improper PC Updates
```rust
// WRONG: Forgetting to mask JALR target
next_pc = rs1 + imm;

// CORRECT: Clear LSB
next_pc = (rs1 + imm) & !1;
```

## Testing Guidelines

### Unit Tests
- Test each instruction variant
- Include edge cases (x0, max values)
- Verify immediate encoding/decoding
- Check alignment handling

### Integration Tests
- Run RISC-V compliance tests
- Test instruction sequences
- Verify state transitions
- Check memory consistency

### Constraint Verification
- Ensure constraints are complete
- No unconstrained operations
- Verify range checks
- Test witness generation

## Performance Optimization Tips

1. **Batch Operations**: LoadStore batches to 4 bytes
2. **Lookup Tables**: Use for bitwise operations
3. **Conditional Logic**: Minimize constraint degree
4. **Memory Access**: Reduce number of reads/writes

## Code Review Checklist

When reviewing RV32IM adapter code:
- [ ] RISC-V spec compliance
- [ ] Correct immediate handling
- [ ] Proper sign extension
- [ ] x0 register safety
- [ ] Complete constraints
- [ ] Memory operation correctness
- [ ] PC update accuracy
- [ ] Range check presence
- [ ] Error handling
- [ ] Test coverage

## Debugging Tips

### Common Issues
1. **Wrong Results**: Check immediate sign extension
2. **Constraint Failures**: Verify all operations constrained
3. **Memory Errors**: Check alignment calculations
4. **PC Issues**: Verify branch offset handling
5. **Register Corruption**: Ensure x0 protection

### Debug Techniques
- Add assertions for invariants
- Trace execution step by step
- Verify against RISC-V simulator
- Check constraint satisfiability
- Use small test programs

## Remember
You are implementing a critical component of a zero-knowledge proof system. Every operation must be:
1. Correct according to RISC-V specification
2. Fully constrained for soundness
3. Efficiently implemented for performance
4. Thoroughly tested for reliability

When in doubt, refer to the RISC-V specification and existing adapter implementations.