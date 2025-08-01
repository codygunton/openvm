# RV32IM Circuit Extension - Component-Specific Instructions

## Overview

This component implements the RISC-V 32-bit Integer (I) and Multiplication (M) instruction set extensions as zero-knowledge proof circuits for OpenVM. When working with this component, follow these specific guidelines.

## Key Principles

1. **Two-Layer Architecture is Mandatory**
   - ALWAYS separate adapter (VM interface) and core (instruction logic) layers
   - Never mix VM interaction code with instruction implementation

2. **Register x0 is Special**
   - ALWAYS check `rd != 0` before writing to registers
   - Register 0 is hardwired to zero and writes are ignored

3. **Limb Decomposition**
   - RV32 registers use 4 limbs of 8 bits each
   - ALWAYS constrain limbs to valid ranges using `assert_u8`

4. **Memory Address Spaces**
   - 1: Registers
   - 2: Heap memory
   - 3: Global memory  
   - 4: Constant memory

## When Modifying This Component

### Adding New Instructions

1. First check if the instruction already exists in subdirectories
2. Follow the existing pattern:
   - Adapter in `adapters/`
   - Core in dedicated subdirectory
   - Tests alongside implementation
3. Register in `extension.rs` with proper opcode mapping

### Modifying Existing Instructions

1. Locate the instruction in its subdirectory
2. Understand both adapter and core layers before changes
3. Update tests to cover modifications
4. Ensure changes don't break existing behavior

### Working with Tests

- Use `test_utils.rs` for generating test inputs
- Test edge cases: zero operands, maximum values, overflow
- Run both unit tests and integration tests

## Common Patterns to Follow

```rust
// Standard adapter pattern
pub struct Rv32InstructionAdapterChip<F> {
    execution_bus: ExecutionBus,
    program_bus: ProgramBus,
    memory_bridge: MemoryBridge<F>,
    // Additional periphery chips as needed
}

// Standard core pattern
pub struct InstructionCoreChip<F> {
    // Instruction-specific fields
    offset: usize,
}

// Always use composite chip
pub type Rv32InstructionChip<F> = CompositeChip<F, Adapter, Core>;
```

## Integration Points

- This extension integrates with:
  - Base OpenVM system (execution bus, program bus)
  - Shared periphery chips (bitwise operations, range checking)
  - Other extensions through the VM inventory system

## Performance Considerations

- Bitwise operations use lookup tables for efficiency
- Range checking is shared across instructions
- Division/remainder uses optimized constraint generation

## Security Requirements

- All arithmetic operations MUST include overflow checks
- Memory accesses MUST be bounds-checked
- Sign extension MUST be implemented correctly
- Division by zero MUST be handled per RISC-V spec

## Testing Requirements

Before submitting changes:
1. Run all existing tests: `cargo test -p openvm-rv32im-circuit`
2. Add tests for new functionality
3. Test with actual RISC-V programs if possible
4. Verify constraint counts haven't increased unnecessarily

## Documentation Updates

When making changes:
1. Update inline documentation in code
2. Update README.md if adding new instructions
3. Keep AI documentation files current
4. Document any breaking changes

## Debugging Tips

1. Use `builder.assert_eq` for debugging constraints
2. Check program counter updates in control flow instructions
3. Verify memory timestamps are correct
4. Use trace output to debug instruction execution