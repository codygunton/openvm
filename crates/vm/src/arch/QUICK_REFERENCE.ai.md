# OpenVM Architecture Component - Quick Reference

## Essential Imports

```rust
use openvm_stark_backend::{p3_field::PrimeField32, Chip};
use openvm_circuit_derive::{AnyEnum, InstructionExecutor};
use openvm_circuit_primitives_derive::{Chip, ChipUsageGetter};
use crate::arch::{
    VmExtension, VmInventoryBuilder, VmInventory, 
    InstructionExecutor, VmChipWrapper,
    VmAdapterChip, VmCoreChip, VmAdapterInterface,
    ExecutionState, ExecutionError, SystemPort,
    BasicAdapterInterface, MinimalInstruction,
    AdapterRuntimeContext, AdapterAirContext,
};
```

## Quick Patterns

### Minimal Extension
```rust
impl<F: PrimeField32> VmExtension<F> for MyExt {
    type Executor = MyExecutor<F>;
    type Periphery = ();
    
    fn build(&self, builder: &mut VmInventoryBuilder<F>) -> Result<VmInventory<Self::Executor, Self::Periphery>, VmInventoryError> {
        let mut inventory = VmInventory::new();
        let executor = MyExecutor::new(builder.system_port());
        inventory.add_executor(executor, [MY_OPCODE])?;
        Ok(inventory)
    }
}
```

### Simple Executor
```rust
impl<F: PrimeField32> InstructionExecutor<F> for MyExecutor<F> {
    fn execute(&mut self, memory: &mut MemoryController<F>, instruction: &Instruction<F>, from_state: ExecutionState<u32>) -> Result<ExecutionState<u32>> {
        // Your logic here
        Ok(ExecutionState {
            pc: from_state.pc + DEFAULT_PC_STEP,
            timestamp: from_state.timestamp + 1,
        })
    }
    
    fn get_opcode_name(&self, opcode: usize) -> String {
        "MY_OP".to_string()
    }
}
```

### Basic Adapter Pattern
```rust
type MyChip<F> = VmChipWrapper<F, MyAdapter<F>, MyCore<F>>;

// Adapter: handles memory
impl<F: PrimeField32> VmAdapterChip<F> for MyAdapter<F> {
    type Interface = BasicAdapterInterface<F, MinimalInstruction<F>, 2, 1, 1, 1>;
    type ReadRecord = (u32, u32); // addresses
    type WriteRecord = u32; // address
    type Air = MyAdapterAir;
    
    fn preprocess(&mut self, memory: &mut MemoryController<F>, instruction: &Instruction<F>) 
        -> Result<([[F; 1]; 2], Self::ReadRecord)> {
        let a = memory.read::<1>(instruction.a.as_canonical_u32(), 0);
        let b = memory.read::<1>(instruction.b.as_canonical_u32(), 0);
        Ok(([a, b], (instruction.a.as_canonical_u32(), instruction.b.as_canonical_u32())))
    }
    
    fn postprocess(&mut self, memory: &mut MemoryController<F>, instruction: &Instruction<F>, 
                   from_state: ExecutionState<u32>, output: AdapterRuntimeContext<F, Self::Interface>, 
                   _: &Self::ReadRecord) -> Result<(ExecutionState<u32>, Self::WriteRecord)> {
        memory.write(instruction.c.as_canonical_u32(), output.writes[0][0], from_state.timestamp);
        Ok((ExecutionState::new(from_state.pc + 1, from_state.timestamp + 3), instruction.c.as_canonical_u32()))
    }
    
    fn generate_trace_row(&self, row_slice: &mut [F], read: Self::ReadRecord, write: Self::WriteRecord, memory: &OfflineMemory<F>) {
        // Fill trace
    }
    
    fn air(&self) -> &Self::Air { &MyAdapterAir }
}

// Core: handles computation
impl<F: PrimeField32> VmCoreChip<F, <MyAdapter<F> as VmAdapterChip<F>>::Interface> for MyCore<F> {
    type Record = (F, F, F); // a, b, result
    type Air = MyCoreAir;
    
    fn execute_instruction(&self, _: &Instruction<F>, _: u32, reads: [[F; 1]; 2]) 
        -> Result<(AdapterRuntimeContext<F, _>, Self::Record)> {
        let a = reads[0][0];
        let b = reads[1][0];
        let result = a + b;
        Ok((AdapterRuntimeContext::without_pc([[result]]), (a, b, result)))
    }
    
    fn generate_trace_row(&self, row_slice: &mut [F], record: Self::Record) {
        row_slice[0] = record.0; // a
        row_slice[1] = record.1; // b
        row_slice[2] = record.2; // result
    }
    
    fn air(&self) -> &Self::Air { &MyCoreAir }
    fn get_opcode_name(&self, _: usize) -> String { "ADD".to_string() }
}
```

## Common Operations

### Memory Access
```rust
// Read single word
let value = memory.read::<1>(address, timestamp)[0];

// Read multiple words
let values = memory.read::<4>(address, timestamp);

// Write single word
memory.write(address, value, timestamp);

// Get current timestamp
let ts = memory.timestamp();
```

### Bus Operations
```rust
// Execution bus
execution_bus.execute(builder, enabled, from_state, to_state);

// Memory bridge
let read_aux = memory_bridge.read_aux(address, timestamp);
let write_aux = memory_bridge.write_aux(address, value, timestamp);

// Program bus
program_bus.lookup_instruction(builder, pc, opcode, operands, enabled);
```

### State Management
```rust
// Create execution state
let state = ExecutionState::new(pc, timestamp);

// Increment PC
let next_state = ExecutionState {
    pc: state.pc + DEFAULT_PC_STEP,
    timestamp: state.timestamp + memory_ops_count,
};

// Custom PC
let jump_state = ExecutionState {
    pc: target_pc,
    timestamp: state.timestamp + 1,
};
```

## Interface Types

### BasicAdapterInterface
```rust
// Template: <F, PI, NUM_READS, NUM_WRITES, READ_SIZE, WRITE_SIZE>
type Simple = BasicAdapterInterface<F, MinimalInstruction<F>, 2, 1, 1, 1>;
// 2 reads of 1 word, 1 write of 1 word

type Complex = BasicAdapterInterface<F, MinimalInstruction<F>, 3, 2, 4, 4>;
// 3 reads of 4 words, 2 writes of 4 words
```

### VecHeapAdapterInterface
```rust
// Template: <F, NUM_READS, BLOCKS_PER_READ, BLOCKS_PER_WRITE, READ_SIZE, WRITE_SIZE>
type VecInterface = VecHeapAdapterInterface<F, 1, 8, 8, 1, 1>;
// 1 read of up to 8 blocks, write up to 8 blocks
```

### ProcessedInstruction Types
```rust
// Minimal: just validity and opcode
MinimalInstruction { is_valid: F, opcode: F }

// With immediate
ImmInstruction { is_valid: F, opcode: F, immediate: F }

// With signed immediate
SignedImmInstruction { is_valid: F, opcode: F, immediate: F, imm_sign: F }
```

## Error Handling

```rust
// Common execution errors
ExecutionError::DisabledOperation { pc, opcode }
ExecutionError::PcNotFound { pc, step, pc_base, program_len }
ExecutionError::PublicValueIndexOutOfBounds { pc, num_public_values, public_value_index }
ExecutionError::FailedWithExitCode(code)

// Check opcode
match opcode.local_opcode_idx(Self::OPCODE_OFFSET) {
    Some(MY_OP) => { /* handle */ }
    _ => Err(ExecutionError::DisabledOperation { pc: from_state.pc, opcode })
}
```

## Configuration

### Basic VM Config
```rust
let config = SystemConfig::default()
    .with_max_constraint_degree(3)
    .with_continuations()
    .with_public_values(32);
```

### Memory Config
```rust
let mem_config = MemoryConfig::new(
    3,      // as_height (8 address spaces)
    1,      // as_offset
    29,     // pointer_max_bits
    29,     // clk_max_bits
    17,     // decomp
    32,     // max_access_adapter_n
    1 << 24 // access_capacity
);
```

## VM Usage

### Setup and Execute
```rust
let config = MyVmConfig::default();
let engine = BabyBearBlake3Engine::new(proving_config);
let vm = VirtualMachine::new(engine, config);

// Execute
let exe = VmExe { program, pc_start: 0, init_memory: vec![] };
let input = Streams::new(vec![vec![input_value]]);
let memory_state = vm.execute(exe, input)?;
```

### Generate Proof
```rust
// Execute and generate
let result = vm.execute_and_generate(exe, input)?;

// Prove
let pk = vm.keygen();
let proofs = vm.prove(&pk, result);

// Verify
let vk = pk.get_vk();
vm.verify(&vk, proofs)?;
```

## Chip Registration

### Add to Inventory
```rust
// Single opcode
inventory.add_executor(executor, [OPCODE])?;

// Multiple opcodes
inventory.add_executor(executor, [ADD, SUB, MUL])?;

// Periphery chip
inventory.add_periphery_chip(periphery);
```

### Access System Resources
```rust
let SystemPort { execution_bus, program_bus, memory_bridge } = builder.system_port();
let range_bus = builder.system_base().range_checker_bus();
let memory_controller = builder.system_base().memory_controller();
```

## Testing Helpers

```rust
// Create test instruction
let instr = Instruction::new(OPCODE, a, b, c);

// Test execution state
let state = ExecutionState::new(0, 100);

// Mock memory controller
let mut memory = MemoryController::new(config);
memory.write(addr, value, 0);

// Verify trace
assert_eq!(chip.current_trace_height(), expected_height);
```