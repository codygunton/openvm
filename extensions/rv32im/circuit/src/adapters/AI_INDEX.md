# RV32IM Circuit Adapters AI Documentation Index

This directory contains AI-focused documentation for the OpenVM RV32IM Circuit Adapters component.

## Documentation Files

### [AI_DOCS.md](./AI_DOCS.md)
High-level architectural overview of the RV32IM Circuit Adapters component, including:
- Core adapter types for RISC-V 32-bit integer instructions
- Memory access patterns and register handling
- Integration with the OpenVM execution framework
- Design principles for RISC-V instruction execution

### [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md)
Detailed implementation patterns and code examples, including:
- Creating RISC-V adapter chips
- Register read/write patterns
- Immediate value handling
- ALU, branch, and memory operations
- Integration with bitwise lookup tables

### [CLAUDE.md](./CLAUDE.md)
Instructions for AI assistants working with this component, including:
- Key principles for RISC-V adapter implementation
- Common pitfalls with register operations
- Testing requirements for instruction correctness
- Performance optimization guidelines

### [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md)
Concise reference for common operations:
- Adapter chip construction patterns
- Register operation snippets
- Immediate value handling
- Key constants and utilities

## Component Source Files

### Core Adapters
- [`alu.rs`](./alu.rs) - ALU operations adapter (ADD, SUB, XOR, OR, AND, SLT, etc.)
- [`branch.rs`](./branch.rs) - Branch operations adapter (BEQ, BNE, BLT, BGE, etc.)
- [`jalr.rs`](./jalr.rs) - Jump and link register adapter
- [`loadstore.rs`](./loadstore.rs) - Load/store operations adapter (LW, SW, LH, SH, LB, SB)
- [`mul.rs`](./mul.rs) - Multiplication/division adapter (MUL, MULH, DIV, REM)
- [`rdwrite.rs`](./rdwrite.rs) - Register direct write adapter
- [`mod.rs`](./mod.rs) - Module exports and utility functions

## Related Components

The RV32IM Circuit Adapters interact with several other components:
- `openvm_circuit::arch` - Core VM adapter interfaces and execution framework
- `openvm_circuit::system::memory` - Memory subsystem for register/memory operations
- `openvm_circuit_primitives::bitwise_op_lookup` - Bitwise operation lookup tables
- `openvm_circuit_primitives::var_range` - Variable range checking for offsets
- `openvm_rv32im_transpiler` - RISC-V instruction definitions and opcodes

## Quick Start

For AI assistants new to this component:
1. Start with [AI_DOCS.md](./AI_DOCS.md) for architectural understanding
2. Review specific adapter files for implementation details
3. Reference [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md) for common patterns
4. Use [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md) for detailed examples
5. Follow [CLAUDE.md](./CLAUDE.md) for best practices

## Key Concepts Summary

- **RISC-V Registers**: 32-bit values stored as 4 bytes (limbs) in memory
- **Address Spaces**: Register space (AS=1) vs immediate space (AS=0)
- **Instruction Format**: Standard RISC-V encoding with OpenVM extensions
- **Memory Operations**: Aligned and unaligned loads/stores with shift handling
- **Immediate Values**: Support for I-type, S-type, B-type, and J-type immediates

## Component Responsibilities

1. **Instruction Execution**: Implement RISC-V RV32IM instruction set
2. **Register Management**: Handle 32 general-purpose registers (x0-x31)
3. **Memory Access**: Support byte, halfword, and word operations
4. **PC Management**: Update program counter for sequential and branch execution
5. **Constraint Generation**: Produce AIR constraints for proof generation

## Common Use Cases

1. **Arithmetic Operations**: ADD, SUB, AND, OR, XOR, shifts
2. **Comparisons**: SLT, SLTU for conditional logic
3. **Branching**: Conditional jumps based on register comparisons
4. **Memory Access**: Load/store with various widths and alignments
5. **Function Calls**: JAL, JALR for procedure calls

## Adapter Types Overview

### ALU Adapter
- **Purpose**: Arithmetic and logic operations
- **Reads**: 2 (rs1, rs2 or immediate)
- **Writes**: 1 (rd)
- **Features**: Immediate support, bitwise operations

### Branch Adapter  
- **Purpose**: Conditional branching
- **Reads**: 2 (rs1, rs2)
- **Writes**: 0
- **Features**: PC modification, immediate offsets

### JALR Adapter
- **Purpose**: Jump and link register
- **Reads**: 1 (rs1)
- **Writes**: 1 (rd, optional)
- **Features**: PC+4 storage, register-based jumps

### LoadStore Adapter
- **Purpose**: Memory load/store operations
- **Reads**: Variable (register + memory)
- **Writes**: Variable (register or memory)
- **Features**: Byte/halfword/word support, alignment handling

### Mul Adapter
- **Purpose**: Multiplication and division
- **Reads**: 2 (rs1, rs2)
- **Writes**: 1 (rd)
- **Features**: Full product, high bits, signed/unsigned

### RdWrite Adapter
- **Purpose**: Direct register writes
- **Reads**: 0
- **Writes**: 1 (rd)
- **Features**: Immediate to register, LUI/AUIPC support

## Key Constants

- `RV32_REGISTER_NUM_LIMBS`: 4 (32-bit registers as 4 bytes)
- `RV32_CELL_BITS`: 8 (bits per limb)
- `RV32_REGISTER_AS`: 1 (address space for registers)
- `RV32_IMM_AS`: 0 (address space for immediates)
- `INT256_NUM_LIMBS`: 32 (for 256-bit operations)