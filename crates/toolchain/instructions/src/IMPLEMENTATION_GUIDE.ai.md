# IMPLEMENTATION_GUIDE: OpenVM Instructions

## Overview
This guide provides practical patterns and examples for implementing features using the OpenVM instructions system. It covers common tasks, best practices, and integration patterns.

## Creating Custom Instruction Sets

### 1. Define Your Opcode Enum
```rust
use openvm_instructions_derive::LocalOpcode;
use strum_macros::{EnumCount, EnumIter, FromRepr};

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, 
    EnumCount, EnumIter, FromRepr, LocalOpcode,
)]
#[opcode_offset = 0x100]  // Choose unique offset
#[repr(usize)]
pub enum MyCustomOpcode {
    ADD,
    SUB,
    MUL,
    CUSTOM_OP,
}
```

### 2. Create Instructions
```rust
use openvm_instructions::{Instruction, VmOpcode};
use openvm_stark_backend::p3_field::Field;

fn create_add_instruction<F: Field>(
    dst: usize,
    src1: usize, 
    src2: usize,
) -> Instruction<F> {
    Instruction::from_usize(
        MyCustomOpcode::ADD.global_opcode(),
        [dst, src1, src2]
    )
}

// Using all operands for complex instruction
fn create_complex_instruction<F: Field>(
    opcode: MyCustomOpcode,
    operands: [isize; 7],
) -> Instruction<F> {
    Instruction::large_from_isize(
        opcode.global_opcode(),
        operands[0], operands[1], operands[2], operands[3],
        operands[4], operands[5], operands[6]
    )
}
```

## Building Programs

### Basic Program Construction
```rust
use openvm_instructions::{Program, Instruction};

fn build_simple_program<F: Field>() -> Program<F> {
    let mut program = Program::new_empty(4, 0); // step=4, base=0
    
    // Add instructions
    program.push_instruction(create_add_instruction(0, 1, 2));
    program.push_instruction(create_sub_instruction(3, 0, 1));
    
    // Add instruction with debug info
    let debug_info = DebugInfo::new(
        "add r0, r1, r2".to_string(),
        Some(Backtrace::capture())
    );
    program.push_instruction_and_debug_info(
        create_add_instruction(0, 1, 2),
        Some(debug_info)
    );
    
    program
}
```

### Program with Sparse Instructions
```rust
fn build_sparse_program<F: Field>() -> Program<F> {
    let mut instructions = vec![None; 10];
    
    // Place instructions at specific indices
    instructions[0] = Some(create_add_instruction(0, 1, 2));
    instructions[3] = Some(create_jump_instruction(8));
    instructions[8] = Some(create_halt_instruction());
    
    Program::new_without_debug_infos_with_option(
        &instructions,
        4,  // step
        0   // base
    )
}
```

### Merging Programs
```rust
fn merge_programs<F: Field>(
    main: &mut Program<F>,
    subroutine: Program<F>
) {
    // Append subroutine to main program
    main.append(subroutine);
}
```

## Creating Executables

### Basic Executable
```rust
use openvm_instructions::{VmExe, MemoryImage};
use std::collections::BTreeMap;

fn create_executable<F: Field>(program: Program<F>) -> VmExe<F> {
    let mut exe = VmExe::new(program)
        .with_pc_start(0x1000);
    
    // Initialize memory
    let mut init_memory: MemoryImage<F> = BTreeMap::new();
    
    // Set up stack pointer
    init_memory.insert(
        (RV32_REGISTER_AS, 2), // sp register
        F::from_canonical_u32(0x10000)
    );
    
    // Load initial data
    for (i, value) in initial_data.iter().enumerate() {
        init_memory.insert(
            (RV32_MEMORY_AS, 0x2000 + i as u32),
            F::from_canonical_u32(*value)
        );
    }
    
    exe.with_init_memory(init_memory)
}
```

### Executable with Function Metadata
```rust
use openvm_instructions::{FnBound, FnBounds};

fn add_function_bounds(exe: &mut VmExe<F>) {
    // Main function
    exe.fn_bounds.insert(0x1000, FnBound {
        start: 0x1000,
        end: 0x1100,
        name: "main".to_string(),
    });
    
    // Helper function
    exe.fn_bounds.insert(0x1100, FnBound {
        start: 0x1100,
        end: 0x1200,
        name: "helper".to_string(),
    });
}
```

## Working with Phantom Instructions

### Debug Support
```rust
fn add_debug_support<F: Field>(program: &mut Program<F>) {
    use openvm_instructions::{PhantomDiscriminant, SysPhantom};
    
    // Add debug panic instruction
    let panic_inst = Instruction::<F>::debug(
        PhantomDiscriminant(SysPhantom::DebugPanic as u16)
    );
    program.push_instruction(panic_inst);
    
    // Add tracing
    let start_trace = Instruction::<F>::debug(
        PhantomDiscriminant(SysPhantom::CtStart as u16)
    );
    let end_trace = Instruction::<F>::debug(
        PhantomDiscriminant(SysPhantom::CtEnd as u16)
    );
    
    program.push_instruction(start_trace);
    // ... code to trace ...
    program.push_instruction(end_trace);
}
```

### Custom Phantom Instructions
```rust
fn create_custom_phantom<F: Field>(
    discriminant: u16,
    data1: F,
    data2: F,
    extra: u16,
) -> Instruction<F> {
    Instruction::phantom(
        PhantomDiscriminant(discriminant),
        data1,
        data2,
        extra
    )
}
```

## Program Analysis

### Iterating Over Instructions
```rust
fn analyze_program<F: Field>(program: &Program<F>) {
    // Get all defined instructions
    for inst in program.defined_instructions() {
        match inst.opcode.as_usize() {
            x if x == MyCustomOpcode::ADD.global_opcode().as_usize() => {
                println!("Found ADD: dst={}, src1={}, src2={}", 
                    inst.a, inst.b, inst.c);
            }
            _ => {}
        }
    }
    
    // Iterate with PC values
    for (pc, inst, debug_info) in program.enumerate_by_pc() {
        println!("PC {:#x}: opcode={}", pc, inst.opcode);
        if let Some(debug) = debug_info {
            println!("  DSL: {}", debug.dsl_instruction);
        }
    }
}
```

### Finding Instruction at PC
```rust
fn find_instruction_at_pc<F: Field>(
    program: &Program<F>,
    target_pc: u32,
) -> Option<&Instruction<F>> {
    if target_pc < program.pc_base {
        return None;
    }
    
    let offset = target_pc - program.pc_base;
    if offset % program.step != 0 {
        return None;
    }
    
    let index = (offset / program.step) as usize;
    program.get_instruction_and_debug_info(index)
        .map(|(inst, _)| inst)
}
```

## Serialization Patterns

### Saving Programs
```rust
fn save_program<F: Field + Serialize>(
    program: &Program<F>,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    let bytes = bitcode::serialize(program)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

fn load_program<F: Field + for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<Program<F>, Box<dyn Error>> {
    let bytes = std::fs::read(path)?;
    let program = bitcode::deserialize(&bytes)?;
    Ok(program)
}
```

### Executable Serialization
```rust
fn save_executable<F: Field + Serialize + Ord>(
    exe: &VmExe<F>,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(exe)?;
    std::fs::write(path, json)?;
    Ok(())
}
```

## Integration Examples

### With Compiler Output
```rust
struct CompilerOutput<F> {
    instructions: Vec<Instruction<F>>,
    debug_infos: Vec<Option<DebugInfo>>,
    entry_point: u32,
    memory_init: Vec<(u32, F)>,
}

fn compile_to_exe<F: Field>(output: CompilerOutput<F>) -> VmExe<F> {
    // Create program
    let program = Program::from_instructions_and_debug_infos(
        &output.instructions,
        &output.debug_infos
    );
    
    // Create memory image
    let mut memory = BTreeMap::new();
    for (addr, value) in output.memory_init {
        memory.insert((RV32_MEMORY_AS, addr), value);
    }
    
    // Build executable
    VmExe::new(program)
        .with_pc_start(output.entry_point)
        .with_init_memory(memory)
}
```

### With VM Runtime
```rust
fn prepare_for_execution<F: Field>(exe: &VmExe<F>) -> VmState<F> {
    VmState {
        pc: exe.pc_start,
        memory: exe.init_memory.clone(),
        program: &exe.program,
        // ... other state
    }
}
```

## Best Practices

### 1. Opcode Offset Selection
- Choose offsets that won't conflict with other instruction sets
- Leave gaps between sets for future expansion
- Document your offset choice

### 2. Operand Usage
- Use consistent operand positions across similar instructions
- Document operand meanings in instruction implementations
- Consider using helper functions for common patterns

### 3. Debug Information
- Include debug info during development
- Strip before production deployment
- Use phantom instructions for runtime debugging

### 4. Memory Layout
- Follow RISC-V conventions when applicable
- Use appropriate address spaces
- Document memory layout assumptions

### 5. Error Handling
- Validate PC values before access
- Check for instruction existence
- Handle field element conversions carefully

## Common Patterns

### Instruction Dispatch
```rust
fn dispatch_instruction<F: Field>(inst: &Instruction<F>) {
    match inst.opcode.as_usize() {
        x if x == SystemOpcode::TERMINATE.global_opcode().as_usize() => {
            // Handle termination
        }
        x if x >= 0x100 && x < 0x100 + MyCustomOpcode::COUNT => {
            let local = inst.opcode.local_opcode_idx(0x100);
            match MyCustomOpcode::from_repr(local) {
                Some(MyCustomOpcode::ADD) => handle_add(inst),
                Some(MyCustomOpcode::SUB) => handle_sub(inst),
                // ...
                None => panic!("Invalid opcode"),
            }
        }
        _ => panic!("Unknown opcode: {}", inst.opcode),
    }
}
```

### Program Validation
```rust
fn validate_program<F: Field>(program: &Program<F>) -> Result<(), String> {
    let max_pc = program.pc_base + 
        (program.step * (program.len() as u32 - 1));
    
    if max_pc > MAX_ALLOWED_PC {
        return Err("Program exceeds maximum PC".to_string());
    }
    
    // Check for at least one instruction
    if program.num_defined_instructions() == 0 {
        return Err("Program has no instructions".to_string());
    }
    
    Ok(())
}
```