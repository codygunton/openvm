# JALR Implementation Guide

## Overview
This guide provides practical patterns and examples for working with the JALR instruction implementation in OpenVM.

## Common Implementation Patterns

### 1. Creating a JALR Instruction

```rust
use openvm_instructions::Instruction;
use openvm_rv32im_transpiler::Rv32JalrOpcode::JALR;

// Create JALR instruction: rd = x1, rs1 = x2, imm = -8
let instruction = Instruction::from_usize(
    JALR.global_opcode(),
    [
        1,      // a: rd register (x1)
        2,      // b: rs1 register (x2)
        0xfff8, // c: immediate value (-8 in 12-bit two's complement)
        1,      // d: register address space (always 1 for RV32)
        0,      // e: unused
        1,      // f: write enable (1 for rd != x0)
        1,      // g: immediate sign bit
    ],
);
```

### 2. Executing JALR in Tests

```rust
use openvm_circuit::arch::testing::VmChipTestBuilder;

fn test_jalr_execution(tester: &mut VmChipTestBuilder<F>, chip: &mut Rv32JalrChip<F>) {
    // Setup: Write value to rs1
    let rs1_value = [0x12, 0x34, 0x56, 0x78]; // As 4 bytes
    tester.write(1, 2, rs1_value.map(F::from_canonical_u32));
    
    // Execute JALR from PC = 0x1000
    let initial_pc = 0x1000;
    tester.execute_with_pc(chip, &instruction, initial_pc);
    
    // Verify: Check rd value (should be initial_pc + 4)
    let rd_value = tester.read::<4>(1, 1);
    assert_eq!(compose(rd_value), initial_pc + 4);
    
    // Verify: Check new PC
    let new_pc = tester.execution.last_to_pc().as_canonical_u32();
    let expected_pc = (0x78563412_u32.wrapping_sub(8)) & !1;
    assert_eq!(new_pc, expected_pc);
}
```

### 3. Implementing Custom JALR Logic

```rust
use crate::jalr::run_jalr;

fn custom_jalr_operation(pc: u32, rs1: u32, imm: i32) -> (u32, [u32; 4]) {
    // Sign-extend immediate to 32 bits
    let imm_extended = imm as u32;
    
    // Standard JALR computation
    let (to_pc, rd_data) = run_jalr(JALR, pc, imm_extended, rs1);
    
    // Custom logic example: validate jump target
    if to_pc >= 0x8000_0000 {
        panic!("Jump to kernel space not allowed");
    }
    
    (to_pc, rd_data)
}
```

### 4. Integrating with Memory System

```rust
impl<F: PrimeField32> VmChipWrapper<F> {
    fn execute_jalr_with_memory(
        &mut self,
        memory: &mut MemoryController<F>,
        rs1_addr: u32,
        rd_addr: u32,
        imm: i32,
        from_pc: u32,
    ) -> Result<u32> {
        // Read rs1 value
        let (_, rs1_data) = memory.read::<4>(1, rs1_addr);
        let rs1_value = compose(rs1_data);
        
        // Calculate jump target
        let to_pc = (rs1_value.wrapping_add(imm as u32)) & !1;
        
        // Calculate return address
        let rd_value = from_pc + 4;
        let rd_data = decompose(rd_value);
        
        // Write to rd (if not x0)
        if rd_addr != 0 {
            memory.write(1, rd_addr, rd_data);
        }
        
        Ok(to_pc)
    }
}
```

## Constraint Debugging

### 1. Verifying Address Calculation

```rust
fn debug_address_calculation(record: &Rv32JalrCoreRecord<F>) {
    // Reconstruct rs1 value
    let rs1 = compose(record.rs1_data);
    
    // Reconstruct immediate with sign extension
    let imm = record.imm.as_canonical_u32();
    let imm_extended = imm + (record.imm_sign.as_canonical_u32() * 0xffff0000);
    
    // Calculate expected address
    let raw_addr = rs1.wrapping_add(imm_extended);
    let expected_lsb = raw_addr & 1;
    let expected_pc = raw_addr & !1;
    
    // Verify to_pc calculation
    let to_pc = (record.to_pc_limbs[0] << 1) + (record.to_pc_limbs[1] << 16);
    assert_eq!(to_pc, expected_pc);
    assert_eq!(record.to_pc_least_sig_bit.as_canonical_u32(), expected_lsb);
}
```

### 2. Checking Range Constraints

```rust
fn verify_range_constraints(cols: &Rv32JalrCoreCols<F>) {
    // Verify rd_data ranges
    let rd0 = cols.rd_data[0].as_canonical_u32();
    let rd1 = cols.rd_data[1].as_canonical_u32();
    let rd2 = cols.rd_data[2].as_canonical_u32();
    let rd3 = cols.rd_data[3].as_canonical_u32();
    
    assert!(rd0 < (1 << RV32_CELL_BITS));
    assert!(rd1 < (1 << RV32_CELL_BITS));
    assert!(rd2 < (1 << RV32_CELL_BITS));
    assert!(rd3 < (1 << (PC_BITS - 3 * RV32_CELL_BITS)));
    
    // Verify to_pc ranges
    assert!(cols.to_pc_limbs[0].as_canonical_u32() < (1 << 15));
    assert!(cols.to_pc_limbs[1].as_canonical_u32() < (1 << (PC_BITS - 16)));
}
```

## Performance Optimization

### 1. Batch Processing

```rust
fn batch_jalr_execution(
    chip: &mut Rv32JalrChip<F>,
    instructions: &[JalrInstruction],
) {
    // Pre-request all range checks
    for inst in instructions {
        let (_, rd_data) = run_jalr(JALR, inst.pc, inst.imm, inst.rs1);
        chip.core.bitwise_lookup_chip.request_range(rd_data[0], rd_data[1]);
        chip.core.range_checker_chip.add_count(rd_data[2], RV32_CELL_BITS);
    }
    
    // Execute instructions
    for inst in instructions {
        chip.execute_instruction(&inst.to_instruction(), inst.pc, inst.reads)?;
    }
}
```

### 2. Optimized Trace Generation

```rust
impl Rv32JalrCoreChip {
    fn generate_optimized_trace(&self, records: &[Rv32JalrCoreRecord<F>]) -> RowMajorMatrix<F> {
        let width = self.air().width();
        let mut trace = RowMajorMatrix::new(vec![F::ZERO; records.len() * width], width);
        
        // Batch range check requests
        for record in records {
            self.range_checker_chip.add_count(record.to_pc_limbs[0], 15);
            self.range_checker_chip.add_count(record.to_pc_limbs[1], PC_BITS - 16);
        }
        
        // Generate trace rows
        for (i, record) in records.iter().enumerate() {
            let row = trace.row_slice_mut(i);
            self.generate_trace_row(row, record.clone());
        }
        
        trace
    }
}
```

## Common Pitfalls and Solutions

### 1. Incorrect Sign Extension

**Problem**: Forgetting to sign-extend the immediate value.

```rust
// WRONG
let to_pc = rs1.wrapping_add(imm); // imm is only 12 bits

// CORRECT
let imm_extended = imm + (imm_sign * 0xffff0000);
let to_pc = rs1.wrapping_add(imm_extended);
```

### 2. Missing LSB Clear

**Problem**: Not clearing the least significant bit of the jump target.

```rust
// WRONG
let to_pc = rs1.wrapping_add(imm_extended);

// CORRECT
let to_pc = rs1.wrapping_add(imm_extended) & !1;
```

### 3. Overflow Handling

**Problem**: Not checking for PC overflow.

```rust
fn safe_jalr(pc: u32, rs1: u32, imm: u32) -> Result<(u32, [u32; 4])> {
    let to_pc = (rs1.wrapping_add(imm)) & !1;
    
    if to_pc >= (1 << PC_BITS) {
        return Err("PC overflow");
    }
    
    let rd_data = decompose(pc + 4);
    Ok((to_pc, rd_data))
}
```

## Testing Strategies

### 1. Edge Case Testing

```rust
#[test]
fn test_jalr_edge_cases() {
    // Test maximum positive offset
    test_jalr(0x1000, 0x2000, 0x7ff);
    
    // Test maximum negative offset
    test_jalr(0x1000, 0x2000, -2048);
    
    // Test LSB clearing
    test_jalr(0x1000, 0x2001, 0); // Should jump to 0x2000
    
    // Test near PC limit
    test_jalr((1 << PC_BITS) - 8, 0, 4);
}
```

### 2. Constraint Violation Testing

```rust
fn test_constraint_violation(
    modification: impl FnOnce(&mut Rv32JalrCoreCols<F>),
    expected_error: VerificationError,
) {
    // Setup normal execution
    let mut trace = generate_valid_trace();
    
    // Apply modification
    modification(&mut trace.cols);
    
    // Verify constraint failure
    let result = verify_constraints(trace);
    assert_eq!(result.unwrap_err(), expected_error);
}
```

## Integration Examples

### With Compiler

```rust
use openvm_transpiler::elf::RV32_JALR;

fn compile_jalr(rd: u8, rs1: u8, offset: i32) -> u32 {
    // Encode JALR instruction in RISC-V format
    let imm = offset & 0xfff;
    let opcode = 0b1100111;
    
    (imm << 20) | ((rs1 as u32) << 15) | (0b000 << 12) | ((rd as u32) << 7) | opcode
}
```

### With Debugger

```rust
impl JalrDebugger {
    fn trace_jalr(&self, record: &Rv32JalrCoreRecord<F>) {
        println!("JALR execution:");
        println!("  PC: 0x{:08x}", self.from_pc);
        println!("  RS1: 0x{:08x}", compose(record.rs1_data));
        println!("  IMM: {:d}", record.imm.as_canonical_i32());
        println!("  Target: 0x{:08x}", self.to_pc);
        println!("  RD: 0x{:08x}", self.from_pc + 4);
    }
}
```