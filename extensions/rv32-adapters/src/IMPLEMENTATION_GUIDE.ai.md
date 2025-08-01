# RV32 Adapters - Implementation Guide

## Core Concepts

### Adapter Architecture
RV32 adapters extend the base RISC-V instruction set with specialized memory operations. Each adapter consists of:

1. **AIR (Algebraic Intermediate Representation)**
   - Defines polynomial constraints for the operation
   - Handles interaction with memory and execution buses
   - Validates address ranges and data integrity

2. **Chip Implementation**
   - Manages runtime execution
   - Generates execution traces
   - Handles pre/post-processing of instructions

3. **Column Layout**
   - Structured data representation for constraint system
   - Auxiliary columns for memory operations
   - State tracking for execution flow

## Implementation Patterns

### Creating a Custom Adapter

```rust
pub struct MyAdapterChip<F: Field, const PARAM: usize> {
    pub air: MyAdapterAir<PARAM>,
    pub auxiliary_chips: AuxiliaryChips,
    _marker: PhantomData<F>,
}

impl<F: PrimeField32, const PARAM: usize> MyAdapterChip<F, PARAM> {
    pub fn new(
        execution_bus: ExecutionBus,
        program_bus: ProgramBus,
        memory_bridge: MemoryBridge,
        address_bits: usize,
    ) -> Self {
        // Initialize adapter with bus connections
    }
}
```

### Memory Access Patterns

#### Single Read Pattern
```rust
// Read from register to get pointer
let (record, val) = read_rv32_register(memory, register_as, register_addr);

// Read from heap using pointer
let heap_data = memory.read::<SIZE>(heap_as, F::from_canonical_u32(pointer));
```

#### Vectorized Read Pattern
```rust
// Read multiple consecutive blocks
for i in 0..BLOCKS_PER_READ {
    let offset = i * BLOCK_SIZE;
    let data = memory.read::<BLOCK_SIZE>(
        heap_as, 
        base_addr + F::from_canonical_u32(offset)
    );
}
```

### AIR Constraint Implementation

#### Basic Structure
```rust
impl<AB: InteractionBuilder> VmAdapterAir<AB> for MyAdapterAir {
    fn eval(
        &self,
        builder: &mut AB,
        local: &[AB::Var],
        ctx: AdapterAirContext<AB::Expr, Self::Interface>,
    ) {
        // 1. Extract column data
        let cols: &MyAdapterCols<_> = local.borrow();
        
        // 2. Setup timestamp tracking
        let timestamp = cols.from_state.timestamp;
        let mut timestamp_delta = 0;
        
        // 3. Perform memory operations with constraints
        // 4. Update execution state
        // 5. Send bus interactions
    }
}
```

#### Memory Bridge Integration
```rust
// Read operation with auxiliary columns
self.memory_bridge
    .read(
        MemoryAddress::new(address_space, pointer),
        data_array,
        timestamp,
        &aux_cols
    )
    .eval(builder, is_valid);

// Write operation
self.memory_bridge
    .write(
        MemoryAddress::new(address_space, pointer),
        data,
        timestamp,
        &write_aux
    )
    .eval(builder, is_valid);
```

### Address Range Validation

```rust
// Validate address fits within configured bits
let address_in_range = address < (1 << self.address_bits);

// Use bitwise lookup for range checks
let limb_shift = 1 << (CELL_BITS * NUM_LIMBS - address_bits);
self.bitwise_bus
    .send_range(value * limb_shift, 0)
    .eval(builder, is_valid);
```

## Advanced Patterns

### Conditional Execution
```rust
// Branch adapter pattern
let condition = evaluate_branch_condition(operands);
let result = if condition {
    execute_true_branch()
} else {
    execute_false_branch()
};
```

### Bulk Operations
```rust
// Efficient bulk memory access
pub fn process_bulk_data<const BLOCKS: usize, const BLOCK_SIZE: usize>(
    memory: &mut MemoryController<F>,
    base_addr: u32,
) -> [[F; BLOCK_SIZE]; BLOCKS] {
    from_fn(|i| {
        let offset = (i * BLOCK_SIZE) as u32;
        memory.read::<BLOCK_SIZE>(heap_as, F::from_canonical_u32(base_addr + offset)).1
    })
}
```

### Custom Column Layouts
```rust
#[repr(C)]
#[derive(AlignedBorrow)]
pub struct CustomAdapterCols<T, const PARAM: usize> {
    // Execution state
    pub from_state: ExecutionState<T>,
    
    // Input operands
    pub operand_ptrs: [T; 2],
    pub operand_vals: [[T; 4]; 2],
    pub operand_aux: [MemoryReadAuxCols<T>; 2],
    
    // Operation-specific data
    pub intermediate: [T; PARAM],
    
    // Output
    pub result_ptr: T,
    pub result_aux: MemoryWriteAuxCols<T, 4>,
}
```

## Performance Considerations

### Optimization Strategies
1. **Minimize Bus Interactions**: Batch operations when possible
2. **Efficient Timestamp Management**: Increment only when necessary
3. **Parallel Reads**: Use multiple read pointers for independent data
4. **Block Size Selection**: Choose sizes that align with your data structures

### Memory Layout Optimization
```rust
// Align data structures to block boundaries
const OPTIMAL_BLOCK_SIZE: usize = 32; // Example: 256-bit blocks
const BLOCKS_PER_OPERATION: usize = 4; // Process 1024 bits at once
```

## Testing Patterns

### Unit Test Structure
```rust
#[test]
fn test_adapter_operation() {
    let mut tester = VmChipTestBuilder::new();
    
    // Setup test data
    let test_data = setup_test_memory(&mut tester);
    
    // Create instruction
    let instruction = Instruction::from_isize(
        OPCODE,
        rd as isize,
        rs1 as isize,
        rs2 as isize,
        RV32_REGISTER_AS as isize,
        RV32_MEMORY_AS as isize,
    );
    
    // Execute and verify
    tester.execute_instruction(instruction);
    verify_results(&tester, expected);
}
```

## Common Pitfalls and Solutions

### Address Space Confusion
- Always verify address space constants match expected values
- Use type-safe wrappers for address spaces

### Timestamp Synchronization
- Track timestamp deltas carefully
- Ensure all operations increment consistently

### Range Check Failures
- Validate addresses before use
- Implement proper bounds checking in constraints

### Memory Alignment
- Respect natural alignment for multi-limb values
- Consider endianness in data layout