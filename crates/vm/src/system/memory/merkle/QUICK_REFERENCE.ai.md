# Merkle Memory System Quick Reference

## Core Components

### MemoryMerkleChip
```rust
// Creation
let chip = MemoryMerkleChip::<CHUNK, F>::new(
    memory_dimensions,  // Memory layout config
    merkle_bus,        // Memory interaction bus  
    compression_bus    // Hash computation bus
);

// Key methods
chip.touch_range(address_space, address, length);
chip.set_overridden_height(height);
chip.finalize(&initial_tree, &final_memory, &mut hasher);
chip.generate_air_proof_input() -> AirProofInput<SC>;
```

### MemoryDimensions
```rust
struct MemoryDimensions {
    as_height: usize,        // Bits for address spaces (2^n spaces)
    address_height: usize,   // Bits for addresses per space
    as_offset: u32,         // Starting address space number
}

// Total tree height = as_height + address_height
dimensions.overall_height()
```

### MemoryMerkleCols (Trace Row)
```rust
struct MemoryMerkleCols<T, CHUNK> {
    expand_direction: T,      // 1=initial, -1=final, 0=padding
    height_section: T,        // 0=address, 1=address_space
    parent_height: T,         // Level in tree
    is_root: T,              // Root node flag
    parent_as_label: T,      // Address space label
    parent_address_label: T,  // Address label
    parent_hash: [T; CHUNK], // Node hash
    left_child_hash: [T; CHUNK],
    right_child_hash: [T; CHUNK],
    left_direction_different: T,  // Optimization flags
    right_direction_different: T,
}
```

## Common Operations

### Initialize and Use Chip
```rust
// 1. Create chip
let mut chip = MemoryMerkleChip::<8, BabyBear>::new(dims, merkle_bus, compression_bus);

// 2. Mark accessed memory
chip.touch_range(address_space, start_addr, length);

// 3. Finalize with initial tree and final memory
chip.finalize(&initial_tree, &final_partition, &mut hasher);

// 4. Generate proof
let proof_input = chip.generate_air_proof_input();
```

### Touch Patterns
```rust
// Single chunk access
chip.touch_range(as_id, chunk_addr * CHUNK, CHUNK);

// Multi-chunk range
chip.touch_range(as_id, start_addr, total_length);

// Sparse access - touch each chunk
for addr in addresses {
    let chunk_start = (addr / CHUNK) * CHUNK;
    chip.touch_range(as_id, chunk_start, CHUNK);
}
```

## Key Constraints (from AIR)

1. **Direction validity**: `expand_direction ∈ {-1, 0, 1}`
2. **Height ordering**: Rows sorted by parent_height (descending)
3. **Root constraints**: First two rows must be roots
4. **Tree structure**: Proper parent-child relationships
5. **Hash validity**: Compression function correctly applied

## Bus Interactions

### Merkle Bus
- **Purpose**: Coordinate memory accesses
- **Fields**: [direction, height, as_label, address_label, hash...]
- **Multiplicity**: Based on expand_direction

### Compression Bus  
- **Purpose**: Record hash computations
- **Fields**: [left_hash..., right_hash..., parent_hash...]
- **Multiplicity**: expand_direction²

## Testing Utilities

### HashTestChip (Mock Hasher)
```rust
let mut hasher = HashTestChip::new();
// Simple deterministic hash: sum of inputs
hasher.compress_and_record(&left, &right); // Returns left + right
```

### Test Template
```rust
fn test_merkle_memory() {
    // Setup
    let dims = MemoryDimensions::new(as_height, addr_height, as_offset);
    let mut chip = MemoryMerkleChip::new(dims, merkle_bus, compression_bus);
    
    // Operations
    chip.touch_range(as_id, addr, len);
    
    // Finalize
    let initial_tree = MemoryNode::tree_from_memory(dims, &initial_mem, &hasher);
    chip.finalize(&initial_tree, &final_partition, &mut hasher);
    
    // Verify
    assert_eq!(chip.final_state.unwrap().final_root, expected_root);
}
```

## Performance Metrics

```rust
// Trace height (number of rows)
chip.current_trace_height() // = 2 * num_touched_nonleaves

// Efficiency metric
chip.num_touched_nonleaves // Minimize for better performance
```

## Common Patterns

### Memory to Partition Conversion
```rust
fn memory_to_partition<F, const N: usize>(
    memory: &MemoryImage<F>
) -> Equipartition<F, N> {
    let mut partition = Equipartition::new();
    for ((as_id, ptr), val) in memory.items() {
        let label = (as_id, ptr / N as u32);
        partition.entry(label)
            .or_insert([F::default(); N])
            [ptr % N as u32] = val;
    }
    partition
}
```

### Tree Construction
```rust
// From memory image
let tree = MemoryNode::tree_from_memory(
    dimensions,
    &memory_image,
    &hasher
);

// Uniform tree (all zeros)
let zero_tree = MemoryNode::construct_uniform(
    height,
    [F::ZERO; CHUNK],
    &hasher
);
```

## Debugging Tips

1. **Constraint Failures**
   - Check trace row ordering
   - Verify expand_direction values
   - Ensure hash computations match

2. **Missing Nodes**
   - All accessed memory must be touched
   - Parent nodes automatically touched
   - Root always touched

3. **Performance Issues**
   - Minimize scattered accesses
   - Align to chunk boundaries
   - Batch nearby operations

## Constants

- `CHUNK`: Elements per memory location (typically 8)
- `PAGE_SIZE`: Memory page size (from paged_vec)
- Tree height = `as_height + address_height`

## Error Conditions

1. **Assertion Failures**
   - `as_height > 0` required
   - `address_height > 0` required
   - Cannot finalize twice
   - Must finalize before trace generation

2. **Constraint Violations**
   - Invalid expand_direction
   - Incorrect tree structure
   - Hash mismatches
   - Wrong public values

## Integration Example

```rust
// In memory controller
impl MemoryController {
    fn execute_with_merkle_proof(&mut self) {
        // Track accesses
        for access in memory_accesses {
            self.merkle_chip.touch_range(
                access.address_space,
                access.pointer,
                access.length
            );
        }
        
        // Finalize at end
        self.merkle_chip.finalize(
            &self.initial_tree,
            &self.final_memory,
            &mut self.hasher
        );
    }
}
```