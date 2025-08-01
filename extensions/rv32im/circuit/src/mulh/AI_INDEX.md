# MulH Component AI Index

## Component Overview
The MulH component implements the high-multiplication operations for RISC-V 32-bit integer multiplication extension (RV32IM). It provides support for `MULH`, `MULHSU`, and `MULHU` instructions, which compute the upper 32 bits of 32-bit × 32-bit multiplication.

## Key Files

### Core Implementation
- **mod.rs** (13 lines)
  - Module definitions and type aliases
  - Exports the core chip implementation
  - Defines `Rv32MulHChip` as a wrapper combining adapter and core functionality

- **core.rs** (365 lines)
  - Core MulH implementation with constraint evaluation
  - `MulHCoreAir`: AIR (Algebraic Intermediate Representation) for constraint system
  - `MulHCoreChip`: Main chip implementation with execution logic
  - `MulHCoreCols`: Column layout for trace generation
  - `run_mulh()`: Core computation function for high multiplication

### Testing
- **tests.rs** (453 lines)
  - Comprehensive test suite including:
    - Positive tests with random inputs
    - Negative tests for constraint validation
    - Sanity tests with known values
  - Tests for all three opcodes: MULH, MULHSU, MULHU

## Architecture Components

### Column Structure (`MulHCoreCols`)
- `a[NUM_LIMBS]`: Result (upper bits of multiplication)
- `b[NUM_LIMBS]`: First operand
- `c[NUM_LIMBS]`: Second operand
- `a_mul[NUM_LIMBS]`: Lower bits of multiplication
- `b_ext`: Sign extension for operand b
- `c_ext`: Sign extension for operand c
- `opcode_*_flag`: Flags for MULH, MULHSU, MULHU

### Dependencies
- `BitwiseOperationLookupBus`: For sign extension verification
- `RangeTupleCheckerBus`: For range checking intermediate values
- `VmAdapterInterface`: For integration with VM execution environment

## Opcodes Supported
1. **MULH**: Signed × Signed multiplication (upper bits)
2. **MULHSU**: Signed × Unsigned multiplication (upper bits)
3. **MULHU**: Unsigned × Unsigned multiplication (upper bits)

## Key Functions
- `eval()`: Constraint evaluation for AIR
- `execute_instruction()`: Runtime execution of MulH operations
- `generate_trace_row()`: Trace generation for proving
- `run_mulh()`: Core multiplication algorithm with carry propagation

## Integration Points
- Uses RV32 adapter for memory operations
- Integrates with OpenVM's modular chip architecture
- Shares bitwise and range checking infrastructure