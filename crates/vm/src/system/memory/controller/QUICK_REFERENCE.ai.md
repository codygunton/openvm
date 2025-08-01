# Memory Controller Quick Reference

## Construction
```rust
// Volatile (default)
let mc = MemoryController::with_volatile_memory(bus, config, range_checker);

// Persistent (for continuations)
let mc = MemoryController::with_persistent_memory(bus, config, range_checker, merkle_bus, compression_bus);
mc.set_initial_memory(initial_state);
```

## Basic Operations
```rust
// Single cell read/write
let (id, val) = mc.read_cell(addr_space, ptr);
let (id, old) = mc.write_cell(addr_space, ptr, new_val);

// Batch operations (more efficient)
let (id, vals) = mc.read::<8>(addr_space, ptr);
let (id, olds) = mc.write::<8>(addr_space, ptr, new_vals);

// Unsafe reads (no proof generation)
let val = mc.unsafe_read_cell(addr_space, ptr);
```

## Timestamp Management
```rust
mc.increment_timestamp();        // +1
mc.increment_timestamp_by(10);   // +10
let ts = mc.timestamp();         // Current
```

## Proof Generation Flow
```rust
// 1. Execute operations
mc.write_cell(F::ONE, F::ZERO, data);

// 2. Finalize (required!)
mc.finalize(Some(&mut hasher));  // Persistent
mc.finalize(None);                // Volatile

// 3. Generate proofs
let proofs = mc.generate_air_proof_inputs::<SC>();
```

## Memory Layout
- Address Space 0: Immediate values (read-only)
- Address Space 1+: Regular memory
- Max pointer: `2^pointer_max_bits - 1`
- Page size: `PAGE_SIZE` (from paged_vec)

## Key Types
```rust
RecordId(usize)                                    // Memory operation ID
TimestampedValues<T, N> { timestamp, values }     // Timestamped data
MemoryLogEntry<F>                                  // Access log entry
MemoryTraceHeights                                 // Trace dimensions
```

## Common Patterns

### Check if persistent
```rust
let is_persistent = mc.continuation_enabled();
```

### Get memory state
```rust
let image = mc.memory_image();  // Current state
let logs = mc.get_memory_logs(); // Access history
```

### Auxiliary columns
```rust
let factory = mc.aux_cols_factory();
factory.generate_read_aux(&record, &mut aux_cols);
```

### Trace heights
```rust
let heights = mc.current_trace_heights();
let cells = mc.current_trace_cells();
```

## Constants
- `CHUNK = 8`: Memory access granularity
- `MERKLE_AIR_OFFSET = 1`: AIR index for Merkle
- `BOUNDARY_AIR_OFFSET = 0`: AIR index for boundary

## Error Cases
- Write to address space 0 → panic
- Pointer >= 2^pointer_max_bits → panic  
- Finalize twice → no-op (idempotent)
- Access after finalize → undefined

## Performance Tips
1. Use batch operations when possible
2. Sequential access → automatic optimization
3. Pre-allocate with proper access_capacity
4. Reuse RecordId for related operations