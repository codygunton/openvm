# RV32IM Circuit Quick Reference

## Common Tasks

### Adding a New RV32 Instruction

```rust
// 1. Create adapter in adapters/new_instruction.rs
pub struct Rv32NewInstructionAdapterChip<F> {
    // Standard buses
    execution_bus: ExecutionBus,
    program_bus: ProgramBus,
    memory_bridge: MemoryBridge<F>,
}

// 2. Create core in new_instruction/core.rs
pub struct NewInstructionCoreChip<F> {
    // Instruction-specific fields
    offset: usize,
}

// 3. Implement eval function
impl<AB: InteractionBuilder> VmAdapterChip<AB> for Rv32NewInstructionAdapterChip<AB::F> {
    fn eval(
        &self,
        builder: &mut AB,
        // Adapter inputs...
    ) {
        // Read operands
        let rs1_data = self.memory_bridge.read(/*...*/);
        // Execute core
        let result = NewInstructionCoreChip::eval(/*...*/);
        // Write result
        self.memory_bridge.write(/*...*/);
    }
}
```

### Working with Registers

```rust
// Read from register
let (rs1_data, _) = self.memory_bridge.read(
    builder,
    1,  // Address space 1 for registers
    rs1.expr(),
    RV32_REGISTER_NUM_LIMBS,
    reads_aux,
);

// Write to register (checking for x0)
self.memory_bridge.write(
    builder,
    1,  // Address space 1
    rd.expr(),
    rd_data,
    timestamp,
    &(rd.expr() * builder.felt(RV32_REGISTER_NUM_LIMBS)),
);
```

### Implementing ALU Operations

```rust
// Example: ADD operation
let result = a_limbs.iter()
    .zip(b_limbs.iter())
    .map(|(a, b)| builder.eval(a + b))
    .collect::<Vec<_>>();

// Don't forget range checks!
for &limb in &result {
    builder.assert_u8(limb);
}
```

### Branch Implementation

```rust
// Calculate branch condition
let condition = builder.eval(rs1_val.eq(rs2_val)); // For BEQ

// Calculate next PC
let branch_target = builder.eval(pc + imm);
let next_pc = builder.select(condition, branch_target, pc + 4);

// Update program counter
self.program_bus.set_next_pc(builder, next_pc);
```

### Memory Operations

```rust
// Load operation
let addr = builder.eval(base + offset);
let data = self.memory_bridge.read(
    builder,
    mem_as,  // Memory address space (2 for heap)
    addr,
    width,   // 1, 2, or 4 bytes
    aux,
);

// Store operation
self.memory_bridge.write(
    builder,
    mem_as,
    addr,
    data,
    timestamp,
    aux,
);
```

### Testing an Instruction

```rust
#[test]
fn test_add_instruction() {
    let mut tester = VmChipTestBuilder::new();
    let mut rng = StdRng::seed_from_u64(0);
    
    // Set up test values
    let (instruction, rd) = rv32_rand_write_register_or_imm(
        &mut tester,
        [10, 0, 0, 0],  // rs1 value
        [20, 0, 0, 0],  // rs2 value
        None,           // Not immediate
        BaseAluOpcode::ADD.global_opcode().as_usize(),
        &mut rng,
    );
    
    // Run test
    tester.execute(&instruction);
    
    // Check result
    let result = tester.read::<4>(1, rd);
    assert_eq!(result[0], 30); // 10 + 20
}
```

### Common Constants

```rust
// Register decomposition
const RV32_REGISTER_NUM_LIMBS: usize = 4;
const RV32_CELL_BITS: usize = 8;

// Address spaces
const REGISTER_AS: usize = 1;
const HEAP_AS: usize = 2;
const GLOBAL_AS: usize = 3;
const CONSTANT_AS: usize = 4;

// Special registers
const X0: usize = 0;  // Always zero

// PC increment
const PC_STEP: usize = 4;
```

### Registering in Extension

```rust
// In extension.rs, add to VmExtension::build()
let new_chip = Rv32NewInstructionChip::new(
    Rv32NewInstructionAdapterChip::new(
        execution_bus,
        program_bus,
        memory_bridge,
    ),
    NewInstructionCoreChip::new(offset),
    offline_memory.clone(),
);

inventory.add_executor(
    new_chip,
    NewInstructionOpcode::iter().map(|x| x.global_opcode()),
)?;
```

## Debugging Tips

1. **Check limb decomposition**: Ensure all limbs are properly constrained
2. **Verify x0 handling**: Register 0 should never be written
3. **Test edge cases**: Maximum values, zero operands, overflow
4. **Use test utilities**: `rv32_rand_write_register_or_imm` for random testing
5. **Check opcode registration**: Ensure opcodes are properly mapped