# OpenVM Memory System Implementation Guide

## Common Implementation Patterns

### 1. Basic Memory Setup

```rust
use openvm_vm::system::memory::*;
use openvm_circuit_primitives::var_range::{SharedVariableRangeCheckerChip, VariableRangeCheckerBus};

// For execution (online memory)
fn setup_execution_memory() -> Memory<BabyBear> {
    let config = MemoryConfig {
        as_offset: 1,
        clk_max_bits: 29,
        access_capacity: 1_000_000,
        max_access_adapter_n: 8,
    };
    Memory::new(&config)
}

// For proof generation (offline memory)
fn setup_proof_memory() -> OfflineMemory<BabyBear> {
    let initial_block_size = 4; // Power of 2
    let memory_bus = MemoryBus::new(0);
    let range_checker = SharedVariableRangeCheckerChip::new(
        VariableRangeCheckerBus::new(1, 29)
    );
    
    let initial_memory = MemoryImage::default();
    let config = MemoryConfig::default();
    
    OfflineMemory::new(
        initial_memory,
        initial_block_size,
        memory_bus,
        range_checker,
        config
    )
}
```

### 2. Memory Access Patterns

#### Sequential Access Pattern
```rust
impl MemoryUser {
    fn sequential_write(&mut self, start: u32, data: &[F]) {
        // Write in chunks for efficiency
        let chunk_size = 8;
        for (i, chunk) in data.chunks(chunk_size).enumerate() {
            let ptr = start + (i * chunk_size) as u32;
            let values = chunk.try_into().unwrap_or_else(|_| {
                // Pad with zeros if needed
                let mut padded = [F::ZERO; 8];
                padded[..chunk.len()].copy_from_slice(chunk);
                padded
            });
            self.memory.write(1, ptr, values);
        }
    }
}
```

#### Random Access Pattern
```rust
impl MemoryUser {
    fn random_access(&mut self, addresses: &[(u32, u32)]) {
        // For random access, use smaller block size
        for &(addr_space, ptr) in addresses {
            let (_, value) = self.memory.read::<1>(addr_space, ptr);
            // Process value...
        }
    }
}
```

### 3. Working with Address Spaces

```rust
#[derive(Debug, Clone, Copy)]
enum AddressSpace {
    Program = 1,
    Stack = 2,
    Heap = 3,
    Global = 4,
}

impl MemoryManager {
    fn allocate_in_space(&mut self, space: AddressSpace, size: u32) -> u32 {
        let as_id = space as u32;
        let ptr = self.next_ptr[as_id as usize];
        self.next_ptr[as_id as usize] += size;
        
        // Initialize memory if needed
        for i in 0..size {
            self.memory.write(as_id, ptr + i, vec![F::ZERO]);
        }
        
        ptr
    }
}
```

### 4. Memory Finalization for Proofs

```rust
fn finalize_memory_for_proof<const CHUNK: usize>(
    offline_memory: &mut OfflineMemory<F>,
    adapter_inventory: &mut AccessAdapterInventory<F>,
) -> Result<TimestampedEquipartition<F, CHUNK>, Error> {
    // Finalize to equipartition
    let final_memory = offline_memory.finalize::<CHUNK>(adapter_inventory);
    
    // Verify all touched addresses are included
    for (key, values) in final_memory.iter() {
        assert_eq!(values.values.len(), CHUNK);
    }
    
    Ok(final_memory)
}
```

### 5. Implementing Custom Memory Chips

```rust
pub struct CustomMemoryChip<F> {
    memory_bus: MemoryBus,
    range_checker: SharedVariableRangeCheckerChip,
    records: Vec<CustomRecord<F>>,
}

impl<F: PrimeField32> CustomMemoryChip<F> {
    pub fn new(
        memory_bus: MemoryBus,
        range_checker: SharedVariableRangeCheckerChip,
    ) -> Self {
        Self {
            memory_bus,
            range_checker,
            records: Vec::new(),
        }
    }
    
    pub fn process_access(
        &mut self,
        addr_space: u32,
        ptr: u32,
        data: Vec<F>,
        timestamp: u32,
    ) {
        // Validate access
        assert!(data.len().is_power_of_two());
        
        // Record for proof generation
        self.records.push(CustomRecord {
            addr_space,
            ptr,
            data,
            timestamp,
        });
    }
}
```

### 6. Memory Bus Integration

```rust
impl<AB: InteractionBuilder> Air<AB> for CustomMemoryAir {
    fn eval(&self, builder: &mut AB) {
        let local = builder.main().row_slice(0);
        
        // Send memory read
        self.memory_bus
            .receive(
                MemoryAddress::new(local.addr_space, local.pointer),
                vec![local.value],
                local.timestamp,
            )
            .eval(builder, local.is_active);
            
        // Send memory write
        self.memory_bus
            .send(
                MemoryAddress::new(local.addr_space, local.pointer),
                vec![local.new_value],
                local.timestamp + AB::F::ONE,
            )
            .eval(builder, local.is_write);
    }
}
```

### 7. Handling Memory Images

```rust
fn load_program_memory(program: &[u8]) -> MemoryImage<F> {
    let mut memory = MemoryImage::new(1); // as_offset = 1
    
    // Load program into address space 1
    for (i, &byte) in program.iter().enumerate() {
        memory.insert(
            &(1, i as u32),
            F::from_canonical_u8(byte)
        );
    }
    
    // Initialize stack in address space 2
    let stack_start = 0x10000;
    let stack_size = 0x1000;
    for i in 0..stack_size {
        memory.insert(
            &(2, stack_start + i),
            F::ZERO
        );
    }
    
    memory
}
```

### 8. Memory Access Adapter Usage

```rust
fn handle_variable_size_access(
    offline_memory: &mut OfflineMemory<F>,
    adapters: &mut AccessAdapterInventory<F>,
    addr_space: u32,
    ptr: u32,
    size: usize,
) {
    // Round up to nearest power of 2
    let access_size = size.next_power_of_two();
    
    match access_size {
        1 => offline_memory.read(addr_space, ptr, 1, adapters),
        2 => offline_memory.read(addr_space, ptr, 2, adapters),
        4 => offline_memory.read(addr_space, ptr, 4, adapters),
        8 => offline_memory.read(addr_space, ptr, 8, adapters),
        16 => offline_memory.read(addr_space, ptr, 16, adapters),
        32 => offline_memory.read(addr_space, ptr, 32, adapters),
        _ => panic!("Unsupported access size: {}", access_size),
    }
}
```

### 9. Memory Verification Pattern

```rust
struct MemoryVerifier<F> {
    expected_final_state: HashMap<(u32, u32), F>,
}

impl<F: PrimeField32> MemoryVerifier<F> {
    fn verify_final_memory(
        &self,
        final_memory: &TimestampedEquipartition<F, 1>,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        for ((as_id, ptr), expected) in &self.expected_final_state {
            match final_memory.get(&(*as_id, *ptr)) {
                Some(timestamped) => {
                    if timestamped.values[0] != *expected {
                        errors.push(format!(
                            "Mismatch at ({}, {}): expected {}, got {}",
                            as_id, ptr, expected, timestamped.values[0]
                        ));
                    }
                }
                None => {
                    errors.push(format!(
                        "Missing memory at ({}, {})",
                        as_id, ptr
                    ));
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

### 10. Performance-Optimized Memory Pattern

```rust
pub struct OptimizedMemory<F> {
    memory: OfflineMemory<F>,
    write_buffer: HashMap<(u32, u32), Vec<F>>,
    buffer_threshold: usize,
}

impl<F: PrimeField32> OptimizedMemory<F> {
    pub fn buffered_write(
        &mut self,
        addr_space: u32,
        ptr: u32,
        values: Vec<F>,
        adapters: &mut AccessAdapterInventory<F>,
    ) {
        self.write_buffer.insert((addr_space, ptr), values);
        
        if self.write_buffer.len() >= self.buffer_threshold {
            self.flush_writes(adapters);
        }
    }
    
    fn flush_writes(&mut self, adapters: &mut AccessAdapterInventory<F>) {
        // Sort by address for better locality
        let mut writes: Vec<_> = self.write_buffer.drain().collect();
        writes.sort_by_key(|((as_id, ptr), _)| (*as_id, *ptr));
        
        for ((as_id, ptr), values) in writes {
            self.memory.write(as_id, ptr, values, adapters);
        }
    }
}
```

## Error Handling Patterns

```rust
#[derive(Debug)]
enum MemoryError {
    InvalidAddressSpace(u32),
    UnalignedAccess { ptr: u32, alignment: u32 },
    SizeNotPowerOfTwo(usize),
    TimestampViolation { current: u32, requested: u32 },
}

fn safe_memory_access(
    memory: &mut OfflineMemory<F>,
    addr_space: u32,
    ptr: u32,
    size: usize,
) -> Result<(), MemoryError> {
    // Validate address space
    if addr_space == 0 || addr_space > MAX_ADDRESS_SPACE {
        return Err(MemoryError::InvalidAddressSpace(addr_space));
    }
    
    // Check size is power of 2
    if !size.is_power_of_two() {
        return Err(MemoryError::SizeNotPowerOfTwo(size));
    }
    
    // Check alignment
    if ptr % size as u32 != 0 {
        return Err(MemoryError::UnalignedAccess {
            ptr,
            alignment: size as u32,
        });
    }
    
    Ok(())
}
```

## Testing Patterns

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn setup_test_memory() -> (OfflineMemory<F>, AccessAdapterInventory<F>) {
        // Standard test setup
        let memory_bus = MemoryBus::new(0);
        let range_checker = SharedVariableRangeCheckerChip::new(
            VariableRangeCheckerBus::new(1, 16)
        );
        let config = MemoryConfig {
            as_offset: 1,
            clk_max_bits: 16,
            access_capacity: 1000,
            max_access_adapter_n: 8,
        };
        
        let memory = OfflineMemory::new(
            MemoryImage::default(),
            4,
            memory_bus,
            range_checker.clone(),
            config,
        );
        
        let adapters = AccessAdapterInventory::new(
            range_checker,
            memory_bus,
            config.clk_max_bits,
            config.max_access_adapter_n,
        );
        
        (memory, adapters)
    }
    
    #[test]
    fn test_memory_pattern() {
        let (mut memory, mut adapters) = setup_test_memory();
        
        // Test implementation...
    }
}
```