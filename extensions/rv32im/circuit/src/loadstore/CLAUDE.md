# LoadStore Component Instructions for Claude

## Component Overview
You are working with the LoadStore component of OpenVM's RV32IM extension. This component implements RISC-V load and store memory operations through a two-chip architecture: a core chip for data transformations and an adapter chip for memory interfacing.

## Key Implementation Details

### Architecture
- **Two-chip design**: LoadStoreCoreChip (logic) + Rv32LoadStoreAdapterChip (interface)
- **4-byte aligned access**: All memory operations work on 4-byte boundaries
- **Sub-word support**: Handles byte and halfword operations via shifting and merging

### Critical Constraints
1. **Flag encoding**: Uses 4 flags to encode 14 different (opcode, shift) combinations
2. **Shift calculation**: `shift = ptr_val % 4`, where ptr_val = rs1 + sign_extended_imm
3. **Memory alignment**: All memory accesses must be to addresses where `(ptr_val - shift) % 4 == 0`
4. **x0 protection**: Writes to register x0 must be disabled by setting `needs_write = 0`

## When Modifying This Component

### Adding New Operations
1. Add opcode to `Rv32LoadStoreOpcode` enum in the transpiler
2. Define flag pattern in `generate_trace_row()` 
3. Implement transformation in `run_write_data()`
4. Add constraint logic if needed in `LoadStoreCoreAir::eval()`
5. Create comprehensive tests including negative cases

### Modifying Existing Operations
- **Data transformations**: Edit `run_write_data()` function
- **Constraints**: Modify `LoadStoreCoreAir::eval()` method
- **Memory interface**: Change adapter chip's `preprocess()`/`postprocess()`
- **Always update tests** to cover your changes

### Common Pitfalls to Avoid
1. **Forgetting prev_data**: Store operations must read existing memory contents
2. **Incorrect shift handling**: Ensure shift is applied consistently
3. **Missing x0 check**: Always verify rd != x0 for load operations
4. **Wrong address space**: Loads use mem_as for memory, stores use it for destination
5. **Flag encoding errors**: Sum of flags must be 0, 1, or 2

## Testing Requirements

### When Writing Tests
1. Test all shift values (0-3 for bytes, 0,2 for halfwords)
2. Include x0 destination cases
3. Test address space boundaries
4. Verify constraint violations with negative tests
5. Check edge cases (max addresses, zero values)

### Test Patterns
```rust
// Positive test
set_and_execute(&mut tester, &mut chip, &mut rng, 
    OPCODE, rs1, imm, imm_sign, mem_as);

// Negative test  
run_negative_loadstore_test(OPCODE, 
    read_data, prev_data, write_data, flags, is_load,
    rs1, imm, imm_sign, mem_as, expected_error);
```

## Code Style Guidelines

### Naming Conventions
- Use descriptive names: `shift_amount` not `shift`
- Prefix memory pointers: `mem_ptr`, `aligned_ptr`
- Boolean flags: `is_load`, `is_valid`, `needs_write`

### Documentation
- Document non-obvious calculations (especially shift logic)
- Explain flag encoding patterns
- Include examples for complex operations

### Performance Considerations
- Minimize constraint degree (keep ≤ 3)
- Batch memory operations when possible
- Use helper functions for repeated calculations

## Integration Points

### With Memory System
- Uses MemoryBridge for all memory operations
- Respects OfflineMemory for trace generation
- Coordinates with MemoryController for timestamps

### With Execution System
- Receives instructions via ExecutionBus
- Updates PC through ExecutionBridge
- Validates addresses with RangeChecker

## Debugging Helpers

### Common Issues
1. **"Wrong write_data"**: Check flag encoding and shift calculation
2. **"Address overflow"**: Verify pointer_max_bits configuration
3. **"Constraint failure"**: Print trace row and verify manually
4. **"Memory mismatch"**: Ensure aligned_ptr calculation is correct

### Debug Output
Add to `generate_trace_row()`:
```rust
#[cfg(debug_assertions)]
println!("LoadStore: op={:?} shift={} flags={:?} write={:?}", 
    record.opcode, record.shift, flags, core_cols.write_data);
```

## Security Considerations
- Always validate pointer bounds before use
- Ensure address spaces are correctly enforced
- Never allow writes outside allocated memory regions
- Maintain x0 as read-only register

## Component-Specific Rules
1. **Never modify** the flag encoding system without updating all tests
2. **Always test** both aligned and unaligned addresses
3. **Maintain backward compatibility** with existing RISC-V semantics
4. **Document any deviations** from standard RISC-V behavior
5. **Preserve the two-chip architecture** for separation of concerns