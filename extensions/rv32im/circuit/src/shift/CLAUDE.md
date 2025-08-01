# Shift Component - Claude Instructions

## Component Context
This component implements RISC-V shift operations (SLL, SRL, SRA) within the OpenVM zkVM. It's part of the RV32IM extension and handles bit shifting with zero-knowledge proof generation.

## Key Implementation Rules

### 1. Limb and Cell Structure
- Always use `RV32_REGISTER_NUM_LIMBS` (4) and `RV32_CELL_BITS` (8) constants
- Shift amounts are taken modulo 32 (NUM_LIMBS * LIMB_BITS)
- Maintain limb-based representation for all operations

### 2. Constraint Generation
- Every algebraic constraint in `eval()` must have a corresponding computation in `execute_instruction()`
- Use bitwise lookup tables for efficient range checking
- Always assert boolean constraints for flag columns

### 3. Sign Extension
- SRA operations must properly handle sign extension
- Use XOR lookup to extract and verify sign bit
- Fill high bits with sign bit for arithmetic right shifts

### 4. Testing Requirements
- Always test with random inputs for correctness
- Include negative tests with pranked traces for soundness
- Verify edge cases: zero shifts, maximum shifts, sign bits
- Use `disable_debug_builder()` in negative tests

### 5. Integration Points
- Inherit from `VmCoreChip` and `VmCoreAir` traits
- Use adapter pattern for memory and instruction handling
- Integrate with shared bitwise lookup and range checker chips

## Common Pitfalls to Avoid
1. **Incorrect shift amount handling**: Always take modulo (NUM_LIMBS * LIMB_BITS)
2. **Missing range checks**: Verify all intermediate values are in valid ranges
3. **Sign bit errors**: Test SRA thoroughly with negative numbers
4. **Carry bit miscalculation**: Verify bit_shift_carry logic for both left and right shifts

## Performance Considerations
- Batch bitwise operations when possible
- Use lookup tables instead of computing bitwise ops in constraints
- Minimize the number of range checks by reusing computations

## Security Notes
- All shift amounts must be validated to prevent overflow
- Sign extension must be constant-time for zkVM security
- Test with adversarial inputs in negative tests