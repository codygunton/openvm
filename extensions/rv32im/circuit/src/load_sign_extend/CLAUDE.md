# Load Sign Extend Component - Claude Rules

## Component-Specific Guidelines

### Code Modification Rules

1. **Constraint Preservation**
   - NEVER modify the sign bit extraction logic without updating range checks
   - ALWAYS ensure opcode flags remain mutually exclusive
   - PRESERVE the relationship between shift_amount and shifted_read_data

2. **Memory Layout Assumptions**
   - This component assumes 32-bit words stored as 4 bytes (limbs)
   - Each limb is 8 bits (RV32_CELL_BITS)
   - DO NOT change these constants without updating all array operations

3. **Shift Handling**
   - The `shift_amount & 2` optimization is critical for performance
   - LOADB uses both flag0 and flag1 to handle shifts 0-3
   - LOADH only uses flag with shifts 0 or 2 (no flag1 needed)

### Testing Requirements

When modifying this component:

1. **Run Full Test Suite**
   ```bash
   cargo test -p openvm-rv32im-circuit load_sign_extend
   ```

2. **Add Tests For**:
   - Any new edge cases in alignment
   - Sign bit boundary conditions
   - New constraint violations in negative tests

3. **Verify Against RISC-V Spec**
   - LOADB: Sign-extend byte to 32 bits
   - LOADH: Sign-extend halfword to 32 bits
   - Results must match RISC-V ISA manual

### Common Pitfalls

1. **Array Indexing**
   - shifted_read_data indices depend on NUM_CELLS
   - Sign bit position: byte[0], halfword[NUM_CELLS/2-1]
   - Upper limbs are always sign-extended

2. **Shift Amount Handling**
   - read_shift = shift_amount & 2 (not shift_amount itself)
   - LOADB with shift 1 uses opcode_loadb_flag1, not flag0
   - Unshift operation: `(i + NUM_CELLS - 2) % NUM_CELLS`

3. **Range Checker Integration**
   - Must add count for sign bit validation
   - Range check size is LIMB_BITS - 1 (7 bits for 8-bit limbs)

### Performance Considerations

1. **Constraint Minimization**
   - Use select() for conditional expressions
   - Batch similar operations with array::from_fn
   - Avoid unnecessary field operations

2. **Trace Generation**
   - Pre-compute boolean flags
   - Use bitwise operations for sign extension
   - Minimize field element conversions

### Integration Points

When integrating with other components:

1. **Adapter Requirements**
   - Expects aligned memory addresses
   - Provides shift_amount in read data
   - Handles memory bridge interactions

2. **Opcode Coordination**
   - LOADB = 0x216, LOADH = 0x217 (from CLASS_OFFSET 0x210)
   - Must not conflict with unsigned load opcodes
   - Coordinate with transpiler for instruction encoding

### Debugging Guidelines

1. **Trace Inspection**
   - Check opcode flags match instruction
   - Verify shifted_read_data alignment
   - Confirm sign bit extraction matches most significant bit

2. **Common Issues**
   - Wrong opcode flag set: Check shift_amount calculation
   - Sign extension failure: Verify bit position logic
   - Range check errors: Ensure proper count registration

### Security Audit Points

1. **Soundness Critical**:
   - Sign bit must be correctly extracted
   - All array accesses must be bounds-checked
   - Shift operations must handle all valid inputs

2. **Do NOT**:
   - Remove range checks without proof of safety
   - Change array sizes without updating all usages
   - Modify opcode encoding without transpiler sync