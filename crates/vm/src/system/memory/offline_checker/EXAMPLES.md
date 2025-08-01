# Memory Offline Checker Examples

## Basic Usage Examples

### Setting Up Memory Bridge

```rust
use openvm_circuit_primitives::var_range::VariableRangeCheckerBus;
use crate::system::memory::offline_checker::{MemoryBridge, MemoryBus};

// System configuration
const CLK_MAX_BITS: usize = 16;  // 2^16 = 65536 max timestamp
const MEMORY_BUS_INDEX: usize = 1;
const RANGE_BUS_INDEX: usize = 2;

// Create components
let memory_bus = MemoryBus::new(MEMORY_BUS_INDEX);
let range_bus = VariableRangeCheckerBus::new(RANGE_BUS_INDEX);

// Create memory bridge
let bridge = MemoryBridge::new(memory_bus, CLK_MAX_BITS, range_bus);
```

### Simple Memory Read

```rust
use crate::system::memory::{MemoryAddress, offline_checker::columns::MemoryReadAuxCols};

// Prepare memory read operation
let address = MemoryAddress::new(1, 0x1000);  // AS=1, pointer=0x1000
let data = [F::from_canonical_u32(42), F::from_canonical_u32(43)]; // 2-word read
let timestamp = F::from_canonical_u32(100);

// Auxiliary columns (typically populated by memory controller)
let aux_cols = MemoryReadAuxCols::new(
    99,  // prev_timestamp
    lt_aux_data,  // timestamp comparison auxiliary data
);

// Create and evaluate read operation
let read_op = bridge.read(address, data, timestamp, &aux_cols);
read_op.eval(builder, enabled_condition);
```

### Memory Write Operation

```rust
use crate::system::memory::offline_checker::columns::MemoryWriteAuxCols;

// Memory write with previous data
let address = MemoryAddress::new(1, 0x2000);
let new_data = [F::from_canonical_u32(100), F::from_canonical_u32(101)];
let prev_data = [F::from_canonical_u32(50), F::from_canonical_u32(51)];
let timestamp = F::from_canonical_u32(200);

// Write auxiliary columns include previous data
let aux_cols = MemoryWriteAuxCols::new(
    prev_data,
    199,  // prev_timestamp  
    lt_aux_data,
);

// Create and evaluate write operation
let write_op = bridge.write(address, new_data, timestamp, &aux_cols);
write_op.eval(builder, enabled_condition);
```

### Read or Immediate Operation

```rust
use crate::system::memory::offline_checker::columns::MemoryReadOrImmediateAuxCols;

// Address space 0 for immediate value
let immediate_address = MemoryAddress::new(0, 42);  // AS=0, immediate value=42
let expected_data = F::from_canonical_u32(42);  // Should equal pointer for AS=0
let timestamp = F::from_canonical_u32(300);

// Auxiliary columns for immediate detection
let aux_cols = MemoryReadOrImmediateAuxCols {
    base: MemoryBaseAuxCols {
        prev_timestamp: F::from_canonical_u32(299),
        timestamp_lt_aux: lt_aux_data,
    },
    is_immediate: F::ONE,  // Flag indicating immediate value
    is_zero_aux: F::ZERO,  // Auxiliary for is_zero check
};

// Create and evaluate read-or-immediate operation
let read_imm_op = bridge.read_or_immediate(
    immediate_address, 
    expected_data, 
    timestamp, 
    &aux_cols
);
read_imm_op.eval(builder, enabled_condition);
```

## Advanced Usage Patterns

### Conditional Memory Operations

```rust
// Memory operation that may be disabled
let is_memory_enabled = some_condition_expr;

let read_op = bridge.read(address, data, timestamp, &aux_cols);
read_op.eval(builder, is_memory_enabled);

// When disabled (is_memory_enabled = 0), no constraints are added
```

### Batch Memory Operations

```rust
// Process multiple memory operations efficiently
struct MemoryOperationBatch {
    addresses: Vec<MemoryAddress<F, F>>,
    data: Vec<[F; 2]>,
    timestamps: Vec<F>,
    aux_cols: Vec<MemoryReadAuxCols<F>>,
}

impl MemoryOperationBatch {
    fn eval_all<AB>(&self, builder: &mut AB, bridge: &MemoryBridge)
    where
        AB: InteractionBuilder<Var = F, Expr = F>,
    {
        for (i, ((address, data), timestamp)) in self.addresses
            .iter()
            .zip(&self.data)
            .zip(&self.timestamps)
            .enumerate()
        {
            let read_op = bridge.read(*address, *data, *timestamp, &self.aux_cols[i]);
            read_op.eval(builder, F::ONE);  // Always enabled
        }
    }
}
```

### Memory Initialization Pattern

```rust
// Initialize memory locations with known values
fn initialize_memory<AB>(
    builder: &mut AB,
    bridge: &MemoryBridge,
    initial_values: &[(MemoryAddress<F, F>, Vec<F>)],
) where
    AB: InteractionBuilder<Var = F, Expr = F>,
{
    for (i, (address, data)) in initial_values.iter().enumerate() {
        // First access has prev_timestamp = 0, no previous data
        let aux_cols = MemoryWriteAuxCols::new(
            vec![F::ZERO; data.len()].try_into().unwrap(),  // prev_data = 0
            0,  // prev_timestamp = 0
            create_lt_aux(0, i + 1),  // timestamp comparison
        );
        
        let write_op = bridge.write(
            *address,
            data.clone().try_into().unwrap(),
            F::from_canonical_usize(i + 1),  // timestamps 1, 2, 3, ...
            &aux_cols,
        );
        write_op.eval(builder, F::ONE);
    }
}
```

## Integration Examples

### With Memory Controller

```rust
use crate::system::memory::controller::MemoryController;

struct MemorySystem<F: PrimeField32> {
    controller: MemoryController<F>,
    bridge: MemoryBridge,
}

impl<F: PrimeField32> MemorySystem<F> {
    fn constrain_read<AB>(&self, builder: &mut AB, addr: u32, expected_data: Vec<F>)
    where
        AB: InteractionBuilder<Var = F, Expr = F>,
    {
        // Get auxiliary data from controller
        let (timestamp, aux_cols) = self.controller.prepare_read(addr);
        let address = MemoryAddress::new(1, addr);
        
        // Constrain the read operation
        let read_op = self.bridge.read(
            address,
            expected_data.try_into().unwrap(),
            timestamp,
            &aux_cols,
        );
        read_op.eval(builder, F::ONE);
    }
}
```

### Custom AIR Implementation

```rust
use openvm_stark_backend::p3_air::{Air, AirBuilder};

#[derive(Clone)]
struct CustomMemoryAir {
    bridge: MemoryBridge,
}

impl<AB: AirBuilder> Air<AB> for CustomMemoryAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let aux = builder.aux();
        
        // Extract values from trace
        let address_space = main.address_space();
        let pointer = main.pointer();
        let data = main.data();
        let timestamp = main.timestamp();
        let enabled = main.enabled();
        
        // Extract auxiliary columns
        let memory_aux: &MemoryReadAuxCols<AB::Var> = aux.memory_aux();
        
        // Constrain memory operation
        let address = MemoryAddress::new(address_space, pointer);
        let read_op = self.bridge.read(address, data, timestamp, memory_aux);
        read_op.eval(builder, enabled);
    }
}
```

## Testing Examples

### Unit Test Setup

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use openvm_stark_backend::p3_field::PrimeField32;
    
    type F = openvm_stark_backend::p3_baby_bear::BabyBear;
    
    fn create_test_bridge() -> MemoryBridge {
        let memory_bus = MemoryBus::new(1);
        let range_bus = VariableRangeCheckerBus::new(2);
        MemoryBridge::new(memory_bus, 16, range_bus)
    }
    
    #[test]
    fn test_simple_read() {
        let bridge = create_test_bridge();
        let mut builder = MockInteractionBuilder::new();
        
        let address = MemoryAddress::new(F::ONE, F::from_canonical_u32(0x1000));
        let data = [F::from_canonical_u32(42)];
        let timestamp = F::from_canonical_u32(100);
        let aux_cols = create_mock_aux_cols(99);
        
        let read_op = bridge.read(address, data, timestamp, &aux_cols);
        read_op.eval(&mut builder, F::ONE);
        
        // Verify constraints were added correctly
        assert!(builder.verify_constraints());
    }
}
```

### Integration Test Pattern

```rust
#[test]
fn test_memory_sequence() {
    let bridge = create_test_bridge();
    let mut builder = MockInteractionBuilder::new();
    
    // Sequence: write, read, write, read
    let address = MemoryAddress::new(F::ONE, F::from_canonical_u32(0x1000));
    
    // Initial write
    let write_op1 = bridge.write(
        address,
        [F::from_canonical_u32(100)],
        F::from_canonical_u32(1),
        &create_write_aux(vec![F::ZERO], 0),
    );
    write_op1.eval(&mut builder, F::ONE);
    
    // Read back
    let read_op = bridge.read(
        address,
        [F::from_canonical_u32(100)],
        F::from_canonical_u32(2),
        &create_read_aux(1),
    );
    read_op.eval(&mut builder, F::ONE);
    
    // Verify all bus interactions balance
    assert!(builder.verify_bus_balance());
}
```

## Best Practices

### Auxiliary Column Management
```rust
// Always populate auxiliary columns correctly
// Incorrect auxiliary data leads to constraint failures

// GOOD: Use controller-generated auxiliary data
let aux_cols = memory_controller.prepare_read_aux(address, timestamp);

// BAD: Manual auxiliary data (error-prone)
let aux_cols = MemoryReadAuxCols { /* manual values */ };
```

### Timestamp Management
```rust
// GOOD: Monotonically increasing timestamps
let mut current_time = 0;
for operation in operations {
    current_time += 1;
    // Use current_time for this operation
}

// BAD: Non-monotonic timestamps cause constraint failures
```

### Error Handling
```rust
// Memory operations can fail constraint evaluation
// Handle gracefully in production systems

match memory_operation.try_eval(builder, enabled) {
    Ok(()) => { /* success */ },
    Err(constraint_error) => {
        log::error!("Memory constraint failed: {}", constraint_error);
        // Handle error appropriately
    }
}
```