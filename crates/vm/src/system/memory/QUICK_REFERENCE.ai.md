# OpenVM Memory System Quick Reference

## Essential Types

```rust
// Memory address with address space and pointer
MemoryAddress<S, T> { address_space: S, pointer: T }

// Memory operation types
OpType::Read = 0
OpType::Write = 1
```

## Memory Types Overview

| Type | Purpose | Use Case |
|------|---------|----------|
| `Memory<F>` | Online execution | Runtime memory access |
| `OfflineMemory<F>` | Proof generation | Creating memory proofs |
| `MemoryImage<F>` | Initial state | Loading program memory |
| `PersistentBoundaryChip` | Persistent proofs | Initial/final state verification |
| `VolatileBoundaryChip` | Volatile proofs | Temporary memory verification |

## Quick Setup

### Online Memory (Execution)
```rust
let config = MemoryConfig::default();
let mut memory = Memory::new(&config);

// Read/Write
let (id, values) = memory.read::<4>(addr_space, pointer);
let (id, prev) = memory.write::<4>(addr_space, pointer, [v1, v2, v3, v4]);
```

### Offline Memory (Proofs)
```rust
let mut offline_mem = OfflineMemory::new(
    initial_memory,
    initial_block_size, // Must be power of 2
    memory_bus,
    range_checker,
    config
);

// Always pass adapters
offline_mem.read(as, ptr, len, &mut adapters);
offline_mem.write(as, ptr, values, &mut adapters);

// Finalize before proof
let final_memory = offline_mem.finalize::<CHUNK>(&mut adapters);
```

## Valid Access Sizes

| Size | Use Case |
|------|----------|
| 1 | Byte access |
| 2 | Short/word access |
| 4 | Standard word |
| 8 | Double word |
| 16 | Vector operations |
| 32 | Large batch ops |

## Address Spaces

| AS | Purpose | Special Rules |
|----|---------|---------------|
| 0 | Identity | Returns pointer value, no writes |
| 1+ | User memory | Normal read/write operations |

## Common Operations

### Initialize Memory
```rust
let mut image = MemoryImage::new(as_offset);
image.insert(&(addr_space, pointer), value);
```

### Batch Operations
```rust
// Efficient batch write
memory.write::<8>(as, ptr, [v0, v1, v2, v3, v4, v5, v6, v7]);

// Inefficient individual writes - AVOID
for i in 0..8 {
    memory.write::<1>(as, ptr + i, [values[i]]);
}
```

### Memory Configuration
```rust
MemoryConfig {
    as_offset: 1,              // First user address space
    clk_max_bits: 29,          // Timestamp bits
    access_capacity: 1_000_000, // Expected accesses
    max_access_adapter_n: 8,    // Max adapter size
}
```

## Adapter Inventory

```rust
let mut adapters = AccessAdapterInventory::new(
    range_checker,
    memory_bus,
    clk_max_bits,
    max_adapter_n  // 2, 4, 8, 16, or 32
);
```

## Memory Bus Operations

```rust
// In AIR constraints
memory_bus.send(
    MemoryAddress::new(addr_space, pointer),
    vec![value],
    timestamp
).eval(builder, condition);

memory_bus.receive(
    MemoryAddress::new(addr_space, pointer),
    vec![value],
    timestamp
).eval(builder, condition);
```

## Finalization Steps

```rust
// 1. Finalize offline memory
let final_memory = offline_memory.finalize::<CHUNK>(&mut adapters);

// 2. Finalize chips
persistent_chip.finalize(&initial_memory, &final_memory, &mut hasher);
volatile_chip.finalize(final_memory_volatile);
```

## Common Patterns

### Sequential Access
```rust
for i in 0..len {
    memory.read::<1>(as, start + i);
}
```

### Aligned Block Access
```rust
let aligned_ptr = (ptr / block_size) * block_size;
memory.read::<BLOCK_SIZE>(as, aligned_ptr);
```

### Memory Allocation
```rust
fn allocate(size: u32) -> u32 {
    let ptr = self.next_free;
    self.next_free += size;
    ptr
}
```

## Error Checklist

- ✓ Access size is power of 2
- ✓ Not writing to AS 0
- ✓ Adapters passed to offline ops
- ✓ Memory finalized before proof
- ✓ Timestamps monotonic
- ✓ Block size configured correctly

## Performance Tips

1. **Use larger access sizes** when possible
2. **Batch operations** to reduce overhead
3. **Align to block boundaries** for efficiency
4. **Choose initial block size** based on access pattern:
   - Sequential: 8
   - Random: 1-2
   - Mixed: 4

## Debug Commands

```rust
// Check memory state
println!("Timestamp: {}", memory.timestamp());
println!("Value at (1, 100): {:?}", memory.get(1, 100));

// Verify adapter records
println!("Total adapter records: {}", adapters.total_records());

// Check final memory
for ((as_id, ptr), values) in final_memory.iter() {
    println!("({}, {}): {:?}", as_id, ptr, values);
}
```

## Key Constants

```rust
const INITIAL_TIMESTAMP: u32 = 0;
const PAGE_SIZE: usize = 1024;
const MAX_ADDRESS_SPACE: u32 = 256;
```

## Chip Integration

```rust
// Create chips
let persistent_chip = PersistentBoundaryChip::new(
    memory_dims,
    memory_bus,
    merkle_bus,
    compression_bus
);

let volatile_chip = VolatileBoundaryChip::new(
    memory_bus,
    addr_space_max_bits,
    pointer_max_bits,
    range_checker
);
```

## Quick Troubleshooting

| Problem | Check |
|---------|-------|
| Access fails | Size is power of 2? AS != 0 for writes? |
| Proof fails | Memory finalized? Adapters used? |
| Performance slow | Block size appropriate? Batching used? |
| Timestamp error | Using increment methods? Not manual? |
| Missing memory | All accesses recorded? AS configured? |