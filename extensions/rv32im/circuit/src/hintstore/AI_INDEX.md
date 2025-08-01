# Hintstore Component Index

## Overview
The hintstore component implements RISC-V extension opcodes for storing hints (external witness data) into memory within the OpenVM zkVM. It supports both single-word and buffered multi-word hint storage operations.

## Key Files

### Core Implementation
- `mod.rs` - Main implementation containing AIR constraints, chip logic, and execution
  - `Rv32HintStoreCols` - Trace columns structure
  - `Rv32HintStoreAir` - AIR constraints for hint store operations
  - `Rv32HintStoreChip` - Main chip implementation
  - `Rv32HintStoreRecord` - Execution record structure

### Testing
- `tests.rs` - Comprehensive test suite including:
  - Random instruction tests
  - Negative tests for constraint verification
  - Sanity tests for execution correctness

## Dependencies

### Internal
- `openvm_circuit` - Core circuit architecture
- `openvm_circuit_primitives` - Bitwise operations and utilities
- `openvm_instructions` - Instruction definitions
- `openvm_rv32im_transpiler` - Opcode definitions
- `openvm_stark_backend` - STARK proving backend

### External
- Standard Rust libraries for concurrency and serialization

## Architecture

### Supported Opcodes
1. **HINT_STOREW** - Store a single word from hint stream to memory
2. **HINT_BUFFER** - Store multiple words from hint stream to memory

### Key Components
- **Execution Bridge** - Handles instruction execution and PC updates
- **Memory Bridge** - Manages memory read/write operations
- **Bitwise Lookup** - Validates data is within byte range
- **Hint Stream** - Source of external witness data

### Trace Layout
- Row types: single word operations, buffer start, buffer continuation
- Handles multi-row traces for buffer operations
- Includes memory auxiliary columns for offline memory checking

## Integration Points
- Integrates with VM execution bus for instruction handling
- Uses memory controller for address space management
- Connects to bitwise operation lookup tables for range checks
- Consumes data from hint stream (external witness data)