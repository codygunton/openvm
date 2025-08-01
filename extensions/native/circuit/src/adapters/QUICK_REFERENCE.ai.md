# Native Circuit Adapters Quick Reference

## Adapter Creation

### Basic Adapter Chip
```rust
let adapter = MyAdapterChip::<F>::new(
    execution_bus,
    program_bus, 
    memory_bridge,
);
```

### ALU Adapter (2 reads, 1 write, immediate support)
```rust
let alu_adapter = AluNativeAdapterChip::<F>::new(
    execution_bus,
    program_bus,
    memory_bridge,
);
```

### Branch Adapter (2 reads, 0 writes, jump support)
```rust
let branch_adapter = BranchNativeAdapterChip::<F>::new(
    execution_bus,
    program_bus,
    memory_bridge,
);
```

### Convert Adapter (variable sizes)
```rust
let convert_adapter = ConvertAdapterChip::<F, READ_SIZE, WRITE_SIZE>::new(
    execution_bus,
    program_bus,
    memory_bridge,
);
```

### LoadStore Adapter
```rust
let loadstore_adapter = NativeLoadStoreAdapterChip::<F, ELEMENT_SIZE>::new(
    execution_bus,
    program_bus,
    memory_bridge,
);
```

### Vectorized Adapter
```rust
let vector_adapter = NativeVectorizedAdapterChip::<F, VECTOR_SIZE>::new(
    execution_bus,
    program_bus,
    memory_bridge,
);
```

## Column Structures

### Basic Columns Pattern
```rust
#[repr(C)]
#[derive(AlignedBorrow)]
pub struct MyAdapterCols<T> {
    pub from_state: ExecutionState<T>,
    pub reads_aux: [MemoryReadAuxCols<T>; NUM_READS],
    pub writes_aux: [MemoryWriteAuxCols<T>; NUM_WRITES],
}
```

### With Immediate Support
```rust
#[repr(C)]
#[derive(AlignedBorrow)]
pub struct AdapterReadCols<T> {
    pub address: MemoryAddress<T, T>,
    pub read_aux: MemoryReadOrImmediateAuxCols<T>,
}
```

## Memory Operations

### Reading with Immediate
```rust
let value = self.memory_bridge.read_or_immediate(
    builder,
    &address,
    &aux_cols,
    is_immediate,
    immediate_value,
);
```

### Simple Read
```rust
let value = self.memory_bridge.read(
    builder,
    &address,
    &read_aux,
);
```

### Writing
```rust
self.memory_bridge.write(
    builder,
    &address,
    &write_aux,
    vec![value],
);
```

## Address Construction

### From Instruction Context
```rust
let address = MemoryAddress {
    address_space: ctx.reads[0].address_space.clone(),
    pointer: ctx.reads[0].pointer.clone(),
};
```

### From Operands
```rust
let address = MemoryAddress {
    address_space: F::from_canonical_u32(operands[0].as_canonical_u32()),
    pointer: F::from_canonical_u32(operands[1].as_canonical_u32()),
};
```

## Interface Definitions

### Basic Interface Types
```rust
// MinimalInstruction - basic operations
type Interface = BasicAdapterInterface<
    F, MinimalInstruction<F>, 2, 1, 1, 1
>;

// ImmInstruction - with immediate support  
type Interface = BasicAdapterInterface<
    F, ImmInstruction<F>, 2, 0, 1, 1
>;
```

### Interface Parameters
```
BasicAdapterInterface<Expr, Instruction, NUM_READS, NUM_WRITES, READ_SIZE, WRITE_SIZE>
```

## Runtime Patterns

### Preprocess (Memory Operations)
```rust
fn preprocess(
    &mut self,
    memory: &mut OfflineMemory<F>,
    instruction: &Instruction<F>,
) -> Result<(ReadRecord, WriteRecord)> {
    // Decode operands
    let (rs1, rs2, rd) = decode_operands(&instruction.operands);
    
    // Read from memory
    let a = memory.read(rs1.address_space, rs1.pointer)?;
    let b = memory.read(rs2.address_space, rs2.pointer)?;
    
    // Perform operation
    let result = a + b; // Example
    
    // Write to memory
    memory.write(rd.address_space, rd.pointer, result)?;
    
    Ok((read_record, write_record))
}
```

### Trace Generation
```rust
fn generate_trace_row(
    &self,
    row_slice: &mut [F],
    instruction: &Instruction<F>,
    from_state: ExecutionState<u32>,
    read_record: Self::ReadRecord,
    write_record: Self::WriteRecord,
) {
    let cols: &mut MyAdapterCols<F> = row_slice.borrow_mut();
    cols.from_state = from_state.map(F::from_canonical_u32);
    // Populate auxiliary columns...
}
```

## Common Instruction Patterns

### ALU Operation
```rust
// Operands: [rs1_space, rs1_ptr, rs2_space, rs2_ptr, rd_space, rd_ptr]
let instruction = Instruction {
    opcode: ALU_ADD,
    operands: vec![0, addr1, 0, addr2, 0, dest],
};
```

### Branch Operation
```rust
// Operands: [rs1_space, rs1_ptr, rs2_space, rs2_ptr, jump_target]
let instruction = Instruction {
    opcode: BRANCH_EQ,
    operands: vec![0, addr1, 0, addr2, target_pc],
};
```

### Load/Store
```rust
// Load: [src_space, src_ptr, dst_space, dst_ptr]
let load_inst = Instruction {
    opcode: LOAD,
    operands: vec![0, src, 0, dst],
};

// Store: [src_space, src_ptr, dst_space, dst_ptr]
let store_inst = Instruction {
    opcode: STORE,
    operands: vec![0, src, 0, dst],
};
```

## Testing Snippets

### Basic Test Setup
```rust
#[test]
fn test_adapter() {
    let execution_bus = ExecutionBus::new(0);
    let program_bus = ProgramBus::new(1);
    let memory_bridge = MemoryBridge::new(2, 3);
    
    let adapter = MyAdapterChip::new(
        execution_bus,
        program_bus,
        memory_bridge,
    );
    
    // Test logic...
}
```

### Memory Setup
```rust
let mut memory = OfflineMemory::<F>::new();
memory.write(0, 100, F::from(42))?;
memory.write(0, 104, F::from(13))?;
```

## Constraint Patterns

### Basic Operation Constraint
```rust
// In eval function
let result = left + right;
builder.assert_eq(cols.result, result);
```

### Conditional Constraint
```rust
builder.when(is_add).assert_eq(
    cols.result,
    cols.operand_a + cols.operand_b,
);
```

### Range Check
```rust
self.range_checker.range_check(
    builder,
    value,
    MAX_BITS,
);
```

## State Updates

### Normal PC Advancement
```rust
ctx.to_pc = ctx.from_pc + DEFAULT_PC_STEP;
```

### Conditional Jump
```rust
let should_jump = builder.eq(a, b);
ctx.to_pc = builder.select(
    should_jump,
    jump_target,
    ctx.from_pc + DEFAULT_PC_STEP,
);
```

## Error Handling

### Runtime Errors
```rust
use openvm_circuit::arch::Result;
use openvm_circuit::arch::VmAdapterError;

// Check bounds
if addr >= memory_size {
    return Err(VmAdapterError::InvalidAddress(addr));
}

// Validate operands
if instruction.operands.len() != 6 {
    return Err(VmAdapterError::InvalidInstruction);
}
```

## Constants

### Common Imports
```rust
use openvm_instructions::program::DEFAULT_PC_STEP; // 4
use openvm_native_compiler::conversion::AS; // address space constant
```

### Typical Sizes
```rust
const NUM_READS: usize = 2;
const NUM_WRITES: usize = 1;
const READ_SIZE: usize = 1;
const WRITE_SIZE: usize = 1;
```