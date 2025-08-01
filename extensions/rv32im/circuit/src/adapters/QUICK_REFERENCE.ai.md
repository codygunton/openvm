# RV32IM Circuit Adapters Quick Reference

## Adapter Creation

### ALU Adapter (arithmetic/logic operations)
```rust
let alu_adapter = Rv32BaseAluAdapterChip::<F>::new(
    execution_bus,
    program_bus,
    memory_bridge,
    bitwise_lookup_chip,
);
```

### Branch Adapter (conditional jumps)
```rust
let branch_adapter = Rv32BranchAdapterChip::<F>::new(
    execution_bus,
    program_bus,
    memory_bridge,
);
```

### JALR Adapter (jump and link register)
```rust
let jalr_adapter = Rv32JalrAdapterChip::<F>::new(
    execution_bus,
    program_bus,
    memory_bridge,
);
```

### LoadStore Adapter (memory operations)
```rust
let loadstore_adapter = Rv32LoadStoreAdapterChip::<F>::new(
    execution_bus,
    program_bus,
    memory_bridge,
    range_checker_chip,
);
```

### Mul Adapter (multiplication/division)
```rust
let mul_adapter = Rv32MulAdapterChip::<F>::new(
    execution_bus,
    program_bus,
    memory_bridge,
);
```

### RdWrite Adapter (immediate writes)
```rust
let rdwrite_adapter = Rv32RdWriteAdapterChip::<F>::new(
    execution_bus,
    program_bus,
    memory_bridge,
);
```

## Register Operations

### Read Register
```rust
// Read a 32-bit register value
let (record_id, value) = read_rv32_register(
    &mut memory,
    F::ONE,  // Address space 1 for registers
    F::from_canonical_u32(register_number)
);
```

### Write Register
```rust
// Write a 32-bit value to register
let limbs = decompose(value);
let record_id = memory.write(
    F::ONE,  // Address space 1
    F::from_canonical_u32(register_number),
    limbs
);
```

### Compose/Decompose
```rust
// Convert 4 limbs to u32
let value: u32 = compose(limbs);

// Convert u32 to 4 limbs
let limbs: [F; 4] = decompose(value);
```

### Abstract Compose (for AIR)
```rust
// Symbolic composition for constraints
let value_expr = abstract_compose::<AB::Expr, _>(limbs_expr);
```

## Immediate Handling

### I-Type (12-bit sign-extended)
```rust
// Runtime
let imm_signed = ((imm as i32) << 20) >> 20;

// AIR
let sign_bit = (imm >> 11) & 1;
let extended = imm - (sign_bit * (1 << 12));
```

### S-Type (store format)
```rust
let imm = (imm11_5 << 5) | imm4_0;
let imm_signed = ((imm as i32) << 20) >> 20;
```

### B-Type (branch format)
```rust
let imm = (imm12 << 12) | (imm11 << 11) | 
          (imm10_5 << 5) | (imm4_1 << 1);
let imm_signed = ((imm as i32) << 19) >> 19;
```

### J-Type (jump format)
```rust
let imm = (imm20 << 20) | (imm19_12 << 12) | 
          (imm11 << 11) | (imm10_1 << 1);
let imm_signed = ((imm as i32) << 11) >> 11;
```

## Common Instruction Patterns

### ALU Operations
```rust
// ADD rd, rs1, rs2
Instruction::new(ADD, [rd, 0, rs1, 0, rs2, 1, 0, 0])

// ADDI rd, rs1, imm
Instruction::new(ADDI, [rd, 0, rs1, 0, imm_lo, 0, imm_hi, 0])

// AND rd, rs1, rs2
Instruction::new(AND, [rd, 0, rs1, 0, rs2, 1, 0, 0])

// ORI rd, rs1, imm
Instruction::new(ORI, [rd, 0, rs1, 0, imm_lo, 0, imm_hi, 0])
```

### Branch Operations
```rust
// BEQ rs1, rs2, offset
Instruction::new(BEQ, [rs1, 0, rs2, 0, off1, off2, off3, off4])

// BLT rs1, rs2, offset
Instruction::new(BLT, [rs1, 0, rs2, 0, off1, off2, off3, off4])
```

### Jump Operations
```rust
// JAL rd, offset
Instruction::new(JAL, [rd, 0, off1, off2, off3, off4, 0, 0])

// JALR rd, rs1, offset
Instruction::new(JALR, [rd, 0, rs1, 0, off_lo, 0, off_hi, 0])
```

### Load/Store Operations
```rust
// LW rd, offset(rs1)
Instruction::new(LW, [rd, 0, rs1, 0, off_lo, 0, off_hi, 0])

// SW rs2, offset(rs1)
Instruction::new(SW, [rs1, 0, rs2, 0, off1, off2, off3, off4])
```

### Immediate Operations
```rust
// LUI rd, imm
Instruction::new(LUI, [rd, 0, 0, 0, 0, 0, imm, 0])

// AUIPC rd, imm
Instruction::new(AUIPC, [rd, 0, 0, 0, 0, 0, imm, 0])
```

## Memory Access Patterns

### Load Word
```rust
let addr = base + offset;
let aligned_addr = addr & !3;
let shift = (addr & 3) * 8;
let data = read_memory_word(aligned_addr);
let value = data >> shift;  // Assumes aligned
```

### Store Word
```rust
let addr = base + offset;
let aligned_addr = addr & !3;
let data = value;  // Assumes aligned
write_memory_word(aligned_addr, data);
```

### Load Halfword (sign-extended)
```rust
let shift = (addr & 3) * 8;
let data = read_memory_word(addr & !3);
let half = (data >> shift) & 0xFFFF;
let value = ((half as i16) as i32) as u32;
```

### Store Byte
```rust
let shift = (addr & 3) * 8;
let mask = !(0xFF << shift);
let current = read_memory_word(addr & !3);
let new_val = (current & mask) | ((value & 0xFF) << shift);
write_memory_word(addr & !3, new_val);
```

## AIR Constraint Patterns

### State Transition
```rust
// PC update for sequential execution
builder.assert_eq(
    ctx.to_state.pc,
    cols.from_state.pc + AB::Expr::from_canonical_u32(DEFAULT_PC_STEP)
);

// Timestamp increment
builder.assert_eq(
    ctx.to_state.timestamp,
    cols.from_state.timestamp + AB::Expr::ONE
);
```

### Memory Read
```rust
self.memory_bridge.read(
    MemoryAddress::new(AB::Expr::ONE, cols.rs1_ptr),
    cols.rs1_data,
    timestamp,
    &cols.reads_aux[0],
);
```

### Memory Write
```rust
self.memory_bridge.write(
    MemoryAddress::new(AB::Expr::ONE, cols.rd_ptr),
    cols.result,
    timestamp,
    &cols.writes_aux,
);
```

### Conditional Logic
```rust
// If-then-else pattern
let result = builder.eval(
    condition * true_value + (AB::Expr::ONE - condition) * false_value
);

// Conditional constraint
builder.when(condition).assert_eq(a, b);
```

### Bitwise Lookup
```rust
// For AND, OR, XOR operations
self.bitwise_lookup_bus.send(
    builder,
    BitwiseOperationLookupBus::new(
        op_type,
        rs1_limb,
        rs2_limb,
        result_limb,
    ).with_multiplicity(timestamp),
);
```

## Common Constants

```rust
// Register configuration
pub const RV32_REGISTER_NUM_LIMBS: usize = 4;
pub const RV32_CELL_BITS: usize = 8;

// Address spaces
pub const RV32_REGISTER_AS: u32 = 1;
pub const RV32_IMM_AS: u32 = 0;

// Immediate bit widths
pub const RV_IS_TYPE_IMM_BITS: usize = 12;
pub const RV_B_TYPE_IMM_BITS: usize = 13;
pub const RV_J_TYPE_IMM_BITS: usize = 21;

// PC increment
pub const DEFAULT_PC_STEP: u32 = 4;

// For 256-bit operations
pub const INT256_NUM_LIMBS: usize = 32;
```

## Error Handling

### Division by Zero
```rust
if divisor == 0 {
    // RISC-V spec: return -1 for quotient, dividend for remainder
    match op {
        DIV | DIVU => u32::MAX,
        REM | REMU => dividend,
    }
}
```

### x0 Register Protection
```rust
// Check before write
if rd != 0 {
    write_register(memory, rd_ptr, value);
}

// Or in AIR
let needs_write = builder.eval(not(cols.rd_ptr.is_zero()));
```

### Overflow Cases
```rust
// Signed division overflow
if dividend == i32::MIN && divisor == -1 {
    match op {
        DIV => dividend as u32,  // Return dividend
        REM => 0,                // Return 0
    }
}
```

## Debugging Snippets

### Trace Instruction
```rust
println!("PC: {:#x} Opcode: {:?} rd={} rs1={} rs2={}", 
    pc, opcode, rd, rs1, rs2);
```

### Dump Register State
```rust
for i in 0..32 {
    let value = unsafe_read_rv32_register(&memory, F::from_canonical_u32(i));
    println!("x{}: {:#x}", i, value);
}
```

### Verify Memory Operation
```rust
println!("Load from {:#x}: shift={} aligned_addr={:#x} value={:#x}",
    addr, shift, aligned_addr, result);
```

## Optimization Patterns

### Batch Register Reads
```rust
// Read both source registers in one go
let records = ctx.memory.batch_read::<RV32_REGISTER_NUM_LIMBS>(&[
    (F::ONE, rs1_ptr),
    (F::ONE, rs2_ptr),
]);
```

### Precompute Constants
```rust
// In adapter initialization
const SIGN_EXTEND_12: u32 = 0xFFFFF000;
const BYTE_MASK: [u32; 4] = [0xFF, 0xFF00, 0xFF0000, 0xFF000000];
```

### Minimize Symbolic Operations
```rust
// Instead of multiple ops
let result = a + b + c + d;

// Group for efficiency
let result = (a + b) + (c + d);
```