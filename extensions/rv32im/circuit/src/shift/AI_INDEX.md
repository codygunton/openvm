# Shift Component - File Index

## mod.rs
**Purpose**: Module entry point and type definitions
- Exports public API from core module
- Defines `Rv32ShiftChip` type alias that combines adapter and core functionality
- Integrates with RV32 adapter infrastructure

## core.rs (400+ lines)
**Purpose**: Core shift operation implementation
- **Structures**:
  - `ShiftCoreCols<T>`: Column layout for shift operation trace
  - `ShiftCoreAir`: AIR (Algebraic Intermediate Representation) constraints
  - `ShiftCoreRecord<T>`: Execution trace record
  - `ShiftCoreChip`: Main chip implementation
- **Key Functions**:
  - `eval()`: Defines algebraic constraints for proof generation
  - `execute_instruction()`: Executes shift operations and generates trace
  - `run_shift()`: Dispatches to specific shift implementations
  - `run_shift_left()`: Implements SLL (shift left logical)
  - `run_shift_right()`: Implements SRL/SRA (shift right logical/arithmetic)
- **Constraints**:
  - Bit shift marker and multiplier consistency
  - Limb shift marker constraints
  - Sign extension for arithmetic shifts
  - Range checking for shift amounts

## tests.rs (400+ lines)
**Purpose**: Comprehensive test suite
- **Test Categories**:
  - **Positive Tests**: Random operation verification
    - `rv32_shift_sll_rand_test()`: SLL with random inputs
    - `rv32_shift_srl_rand_test()`: SRL with random inputs
    - `rv32_shift_sra_rand_test()`: SRA with random inputs
  - **Negative Tests**: Soundness verification with pranked traces
    - Wrong bit/limb shift values
    - Incorrect carry values
    - Wrong multiplier sides
    - Sign bit manipulation
  - **Sanity Tests**: Fixed input/output verification
- **Test Infrastructure**:
  - `ShiftPrankValues`: Structure for injecting incorrect values
  - `run_rv32_shift_negative_test()`: Framework for negative testing
  - Integration with VM test builder and verification