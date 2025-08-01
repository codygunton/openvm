# JALR Component - AI Assistant Instructions

## Component Overview
You are working with the JALR (Jump And Link Register) instruction implementation in OpenVM's RISC-V extension. This component handles indirect jumps with return address saving.

## Key Responsibilities

When working on JALR code:

1. **Maintain Correctness**
   - Always clear LSB of jump target: `to_pc & !1`
   - Sign-extend 12-bit immediate to 32 bits
   - Save `pc + 4` to rd, not `to_pc + 4`
   - Handle x0 writes properly (no-op)

2. **Preserve Optimizations**
   - Keep 3-limb rd_data storage (derive 4th limb)
   - Use 2-limb address calculation for efficiency
   - Batch range check requests when possible

3. **Follow Patterns**
   - Use `compose`/`decompose` for limb conversions
   - Request bitwise lookups before execution
   - Maintain consistent error handling

## Common Tasks

### Adding New JALR Variant
```rust
// 1. Define new opcode in transpiler
pub enum Rv32JalrOpcode {
    JALR,
    JALR_NEW,  // Your new variant
}

// 2. Update run_jalr if needed
pub fn run_jalr(opcode: Rv32JalrOpcode, ...) {
    match opcode {
        JALR => { /* existing */ },
        JALR_NEW => { /* your logic */ },
    }
}

// 3. Add tests
#[test]
fn test_jalr_new() {
    // Test your variant
}
```

### Modifying Constraints
When changing constraints in `eval()`:
1. Ensure all boolean values have `assert_bool`
2. Add appropriate `when(is_valid)` guards
3. Update range checks if bit widths change
4. Test with both positive and negative tests

### Debugging Issues
For constraint failures:
1. Check `run_negative_jalr_test` examples
2. Verify range constraints match actual values
3. Ensure carry propagation is correct
4. Use `debug_address_calculation` pattern

## Testing Requirements

Always include:
1. **Positive tests**: Random valid inputs
2. **Negative tests**: Constraint violations
3. **Edge cases**: PC limits, sign boundaries
4. **Sanity tests**: Known input/output pairs

## Integration Considerations

1. **Memory System**
   - Use correct address space (RV32_REGISTER_AS)
   - Handle timestamps properly
   - Coordinate with OfflineMemory

2. **Execution Bus**
   - Report all 7 instruction fields
   - Update execution state correctly
   - Maintain PC validity

3. **Auxiliary Chips**
   - Request ranges before execution
   - Share chips across instructions
   - Clean up references properly

## Code Style

1. **Naming**
   - Use `to_pc` for jump target
   - Use `from_pc` for current PC
   - Use `rd_data` for return address limbs

2. **Comments**
   - Explain non-obvious optimizations
   - Document constraint purposes
   - Note RISC-V spec references

3. **Structure**
   - Keep core logic in `run_jalr`
   - Put constraints in `eval`
   - Test helpers in test module

## Performance Guidelines

1. **Minimize Operations**
   - Reuse computed values
   - Batch similar operations
   - Avoid redundant checks

2. **Optimize Traces**
   - Pack columns efficiently
   - Derive values when possible
   - Use lookup tables wisely

## Security Considerations

1. **Address Validation**
   - Enforce PC_BITS limit
   - Clear LSB for alignment
   - Prevent integer overflow

2. **Register Safety**
   - Check register bounds
   - Handle x0 specially
   - Validate memory access

## Don'ts

- Don't forget LSB clearing on jump target
- Don't store all 4 rd_data limbs (waste of space)
- Don't skip range checks on derived values
- Don't modify without updating tests
- Don't break existing optimizations

## Quick Checklist

Before submitting JALR changes:
- [ ] All tests pass (positive, negative, sanity)
- [ ] Constraints are sound and complete
- [ ] Range checks cover all values
- [ ] Memory operations are correct
- [ ] Documentation is updated
- [ ] No performance regressions