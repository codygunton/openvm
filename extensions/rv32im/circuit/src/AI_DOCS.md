# RV32IM Circuit Extension - AI Documentation

## Component Overview

The RV32IM circuit extension implements the RISC-V 32-bit Integer and Multiplication instruction set extensions for OpenVM's zkVM framework. This extension provides zero-knowledge proof circuits for executing RV32IM instructions within the OpenVM architecture.

## Purpose

This extension enables OpenVM to execute RISC-V 32-bit programs with integer arithmetic and multiplication operations while generating zero-knowledge proofs of correct execution. It's a critical component for running standard RISC-V programs in a verifiable manner.

## Architecture

### Two-Layer Design
Each instruction implementation follows a two-layer architecture:

1. **Adapter Layer**: Handles VM interactions (memory reads/writes, program counter updates)
2. **Core Layer**: Implements the actual instruction logic and constraints

### Key Components

- **Base ALU Operations**: ADD, SUB, AND, OR, XOR, etc.
- **Comparison Operations**: SLT, SLTU (set less than)
- **Shift Operations**: SLL, SRL, SRA (logical/arithmetic shifts)
- **Memory Operations**: Load/Store with various widths (byte, halfword, word)
- **Control Flow**: JAL, JALR, branches (BEQ, BNE, BLT, BGE, etc.)
- **Multiplication Extension**: MUL, MULH, MULHU, MULHSU
- **Division/Remainder**: DIV, DIVU, REM, REMU
- **Immediate Operations**: LUI, AUIPC
- **IO Operations**: HintStore for input/output

### Extension Structure

The module exports three main extensions:
- `Rv32I`: Base integer instruction set
- `Rv32Io`: Input/output operations
- `Rv32M`: Multiplication and division extension

## Integration with OpenVM

The extension integrates with OpenVM through:
- `VmExtension` trait implementation for each extension type
- Instruction executors that plug into the VM's execution bus
- Shared periphery chips (bitwise operations, range checking)
- Memory bridge for register and memory access

## Testing Infrastructure

- Unit tests for individual instruction implementations
- Test utilities for generating random inputs
- Integration with OpenVM's testing framework

## Security Considerations

- All arithmetic operations include range checks
- Memory accesses are bounds-checked
- Instruction decoding validates operands
- Phantom operations for secure hints and randomness