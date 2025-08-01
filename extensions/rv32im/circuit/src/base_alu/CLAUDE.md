# BaseAlu Component - Implementation Guidelines

## Component Purpose

This component implements the basic ALU operations (ADD, SUB, XOR, OR, AND) for RISC-V 32-bit instructions in a zero-knowledge proof-friendly manner. It's a critical component of the RV32IM extension.

## Code Standards

### When Modifying This Component

1. **Maintain Constraint Degree**: Keep polynomial constraints at degree 3 or lower for performance
2. **Preserve Limb Structure**: Always work with 4 limbs of 8 bits for RV32 compatibility
3. **Use Bitwise Lookups**: All operations must validate through the bitwise operation lookup table
4. **Boolean Constraints**: Ensure all flag and carry values are properly boolean-constrained

### Testing Requirements

When making changes:
- Run all existing tests: `cargo test -p openvm-rv32im-circuit base_alu`
- Add tests for new functionality in `tests.rs`
- Include both positive and negative test cases
- Verify constraint satisfaction and interaction correctness

### Common Pitfalls

1. **Carry Overflow**: Ensure carries are computed with proper field division
2. **Range Checking**: All limb values must be validated as 8-bit values
3. **Opcode Flags**: Exactly one flag must be active per operation
4. **Memory Alignment**: Maintain proper limb ordering for memory operations

## Architecture Decisions

### Why Limb-Based?
- Enables efficient carry propagation in field arithmetic
- Allows bit-level operations through lookups
- Maintains compatibility with memory system

### Why Separate ADD/SUB Constraints?
- Keeps constraint degree at 3 (critical for performance)
- Allows independent carry/borrow validation
- Simplifies debugging and verification

### Why Bitwise Lookups for Range Checking?
- Unified approach for all operations
- Efficient batching of lookups
- Reuses existing infrastructure

## Integration Notes

### With Memory System
- Reads two operands through adapter
- Writes one result through adapter
- Handles both register and immediate operands

### With Bitwise Lookup
- ADD/SUB: Range check all result limbs
- XOR/OR/AND: Validate operation results
- Shared lookup table for efficiency

### With VM Framework
- Implements `VmCoreChip` trait
- Provides proper trace generation
- Handles instruction decoding

## Performance Optimization

1. **Batch Lookups**: Request all bitwise operations together
2. **Minimal Constraints**: Only essential constraints in AIR
3. **Efficient Carry**: Precompute inverse for division
4. **Trace Layout**: Optimize column ordering for cache

## Future Considerations

When extending:
- Consider adding more ALU operations (e.g., shifts)
- Maintain backward compatibility
- Keep the same limb structure
- Document any new constraints thoroughly