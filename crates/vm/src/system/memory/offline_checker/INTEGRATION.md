# Memory Offline Checker Integration Guide

## Integration Overview

The Memory Offline Checker integrates deeply with OpenVM's memory system, circuit primitives, and STARK backend. This document details key integration points and patterns.

## Core Dependencies

### Circuit Primitives Integration

#### Range Checking
```rust
use openvm_circuit_primitives::var_range::VariableRangeCheckerBus;

// Integration with range checker for timestamp validation
let bridge = MemoryBridge::new(memory_bus, clk_max_bits, range_bus);
```

#### Constraint Primitives
```rust
use openvm_circuit_primitives::{
    assert_less_than::{AssertLessThanIo, AssertLtSubAir},
    is_zero::{IsZeroIo, IsZeroSubAir},
};

// Used internally for timestamp and immediate value constraints
```

### STARK Backend Integration

#### Interaction Builder Pattern
```rust
use openvm_stark_backend::interaction::InteractionBuilder;

impl<F: FieldAlgebra, V: Copy + Into<F>, const N: usize> 
    MemoryReadOperation<'_, F, V, N> 
{
    pub fn eval<AB>(self, builder: &mut AB, enabled: impl Into<AB::Expr>)
    where
        AB: InteractionBuilder<Var = V, Expr = F>,
    {
        // Constraint evaluation using builder pattern
    }
}
```

#### Bus System
```rust
use openvm_stark_backend::interaction::{BusIndex, PermutationCheckBus};

// Memory bus wraps permutation check bus
pub struct MemoryBus {
    pub inner: PermutationCheckBus,
}
```

## Memory System Integration

### Address Types
```rust
use crate::system::memory::MemoryAddress;

// Unified address type across memory system
let address = MemoryAddress::new(address_space, pointer);
```

### Column Alignment
```rust
use openvm_circuit_primitives_derive::AlignedBorrow;

#[repr(C)]
#[derive(AlignedBorrow)]
pub struct MemoryBaseAuxCols<T> {
    // Aligned for efficient field access
}
```

## Bus Interaction Patterns

### Send/Receive Model
```rust
// Memory writes send to bus
self.memory_bus
    .send(address, data.to_vec(), timestamp)
    .eval(builder, enabled.clone());

// Memory reads receive from bus  
self.memory_bus
    .receive(address, prev_data.to_vec(), prev_timestamp)
    .eval(builder, enabled);
```

### Direction Control
```rust
// Flexible direction control for complex operations
bus_interaction.eval(builder, direction); // direction ∈ {-1, 0, 1}
```

## AIR Integration Patterns

### Constraint Evaluation
```rust
// Standard pattern for memory operation constraints
operation.eval(builder, enabled_condition);

// Builder automatically handles:
// - Bus interactions
// - Timestamp constraints  
// - Data consistency checks
```

### Auxiliary Column Management
```rust
// Columns must be pre-allocated and populated
let aux_cols = MemoryReadAuxCols::new(prev_timestamp, lt_aux);
let operation = bridge.read(address, data, timestamp, &aux_cols);
```

## Type System Integration

### Generic Expression Types
```rust
// T represents expressions (typically AB::Expr)
// V represents variables (typically AB::Var)
pub struct MemoryReadOperation<'a, T, V, const N: usize> {
    // Flexible type system for different AIR contexts
}
```

### Compile-Time Constants
```rust
// Block size N is compile-time constant for efficiency
const BLOCK_SIZE: usize = 4;
let operation = bridge.read::<BLOCK_SIZE>(address, data, timestamp, &aux);
```

## Bus Index Management

### Unique Bus Allocation
```rust
// Each memory space gets unique bus index
const MEMORY_BUS_INDEX: BusIndex = 1;
let memory_bus = MemoryBus::new(MEMORY_BUS_INDEX);
```

### Multi-Bus Coordination
```rust
// Coordinate with other system buses
let range_bus = VariableRangeCheckerBus::new(RANGE_BUS_INDEX);
let memory_bus = MemoryBus::new(MEMORY_BUS_INDEX);
```

## Error Integration

### Constraint Failures
```rust
// Constraints automatically propagate through builder
// Failed constraints appear in proof generation errors
```

### Debug Integration
```rust
// Use constraint evaluation for debugging
#[cfg(debug_assertions)]
operation.eval(&mut debug_builder, enabled);
```

## Performance Integration

### Batch Operations
```rust
// Batch multiple memory operations for efficiency
for (addr, data, timestamp) in operations {
    bridge.read(addr, data, timestamp, &aux_cols[i])
        .eval(builder, enabled);
}
```

### Constraint Degree Management
```rust
// Keep constraint degrees bounded
// Max degree = deg(enabled) + max(expression_degrees)
```

## Configuration Integration

### Parameterized Setup
```rust
pub fn create_memory_bridge(
    clk_max_bits: usize,      // From system config
    range_bus: VariableRangeCheckerBus, // Shared resource
) -> MemoryBridge {
    let memory_bus = MemoryBus::new(allocate_bus_index());
    MemoryBridge::new(memory_bus, clk_max_bits, range_bus)
}
```

### AUX_LEN Coordination
```rust
// AUX_LEN must satisfy: (clk_max_bits + decomp - 1) / decomp = AUX_LEN
const AUX_LEN: usize = 2;
// Verify this matches system parameters
```

## Testing Integration

### Unit Test Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_operation() {
        let bridge = create_test_bridge();
        let operation = bridge.read(test_address, test_data, timestamp, &aux);
        // Test constraint evaluation
    }
}
```

### Integration Test Support
```rust
// Support for end-to-end memory system testing
// Verify bus interaction balancing
// Check constraint satisfaction
```

## Future Integration Points

### Extensibility Hooks
- Plugin architecture for custom memory models
- Configurable constraint sets
- Alternative bus architectures

### Optimization Opportunities  
- Vectorized operations
- Specialized constraint evaluation
- Custom auxiliary column layouts