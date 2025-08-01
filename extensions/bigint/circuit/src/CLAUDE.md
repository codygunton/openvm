# BigInt Circuit Component Guidelines

## Component Overview

This component implements 256-bit integer arithmetic operations as a circuit extension for OpenVM. It provides hardware-accelerated operations for cryptographic and scientific computing applications requiring 256-bit precision.

## Key Implementation Rules

### 1. Chip Architecture
- Each arithmetic operation MUST be implemented as a separate chip type
- All chips MUST follow the `VmChipWrapper` pattern with appropriate adapters
- Use `Rv32HeapAdapterChip` for standard operations, `Rv32HeapBranchAdapterChip` for branch operations

### 2. Memory Handling
- 256-bit integers are stored as 8 limbs of 32 bits each in little-endian format
- All memory operations MUST go through the heap adapter layer
- Never directly manipulate memory pointers in core arithmetic logic

### 3. Opcode Organization
- Each operation class has a fixed opcode offset (e.g., 0x400 for BaseAlu)
- Maintain consistency with transpiler opcode definitions
- New operations should follow the established offset pattern

### 4. Bus Integration
- Bitwise operations MUST use the shared `BitwiseOperationLookupBus`
- Multiplication MUST use the `RangeTupleCheckerBus` for overflow checking
- Reuse existing bus instances when possible to minimize circuit size

### 5. Testing Requirements
- All new operations MUST include randomized testing
- Test edge cases: zero operands, maximum values, overflow conditions
- Verify branch target calculations for branch operations

## Common Patterns

### Adding a New Operation

1. Define the opcode in the transpiler:
```rust
#[derive(Copy, Clone, Debug, LocalOpcode)]
#[opcode_offset = 0x4XX]
pub struct Rv32NewOp256Opcode(pub NewOpOpcode);
```

2. Create the chip type alias:
```rust
pub type Rv32NewOp256Chip<F> = VmChipWrapper<
    F,
    Rv32HeapAdapterChip<F, 2, INT256_NUM_LIMBS, INT256_NUM_LIMBS>,
    NewOpCoreChip<INT256_NUM_LIMBS, RV32_CELL_BITS>,
>;
```

3. Add to the executor enum:
```rust
#[derive(ChipUsageGetter, Chip, InstructionExecutor, From, AnyEnum)]
pub enum Int256Executor<F: PrimeField32> {
    // ... existing variants
    NewOp256(Rv32NewOp256Chip<F>),
}
```

4. Register in the build function with proper bus connections

### Memory Layout Example

For a 256-bit integer at address `addr`:
- Limb 0 (bits 0-31): `addr + 0`
- Limb 1 (bits 32-63): `addr + 4`
- ...
- Limb 7 (bits 224-255): `addr + 28`

## Performance Optimization

1. **Lookup Table Sharing**: Always check if an existing lookup chip can be reused
2. **Bus Sizing**: Configure range tuple checker sizes based on expected workload
3. **Instruction Batching**: Group related operations to minimize context switches

## Security Considerations

1. **Constant Time**: All operations must execute in constant time
2. **No Data-Dependent Branching**: Branch decisions can be revealed, but not intermediate values
3. **Memory Bounds**: Heap adapters handle bounds checking - never bypass them

## Common Pitfalls

1. **Forgetting Opcode Registration**: Always add new opcodes to the transpiler's iterator
2. **Incorrect Limb Count**: Ensure INT256_NUM_LIMBS (8) is used consistently
3. **Missing Bus Connections**: Verify all required buses are connected in the build function
4. **Opcode Conflicts**: Check that new opcode offsets don't overlap with existing ones

## Integration Checklist

When modifying this component:
- [ ] Update opcode definitions in transpiler
- [ ] Add chip type aliases in lib.rs
- [ ] Extend executor enum
- [ ] Implement guest function externs
- [ ] Add to extension build() method
- [ ] Write comprehensive tests
- [ ] Update AI documentation if adding new concepts

## Debugging Tips

1. **Constraint Failures**: Check bus connections and ensure all lookups are registered
2. **Memory Errors**: Verify heap adapter configuration and pointer arithmetic
3. **Opcode Not Found**: Ensure transpiler and circuit opcode definitions match
4. **Performance Issues**: Profile bus usage and consider increasing lookup table sizes