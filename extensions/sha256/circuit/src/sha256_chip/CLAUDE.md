# SHA256 Chip Component - Claude Instructions

## Component Overview
This is the SHA256 cryptographic hash function implementation for OpenVM's zkVM. It provides hardware-accelerated SHA256 hashing as a custom RISC-V instruction that can process variable-length messages from memory.

## Key Responsibilities
1. Execute SHA256 hashing instructions within the VM
2. Handle automatic message padding according to SHA256 specification
3. Generate constraint-satisfying execution traces for zero-knowledge proofs
4. Integrate with OpenVM's memory system and execution bus

## Critical Implementation Details

### Memory Access Pattern
- Reads input in 16-byte chunks (SHA256_READ_SIZE)
- Processes 64-byte blocks (SHA256_BLOCK_CELLS)
- Writes 32-byte digest in a single operation
- All pointers must fit within ptr_max_bits (minimum 24 bits)

### Padding State Machine
The padding implementation uses explicit flags to track padding state across rows. This is critical for constraint satisfaction. The padding flags encode:
- Where padding starts within a 16-byte read
- Whether the current row is in the last block
- Special handling for the 64-bit message length suffix

### Trace Layout
Each SHA256 block generates 17 rows:
- Rows 0-15: Round computations (Sha256VmRoundCols)
- Row 16: Final digest (Sha256VmDigestCols)
- First 4 rows of each block perform memory reads

## Working with This Component

### When Modifying Core Logic
1. **Instruction Execution (mod.rs)**:
   - Maintain consistency between execute() and trace generation
   - Update Sha256Record if adding new execution data
   - Preserve memory access patterns

2. **Constraints (air.rs)**:
   - Any changes must maintain soundness
   - Test edge cases thoroughly (especially padding boundaries)
   - Document complex constraint interactions

3. **Trace Generation (trace.rs)**:
   - Keep trace generation deterministic
   - Optimize for parallelization where possible
   - Validate against constraint system

### Common Pitfalls
1. **Endianness**: Memory is little-endian but SHA256 operates on big-endian
2. **Padding Edge Cases**: Messages ending at block boundaries need special care
3. **Pointer Arithmetic**: Must not overflow ptr_max_bits
4. **Message Length**: Limited to 2^30 bytes due to field element constraints

### Testing Requirements
When modifying:
- Test standard test vectors (empty string, "abc", etc.)
- Test padding edge cases (55, 56, 64, 119, 120 byte messages)
- Test maximum message length
- Verify trace satisfies all constraints
- Check memory access patterns are correct

## Integration Points

### With OpenVM Core
- Uses ExecutionBridge for instruction handling
- Uses MemoryBridge for memory operations
- Integrates with offline memory for trace generation

### With Other Components
- Shares BitwiseOperationLookupChip for efficient bitwise ops
- Can be composed with other crypto operations (e.g., for HMAC)

## Performance Considerations
- Memory reads are batched for efficiency
- Bitwise operations use lookup tables
- Trace generation is parallelized by block
- Consider caching for repeated hash operations

## Security Notes
- This implements SHA256 to specification - do not modify the core algorithm
- All padding must be explicitly verified in constraints
- Memory access bounds must be checked
- The implementation has been audited as part of OpenVM

## Future Extensions
Potential improvements while maintaining compatibility:
- Streaming hash support (hash across multiple instructions)
- Batch hashing (multiple messages in one instruction)
- Integration with other hash functions (SHA3, Blake3)
- Optimized layouts for common message sizes