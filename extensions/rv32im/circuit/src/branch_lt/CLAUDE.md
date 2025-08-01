# Branch Less Than Component - Claude Instructions

## Component Overview
This component implements RISC-V branch instructions for less-than comparisons (BLT, BLTU, BGE, BGEU). It performs signed and unsigned comparisons between two 32-bit register values and conditionally branches based on the result.

## Key Implementation Details

### Architecture
- Uses 8 limbs of 8 bits each to represent 32-bit values
- Integrates with bitwise lookup chip for range checking
- Implements both signed (two's complement) and unsigned comparisons
- Handles branch PC calculation based on comparison result

### Critical Logic
1. **Comparison Algorithm**: Scans limbs from MSB to LSB to find first difference
2. **Sign Handling**: MSB interpretation differs for signed vs unsigned
3. **Branch Decision**: Maps comparison result to branch decision based on opcode
4. **PC Update**: Adds immediate to PC if branching, else increments by 4

## When Working on This Component

### Do:
- Maintain consistency with RISC-V specification for branch semantics
- Ensure proper two's complement handling for signed operations
- Validate all opcode flags are mutually exclusive
- Test edge cases like equal values, overflow boundaries
- Use the shared bitwise lookup chip for range checks

### Don't:
- Modify the limb structure without updating dependent components
- Change opcode mappings without updating transpiler
- Forget to handle the equal values case for >= operations
- Skip range checking for MSB values

### Common Tasks:
1. **Adding new comparison operations**: Follow existing opcode pattern, update enum
2. **Optimizing constraints**: Ensure mathematical equivalence is preserved
3. **Debugging comparisons**: Check sign bit handling and difference detection
4. **Testing**: Use property-based testing with random values

## Integration Points
- **Bitwise Lookup**: Used for MSB range checking
- **RV32 Adapter**: Handles instruction decoding and register access
- **PC Management**: Updates program counter based on branch result

## Performance Considerations
- Difference detection is O(NUM_LIMBS) in worst case
- Range checks are batched through shared lookup chip
- Constraint evaluation is optimized for common case (no branch)