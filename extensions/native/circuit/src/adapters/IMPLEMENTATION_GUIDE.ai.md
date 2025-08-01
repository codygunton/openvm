# Native Circuit Adapters Implementation Guide

## Creating a New Adapter

### Basic Adapter Structure
```rust
use std::{borrow::{Borrow, BorrowMut}, marker::PhantomData};
use openvm_circuit::{
    arch::{
        AdapterAirContext, AdapterRuntimeContext, BasicAdapterInterface,
        ExecutionBridge, ExecutionBus, ExecutionState, Result,
        VmAdapterAir, VmAdapterChip, VmAdapterInterface,
    },
    system::{
        memory::{
            offline_checker::{MemoryBridge, MemoryReadAuxCols, MemoryWriteAuxCols},
            MemoryAddress, MemoryController, OfflineMemory,
        },
        program::ProgramBus,
    },
};
use openvm_circuit_primitives_derive::AlignedBorrow;
use openvm_stark_backend::{
    interaction::InteractionBuilder,
    p3_air::BaseAir,
    p3_field::{Field, PrimeField32},
};

#[derive(Debug)]
pub struct MyAdapterChip<F: Field> {
    pub air: MyAdapterAir,
    _marker: PhantomData<F>,
}

impl<F: PrimeField32> MyAdapterChip<F> {
    pub fn new(
        execution_bus: ExecutionBus,
        program_bus: ProgramBus,
        memory_bridge: MemoryBridge,
    ) -> Self {
        Self {
            air: MyAdapterAir {
                execution_bridge: ExecutionBridge::new(execution_bus, program_bus),
                memory_bridge,
            },
            _marker: PhantomData,
        }
    }
}
```

### Defining Columns Structure
```rust
#[repr(C)]
#[derive(AlignedBorrow)]
pub struct MyAdapterCols<T> {
    pub from_state: ExecutionState<T>,
    pub reads_aux: [MemoryReadAuxCols<T>; NUM_READS],
    pub writes_aux: [MemoryWriteAuxCols<T>; NUM_WRITES],
    // Custom columns for your operation
    pub custom_field: T,
}
```

### Implementing the Air
```rust
#[derive(Clone, Copy, Debug, derive_new::new)]
pub struct MyAdapterAir {
    pub(super) execution_bridge: ExecutionBridge,
    pub(super) memory_bridge: MemoryBridge,
}

impl<F: Field> BaseAir<F> for MyAdapterAir {
    fn width(&self) -> usize {
        MyAdapterCols::<F>::width()
    }
}

impl<AB: InteractionBuilder> VmAdapterAir<AB> for MyAdapterAir {
    type Interface = BasicAdapterInterface<
        AB::Expr,
        MinimalInstruction<AB::Expr>,
        NUM_READS,
        NUM_WRITES,
        READ_SIZE,
        WRITE_SIZE,
    >;

    fn eval(
        &self,
        builder: &mut AB,
        local: &[AB::Var],
        ctx: AdapterAirContext<AB::Expr, Self::Interface>,
    ) {
        let cols: &MyAdapterCols<AB::Var> = local.borrow();
        let flags = &ctx.instruction.flags;
        
        // Core adapter logic here
        self.eval_transitions(builder, cols, flags);
        self.eval_interactions(builder, cols, ctx);
    }
}
```

## Memory Operations

### Reading from Memory
```rust
// Single read with immediate support
fn eval_read_with_immediate(
    &self,
    builder: &mut AB,
    address: &MemoryAddress<AB::Expr, AB::Expr>,
    aux: &MemoryReadOrImmediateAuxCols<AB::Var>,
    is_immediate: AB::Expr,
    immediate_value: AB::Expr,
) -> AB::Expr {
    let read_value = self.memory_bridge.read_or_immediate(
        builder,
        address,
        aux,
        is_immediate.clone(),
        immediate_value,
    );
    read_value
}

// Multiple reads
fn eval_reads(
    &self,
    builder: &mut AB,
    cols: &MyAdapterCols<AB::Var>,
    ctx: &AdapterAirContext<AB::Expr, Self::Interface>,
) {
    for (i, read_aux) in cols.reads_aux.iter().enumerate() {
        let address = MemoryAddress {
            address_space: ctx.reads[i].address_space.clone(),
            pointer: ctx.reads[i].pointer.clone(),
        };
        let value = self.memory_bridge.read(
            builder,
            &address,
            read_aux,
        );
        // Use value in computation
    }
}
```

### Writing to Memory
```rust
fn eval_write(
    &self,
    builder: &mut AB,
    address: &MemoryAddress<AB::Expr, AB::Expr>,
    aux: &MemoryWriteAuxCols<AB::Var>,
    value: Vec<AB::Expr>,
) {
    self.memory_bridge.write(
        builder,
        address,
        aux,
        value,
    );
}
```

## Runtime Implementation

### Basic Runtime Structure
```rust
impl<F: PrimeField32> VmAdapterChip<F> for MyAdapterChip<F> {
    type ReadRecord = NativeReadRecord<NUM_READS>;
    type WriteRecord = NativeWriteRecord<NUM_WRITES>;
    type Air = MyAdapterAir;
    type Interface = BasicAdapterInterface<F, MinimalInstruction<F>, NUM_READS, NUM_WRITES, READ_SIZE, WRITE_SIZE>;

    fn preprocess(
        &mut self,
        memory: &mut OfflineMemory<F>,
        instruction: &Instruction<F>,
    ) -> Result<(
        Self::ReadRecord,
        Self::WriteRecord,
    )> {
        let Instruction { opcode, operands } = instruction;
        
        // Decode operands
        let reads = self.read_from_memory(memory, operands)?;
        
        // Perform operation
        let result = self.execute_operation(&reads)?;
        
        // Write results
        let writes = self.write_to_memory(memory, operands, result)?;
        
        Ok((reads, writes))
    }

    fn postprocess(
        &mut self,
        memory: &mut OfflineMemory<F>,
        instruction: &Instruction<F>,
        from_state: ExecutionState<u32>,
        read_record: &Self::ReadRecord,
        write_record: &Self::WriteRecord,
    ) -> Result<()> {
        self.air
            .memory_bridge
            .load_offline_memory_evaluations(memory)?;
        Ok(())
    }

    fn generate_trace_row(
        &self,
        row_slice: &mut [F],
        instruction: &Instruction<F>,
        from_state: ExecutionState<u32>,
        read_record: Self::ReadRecord,
        write_record: Self::WriteRecord,
    ) {
        let cols: &mut MyAdapterCols<F> = row_slice.borrow_mut();
        
        // Populate execution state
        cols.from_state = from_state.map(F::from_canonical_u32);
        
        // Populate read auxiliary columns
        for (i, record) in read_record.reads.iter().enumerate() {
            cols.reads_aux[i] = memory.generate_read_aux(*record);
        }
        
        // Populate write auxiliary columns
        for (i, record) in write_record.writes.iter().enumerate() {
            cols.writes_aux[i] = memory.generate_write_aux(*record);
        }
    }
}
```

## Common Patterns

### ALU-Style Operations (2 reads, 1 write)
```rust
// For operations like add, mul, sub
fn execute_alu_op<F: Field>(
    &self,
    memory: &mut OfflineMemory<F>,
    op: AluOp,
    operands: &[F],
) -> Result<F> {
    // Read operands
    let a = self.read_operand(memory, operands[0], operands[1])?;
    let b = self.read_operand(memory, operands[2], operands[3])?;
    
    // Execute operation
    let result = match op {
        AluOp::Add => a + b,
        AluOp::Sub => a - b,
        AluOp::Mul => a * b,
        // ...
    };
    
    // Write result
    self.write_result(memory, operands[4], operands[5], result)?;
    
    Ok(result)
}
```

### Branch-Style Operations (2 reads, 0 writes, jump)
```rust
fn execute_branch<F: Field>(
    &self,
    memory: &mut OfflineMemory<F>,
    cmp: CompareOp,
    operands: &[F],
    pc: &mut u32,
) -> Result<()> {
    // Read comparison values
    let a = self.read_value(memory, operands[0], operands[1])?;
    let b = self.read_value(memory, operands[2], operands[3])?;
    
    // Evaluate condition
    let condition = match cmp {
        CompareOp::Eq => a == b,
        CompareOp::Ne => a != b,
        // ...
    };
    
    // Update PC if condition met
    if condition {
        *pc = operands[4].as_canonical_u32();
    } else {
        *pc += DEFAULT_PC_STEP;
    }
    
    Ok(())
}
```

### Vector Operations
```rust
fn execute_vector_op<F: Field, const N: usize>(
    &self,
    memory: &mut OfflineMemory<F>,
    op: VectorOp,
    operands: &[F],
) -> Result<[F; N]> {
    // Read vector operands
    let a = self.read_vector::<N>(memory, operands[0], operands[1])?;
    let b = self.read_vector::<N>(memory, operands[2], operands[3])?;
    
    // Element-wise operation
    let mut result = [F::ZERO; N];
    for i in 0..N {
        result[i] = match op {
            VectorOp::Add => a[i] + b[i],
            VectorOp::Mul => a[i] * b[i],
            // ...
        };
    }
    
    // Write result vector
    self.write_vector(memory, operands[4], operands[5], result)?;
    
    Ok(result)
}
```

## Testing Patterns

### Unit Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use openvm_circuit::test_utils::*;
    
    #[test]
    fn test_my_adapter() {
        let mut vm = VirtualMachine::<F>::new();
        
        // Add adapter
        let adapter = MyAdapterChip::new(
            vm.execution_bus(),
            vm.program_bus(),
            vm.memory_bridge(),
        );
        vm.add_adapter(adapter);
        
        // Create test program
        let program = vec![
            Instruction {
                opcode: MY_OPCODE,
                operands: vec![/* ... */],
            },
        ];
        
        // Run and verify
        vm.run(program)?;
        vm.verify()?;
    }
}
```

### Integration Test Pattern
```rust
#[test]
fn test_with_core_chip() {
    let mut vm = VirtualMachine::<F>::new();
    
    // Add adapter and core chip
    let adapter = MyAdapterChip::new(/* ... */);
    let core = MyCoreChip::new(/* ... */);
    
    vm.add_adapter(adapter);
    vm.add_chip(core);
    
    // Test complete operation flow
    // ...
}
```

## Performance Optimization

### Immediate Value Usage
```rust
// Prefer immediates for constants
instruction.with_immediate(constant_value)
    .instead_of_memory_read();
```

### Batch Operations
```rust
// Process multiple elements together
fn batch_process<const BATCH_SIZE: usize>(
    &self,
    elements: &[F],
) -> [F; BATCH_SIZE] {
    // Vectorized processing
}
```

### Memory Access Optimization
```rust
// Sequential access pattern
for i in 0..N {
    let addr = base_addr + i;
    // Access memory[addr]
}

// Avoid random access when possible
```

## Common Pitfalls

### Forgetting Memory Constraints
```rust
// BAD: Direct memory access
let value = memory[addr];

// GOOD: Through memory bridge
let value = self.memory_bridge.read(builder, &address, aux);
```

### Incorrect State Updates
```rust
// BAD: Forgetting PC update
// execution continues at same instruction

// GOOD: Proper PC advancement
ctx.to_pc = ctx.from_pc + DEFAULT_PC_STEP;
```

### Missing Range Checks
```rust
// BAD: Unchecked value
let result = a * b;

// GOOD: With proper constraints
let result = a * b;
builder.assert_in_field(&result);
```