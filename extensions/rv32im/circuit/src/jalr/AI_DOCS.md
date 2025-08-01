# JALR (Jump And Link Register) Circuit Implementation

## Overview

The JALR (Jump And Link Register) circuit implements the RISC-V JALR instruction within the OpenVM zkVM framework. This instruction performs an indirect jump to an address computed from a register value plus an immediate offset, while saving the return address.

## Architecture

### Core Components

1. **Rv32JalrCoreChip** (`core.rs`)
   - Main computational logic for JALR instruction
   - Handles address calculation with proper masking
   - Manages program counter updates
   - Enforces constraints on address alignment

2. **Rv32JalrAdapterChip** (`adapters/jalr.rs`)
   - Memory interface adapter
   - Manages register reads (rs1) and writes (rd)
   - Handles execution state transitions
   - Coordinates with memory controller

3. **Rv32JalrChip** (`mod.rs`)
   - Type alias combining core and adapter chips
   - `VmChipWrapper<F, Rv32JalrAdapterChip<F>, Rv32JalrCoreChip>`

### Instruction Format

JALR uses the I-type instruction format:
- **opcode**: JALR (offset 0x23F)
- **rd**: Destination register for return address
- **rs1**: Source register containing base address
- **imm**: 12-bit signed immediate offset (sign-extended to 32 bits)

### Execution Semantics

```
t = rs1 + sign_extend(imm)
pc = t & ~1  // Clear least significant bit
rd = pc + 4  // Save return address
```

## Key Implementation Details

### Address Calculation

The JALR instruction computes the target address by:
1. Adding the rs1 register value to the sign-extended immediate
2. Clearing the least significant bit (LSB) to ensure 2-byte alignment
3. Validating the result is within the valid PC range (PC_BITS)

### Limb Decomposition

The implementation uses 8-bit limbs for 32-bit values:
- **RV32_REGISTER_NUM_LIMBS = 4**: Each 32-bit register is stored as 4 bytes
- **RV32_CELL_BITS = 8**: Each limb represents 8 bits
- **PC decomposition**: The return address (pc + 4) is decomposed into limbs for storage

### Optimization: Reduced Column Usage

To save a column, the implementation only stores 3 most significant limbs of `rd_data` in the trace. The least significant limb is derived using:
```rust
least_sig_limb = from_pc + DEFAULT_PC_STEP - composed
```
Where `composed` is the composition of the 3 stored limbs.

### Constraint System

1. **Address Calculation Constraints**:
   - Validates 32-bit addition with carry propagation
   - Ensures LSB is cleared (stored separately as `to_pc_least_sig_bit`)
   - Prevents overflow beyond PC_BITS range

2. **Range Checks**:
   - `rd_data` limbs are range-checked based on position
   - `to_pc_limbs` are checked to prevent PC overflow
   - Uses bitwise lookup for efficient 8-bit range checks

3. **Memory Consistency**:
   - Adapter ensures proper read from rs1
   - Conditional write to rd (skipped if rd = x0)
   - Maintains correct timestamp ordering

## Constraint Details

### Core Air Constraints

1. **Boolean Constraints**:
   ```rust
   builder.assert_bool(is_valid)
   builder.assert_bool(imm_sign)
   builder.assert_bool(to_pc_least_sig_bit)
   ```

2. **Address Calculation** (32-bit addition with 2 limbs):
   ```rust
   // Lower 16 bits
   carry = (rs1_limbs_01 + imm - to_pc_limbs[0] * 2 - to_pc_least_sig_bit) / 2^16
   
   // Upper 16 bits with sign extension
   carry = (rs1_limbs_23 + imm_extend_limb + carry - to_pc_limbs[1]) / 2^16
   ```

3. **Range Constraints**:
   - `rd_data[0..1]`: Via bitwise lookup bus
   - `rd_data[2]`: 8-bit range check
   - `rd_data[3]`: (PC_BITS - 24)-bit range check
   - `to_pc_limbs[0]`: 15-bit range check
   - `to_pc_limbs[1]`: (PC_BITS - 16)-bit range check

## Testing

The module includes comprehensive tests in `tests.rs`:

1. **Positive Tests** (`rand_jalr_test`):
   - Random instruction generation and execution
   - Verifies trace passes all constraints
   - 100 test iterations with random inputs

2. **Negative Tests** (`invalid_cols_negative_tests`, `overflow_negative_tests`):
   - Tests constraint violations
   - Validates error detection for:
     - Invalid column values
     - Incorrect sign extension
     - Address overflow conditions
     - Mismatched carry bits

3. **Sanity Tests** (`run_jalr_sanity_test`):
   - Known input/output validation
   - Example: `jalr(pc=789456120, imm=-1235, rs1=736482910)` → `pc=736481674, rd=[252,36,14,47]`

## Integration Points

1. **Memory System**:
   - Uses `MemoryController` for register access
   - Coordinates with `OfflineMemory` for trace generation
   - Respects RISC-V register addressing space (RV32_REGISTER_AS)

2. **Execution Bus**:
   - Reports instruction execution to program bus
   - Updates execution state (PC and timestamp)
   - Maintains instruction ordering

3. **Auxiliary Chips**:
   - `SharedBitwiseOperationLookupChip`: For efficient range checks
   - `SharedVariableRangeCheckerChip`: For larger range validations

## Performance Considerations

1. **Column Optimization**: Saves one field element per row by deriving LSB of rd_data
2. **Batched Range Checks**: Uses shared lookup tables for efficiency
3. **Minimal Carry Propagation**: Only 2-limb addition instead of full 4-limb

## Security Notes

1. **Address Alignment**: LSB clearing prevents misaligned jumps
2. **Overflow Protection**: Range checks prevent jumps outside valid PC space
3. **Register Safety**: x0 writes are properly handled (no-op)