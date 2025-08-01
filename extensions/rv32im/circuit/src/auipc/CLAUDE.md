# AUIPC Component - Claude Instructions

## Component Overview
The AUIPC component implements the RISC-V "Add Upper Immediate to PC" instruction in OpenVM. This instruction adds a 20-bit immediate (shifted left by 12 bits) to the program counter and stores the result in a destination register.

## Key Implementation Details

### Instruction Semantics
- **Operation**: `rd = pc + (imm << 12)`
- **Immediate**: 20-bit value, left-shifted by 12 bits
- **Result**: 32-bit value with wrapping arithmetic
- **Special case**: Writing to x0 (register 0) is a no-op

### Data Representation
The component uses a limb-based representation:
- 32-bit values are split into 4 limbs of 8 bits each
- Limbs are stored in little-endian order
- Special optimizations:
  - Immediate LSB is always 0 (not stored)
  - PC limbs are partially reconstructed from result

### Critical Constraints
1. **Carry propagation**: Addition must properly handle carries between limbs
2. **Range checking**: All limbs must be within 8-bit bounds
3. **PC limit**: Most significant PC limb must respect PC_BITS constraint
4. **Boolean carries**: Carry values must be 0 or 1

## Common Tasks

### Adding Tests
When adding new tests, focus on:
- Edge cases in carry propagation
- PC wraparound behavior
- Maximum immediate values
- Interaction with PC_BITS limit

### Modifying the Implementation
Key areas to consider:
- `run_auipc()`: Core execution logic
- `eval()`: Constraint generation
- Range checking strategy
- Limb decomposition logic

### Debugging Constraint Failures
Common issues:
1. Incorrect carry calculation
2. Missing range checks
3. Wrong limb indexing
4. PC MSB calculation errors

## Performance Considerations
- Batch range checks to minimize lookups
- Share bitwise operation tables
- Optimize limb representation
- Consider instruction fusion opportunities

## Security Notes
- All arithmetic operations use wrapping semantics
- Range checks are mandatory for soundness
- No assumptions about field element representation
- Carries must be explicitly constrained

## Integration Points
- Uses `Rv32RdWriteAdapterChip` for register writes
- Shares `BitwiseOperationLookupChip` with other components
- Registered with VM executor for AUIPC opcode
- Part of the RV32IM extension module