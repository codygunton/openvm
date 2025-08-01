# RV32IM Circuit Adapters Implementation Guide

## Overview
This guide provides detailed implementation patterns and code examples for working with RV32IM circuit adapters in OpenVM. Each section includes practical examples and best practices.

## Creating a New RISC-V Adapter

### Step 1: Define the Adapter Structure
```rust
use std::marker::PhantomData;
use openvm_circuit::arch::{VmAdapterChip, VmAdapterAir};
use openvm_stark_backend::p3_field::Field;

pub struct MyRiscVAdapterChip<F: Field> {
    pub air: MyRiscVAdapterAir,
    _marker: PhantomData<F>,
}

pub struct MyRiscVAdapterAir {
    pub execution_bridge: ExecutionBridge,
    pub memory_bridge: MemoryBridge,
    // Add any additional buses needed
}
```

### Step 2: Define Column Layout
```rust
#[repr(C)]
#[derive(AlignedBorrow)]
pub struct MyRiscVAdapterCols<T> {
    pub from_state: ExecutionState<T>,
    
    // Register pointers
    pub rd_ptr: T,
    pub rs1_ptr: T,
    pub rs2_ptr: T,
    
    // Memory auxiliary columns
    pub reads_aux: [MemoryReadAuxCols<T>; 2],
    pub writes_aux: MemoryWriteAuxCols<T, RV32_REGISTER_NUM_LIMBS>,
    
    // Operation-specific columns
    pub immediate: T,
    pub result: [T; RV32_REGISTER_NUM_LIMBS],
}
```

### Step 3: Implement the Adapter Interface
```rust
impl<AB: InteractionBuilder> VmAdapterAir<AB> for MyRiscVAdapterAir {
    type Interface = BasicAdapterInterface<
        AB::Expr,
        ImmInstruction<AB::Expr>,
        2,  // number of reads
        1,  // number of writes  
        RV32_REGISTER_NUM_LIMBS,
        0   // number of memory reads/writes
    >;

    fn eval(
        &self,
        builder: &mut AB,
        local: &[AB::Var],
        ctx: AdapterAirContext<AB::Expr, Self::Interface>,
    ) {
        let cols: &MyRiscVAdapterCols<_> = local.borrow();
        let timestamp = cols.from_state.timestamp;
        
        // Instruction decoding
        let instruction = &ctx.instruction;
        
        // Implement constraints here
    }
}
```

## Register Operations

### Reading Registers
```rust
// Runtime implementation
pub fn read_register<F: PrimeField32>(
    memory: &mut MemoryController<F>,
    register_ptr: F,
) -> (RecordId, u32) {
    let (record_id, limbs) = memory.read::<RV32_REGISTER_NUM_LIMBS>(
        F::ONE,  // Register address space
        register_ptr,
    );
    let value = compose(limbs);
    (record_id, value)
}

// AIR constraints for register read
self.memory_bridge.read(
    MemoryAddress::new(F::ONE, cols.rs1_ptr),
    data,
    timestamp,
    &cols.reads_aux[0],
);
```

### Writing Registers
```rust
// Runtime implementation  
pub fn write_register<F: PrimeField32>(
    memory: &mut MemoryController<F>,
    register_ptr: F,
    value: u32,
) -> RecordId {
    let limbs = decompose(value);
    let (record_id, _) = memory.write(
        F::ONE,  // Register address space
        register_ptr,
        limbs,
    );
    record_id
}

// AIR constraints for register write
self.memory_bridge.write(
    MemoryAddress::new(F::ONE, cols.rd_ptr),
    cols.result,
    timestamp,
    &cols.writes_aux,
);
```

### Handling x0 Register
```rust
// Check if writing to x0 and skip
let needs_write = builder.eval(not(cols.rd_ptr));
builder.when(needs_write).assert_one(cols.writes_aux.enable);

// Or handle in runtime
if rd != 0 {
    write_register(memory, F::from_canonical_u32(rd), result);
}
```

## Immediate Value Handling

### I-Type Immediate (12-bit sign-extended)
```rust
pub fn decode_i_type_immediate(imm12: u32) -> i32 {
    // Sign extend from 12 bits
    ((imm12 as i32) << 20) >> 20
}

// In adapter
let imm = instruction.immediate;
let sign_extended = builder.eval(
    imm - (AB::Expr::from_canonical_u32(1 << 12) * imm[11])
);
```

### S-Type Immediate (store)
```rust
pub fn decode_s_type_immediate(inst: &Instruction<u32>) -> i32 {
    let imm11_5 = inst.operands[6] as i32;
    let imm4_0 = inst.operands[7] as i32;
    let imm12 = (imm11_5 << 5) | imm4_0;
    ((imm12 << 20) >> 20)  // Sign extend
}
```

### B-Type Immediate (branch)
```rust
pub fn decode_b_type_immediate(inst: &Instruction<u32>) -> i32 {
    let imm12 = inst.operands[6] as i32;
    let imm10_5 = inst.operands[7] as i32;
    let imm4_1 = inst.operands[8] as i32;
    let imm11 = inst.operands[9] as i32;
    
    let imm = (imm12 << 12) | (imm11 << 11) | 
              (imm10_5 << 5) | (imm4_1 << 1);
    ((imm << 19) >> 19)  // Sign extend from 13 bits
}
```

## ALU Operations Implementation

### Basic ALU Pattern
```rust
impl<F: PrimeField32> VmAdapterChip<F> for Rv32BaseAluAdapterChip<F> {
    fn execute_instruction(
        &self,
        instruction: &Instruction<F>,
        ctx: &mut AdapterRuntimeContext<F>,
    ) -> Result<()> {
        let Instruction { opcode, operands, .. } = instruction;
        
        let rd = operands[0].as_canonical_u32() as usize;
        let rs1 = operands[2].as_canonical_u32() as usize;
        let rs2_val = operands[4];
        let rs2_as = operands[5];
        
        // Read rs1
        let (rs1_record, rs1_val) = read_rv32_register(
            &mut ctx.memory,
            F::ONE,
            F::from_canonical_u32(rs1 as u32),
        );
        
        // Read rs2 or use immediate
        let rs2_val = if rs2_as == F::ONE {
            let rs2 = rs2_val.as_canonical_u32() as usize;
            let (_, val) = read_rv32_register(
                &mut ctx.memory,
                F::ONE, 
                F::from_canonical_u32(rs2 as u32),
            );
            val
        } else {
            // Sign extend immediate
            let imm = rs2_val.as_canonical_u32();
            ((imm as i32) << 20) >> 20) as u32
        };
        
        // Perform operation
        let result = match opcode.local_opcode_idx() {
            ADD => rs1_val.wrapping_add(rs2_val),
            SUB => rs1_val.wrapping_sub(rs2_val),
            XOR => rs1_val ^ rs2_val,
            OR => rs1_val | rs2_val,
            AND => rs1_val & rs2_val,
            _ => panic!("Unsupported ALU operation"),
        };
        
        // Write result
        if rd != 0 {
            let rd_record = write_register(
                &mut ctx.memory,
                F::from_canonical_u32(rd as u32),
                result,
            );
        }
        
        // Update PC
        ctx.set_pc(ctx.get_pc() + DEFAULT_PC_STEP);
        
        Ok(())
    }
}
```

### Bitwise Operations with Lookup
```rust
// For operations like AND, OR, XOR
let lookup_val = BitwiseOperationLookupBus::new(
    op_type,
    rs1_limb,
    rs2_limb,
    result_limb,
);

self.bitwise_lookup_bus.send(
    builder,
    lookup_val.with_multiplicity(timestamp),
);
```

## Branch Operations Implementation

### Branch Comparison Pattern
```rust
let comparison_result = match opcode {
    BEQ => rs1_val == rs2_val,
    BNE => rs1_val != rs2_val,
    BLT => (rs1_val as i32) < (rs2_val as i32),
    BGE => (rs1_val as i32) >= (rs2_val as i32),
    BLTU => rs1_val < rs2_val,
    BGEU => rs1_val >= rs2_val,
    _ => false,
};

if comparison_result {
    let offset = decode_b_type_immediate(instruction);
    ctx.set_pc((ctx.get_pc() as i32 + offset) as u32);
} else {
    ctx.set_pc(ctx.get_pc() + DEFAULT_PC_STEP);
}
```

### Branch AIR Constraints
```rust
// In eval function
let pc_offset = if_else(
    comparison_passed,
    instruction.immediate,
    AB::Expr::from_canonical_u32(DEFAULT_PC_STEP),
);

let next_pc = cols.from_state.pc + pc_offset;
builder.assert_eq(ctx.to_state.pc, next_pc);
```

## LoadStore Operations Implementation

### Load Implementation Pattern
```rust
// Calculate effective address
let base_addr = read_register(memory, rs1_ptr);
let offset = sign_extend_i_type(immediate);
let addr = base_addr.wrapping_add(offset as u32);

// Determine alignment and shift
let shift = addr & 0x3;
let aligned_addr = addr & !0x3;

// Read 4 bytes from aligned address
let data = read_memory_word(memory, aligned_addr);

// Extract bytes based on operation
let value = match opcode {
    LW => data,
    LH => {
        let half = (data >> (shift * 8)) & 0xFFFF;
        sign_extend_16(half)
    },
    LHU => (data >> (shift * 8)) & 0xFFFF,
    LB => {
        let byte = (data >> (shift * 8)) & 0xFF;
        sign_extend_8(byte)
    },
    LBU => (data >> (shift * 8)) & 0xFF,
    _ => unreachable!(),
};

// Write to destination register
write_register(memory, rd_ptr, value);
```

### Store Implementation Pattern
```rust
// Read value to store
let value = read_register(memory, rs2_ptr);

// Calculate address (same as load)
let addr = base_addr.wrapping_add(offset as u32);
let shift = addr & 0x3;
let aligned_addr = addr & !0x3;

// Read current memory value
let current = read_memory_word(memory, aligned_addr);

// Mask and insert new value
let stored = match opcode {
    SW => value,
    SH => {
        let mask = !(0xFFFF << (shift * 8));
        (current & mask) | ((value & 0xFFFF) << (shift * 8))
    },
    SB => {
        let mask = !(0xFF << (shift * 8));
        (current & mask) | ((value & 0xFF) << (shift * 8))
    },
    _ => unreachable!(),
};

// Write back to memory
write_memory_word(memory, aligned_addr, stored);
```

## JALR Implementation

### Runtime Implementation
```rust
// Read base address
let (_, base) = read_rv32_register(memory, F::ONE, rs1_ptr);

// Calculate target
let offset = sign_extend_i_type(immediate);
let target = base.wrapping_add(offset as u32) & !1;  // Clear LSB

// Save return address if rd != x0
if rd != 0 {
    let return_addr = ctx.get_pc() + DEFAULT_PC_STEP;
    write_register(memory, rd_ptr, return_addr);
}

// Jump to target
ctx.set_pc(target);
```

### AIR Constraints
```rust
// Target calculation
let target = abstract_compose(rs1_data) + instruction.immediate;
let aligned_target = target - (target & AB::Expr::ONE);

// Next PC constraint
builder.assert_eq(ctx.to_state.pc, aligned_target);

// Return address
let return_addr = cols.from_state.pc + DEFAULT_PC_STEP;
builder.when(needs_write).assert_eq(
    abstract_compose(cols.rd_data),
    return_addr,
);
```

## Multiplication Operations

### Basic Multiplication Pattern
```rust
let rs1_signed = rs1_val as i32;
let rs2_signed = rs2_val as i32;

let result = match opcode {
    MUL => rs1_val.wrapping_mul(rs2_val),
    MULH => ((rs1_signed as i64 * rs2_signed as i64) >> 32) as u32,
    MULHSU => ((rs1_signed as i64 * rs2_val as i64) >> 32) as u32,
    MULHU => ((rs1_val as u64 * rs2_val as u64) >> 32) as u32,
    _ => unreachable!(),
};
```

### Division Pattern
```rust
let result = match opcode {
    DIV => {
        if rs2_signed == 0 {
            u32::MAX  // Division by zero
        } else if rs1_signed == i32::MIN && rs2_signed == -1 {
            rs1_val  // Overflow case
        } else {
            (rs1_signed / rs2_signed) as u32
        }
    },
    DIVU => {
        if rs2_val == 0 {
            u32::MAX
        } else {
            rs1_val / rs2_val
        }
    },
    REM => {
        if rs2_signed == 0 {
            rs1_val
        } else if rs1_signed == i32::MIN && rs2_signed == -1 {
            0
        } else {
            (rs1_signed % rs2_signed) as u32
        }
    },
    REMU => {
        if rs2_val == 0 {
            rs1_val
        } else {
            rs1_val % rs2_val
        }
    },
    _ => unreachable!(),
};
```

## Direct Write Operations

### LUI Implementation
```rust
// Load Upper Immediate
let imm20 = instruction.operands[6].as_canonical_u32();
let value = imm20 << 12;

if rd != 0 {
    write_register(memory, rd_ptr, value);
}
```

### AUIPC Implementation
```rust
// Add Upper Immediate to PC
let imm20 = instruction.operands[6].as_canonical_u32();
let value = ctx.get_pc().wrapping_add(imm20 << 12);

if rd != 0 {
    write_register(memory, rd_ptr, value);
}
```

## Common Utility Functions

### Sign Extension
```rust
fn sign_extend_8(value: u32) -> u32 {
    ((value as i8) as i32) as u32
}

fn sign_extend_16(value: u32) -> u32 {
    ((value as i16) as i32) as u32
}

fn sign_extend_n(value: u32, n: u32) -> u32 {
    let shift = 32 - n;
    ((value as i32) << shift >> shift) as u32
}
```

### Abstract Operations for AIR
```rust
// Compose register value from limbs symbolically
fn abstract_compose_register<T: FieldAlgebra>(
    limbs: [T; RV32_REGISTER_NUM_LIMBS]
) -> T {
    limbs.into_iter()
        .enumerate()
        .fold(T::ZERO, |acc, (i, limb)| {
            acc + limb * T::from_canonical_u32(1 << (i * 8))
        })
}

// Conditional selection
fn if_else<T: Field>(condition: T, true_val: T, false_val: T) -> T {
    condition * true_val + (T::ONE - condition) * false_val
}
```

## Testing Patterns

### Unit Test Structure
```rust
#[test]
fn test_add_instruction() {
    let mut memory = MemoryController::new();
    let mut ctx = AdapterRuntimeContext::new(&mut memory);
    
    // Setup registers
    write_register(&mut memory, F::from_canonical_u32(1), 100);
    write_register(&mut memory, F::from_canonical_u32(2), 200);
    
    // Create instruction
    let instruction = Instruction {
        opcode: Opcode::ADD,
        operands: [3, 0, 1, 0, 2, 1, 0, 0], // rd=3, rs1=1, rs2=2
    };
    
    // Execute
    adapter.execute_instruction(&instruction, &mut ctx).unwrap();
    
    // Verify
    let result = read_register(&memory, F::from_canonical_u32(3));
    assert_eq!(result, 300);
}
```

### Integration Test Pattern
```rust
#[test] 
fn test_fibonacci_sequence() {
    // Test a sequence of instructions
    let program = vec![
        // Initialize
        Instruction::new(ADDI, [1, 0, 0, 0, 0, 0, 1, 0]), // x1 = 1
        Instruction::new(ADDI, [2, 0, 0, 0, 0, 0, 1, 0]), // x2 = 1
        
        // Loop
        Instruction::new(ADD, [3, 0, 1, 0, 2, 1, 0, 0]),  // x3 = x1 + x2
        Instruction::new(ADD, [1, 0, 2, 0, 0, 1, 0, 0]),  // x1 = x2
        Instruction::new(ADD, [2, 0, 3, 0, 0, 1, 0, 0]),  // x2 = x3
    ];
    
    // Execute and verify each step
}
```

## Performance Optimization Tips

### 1. Batch Memory Operations
```rust
// Instead of multiple single reads
let val1 = read_register(memory, ptr1);
let val2 = read_register(memory, ptr2);

// Use batch read when possible
let values = memory.batch_read(&[ptr1, ptr2]);
```

### 2. Minimize Constraint Degree
```rust
// High degree
let result = a * b * c * d;

// Lower degree
let temp1 = a * b;
let temp2 = c * d;
let result = temp1 * temp2;
```

### 3. Reuse Computations
```rust
// Store common subexpressions
let rs1_composed = abstract_compose(cols.rs1_data);
// Reuse rs1_composed instead of recomputing
```

## Debugging Techniques

### Add Debug Assertions
```rust
#[cfg(debug_assertions)]
{
    assert!(rs1 < 32, "Invalid register index");
    assert!(shift < 4, "Invalid shift amount");
}
```

### Trace Execution
```rust
println!("Executing {:?} rd={} rs1={} rs2={}", 
    opcode, rd, rs1, rs2);
println!("Values: rs1_val={:#x} rs2_val={:#x} result={:#x}",
    rs1_val, rs2_val, result);
```

### Verify Against Reference
```rust
// Compare with RISC-V reference implementation
let expected = riscv_reference::execute(instruction);
assert_eq!(result, expected, "Mismatch for {:?}", instruction);
```