# OpenVM Architecture Component - Implementation Guide

## Overview

This guide provides detailed instructions for implementing VM extensions, custom instruction executors, and integrating with the OpenVM architecture component.

## Table of Contents

1. [Creating a VM Extension](#creating-a-vm-extension)
2. [Implementing Instruction Executors](#implementing-instruction-executors)
3. [Adapter-Core Pattern](#adapter-core-pattern)
4. [Memory Integration](#memory-integration)
5. [Segmentation Strategies](#segmentation-strategies)
6. [Testing Your Implementation](#testing-your-implementation)

## Creating a VM Extension

### Basic Extension Structure

```rust
use openvm_stark_backend::p3_field::PrimeField32;
use crate::arch::{VmExtension, VmInventoryBuilder, VmInventory, VmInventoryError};

pub struct MyExtension {
    // Extension configuration
}

impl<F: PrimeField32> VmExtension<F> for MyExtension {
    type Executor = MyExecutorEnum<F>;  // Enum of all executors
    type Periphery = MyPeripheryEnum<F>; // Enum of periphery chips
    
    fn build(
        &self,
        builder: &mut VmInventoryBuilder<F>,
    ) -> Result<VmInventory<Self::Executor, Self::Periphery>, VmInventoryError> {
        let mut inventory = VmInventory::new();
        
        // Get system resources
        let system_port = builder.system_port();
        
        // Create and register executors
        let executor = MyInstructionExecutor::new(
            system_port.execution_bus,
            system_port.program_bus,
            system_port.memory_bridge,
        );
        inventory.add_executor(executor, [MY_OPCODE_1, MY_OPCODE_2])?;
        
        // Add periphery chips if needed
        let periphery = MyPeripheryChip::new(builder.new_bus_idx());
        inventory.add_periphery_chip(periphery);
        
        Ok(inventory)
    }
}
```

### Executor Enum with Derive Macros

```rust
use openvm_circuit_derive::{AnyEnum, InstructionExecutor};
use openvm_circuit_primitives_derive::{Chip, ChipUsageGetter};

#[derive(ChipUsageGetter, Chip, AnyEnum, From, InstructionExecutor)]
pub enum MyExecutorEnum<F: PrimeField32> {
    Arithmetic(ArithmeticExecutor<F>),
    Memory(MemoryExecutor<F>),
    #[any_enum]  // For nested enums
    Complex(ComplexExecutorEnum<F>),
}
```

## Implementing Instruction Executors

### Direct Executor Implementation

```rust
pub struct ArithmeticExecutor<F> {
    // Chip state
    records: Vec<ArithmeticRecord>,
}

impl<F: PrimeField32> InstructionExecutor<F> for ArithmeticExecutor<F> {
    fn execute(
        &mut self,
        memory: &mut MemoryController<F>,
        instruction: &Instruction<F>,
        from_state: ExecutionState<u32>,
    ) -> Result<ExecutionState<u32>> {
        let Instruction { opcode, a, b, c, .. } = instruction;
        
        // Validate opcode
        match opcode.local_opcode_idx(Self::OPCODE_OFFSET) {
            Some(ADD) => {
                // Perform addition
                let result = a.as_canonical_u32() + b.as_canonical_u32();
                
                // Write to memory
                memory.write(c.as_canonical_u32(), result.into(), from_state.timestamp);
                
                // Record for trace generation
                self.records.push(ArithmeticRecord { a: *a, b: *b, result });
                
                // Return next state
                Ok(ExecutionState {
                    pc: from_state.pc + DEFAULT_PC_STEP,
                    timestamp: from_state.timestamp + 1,
                })
            }
            _ => Err(ExecutionError::DisabledOperation { 
                pc: from_state.pc, 
                opcode: *opcode 
            }),
        }
    }
    
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode - Self::OPCODE_OFFSET {
            ADD => "ADD".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }
}
```

### Using VmChipWrapper Pattern

For more complex executors, use the adapter-core pattern:

```rust
pub type ArithmeticChip<F> = VmChipWrapper<F, ArithmeticAdapter<F>, ArithmeticCore<F>>;

impl ArithmeticChip<F> {
    pub fn new(memory_bridge: MemoryBridge, offline_memory: Arc<Mutex<OfflineMemory<F>>>) -> Self {
        let adapter = ArithmeticAdapter::new(memory_bridge);
        let core = ArithmeticCore::new();
        VmChipWrapper::new(adapter, core, offline_memory)
    }
}
```

## Adapter-Core Pattern

### Implementing an Adapter

```rust
pub struct ArithmeticAdapter<F> {
    memory_bridge: MemoryBridge,
}

impl<F: PrimeField32> VmAdapterChip<F> for ArithmeticAdapter<F> {
    type ReadRecord = ArithmeticReadRecord<F>;
    type WriteRecord = ArithmeticWriteRecord<F>;
    type Air = ArithmeticAdapterAir;
    type Interface = BasicAdapterInterface<F, MinimalInstruction<F>, 2, 1, 1, 1>;
    
    fn preprocess(
        &mut self,
        memory: &mut MemoryController<F>,
        instruction: &Instruction<F>,
    ) -> Result<(
        <Self::Interface as VmAdapterInterface<F>>::Reads,
        Self::ReadRecord,
    )> {
        // Read operands from memory
        let a_val = memory.read::<1>(instruction.a.as_canonical_u32(), 0);
        let b_val = memory.read::<1>(instruction.b.as_canonical_u32(), 0);
        
        // Create read record
        let record = ArithmeticReadRecord {
            a_ptr: instruction.a.as_canonical_u32(),
            b_ptr: instruction.b.as_canonical_u32(),
            timestamp: memory.timestamp(),
        };
        
        Ok(([[a_val[0]], [b_val[0]]], record))
    }
    
    fn postprocess(
        &mut self,
        memory: &mut MemoryController<F>,
        instruction: &Instruction<F>,
        from_state: ExecutionState<u32>,
        output: AdapterRuntimeContext<F, Self::Interface>,
        read_record: &Self::ReadRecord,
    ) -> Result<(ExecutionState<u32>, Self::WriteRecord)> {
        // Write result to memory
        let [[result]] = output.writes;
        memory.write(instruction.c.as_canonical_u32(), result, read_record.timestamp);
        
        // Create write record
        let record = ArithmeticWriteRecord {
            c_ptr: instruction.c.as_canonical_u32(),
            result,
        };
        
        // Calculate next state
        let to_state = ExecutionState {
            pc: output.to_pc.unwrap_or(from_state.pc + DEFAULT_PC_STEP),
            timestamp: read_record.timestamp + 3, // 2 reads + 1 write
        };
        
        Ok((to_state, record))
    }
    
    fn generate_trace_row(
        &self,
        row_slice: &mut [F],
        read_record: Self::ReadRecord,
        write_record: Self::WriteRecord,
        memory: &OfflineMemory<F>,
    ) {
        // Populate trace row with adapter data
        // This includes memory access traces
    }
    
    fn air(&self) -> &Self::Air {
        &ArithmeticAdapterAir
    }
}
```

### Implementing a Core

```rust
pub struct ArithmeticCore<F> {
    // Core configuration
}

impl<F: PrimeField32, I: VmAdapterInterface<F>> VmCoreChip<F, I> for ArithmeticCore<F> 
where
    I::Reads: Into<[[F; 1]; 2]>,
    I::Writes: From<[[F; 1]; 1]>,
{
    type Record = ArithmeticCoreRecord<F>;
    type Air = ArithmeticCoreAir;
    
    fn execute_instruction(
        &self,
        instruction: &Instruction<F>,
        from_pc: u32,
        reads: I::Reads,
    ) -> Result<(AdapterRuntimeContext<F, I>, Self::Record)> {
        let [[a], [b]] = reads.into();
        
        // Perform computation
        let result = match instruction.opcode.local_opcode_idx(Self::OPCODE_OFFSET) {
            Some(ADD) => a + b,
            Some(SUB) => a - b,
            Some(MUL) => a * b,
            _ => return Err(ExecutionError::DisabledOperation { 
                pc: from_pc, 
                opcode: instruction.opcode 
            }),
        };
        
        // Create record for trace generation
        let record = ArithmeticCoreRecord { a, b, result, opcode: instruction.opcode };
        
        // Return output
        let output = AdapterRuntimeContext::without_pc([[result]].into());
        Ok((output, record))
    }
    
    fn generate_trace_row(&self, row_slice: &mut [F], record: Self::Record) {
        // Populate trace row with core computation data
    }
    
    fn air(&self) -> &Self::Air {
        &ArithmeticCoreAir
    }
}
```

## Memory Integration

### Basic Memory Operations

```rust
// Reading from memory
let value = memory.read::<1>(address, timestamp);  // Read 1 word
let values = memory.read::<4>(address, timestamp); // Read 4 words

// Writing to memory
memory.write(address, value, timestamp);

// Batch operations through memory bridge
let read_aux = memory_bridge.read_aux(address, timestamp);
let write_aux = memory_bridge.write_aux(address, value, timestamp);
```

### Using Different Adapter Interfaces

```rust
// Basic interface: fixed reads/writes
type Interface = BasicAdapterInterface<F, MinimalInstruction<F>, 2, 1, 1, 1>;
// 2 reads of size 1, 1 write of size 1

// Vector heap interface: variable-length operations
type Interface = VecHeapAdapterInterface<F, 1, 4, 4, 1, 1>;
// 1 read of up to 4 blocks, write up to 4 blocks

// Dynamic interface: runtime-determined
type Interface = DynAdapterInterface<F>;
// Flexible but less efficient
```

## Segmentation Strategies

### Custom Segmentation Strategy

```rust
#[derive(Debug)]
pub struct MySegmentationStrategy {
    max_cycles: usize,
    max_memory_accesses: usize,
}

impl SegmentationStrategy for MySegmentationStrategy {
    fn should_segment(
        &self,
        air_names: &[String],
        trace_heights: &[usize],
        trace_cells: &[usize],
    ) -> bool {
        // Check cycle count
        if let Some(connector_idx) = air_names.iter().position(|n| n.contains("Connector")) {
            if trace_heights[connector_idx] > self.max_cycles {
                tracing::info!("Segmenting due to cycle count: {}", trace_heights[connector_idx]);
                return true;
            }
        }
        
        // Check memory accesses
        if let Some(memory_idx) = air_names.iter().position(|n| n.contains("Memory")) {
            if trace_heights[memory_idx] > self.max_memory_accesses {
                tracing::info!("Segmenting due to memory accesses: {}", trace_heights[memory_idx]);
                return true;
            }
        }
        
        false
    }
    
    fn stricter_strategy(&self) -> Arc<dyn SegmentationStrategy> {
        Arc::new(MySegmentationStrategy {
            max_cycles: self.max_cycles / 2,
            max_memory_accesses: self.max_memory_accesses / 2,
        })
    }
}
```

### Setting Custom Strategy

```rust
let mut config = SystemConfig::default().with_continuations();
config.set_segmentation_strategy(Arc::new(MySegmentationStrategy {
    max_cycles: 1_000_000,
    max_memory_accesses: 500_000,
}));
```

## Testing Your Implementation

### Unit Testing Executors

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use openvm_stark_backend::p3_baby_bear::BabyBear;
    
    #[test]
    fn test_arithmetic_executor() {
        let mut executor = ArithmeticExecutor::<BabyBear>::new();
        let mut memory = MemoryController::new(/* config */);
        
        // Setup memory
        memory.write(100, BabyBear::from(5), 0);
        memory.write(101, BabyBear::from(3), 0);
        
        // Create instruction
        let instruction = Instruction {
            opcode: VmOpcode::from_usize(ADD_OPCODE),
            a: BabyBear::from(100),
            b: BabyBear::from(101),
            c: BabyBear::from(102),
            ..Default::default()
        };
        
        // Execute
        let from_state = ExecutionState::new(0, 1);
        let to_state = executor.execute(&mut memory, &instruction, from_state).unwrap();
        
        // Verify
        assert_eq!(to_state.pc, DEFAULT_PC_STEP);
        assert_eq!(memory.read::<1>(102, to_state.timestamp)[0], BabyBear::from(8));
    }
}
```

### Integration Testing

```rust
#[test]
fn test_extension_integration() {
    use openvm_circuit::testing::{VmChipTestBuilder, TestAdapterChip};
    
    let mut builder = VmChipTestBuilder::new();
    let config = MyExtension::new();
    
    // Build extension
    let inventory = config.build(&mut builder).unwrap();
    
    // Create test program
    let program = vec![
        Instruction::new(ADD_OPCODE, 100, 101, 102),
        Instruction::new(TERMINATE_OPCODE, 0, 0, 0),
    ];
    
    // Run test
    builder.run(program, vec![]).unwrap();
}
```

### Constraint Testing

```rust
#[test]
fn test_arithmetic_constraints() {
    use openvm_stark_backend::verifier::VerificationError;
    
    let chip = ArithmeticChip::<BabyBear>::new(/* deps */);
    
    // Generate valid trace
    let trace = generate_valid_trace(&chip);
    
    // Test constraints pass
    assert!(verify_constraints(&chip, &trace).is_ok());
    
    // Corrupt trace
    let mut bad_trace = trace.clone();
    bad_trace.values[0] = BabyBear::from(999);
    
    // Test constraints fail
    assert!(matches!(
        verify_constraints(&chip, &bad_trace),
        Err(VerificationError::ConstraintFailure(_))
    ));
}
```

## Best Practices

1. **Memory Access**: Always use the memory controller for reads/writes
2. **Timestamp Management**: Increment timestamp for each memory operation
3. **Error Handling**: Return proper execution errors with context
4. **Trace Generation**: Ensure trace row generation matches AIR constraints
5. **Testing**: Test both execution and constraint satisfaction
6. **Documentation**: Document opcodes, memory layouts, and constraints

## Common Pitfalls

1. **Forgetting to increment timestamp**: Each memory operation needs unique timestamp
2. **Incorrect opcode offset**: Use `local_opcode_idx` to get local opcode
3. **Memory alignment**: Some operations require aligned addresses
4. **Trace padding**: Ensure padded rows satisfy constraints
5. **Bus interactions**: Register all bus operations correctly