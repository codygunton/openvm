# OpenVM Instructions Component - Examples

This document provides practical examples of using the `openvm-instructions` crate for instruction creation, opcode management, and program construction.

## Basic Setup

```rust
use openvm_instructions::*;
use openvm_stark_backend::p3_field::Field;
use p3_baby_bear::BabyBear;

type F = BabyBear;
```

## Example 1: Basic Instruction Creation

```rust
#[test]
fn example_basic_instruction_creation() {
    // Create instruction with sized operands
    let opcode = SystemOpcode::TERMINATE.global_opcode();
    let inst = Instruction::<F>::from_usize(opcode, [42, 100, 0]);
    
    assert_eq!(inst.opcode, opcode);
    assert_eq!(inst.a, F::from_canonical_usize(42));
    assert_eq!(inst.b, F::from_canonical_usize(100));
    assert_eq!(inst.c, F::ZERO);
    
    // Create instruction with signed operands
    let inst2 = Instruction::<F>::from_isize(opcode, -10, 20, -30, 40, -50);
    assert_eq!(inst2.a, F::from_canonical_usize((-10isize) as usize));
    assert_eq!(inst2.b, F::from_canonical_usize(20));
}
```

## Example 2: Custom Opcode Definition

```rust
use openvm_instructions_derive::LocalOpcode;

#[derive(Copy, Clone, Debug, PartialEq, Eq, LocalOpcode)]
#[opcode_offset = 0x200] // Custom offset for this instruction class
#[repr(usize)]
pub enum ArithmeticOpcode {
    ADD,    // Global opcode: 0x200
    SUB,    // Global opcode: 0x201
    MUL,    // Global opcode: 0x202
    DIV,    // Global opcode: 0x203
}

#[test]
fn example_custom_opcodes() {
    let add_opcode = ArithmeticOpcode::ADD;
    let global_add = add_opcode.global_opcode();
    
    // Verify opcode mapping
    assert_eq!(global_add.as_usize(), 0x200);
    assert_eq!(add_opcode.local_usize(), 0);
    
    // Create instruction with custom opcode
    let add_inst = Instruction::<F>::from_usize(global_add, [10, 20]);
    
    // Verify we can convert back
    let local_idx = global_add.local_opcode_idx(ArithmeticOpcode::CLASS_OFFSET);
    let recovered_opcode = ArithmeticOpcode::from_usize(local_idx);
    assert_eq!(recovered_opcode, ArithmeticOpcode::ADD);
}
```

## Example 3: Program Construction

```rust
#[test]
fn example_program_construction() {
    let step = 4;  // RISC-V style 4-byte steps
    let pc_base = 0x1000;  // Program starts at address 0x1000
    
    // Create empty program
    let mut program = Program::<F>::new_empty(step, pc_base);
    
    // Add instructions one by one
    let add_op = ArithmeticOpcode::ADD.global_opcode();
    let sub_op = ArithmeticOpcode::SUB.global_opcode();
    let term_op = SystemOpcode::TERMINATE.global_opcode();
    
    // ADD r1, r2, r3 (pseudo-assembly)
    let add_inst = Instruction::<F>::from_usize(add_op, [1, 2, 3]);
    program.add_instruction(add_inst, None);
    
    // SUB r4, r1, r2
    let sub_inst = Instruction::<F>::from_usize(sub_op, [4, 1, 2]);
    program.add_instruction(sub_inst, None);
    
    // TERMINATE
    let term_inst = Instruction::<F>::from_usize(term_op, []);
    program.add_instruction(term_inst, None);
    
    // Verify program structure
    assert_eq!(program.len(), 3);
    assert_eq!(program.step, 4);
    assert_eq!(program.pc_base, 0x1000);
    
    // Access instructions by PC
    let first_inst = program.get_instruction(0x1000).unwrap();
    assert_eq!(first_inst.opcode, add_op);
    
    let second_inst = program.get_instruction(0x1004).unwrap();
    assert_eq!(second_inst.opcode, sub_op);
}
```

## Example 4: Program with Debug Information

```rust
use crate::instruction::DebugInfo;

#[test]
fn example_program_with_debug_info() {
    let mut program = Program::<F>::new_empty(4, 0);
    
    // Create debug information
    let debug_info = DebugInfo {
        source_line: Some(42),
        source_file: Some("main.rs".to_string()),
        function_name: Some("compute".to_string()),
    };
    
    let inst = Instruction::<F>::from_usize(
        ArithmeticOpcode::ADD.global_opcode(),
        [10, 20, 30]
    );
    
    // Add instruction with debug info
    program.add_instruction(inst, Some(debug_info.clone()));
    
    // Retrieve instruction with debug info
    let (retrieved_inst, retrieved_debug) = program.get_instruction_with_debug(0).unwrap();
    assert_eq!(retrieved_inst.opcode, ArithmeticOpcode::ADD.global_opcode());
    assert_eq!(retrieved_debug.as_ref().unwrap().source_line, Some(42));
}
```

## Example 5: Bulk Program Creation

```rust
#[test]
fn example_bulk_program_creation() {
    // Create a batch of instructions
    let instructions = vec![
        Instruction::<F>::from_usize(ArithmeticOpcode::ADD.global_opcode(), [1, 2, 3]),
        Instruction::<F>::from_usize(ArithmeticOpcode::SUB.global_opcode(), [4, 5, 6]),
        Instruction::<F>::from_usize(ArithmeticOpcode::MUL.global_opcode(), [7, 8, 9]),
        Instruction::<F>::from_usize(SystemOpcode::TERMINATE.global_opcode(), []),
    ];
    
    // Create program from instruction batch
    let program = Program::<F>::new_without_debug_infos(&instructions, 4, 0);
    
    assert_eq!(program.len(), 4);
    
    // Verify all instructions are accessible
    for (i, expected_inst) in instructions.iter().enumerate() {
        let pc = (i as u32) * 4;
        let actual_inst = program.get_instruction(pc).unwrap();
        assert_eq!(actual_inst.opcode, expected_inst.opcode);
        assert_eq!(actual_inst.a, expected_inst.a);
    }
}
```

## Example 6: Program Serialization

```rust
use serde::{Deserialize, Serialize};

#[test]
fn example_program_serialization() {
    // Create a program
    let mut program = Program::<F>::new_empty(4, 0x2000);
    program.add_instruction(
        Instruction::<F>::from_usize(ArithmeticOpcode::ADD.global_opcode(), [1, 2, 3]),
        None
    );
    program.add_instruction(
        Instruction::<F>::from_usize(SystemOpcode::TERMINATE.global_opcode(), []),
        None
    );
    
    // Serialize program
    let serialized = bincode::serialize(&program).unwrap();
    
    // Deserialize program
    let deserialized: Program<F> = bincode::deserialize(&serialized).unwrap();
    
    // Verify program integrity
    assert_eq!(deserialized.len(), program.len());
    assert_eq!(deserialized.step, program.step);
    assert_eq!(deserialized.pc_base, program.pc_base);
    
    for i in 0..program.len() {
        let pc = program.pc_base + (i as u32) * program.step;
        let original = program.get_instruction(pc).unwrap();
        let restored = deserialized.get_instruction(pc).unwrap();
        assert_eq!(original.opcode, restored.opcode);
        assert_eq!(original.a, restored.a);
    }
}
```

## Example 7: Complex Instruction Patterns

```rust
// Define a more complex instruction set
#[derive(Copy, Clone, Debug, PartialEq, Eq, LocalOpcode)]
#[opcode_offset = 0x400]
#[repr(usize)]
pub enum MemoryOpcode {
    LOAD,       // Load from memory
    STORE,      // Store to memory
    LOAD_IMM,   // Load immediate value
}

#[test]
fn example_complex_instructions() {
    let mut program = Program::<F>::new_empty(4, 0);
    
    // LOAD_IMM r1, #42 (load immediate 42 into register 1)
    let load_imm = Instruction::<F>::from_isize(
        MemoryOpcode::LOAD_IMM.global_opcode(),
        1,    // destination register
        42,   // immediate value
        0, 0, 0  // unused operands
    );
    program.add_instruction(load_imm, None);
    
    // STORE r1, [r2 + #8] (store r1 to memory at r2 + 8)
    let store = Instruction::<F>::from_isize(
        MemoryOpcode::STORE.global_opcode(),
        1,    // source register
        2,    // base register
        8,    // offset
        0, 0  // unused operands
    );
    program.add_instruction(store, None);
    
    // LOAD r3, [r2 + #8] (load from memory at r2 + 8 into r3)
    let load = Instruction::<F>::from_isize(
        MemoryOpcode::LOAD.global_opcode(),
        3,    // destination register
        2,    // base register
        8,    // offset
        0, 0  // unused operands
    );
    program.add_instruction(load, None);
    
    assert_eq!(program.len(), 3);
    
    // Verify instruction encodings
    let first = program.get_instruction(0).unwrap();
    assert_eq!(first.opcode, MemoryOpcode::LOAD_IMM.global_opcode());
    assert_eq!(first.a, F::from_canonical_usize(1));
    assert_eq!(first.b, F::from_canonical_usize(42));
}
```

## Example 8: Field Element Conversion

```rust
#[test]
fn example_field_conversion() {
    let opcode = SystemOpcode::TERMINATE.global_opcode();
    
    // Test opcode to field conversion
    let opcode_field = opcode.to_field::<F>();
    assert_eq!(opcode_field, F::from_canonical_usize(opcode.as_usize()));
    
    // Create instruction with field operands
    let inst = Instruction::<F>::new(
        opcode,
        F::from_canonical_usize(100),
        F::from_canonical_usize(200),
        F::from_canonical_usize(300),
        F::ZERO,
        F::ONE,
        F::TWO,
        F::from_canonical_usize(42)
    );
    
    // Verify field values
    assert_eq!(inst.a, F::from_canonical_usize(100));
    assert_eq!(inst.b, F::from_canonical_usize(200));
    assert_eq!(inst.c, F::from_canonical_usize(300));
    assert_eq!(inst.d, F::ZERO);
    assert_eq!(inst.e, F::ONE);
    assert_eq!(inst.f, F::TWO);
    assert_eq!(inst.g, F::from_canonical_usize(42));
}
```

## Example 9: Error Handling

```rust
#[test]
fn example_error_handling() {
    let program = Program::<F>::new_empty(4, 0);
    
    // Attempt to access non-existent instruction
    assert!(program.get_instruction(100).is_none());
    
    // PC alignment check
    let mut program = Program::<F>::new_empty(4, 0);
    program.add_instruction(
        Instruction::<F>::from_usize(SystemOpcode::TERMINATE.global_opcode(), []),
        None
    );
    
    // Valid PC (aligned to step size)
    assert!(program.get_instruction(0).is_some());
    
    // Invalid PC (not aligned)
    assert!(program.get_instruction(1).is_none());
    assert!(program.get_instruction(2).is_none());
    assert!(program.get_instruction(3).is_none());
    
    // Next valid PC
    program.add_instruction(
        Instruction::<F>::from_usize(SystemOpcode::PHANTOM.global_opcode(), []),
        None
    );
    assert!(program.get_instruction(4).is_some());
}
```

## Example 10: Performance Benchmarking

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_instruction_creation(c: &mut Criterion) {
    let opcode = ArithmeticOpcode::ADD.global_opcode();
    
    c.bench_function("create_instruction_from_usize", |b| {
        b.iter(|| {
            Instruction::<F>::from_usize(opcode, [1, 2, 3, 4, 5])
        })
    });
    
    c.bench_function("create_instruction_from_isize", |b| {
        b.iter(|| {
            Instruction::<F>::from_isize(opcode, -1, 2, -3, 4, -5)
        })
    });
}

fn bench_program_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("program_operations");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("add_instruction", size), size, |b, &size| {
            b.iter(|| {
                let mut program = Program::<F>::new_empty(4, 0);
                for i in 0..size {
                    let inst = Instruction::<F>::from_usize(
                        ArithmeticOpcode::ADD.global_opcode(),
                        [i, i + 1, i + 2]
                    );
                    program.add_instruction(inst, None);
                }
            })
        });
        
        group.bench_with_input(BenchmarkId::new("get_instruction", size), size, |b, &size| {
            let mut program = Program::<F>::new_empty(4, 0);
            for i in 0..size {
                let inst = Instruction::<F>::from_usize(
                    ArithmeticOpcode::ADD.global_opcode(),
                    [i, i + 1, i + 2]
                );
                program.add_instruction(inst, None);
            }
            
            b.iter(|| {
                for i in 0..size {
                    let pc = (i as u32) * 4;
                    program.get_instruction(pc);
                }
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_instruction_creation, bench_program_operations);
criterion_main!(benches);
```

These examples demonstrate the key patterns for working with the OpenVM instructions component, from basic instruction creation to complex program construction and performance optimization. The component provides a flexible foundation for building instruction sets while maintaining type safety and integration with the broader OpenVM ecosystem.