# Shift Component - AI Documentation

## Overview
The shift component implements RISC-V 32-bit shift operations (SLL, SRL, SRA) within the OpenVM zkVM framework. It provides circuit implementations for bit shifting operations that are proven using zero-knowledge proofs.

## Architecture
- **Core Chip**: `ShiftCoreChip` handles the actual shift logic and constraint generation
- **Adapter**: `Rv32ShiftChip` wraps the core with RV32-specific adapters for memory and instruction handling
- **Air**: `ShiftCoreAir` defines the algebraic intermediate representation constraints

## Key Features
- Supports three shift operations: SLL (logical left), SRL (logical right), SRA (arithmetic right)
- Handles both register-register and register-immediate shift amounts
- Efficient constraint generation using bitwise operation lookups
- Sign extension support for arithmetic right shift
- Modular design allowing reuse with different word sizes

## Technical Details
- Uses limb-based representation (4 limbs of 8 bits each for RV32)
- Shift amount is limited to 0-31 bits (5 bits of shift value used)
- Leverages lookup tables for efficient bitwise operations
- Integrates with memory controller for register reads/writes

## Dependencies
- `openvm_circuit`: Core circuit framework
- `openvm_circuit_primitives`: Bitwise operation lookups and range checking
- `openvm_rv32im_transpiler`: RV32 instruction opcodes
- `openvm_stark_backend`: STARK proof system backend

## Testing
Comprehensive test suite includes:
- Random operation tests for correctness
- Negative tests with pranked traces for soundness
- Sanity tests with known input/output values
- Coverage of edge cases (zero shifts, maximum shifts, sign bits)