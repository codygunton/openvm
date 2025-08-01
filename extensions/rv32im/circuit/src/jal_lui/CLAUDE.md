# JAL/LUI Component - Claude Instructions

## Component Purpose
This component implements the RISC-V JAL (Jump And Link) and LUI (Load Upper Immediate) instructions within the OpenVM zkVM framework. These instructions share a combined implementation for efficiency.

## Key Implementation Details

### Instruction Behavior
- **JAL**: Saves PC+4 to rd, then jumps to PC + immediate
- **LUI**: Loads immediate << 12 into rd, increments PC by 4

### Critical Constraints
1. Exactly one of `is_jal` or `is_lui` must be true
2. All limbs must be range-checked to 8 bits
3. JAL must enforce PC_BITS limit on the last limb
4. LUI must have first limb (LSB) equal to zero

### Common Pitfalls
- JAL immediate is sign-extended and shifted (bit 0 implicit)
- PC is limited to 24 bits, not 32 bits
- LUI shifts by exactly 12 bits (not 16)
- Register x0 writes handled by adapter layer

## Testing Requirements
When modifying this component:
1. Run all existing tests to ensure compatibility
2. Add tests for any new edge cases
3. Verify both positive and negative test cases
4. Check immediate encoding edge cases

## Performance Considerations
- Shared implementation reduces chip count
- Bitwise lookups are expensive - minimize usage
- Range checking is done in pairs for efficiency

## Security Notes
- All arithmetic must handle wraparound correctly
- Sign extension must be explicit and verified
- Range constraints are critical for soundness
- Never trust unchecked immediate values