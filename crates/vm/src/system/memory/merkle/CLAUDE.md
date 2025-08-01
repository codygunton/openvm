# Claude Instructions for Merkle Memory System

## Component Overview

You are working with the Merkle memory system in OpenVM, which provides cryptographic verification of memory operations. This system is critical for proving memory integrity in zero-knowledge proofs.

## Key Concepts to Remember

1. **Memory is Merkle-ized**: All memory operations must maintain consistency with a Merkle tree root
2. **Lazy evaluation**: Only touched parts of the tree are materialized for efficiency  
3. **Chunked storage**: Memory is divided into chunks (typically 8 field elements)
4. **Two-phase operation**: Touch nodes during execution, finalize once at the end

## When Working on This Component

### Always Check
- Memory dimensions configuration is valid (non-zero heights)
- All memory accesses are properly touched before finalization
- Hash operations are recorded for proof generation
- Bus interactions are balanced (sends match receives)

### Never
- Finalize the chip more than once
- Access memory without touching the corresponding range
- Modify constraints without understanding security implications
- Assume a specific hash function - it's configurable

## Common Tasks

### Adding Memory Access Tracking
```rust
// Always touch before access
chip.touch_range(address_space, start_address, length);
// Then perform the actual memory operation
```

### Debugging Proof Failures
1. Check trace row ordering (must be sorted by height)
2. Verify expand_direction values (1, -1, or 0)
3. Ensure all touched nodes have corresponding trace rows
4. Validate hash computations match between initial and final states

### Performance Optimization
- Minimize the number of touched nodes
- Batch nearby memory accesses
- Align accesses to chunk boundaries when possible
- Monitor `num_touched_nonleaves` metric

## Architecture Context

The Merkle memory system sits between:
- **Memory Controller**: High-level memory management
- **Hash Functions**: Poseidon2 or other cryptographic hashes
- **Proof System**: Generates ZK proofs of memory operations

## Key Files and Their Purposes

- `mod.rs`: Core chip logic and node tracking
- `air.rs`: Constraints that ensure proof security
- `trace.rs`: Converts touched nodes into proof trace
- `columns.rs`: Data layout for proof generation

## Testing Philosophy

- Use `HashTestChip` for deterministic testing
- Always test edge cases (empty tree, single access)
- Verify both positive and negative test cases
- Random testing helps find unexpected issues

## Security Critical Points

1. **Root Hash Integrity**: The Merkle root must accurately reflect all memory changes
2. **Tree Structure**: Parent-child relationships must be maintained
3. **Hash Function**: Must be collision-resistant and match the proof system
4. **Public Values**: Initial and final roots enable proof continuation

## Common Pitfalls

1. **Forgetting to touch**: Every memory access needs `touch_range()`
2. **Double finalization**: Can only finalize once per proof
3. **Incorrect dimensions**: Tree structure must match memory layout
4. **Bus mismatches**: Interaction multiplicities must balance

## Performance Considerations

The Merkle memory system can be a bottleneck. Consider:
- Access pattern locality (nearby accesses are cheaper)
- Tree depth (shallower trees = fewer operations)
- Chunk size (larger chunks = fewer nodes but coarser granularity)

## Integration Guidelines

When integrating with other components:
1. Ensure consistent memory dimensions across all components
2. Use the same hasher instance for all operations
3. Coordinate bus indices to avoid conflicts
4. Handle public values correctly for proof chaining

## Debugging Commands

```rust
// Check current state
println!("Touched nodes: {}", chip.num_touched_nonleaves);
println!("Trace height: {}", chip.current_trace_height());

// Verify tree structure
assert!(chip.final_state.is_some(), "Must finalize before use");
```

## Code Style for This Component

- Use explicit types for clarity (avoid excessive type inference)
- Document any modifications to constraints
- Keep test cases focused and well-named
- Preserve existing optimization strategies

## When Modifying Constraints

1. Understand the security property being enforced
2. Test with malicious inputs to verify enforcement  
3. Document the mathematical reasoning
4. Ensure backward compatibility or version appropriately

Remember: The Merkle memory system is foundational to OpenVM's security. Changes here affect the entire proof system's integrity.