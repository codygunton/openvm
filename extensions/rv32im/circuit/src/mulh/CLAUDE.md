# MulH Component Instructions for Claude

## Component Overview
The MulH component implements high-multiplication operations (MULH, MULHSU, MULHU) for RISC-V 32-bit integers in the OpenVM zkVM. It computes the upper 32 bits of 64-bit multiplication results.

## Key Implementation Details

### Architecture
- **Limb-based computation**: 32-bit values split into 4 limbs of 8 bits each
- **Constraint system**: Enforces correct multiplication with carry propagation
- **Sign extension**: Handled differently for each opcode variant
- **Shared resources**: Uses common bitwise and range checking infrastructure

### Critical Files
- `core.rs`: Main implementation with AIR constraints and execution logic
- `mod.rs`: Module structure and type definitions
- `tests.rs`: Comprehensive test suite including edge cases

## Code Modification Guidelines

### When Modifying Constraints
1. **Preserve mathematical correctness**: The multiplication algorithm must remain sound
2. **Maintain carry propagation**: Carries must flow correctly between limbs
3. **Check sign extension logic**: Each opcode has specific sign extension requirements
4. **Test with edge cases**: Include maximum values, sign boundaries, and zero

### When Adding Features
1. **Follow existing patterns**: Use the same limb-based approach
2. **Share lookup tables**: Reuse bitwise and range checking infrastructure
3. **Update all three opcodes**: Changes often affect MULH, MULHSU, and MULHU
4. **Add comprehensive tests**: Include positive, negative, and sanity tests

### Testing Requirements
- **Positive tests**: Random inputs with constraint verification
- **Negative tests**: Corrupted traces to verify constraint robustness
- **Sanity tests**: Known input/output pairs for correctness
- **Edge cases**: Sign bits, overflow scenarios, zero operands

## Common Pitfalls to Avoid

1. **Incorrect carry handling**: Carries must account for all partial products
2. **Sign extension errors**: MULHSU only extends first operand, MULHU extends neither
3. **Range check bounds**: Carries can exceed single limb values
4. **Endianness confusion**: Limbs are little-endian, MSB in last position

## Integration Points

### With RV32 System
- Uses standard RV32 register addressing
- Integrates through `Rv32MultAdapterChip`
- Shares memory interface with other RV32 operations

### With Lookup Infrastructure
- `BitwiseOperationLookupBus`: For sign bit extraction
- `RangeTupleCheckerBus`: For carry validation
- Must coordinate table sizes with other components

## Performance Considerations

1. **Constraint optimization**: Minimize redundant checks
2. **Lookup table efficiency**: Batch similar operations
3. **Memory access patterns**: Sequential reads, single write
4. **Trace generation**: Efficient column layout

## Security Notes

The MulH implementation is critical for:
- Overflow detection in multiplication
- Multi-precision arithmetic
- Cryptographic operations requiring full multiplication results

Any modifications must maintain the security properties of the constraint system.