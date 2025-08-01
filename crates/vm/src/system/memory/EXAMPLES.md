# OpenVM Memory System Examples

## Basic Memory Operations

### Online Memory Usage

```rust
use openvm_vm::system::memory::{Memory, MemoryConfig};
use openvm_stark_backend::p3_field::AbstractField;

// Initialize memory with configuration
let mem_config = MemoryConfig {
    access_capacity: 1000,
    // ... other config fields
};
let mut memory = Memory::new(&mem_config);

// Write data to memory
let address_space = 1;
let pointer = 0x1000;
let data = vec![42u32, 100u32, 200u32];
memory.write(address_space, pointer, data);

// Read data back
let (record_id, values) = memory.read::<3>(address_space, pointer);
assert_eq!(values, [42u32, 100u32, 200u32]);

// Increment timestamp for next operation
memory.increment_timestamp();
```

### Offline Memory for Proof Generation

```rust
use openvm_vm::system::memory::{OfflineMemory, MemoryImage, AccessAdapterInventory};
use openvm_circuit_primitives::var_range::SharedVariableRangeCheckerChip;

// Setup for offline memory
let image = MemoryImage::default();
let initial_block_size = 4; // Must be power of two
let range_checker = SharedVariableRangeCheckerChip::new();
let memory_bus = MemoryBus::new();
let mem_config = MemoryConfig { /* ... */ };

let mut offline_memory = OfflineMemory::new(
    image,
    initial_block_size,
    memory_bus,
    range_checker,
    &mem_config,
);

// Track adapters for proof generation
let mut adapter_inventory = AccessAdapterInventory::default();

// Perform operations with adapter tracking
offline_memory.write(1, 0x2000, vec![1, 2, 3, 4], &mut adapter_inventory);
let (_, values) = offline_memory.read::<4>(1, 0x2000, &mut adapter_inventory);

// Finalize for proof generation
let final_memory = offline_memory.finalize::<8>(&mut adapters);
```

## Address Space Management

### Working with Multiple Address Spaces

```rust
// Address space 0 is special - returns pointer value on read
let (_, immediate_value) = memory.read::<1>(0, 42);
assert_eq!(immediate_value[0], 42); // Returns the pointer itself

// Normal address spaces for data storage
let data_space = 1;
let stack_space = 2;
let heap_space = 3;

// Store different types of data in different spaces
memory.write(data_space, 0x1000, vec![1, 2, 3, 4]);     // Program data
memory.write(stack_space, 0x2000, vec![10, 20, 30]);    // Stack data
memory.write(heap_space, 0x3000, vec![100, 200]);       // Heap data
```

### Memory Address Structure

```rust
use openvm_vm::system::memory::MemoryAddress;

// Create memory addresses
let addr1 = MemoryAddress::new(1u32, 0x1000u32);
let addr2 = MemoryAddress::new(2u32, 0x2000u32);

// Convert between address types
let converted: MemoryAddress<u32, u32> = MemoryAddress::from(addr1);

// Use in memory operations
let data = vec![42];
memory.write(addr1.address_space, addr1.pointer, data);
```

## Power-of-Two Access Patterns

### Correct Access Sizes

```rust
// All access sizes must be powers of two
memory.write(1, 0x1000, vec![1]);                    // 1 byte ✓
memory.write(1, 0x1001, vec![1, 2]);                 // 2 bytes ✓
memory.write(1, 0x1004, vec![1, 2, 3, 4]);           // 4 bytes ✓
memory.write(1, 0x1008, vec![1, 2, 3, 4, 5, 6, 7, 8]); // 8 bytes ✓

// Read with type parameters for compile-time size checking
let (_, data1) = memory.read::<1>(1, 0x1000);  // Read 1 element
let (_, data2) = memory.read::<2>(1, 0x1001);  // Read 2 elements
let (_, data4) = memory.read::<4>(1, 0x1004);  // Read 4 elements
let (_, data8) = memory.read::<8>(1, 0x1008);  // Read 8 elements
```

### Batch Operations for Efficiency

```rust
// Inefficient: Multiple small writes
for i in 0..8 {
    memory.write(1, 0x1000 + i, vec![data[i as usize]]);
    memory.increment_timestamp();
}

// Efficient: Single batch write
memory.write(1, 0x1000, data[0..8].to_vec());
memory.increment_timestamp();

// Efficient: Aligned block operations
let block_size = 8;
for chunk_start in (0..data.len()).step_by(block_size) {
    let chunk_end = (chunk_start + block_size).min(data.len());
    let chunk = &data[chunk_start..chunk_end];
    memory.write(1, 0x1000 + chunk_start as u32, chunk.to_vec());
    memory.increment_timestamp();
}
```

## Memory Configuration Examples

### Basic Configuration

```rust
use openvm_vm::arch::MemoryConfig;

let basic_config = MemoryConfig {
    access_capacity: 1000,              // Expected number of memory operations
    // Additional configuration fields...
};
```

### Performance-Optimized Configuration

```rust
// Configuration for high-throughput applications
let high_perf_config = MemoryConfig {
    access_capacity: 10000,             // Higher capacity for more operations
    // Optimized settings for batch operations
};

// Configuration for memory-constrained environments
let low_memory_config = MemoryConfig {
    access_capacity: 100,               // Lower capacity to save memory
    // Conservative settings
};
```

## Timestamp Management

### Proper Timestamp Handling

```rust
// Timestamps start at 0 and must increase monotonically
assert_eq!(memory.timestamp(), 1); // Initial timestamp after creation

// Single increment
memory.increment_timestamp();
assert_eq!(memory.timestamp(), 2);

// Batch increment for efficiency
memory.increment_timestamp_by(5);
assert_eq!(memory.timestamp(), 7);

// Each operation should be followed by timestamp increment
memory.write(1, 0x1000, vec![42]);
memory.increment_timestamp();

let (_, value) = memory.read::<1>(1, 0x1000);
memory.increment_timestamp();
```

### Timestamp Ordering in Complex Operations

```rust
// Complex operation with multiple memory accesses
fn complex_operation(memory: &mut Memory<F>) {
    // Read input data
    let (_, input) = memory.read::<4>(1, 0x1000);
    memory.increment_timestamp();
    
    // Process data (compute result)
    let result: Vec<_> = input.iter().map(|x| x * 2).collect();
    
    // Write result
    memory.write(1, 0x2000, result);
    memory.increment_timestamp();
    
    // Update status flag
    memory.write(2, 0x3000, vec![1]); // Status: complete
    memory.increment_timestamp();
}
```

## Error Handling Examples

### Handling Common Errors

```rust
use std::result::Result;

fn safe_memory_operation(memory: &mut Memory<F>) -> Result<Vec<F>, &'static str> {
    // Check address space bounds
    let address_space = 1;
    if address_space == 0 {
        return Err("Cannot write to address space 0");
    }
    
    // Ensure access size is power of two
    let access_size = 4;
    if !access_size.is_power_of_two() {
        return Err("Access size must be power of two");
    }
    
    // Perform safe operation
    let data = vec![1, 2, 3, 4];
    memory.write(address_space, 0x1000, data);
    memory.increment_timestamp();
    
    let (_, result) = memory.read::<4>(address_space, 0x1000);
    memory.increment_timestamp();
    
    Ok(result.to_vec())
}
```

### Capacity Management

```rust
fn check_memory_capacity(memory: &Memory<F>) -> bool {
    // Check if approaching capacity limit
    let usage_ratio = memory.log.len() as f64 / memory.config.access_capacity as f64;
    
    if usage_ratio > 0.9 {
        println!("Warning: Memory capacity at {}%", usage_ratio * 100.0);
        return false;
    }
    
    true
}

fn memory_operation_with_capacity_check(memory: &mut Memory<F>) -> Result<(), &'static str> {
    if !check_memory_capacity(memory) {
        return Err("Memory capacity exceeded");
    }
    
    // Proceed with operation
    memory.write(1, 0x1000, vec![42]);
    memory.increment_timestamp();
    
    Ok(())
}
```

## Integration with Proof System

### Preparing Memory for Proof Generation

```rust
use openvm_vm::system::memory::offline_checker::{MemoryBridge, MemoryBus};

fn generate_memory_proof(memory_log: Vec<MemoryLogEntry<F>>) {
    // Setup offline memory from log
    let mut offline_memory = OfflineMemory::from_log(memory_log, /* config */);
    let mut adapter_inventory = AccessAdapterInventory::default();
    
    // Replay operations to generate adapters
    for entry in &memory_log {
        match entry {
            MemoryLogEntry::Read { address_space, pointer, len } => {
                offline_memory.read_dynamic(*address_space, *pointer, *len, &mut adapter_inventory);
            }
            MemoryLogEntry::Write { address_space, pointer, data } => {
                offline_memory.write(*address_space, *pointer, data.clone(), &mut adapter_inventory);
            }
            MemoryLogEntry::IncrementTimestampBy(n) => {
                offline_memory.increment_timestamp_by(*n);
            }
        }
    }
    
    // Finalize and generate proof components
    let final_memory = offline_memory.finalize::<8>(&mut adapter_inventory);
    // Continue with proof generation...
}
```

## Memory Image Operations

### Loading Initial State

```rust
use openvm_vm::system::memory::MemoryImage;

// Create memory image with initial data
let mut image = MemoryImage::default();

// Load program data
image.set_range(1, 0x1000, vec![1, 2, 3, 4, 5]); // Program instructions
image.set_range(2, 0x2000, vec![100, 200, 300]);  // Initial data

// Create memory from image
let memory = Memory::from_image(image, 1000);

// Verify initial state
let (_, program_data) = memory.read::<5>(1, 0x1000);
assert_eq!(program_data, [1, 2, 3, 4, 5]);
```

### Persisting Memory State

```rust
fn save_memory_state(memory: &Memory<F>) -> MemoryImage<F> {
    // Extract current state as image
    memory.data.clone()
}

fn restore_memory_state(image: MemoryImage<F>) -> Memory<F> {
    Memory::from_image(image, 1000)
}

// Usage example
let checkpoint = save_memory_state(&memory);
// ... perform operations ...
let restored_memory = restore_memory_state(checkpoint);
```

## Advanced Usage Patterns

### Memory Pool Management

```rust
struct MemoryPool<F> {
    memories: Vec<Memory<F>>,
    current: usize,
}

impl<F> MemoryPool<F> {
    fn new(pool_size: usize, config: &MemoryConfig) -> Self {
        let memories = (0..pool_size)
            .map(|_| Memory::new(config))
            .collect();
        
        Self {
            memories,
            current: 0,
        }
    }
    
    fn get_memory(&mut self) -> &mut Memory<F> {
        &mut self.memories[self.current]
    }
    
    fn rotate(&mut self) {
        self.current = (self.current + 1) % self.memories.len();
    }
}
```

### Memory Access Patterns

```rust
// Sequential access pattern
fn sequential_write(memory: &mut Memory<F>, base_addr: u32, data: &[F]) {
    for (i, &value) in data.iter().enumerate() {
        memory.write(1, base_addr + i as u32, vec![value]);
        memory.increment_timestamp();
    }
}

// Block access pattern
fn block_write(memory: &mut Memory<F>, base_addr: u32, data: &[F], block_size: usize) {
    for chunk in data.chunks(block_size) {
        let offset = (chunk.as_ptr() as usize - data.as_ptr() as usize) / std::mem::size_of::<F>();
        memory.write(1, base_addr + offset as u32, chunk.to_vec());
        memory.increment_timestamp();
    }
}

// Strided access pattern
fn strided_access(memory: &mut Memory<F>, base_addr: u32, stride: u32, count: usize) -> Vec<F> {
    let mut result = Vec::new();
    
    for i in 0..count {
        let addr = base_addr + i as u32 * stride;
        let (_, value) = memory.read::<1>(1, addr);
        result.push(value[0]);
        memory.increment_timestamp();
    }
    
    result
}
```

These examples demonstrate the proper usage patterns for the OpenVM memory system, including best practices for performance, error handling, and integration with the broader zkVM framework.