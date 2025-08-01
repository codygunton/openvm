# Memory Controller Component Instructions

## Component Overview
You are working with the Memory Controller component of OpenVM, which manages all memory operations and generates proofs for memory accesses. This is a critical security component that ensures memory integrity in the zkVM.

## Key Principles

1. **Memory Safety First**: All memory accesses must be bounds-checked and authenticated
2. **Proof Correctness**: Any modification must maintain proof soundness
3. **Performance Awareness**: Memory operations are on the critical path

## Working with This Component

### Before Making Changes
1. Read `AI_DOCS.md` for architectural overview
2. Review `IMPLEMENTATION_GUIDE.ai.md` for patterns
3. Check if changes affect proof generation
4. Consider both volatile and persistent memory modes

### Code Modification Guidelines

#### Adding New Features
- Extend `MemoryInterface` enum for mode-specific features
- Update both volatile and persistent implementations
- Add corresponding methods to `MemoryController`
- Update trace generation in `generate_air_proof_inputs`

#### Modifying Memory Access
- Maintain record ID generation for proof system
- Update both online (`Memory`) and offline (`OfflineMemory`) paths
- Ensure `replay_access_log` handles new access patterns
- Test with access adapter optimizations

#### Performance Optimizations
- Benchmark with realistic memory access patterns
- Consider batch operations for repeated accesses
- Profile trace generation time
- Minimize auxiliary column generation overhead

### Common Pitfalls to Avoid

1. **Forgetting Address Space 0**: Reserved for immediate values, no writes allowed
2. **Skipping Finalization**: Must call `finalize()` before proof generation
3. **Timestamp Overflow**: Check against `clk_max_bits` limit
4. **Thread Safety**: `offline_memory` is shared, use mutex properly
5. **Mode Confusion**: Volatile and persistent have different requirements

### Testing Requirements

When modifying this component:
1. Test both memory modes (volatile/persistent)
2. Verify proof generation succeeds
3. Check edge cases (bounds, address space 0)
4. Benchmark performance impact
5. Test with continuation system (persistent mode)

### Integration Considerations

This component interacts with:
- **Range Checker**: For address validation
- **Access Adapters**: For optimized access patterns
- **Hasher Chip**: For Merkle tree operations (persistent)
- **VM Core**: For instruction execution
- **Continuation System**: For state persistence

### Security Checklist

Before submitting changes:
- [ ] No unbounded memory allocations
- [ ] All accesses are range-checked
- [ ] Timestamp ordering is maintained
- [ ] Proof generation remains sound
- [ ] No information leakage in traces

### Performance Targets

Maintain or improve:
- Single access: < 100ns
- Batch access: < 20ns per element
- Trace generation: Linear in access count
- Memory overhead: < 2x working set

### Documentation Updates

When changing functionality:
1. Update inline documentation
2. Modify AI_DOCS.md if architecture changes
3. Update IMPLEMENTATION_GUIDE.ai.md for new patterns
4. Add examples for new APIs

## Component-Specific Commands

### Building and Testing
```bash
# Run component tests
cargo test -p openvm-circuit memory::controller

# Benchmark memory operations
cargo bench -p openvm-circuit memory_controller

# Check proof generation
cargo test -p openvm-circuit memory_proof_generation
```

### Debugging
```rust
// Enable detailed memory logs
std::env::set_var("RUST_LOG", "openvm_circuit::memory=trace");

// Dump memory state
println!("{:#?}", controller.memory_image());

// Trace access patterns
for entry in controller.get_memory_logs() {
    println!("{:?}", entry);
}
```

## Future Considerations

When extending this component, consider:
1. Hardware acceleration interfaces
2. Alternative proof systems
3. Compressed memory representations
4. Streaming proof generation
5. Multi-level memory hierarchies