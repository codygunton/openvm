# OpenVM Memory System Rules and Guidelines

## Component-Specific Rules

### Memory Access Rules

1. **Address Space 0 is Special**
   - Address space 0 implements identity mapping
   - Reads from AS 0 return the pointer value itself
   - Never write to address space 0
   - Used for immediate values in the VM

2. **Power-of-Two Requirements**
   - All memory access sizes MUST be powers of two
   - Initial block sizes MUST be powers of two
   - Valid sizes: 1, 2, 4, 8, 16, 32

3. **Timestamp Monotonicity**
   - Timestamps must strictly increase
   - Initial timestamp is always 0
   - Never manually set timestamps - use increment methods

### Implementation Guidelines

#### When Working with Online Memory
```rust
// CORRECT: Use type parameters for access size
let (id, values) = memory.read::<4>(addr_space, pointer);

// WRONG: Don't use dynamic sizes
let (id, values) = memory.read(addr_space, pointer, 4); // This doesn't exist
```

#### When Working with Offline Memory
```rust
// ALWAYS pass adapter inventory for record keeping
offline_memory.write(as, ptr, values, &mut adapter_inventory);

// NEVER forget to finalize before proof generation
let final_memory = offline_memory.finalize::<CHUNK>(&mut adapters);
```

### Memory Configuration Best Practices

1. **Choose Initial Block Size Wisely**
   - Larger blocks (8) for sequential access patterns
   - Smaller blocks (1) for random access patterns
   - Default to 4 for mixed workloads

2. **Configure Access Capacity**
   - Set based on expected program complexity
   - Under-provisioning causes reallocation
   - Over-provisioning wastes memory

3. **Max Access Adapter Size**
   - Must be one of: 2, 4, 8, 16, 32
   - Larger values support batch operations
   - Default to 8 for most applications

### Common Pitfalls to Avoid

1. **Don't Mix Memory Types**
   ```rust
   // WRONG: Using online memory for proof generation
   let online_mem = Memory::new(&config);
   // ... use in proof generation ...
   
   // CORRECT: Use offline memory for proofs
   let offline_mem = OfflineMemory::new(image, block_size, bus, rc, config);
   ```

2. **Don't Forget Adapter Records**
   ```rust
   // WRONG: Direct block manipulation
   offline_memory.block_data.set_range(...);
   
   // CORRECT: Use proper access methods
   offline_memory.write(as, ptr, values, &mut adapters);
   ```

3. **Don't Assume Memory Layout**
   ```rust
   // WRONG: Assuming contiguous memory
   for i in 0..100 {
       memory.read(1, i);
   }
   
   // CORRECT: Track actual allocations
   for ptr in allocated_pointers {
       memory.read(1, ptr);
   }
   ```

### Testing Guidelines

1. **Test Edge Cases**
   - Empty memory access
   - Maximum address values
   - Power-of-two boundaries
   - Cross-block accesses

2. **Verify Timestamps**
   - Check monotonic increase
   - Verify initial state
   - Test increment operations

3. **Check Adapter Generation**
   - Verify split operations
   - Verify merge operations
   - Check record counts

### Performance Optimization Rules

1. **Batch Operations When Possible**
   ```rust
   // SLOW: Individual byte writes
   for i in 0..8 {
       memory.write(as, ptr + i, vec![data[i]], &mut adapters);
   }
   
   // FAST: Single batch write
   memory.write(as, ptr, data[0..8].to_vec(), &mut adapters);
   ```

2. **Minimize Address Space Switches**
   - Group operations by address space
   - Use fewer, larger address spaces
   - Consider memory layout carefully

3. **Align Access to Block Boundaries**
   - Reduces split/merge operations
   - Improves adapter efficiency
   - Simplifies proof generation

### Integration Rules

1. **With Hasher**
   - Always use provided hasher interface
   - Don't implement custom hashing
   - Ensure consistent hash parameters

2. **With Range Checker**
   - All addresses must be range-checked
   - Use shared range checker instance
   - Respect configured bit limits

3. **With Memory Bus**
   - All operations must use bus
   - Maintain correct operation order
   - Handle bus IDs properly

### Debugging Tips

1. **Memory Access Issues**
   - Check address space configuration
   - Verify pointer alignment
   - Examine adapter records

2. **Proof Generation Failures**
   - Verify finalization was called
   - Check timestamp consistency
   - Examine bus interactions

3. **Performance Problems**
   - Profile adapter usage
   - Check block fragmentation
   - Analyze access patterns

### Code Review Checklist

- [ ] All access sizes are powers of two
- [ ] Adapter inventory passed to all operations
- [ ] Memory finalized before proof generation
- [ ] Timestamps handled correctly
- [ ] Address space 0 not written to
- [ ] Error handling for edge cases
- [ ] Appropriate initial block size chosen
- [ ] Memory configuration documented
- [ ] Test coverage for access patterns
- [ ] Performance implications considered