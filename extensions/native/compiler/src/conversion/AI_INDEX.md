# Conversion Module AI Index

## Overview
The conversion module is responsible for converting high-level assembly instructions into low-level VM instructions that can be executed by the OpenVM runtime. It acts as the final stage of the compilation pipeline, translating abstract assembly operations into concrete machine instructions.

## Key Components

### `mod.rs`
Main module file containing:
- `CompilerOptions` - Configuration for the conversion process
- `convert_instruction` - Core function that converts individual assembly instructions
- `convert_program` - Converts an entire assembly program to VM instructions

## Purpose
This module bridges the gap between the human-readable assembly language (ASM) and the VM's instruction format, handling:
- Instruction encoding
- Memory addressing modes (Native vs Immediate)
- Branch offset calculations
- Extension field operations
- Phantom instruction generation

## Architecture Role
```
IR Layer → ASM Layer → Conversion Layer → VM Instructions
                            ↑
                      (This module)
```

## Key Responsibilities
1. **Instruction Translation**: Converts ASM instructions to VM opcodes
2. **Address Mode Handling**: Manages Native (memory) vs Immediate (constant) operands
3. **Branch Resolution**: Calculates relative offsets for jumps and branches
4. **Extension Field Support**: Handles operations on extension field elements
5. **Debug Information**: Preserves debug metadata through conversion

## Dependencies
- `openvm_circuit::arch::instructions::program::Program` - Output format
- `openvm_instructions` - VM instruction definitions
- `openvm_rv32im_transpiler` - RISC-V opcode definitions
- `openvm_stark_backend::p3_field` - Field arithmetic types
- Parent module ASM types - Input instruction format

## Related Components
- **ASM Module** (`../asm/`): Provides the assembly instructions to convert
- **IR Module** (`../ir/`): Higher-level intermediate representation
- **Constraints Module** (`../constraints/`): Constraint generation for proving

## Usage Context
The conversion module is invoked as the final step in the compilation pipeline:
1. IR code is generated from high-level constructs
2. IR is lowered to ASM instructions
3. ASM instructions are converted to VM instructions (this module)
4. VM instructions are executed or proven

## Important Notes
- Supports both base field (F) and extension field (EF) operations
- Handles special instructions like FRI operations and batch verification
- Preserves debug information for better error reporting
- PC (program counter) increments by DEFAULT_PC_STEP (default 4)