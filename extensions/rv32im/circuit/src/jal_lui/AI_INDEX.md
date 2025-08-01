# JAL/LUI Component - AI Index

## Quick Navigation

### Core Implementation Files
- [`core.rs`](./core.rs) - Main implementation of JAL and LUI instructions
- [`mod.rs`](./mod.rs) - Module exports and type definitions
- [`tests.rs`](./tests.rs) - Comprehensive test suite

### Key Types and Structures

#### Main Chip Types
- `Rv32JalLuiChip<F>` - Main chip wrapper with adapter
- `Rv32JalLuiCoreChip` - Core implementation chip
- `Rv32JalLuiCoreAir` - AIR constraint implementation

#### Data Structures
- `Rv32JalLuiCoreCols<T>` - Trace columns layout
- `Rv32JalLuiCoreRecord<F>` - Execution record

### Key Functions
- `run_jal_lui()` - Core execution logic for both instructions
- `execute_instruction()` - VM interface implementation
- `eval()` - AIR constraint evaluation

### Constants
- `RV32_REGISTER_NUM_LIMBS = 4` - Number of 8-bit limbs per register
- `RV32_CELL_BITS = 8` - Bits per limb
- `PC_BITS = 24` - Program counter bit width
- `DEFAULT_PC_STEP = 4` - Standard PC increment
- `RV_J_TYPE_IMM_BITS = 21` - JAL immediate bit width

## Component Overview

This component implements two RISC-V instructions:

1. **JAL (Jump And Link)**
   - Unconditional jump with return address save
   - PC-relative addressing
   - Used for function calls

2. **LUI (Load Upper Immediate)**
   - Loads 20-bit immediate into upper register bits
   - Used for building 32-bit constants
   - Often paired with ADDI

## Key Concepts

### Limb-Based Arithmetic
32-bit values are split into 4 limbs of 8 bits each for efficient field arithmetic.

### Instruction Selection
Uses boolean flags `is_jal` and `is_lui` to select instruction behavior within shared constraints.

### Range Checking
All limbs are verified to be within 8-bit range using bitwise operation lookups.

### PC Handling
Special constraints ensure PC values respect the 24-bit limit.

## Testing Guide

### Running Tests
```bash
cargo test -p openvm-rv32im-circuit jal_lui
```

### Test Categories
- `rand_jal_lui_test` - Randomized positive tests
- `*_negative_test` - Constraint violation tests
- `*_sanity_test` - Known value verification

## Common Tasks

### Adding New Constraints
1. Modify `eval()` in `core.rs`
2. Update trace generation in `generate_trace_row()`
3. Add corresponding tests

### Debugging Failures
1. Check immediate encoding (JAL vs LUI format)
2. Verify PC calculations
3. Ensure proper limb decomposition
4. Check range constraint satisfaction

## Performance Notes

- Combined implementation saves ~50% chip count
- Bitwise lookups are shared across instructions
- Constraint evaluation is optimized for common path

## Security Checklist

- [ ] All limbs range-checked
- [ ] PC bounds enforced
- [ ] Sign extension handled correctly
- [ ] Boolean flags validated
- [ ] No field overflow possible