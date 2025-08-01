# AUIPC Component - Implementation Guide

## Overview

This guide provides detailed implementation information for the AUIPC component in OpenVM's RV32IM extension.

## Architecture Deep Dive

### Component Hierarchy

```
Rv32AuipcChip (Public Interface)
    ├── Rv32RdWriteAdapterChip (Memory Interface)
    └── Rv32AuipcCoreChip (Core Logic)
        ├── Rv32AuipcCoreAir (Constraints)
        └── BitwiseOperationLookupChip (Range Checking)
```

### Data Flow

1. **Instruction Fetch**: VM provides instruction with PC
2. **Decode**: Extract immediate and destination register
3. **Execute**: Compute `pc + (imm << 12)`
4. **Limb Split**: Convert result to field elements
5. **Constraint Check**: Verify arithmetic and ranges
6. **Write Back**: Update destination register

## Implementation Details

### Limb-Based Arithmetic

The implementation uses 8-bit limbs to represent 32-bit values in the constraint system:

```rust
// 32-bit value representation
value = limb[0] + limb[1]*256 + limb[2]*65536 + limb[3]*16777216

// For AUIPC calculation
rd = pc + (imm << 12)
```

### Special Limb Handling

1. **Immediate Value**
   - Only upper 20 bits stored (lower 12 are zeros)
   - `imm_limbs` contains 3 limbs (indices 1-3)
   - LSB limb omitted as it's always 0

2. **PC Value**
   - `pc_limbs` contains middle 2 limbs (indices 1-2)
   - LSB reused from `rd_data[0]`
   - MSB computed from constraint

3. **Result (rd_data)**
   - Full 4 limbs stored
   - `rd_data[0]` equals PC's LSB (since imm LSB is 0)

### Constraint Implementation

#### Carry Propagation
```rust
// For each limb position i (1 to 3)
carry[i] = (pc_limbs[i] + imm_limbs[i-1] - rd_data[i] + carry[i-1]) / 256

// Constraints:
// 1. Each carry must be 0 or 1 (boolean)
// 2. The equation must hold exactly in the field
```

#### Range Checking
```rust
// Pairs of limbs checked together
for i in 0..2 {
    send_range(rd_data[i*2], rd_data[i*2+1])
}

// Special handling for PC MSB (limited to PC_BITS)
pc_msb_scaled = pc_msb * (1 << (32 - PC_BITS))
send_range(pc_msb_scaled, other_limb)
```

### Key Algorithms

#### PC MSB Recovery
```rust
// Given: rd_data[0], pc_limbs[0..2], and from_pc
// Compute intermediate value (PC without MSB)
intermed = rd_data[0] + pc_limbs[0]*256 + pc_limbs[1]*65536

// Recover MSB
pc_msb = (from_pc - intermed) / 16777216
```

#### Efficient Range Checking
- Groups limbs in pairs for bitwise lookup
- Combines immediate and PC limbs for efficiency
- Total lookups: 4 (for rd_data) + 3 (for imm/pc limbs)

## Edge Cases and Gotchas

### 1. Register Zero Handling
```rust
if dec_insn.rd == 0 {
    return nop();  // RISC-V convention: x0 always zero
}
```

### 2. Overflow Behavior
- Uses wrapping arithmetic (standard for RV32)
- No overflow flags or exceptions
- Result truncated to 32 bits

### 3. PC Bits Limitation
- PC typically limited to 24 bits (16MB address space)
- MSB range check accounts for this
- Scaling factor: `2^(32-24) = 256`

### 4. Immediate Encoding
- Instruction stores immediate shifted right by 12
- Implementation shifts left by 12 before addition
- Effective range: ±512KB from PC

## Performance Optimization

### Lookup Table Sharing
- Bitwise operations shared across instructions
- Amortizes setup cost across circuit
- Reduces overall constraint count

### Batched Range Checks
- Checks two values per lookup
- Reduces interaction count by ~50%
- Critical for prover performance

### Precomputed Values
- PC limb decomposition cached
- Immediate shift computed once
- Minimizes field operations

## Testing Approach

### Positive Tests
```rust
// Random valid operations
for _ in 0..100 {
    let pc = rng.gen_range(0..(1 << PC_BITS));
    let imm = rng.gen_range(0..(1 << 20));
    test_auipc(pc, imm);
}
```

### Negative Tests
```rust
// Tamper with trace to verify constraints
let mut trace = generate_valid_trace();
trace.rd_data[0] += 1;  // Should fail carry constraint
verify_error(ConstraintViolation);
```

### Edge Case Tests
- Maximum PC value: `(1 << 24) - 1`
- Maximum immediate: `0xFFFFF`
- Zero values for PC and immediate
- All combinations of carry patterns

## Integration Guidelines

### 1. Chip Instantiation
```rust
let bitwise_chip = SharedBitwiseOperationLookupChip::new(bus);
let core = Rv32AuipcCoreChip::new(bitwise_chip.clone());
let adapter = Rv32RdWriteAdapterChip::new(buses...);
let auipc_chip = Rv32AuipcChip::new(adapter, core, memory);
```

### 2. Executor Registration
```rust
executor.register_instruction(
    auipc_chip,
    Rv32AuipcOpcode::AUIPC.global_opcode()
);
```

### 3. Program Usage
```asm
# Example: Load address of data
auipc t0, %hi(data_label)
addi  t0, t0, %lo(data_label)
```

## Debugging Tips

### Common Issues

1. **Constraint Failures**
   - Check limb decomposition
   - Verify carry propagation
   - Ensure proper range checking

2. **Wrong Results**
   - Verify immediate shift (× 4096)
   - Check wrapping arithmetic
   - Confirm PC value source

3. **Performance Issues**
   - Profile range check calls
   - Verify lookup table sharing
   - Check trace size growth

### Debug Output
```rust
// Add debug prints in execute_instruction
dbg!(pc, imm, rd_data);
dbg!(pc_limbs, imm_limbs);
dbg!(carries);
```

## Future Improvements

### Potential Optimizations
1. Combine with other U-type instructions (LUI)
2. Optimize limb representation for sparse immediates
3. Parallelize range checking across instructions

### Extension Points
1. Support for compressed instructions
2. Integration with branch prediction
3. Custom immediate encodings