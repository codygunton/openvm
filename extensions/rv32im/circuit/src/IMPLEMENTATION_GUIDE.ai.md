# RV32IM Circuit Implementation Guide

## Core Concepts

### 1. Two-Layer Architecture

Every RV32IM instruction implementation follows a consistent two-layer pattern:

```rust
// Adapter Layer - Handles VM interactions
pub struct Rv32BaseAluAdapterChip<F> {
    execution_bus: ExecutionBus,
    program_bus: ProgramBus,
    memory_bridge: MemoryBridge<F>,
    bitwise_chip: SharedBitwiseOperationLookupChip<8>,
}

// Core Layer - Implements instruction logic
pub struct BaseAluCoreChip<F> {
    bitwise_chip: SharedBitwiseOperationLookupChip<8>,
    offset: usize,
}

// Combined Chip - Connects both layers
pub type Rv32BaseAluChip<F> = CompositeChip<F, Rv32BaseAluAdapterChip<F>, BaseAluCoreChip<F>>;
```

### 2. Register Representation

RV32 registers are 32-bit values decomposed into limbs:
- Standard decomposition: 4 limbs of 8 bits each
- Constants: `RV32_REGISTER_NUM_LIMBS = 4`, `RV32_CELL_BITS = 8`

### 3. Memory Access Patterns

**Register Access:**
```rust
// Read from register
memory_bridge.read(1, rs1, RV32_REGISTER_NUM_LIMBS)

// Write to register (skip if rd == 0)
if rd != 0 {
    memory_bridge.write(1, rd, result_limbs)
}
```

**Memory Load/Store:**
```rust
// Calculate effective address
let addr = rs1_val + imm

// Load from memory
memory_bridge.read(mem_as, addr, width)

// Store to memory
memory_bridge.write(mem_as, addr, data)
```

### 4. Instruction Encoding

Instructions follow RISC-V encoding with OpenVM adaptations:
- Opcode determines the operation
- Register addresses (rd, rs1, rs2)
- Immediate values for I-type instructions
- Memory address space indicators

### 5. Constraint Patterns

**Range Checking:**
```rust
// Ensure limbs are within valid range
for &limb in &result_limbs {
    builder.assert_u8(limb);
}
```

**Conditional Logic:**
```rust
// Branch taken logic
let branch_taken = builder.eval(condition);
let next_pc = builder.select(branch_taken, pc + imm, pc + 4);
```

## Implementation Patterns

### Creating a New Instruction

1. **Define the Adapter** (in `adapters/`):
   - Handle memory reads for operands
   - Set up instruction fetching
   - Write results back

2. **Define the Core** (in instruction directory):
   - Implement actual operation logic
   - Add necessary constraints
   - Handle edge cases

3. **Create Tests**:
   - Unit tests for core logic
   - Integration tests with full chip

4. **Register in Extension**:
   - Add to appropriate executor enum
   - Register opcodes in extension builder

### Common Pitfalls

1. **Forgetting x0 is hardwired to zero**
   - Always check `rd != 0` before writing

2. **Missing range checks**
   - All limbs must be constrained to valid ranges

3. **Incorrect sign extension**
   - Be careful with signed vs unsigned operations

4. **Program counter updates**
   - Branches update PC conditionally
   - Regular instructions increment by 4

## Extension Points

### Adding New Instructions

1. Create new module directory
2. Implement adapter and core chips
3. Add to executor enum in `extension.rs`
4. Register opcodes in VmExtension implementation

### Modifying Existing Instructions

1. Locate instruction in its module
2. Modify core logic in `core.rs`
3. Update tests accordingly
4. Ensure backward compatibility

### Integration with Other Extensions

- Share periphery chips (bitwise, range checking)
- Use consistent bus indices
- Follow OpenVM architecture patterns