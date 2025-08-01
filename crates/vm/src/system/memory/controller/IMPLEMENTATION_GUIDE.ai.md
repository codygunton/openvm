# Memory Controller Implementation Guide

## Module Dependencies

### Internal Dependencies
```rust
use crate::{
    arch::{hasher::HasherChip, MemoryConfig},
    system::memory::{
        adapter::AccessAdapterInventory,
        dimensions::MemoryDimensions,
        merkle::{MemoryMerkleChip, SerialReceiver},
        offline::{MemoryRecord, OfflineMemory, INITIAL_TIMESTAMP},
        offline_checker::{MemoryBus, MemoryBridge, ...},
        online::{Memory, MemoryLogEntry},
        persistent::PersistentBoundaryChip,
        tree::MemoryNode,
        volatile::VolatileBoundaryChip,
        paged_vec::{AddressMap, PAGE_SIZE},
    },
};
```

### External Dependencies
- `openvm_circuit_primitives`: Circuit building blocks
- `openvm_stark_backend`: STARK proof system integration
- `p3_field::PrimeField32`: Field arithmetic traits

## Implementation Patterns

### 1. Memory Controller Construction

#### Volatile Memory Setup
```rust
let controller = MemoryController::with_volatile_memory(
    memory_bus,
    mem_config,
    range_checker,
);
```

#### Persistent Memory Setup
```rust
let mut controller = MemoryController::with_persistent_memory(
    memory_bus,
    mem_config,
    range_checker,
    merkle_bus,
    compression_bus,
);
controller.set_initial_memory(initial_state);
```

### 2. Memory Access Patterns

#### Single Cell Access
```rust
// Read
let (record_id, value) = controller.read_cell(address_space, pointer);

// Write
let (record_id, prev_value) = controller.write_cell(address_space, pointer, new_value);
```

#### Batch Access
```rust
// Read N consecutive cells
let (record_id, values) = controller.read::<N>(address_space, pointer);

// Write N consecutive cells
let (record_id, prev_values) = controller.write::<N>(address_space, pointer, new_values);
```

#### Unsafe Access (No State Update)
```rust
// Direct memory read without logging
let value = controller.unsafe_read_cell(address_space, pointer);
```

### 3. Timestamp Management

```rust
// Increment by 1 (common case)
controller.increment_timestamp();

// Increment by custom amount
controller.increment_timestamp_by(delta);

// Query current timestamp
let current = controller.timestamp();
```

### 4. Trace Generation Workflow

```rust
// 1. Perform memory operations
controller.write_cell(addr_space, ptr, value);

// 2. Finalize (required before proof generation)
controller.finalize(Some(&mut hasher));

// 3. Generate proof inputs
let proof_inputs = controller.generate_air_proof_inputs::<StarkConfig>();
```

## State Machine Model

### States
1. **Active**: Accepting memory operations
2. **Finalized**: Ready for proof generation
3. **Consumed**: Controller moved for proof generation

### State Transitions
```
Active --[finalize()]--> Finalized --[generate_air_proof_inputs()]--> Consumed
```

## Memory Layout

### Address Space Organization
- Space 0: Reserved for immediate values
- Space 1+: User-defined memory regions
- Each space: 2^pointer_max_bits addresses

### Page Structure
- Memory organized in pages of PAGE_SIZE
- Sparse representation via BTreeMap
- Zero-initialized by default

## Auxiliary Column Generation

### Factory Pattern
```rust
let aux_factory = controller.aux_cols_factory();

// Generate auxiliary columns for reads
let mut read_aux = MemoryReadAuxCols::default();
aux_factory.generate_read_aux(&record, &mut read_aux);

// Generate auxiliary columns for writes
let mut write_aux = MemoryWriteAuxCols::default();
aux_factory.generate_write_aux(&record, &mut write_aux);
```

### Auxiliary Column Types
1. **MemoryReadAuxCols**: Timestamp verification for reads
2. **MemoryWriteAuxCols**: Previous data + timestamp verification
3. **MemoryReadOrImmediateAuxCols**: Handles address space 0

## Offline Memory Synchronization

### Access Log Replay
```rust
// Automatic during finalization
controller.finalize(hasher);

// Manual replay (advanced usage)
let entry = MemoryLogEntry::Read { address_space, pointer, len };
MemoryController::replay_access(entry, &mut offline_memory, &mut interface, &mut adapters);
```

### Log Entry Types
```rust
enum MemoryLogEntry<F> {
    Read { address_space: u32, pointer: u32, len: usize },
    Write { address_space: u32, pointer: u32, data: Vec<F> },
    IncrementTimestampBy(u32),
}
```

## Performance Optimization

### 1. Batch Operations
- Prefer `read::<N>()` over N individual reads
- Reduces proof size and generation time

### 2. Access Adapter Selection
- Sequential accesses automatically use optimized adapters
- Singleton accesses bypass adapter overhead

### 3. Memory Allocation
- Pre-allocate with appropriate `access_capacity`
- Avoid frequent reallocation during execution

### 4. Parallel Trace Generation
- Traces generated in parallel via rayon
- Ensure thread-safe access to shared resources

## Common Integration Patterns

### 1. With VM Execution
```rust
// During instruction execution
let (record_id, operand) = memory.read_cell(F::ONE, pc);
// Process instruction...
memory.increment_timestamp();
```

### 2. With Continuation System
```rust
// Save state
let merkle_root = match &memory.interface_chip {
    MemoryInterface::Persistent { boundary_chip, .. } => {
        boundary_chip.final_root()
    }
    _ => panic!("Continuation requires persistent memory"),
};

// Resume from state
memory.set_initial_memory(saved_state);
```

### 3. With Custom Instructions
```rust
// Register memory access in custom chip
let record = memory.read::<4>(addr_space, ptr);
// Use record_id for interaction with memory bus
```

## Debugging Tips

### 1. Trace Height Monitoring
```rust
let heights = controller.current_trace_heights();
println!("Memory trace heights: {:?}", heights);
```

### 2. Memory State Inspection
```rust
let image = controller.memory_image();
// Inspect specific addresses
```

### 3. Access Pattern Analysis
```rust
let logs = controller.get_memory_logs();
// Analyze access patterns for optimization
```

## Error Handling Best Practices

1. **Validate Before Access**
   ```rust
   assert!(ptr < (1 << mem_config.pointer_max_bits));
   assert_ne!(address_space, F::ZERO); // for writes
   ```

2. **Check Finalization State**
   ```rust
   if controller.final_state.is_some() {
       panic!("Controller already finalized");
   }
   ```

3. **Handle Capacity Limits**
   ```rust
   // Monitor timestamp to avoid overflow
   assert!(controller.timestamp() < (1 << mem_config.clk_max_bits));
   ```