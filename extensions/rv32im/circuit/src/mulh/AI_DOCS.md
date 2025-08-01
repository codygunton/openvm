# MulH Component Documentation

## Overview

The MulH component implements high-multiplication operations for RISC-V 32-bit integers, computing the upper 32 bits of 64-bit multiplication results. This is essential for multi-precision arithmetic and overflow detection in 32-bit systems.

## Purpose

When multiplying two 32-bit numbers, the result can be up to 64 bits. Standard multiplication instructions only return the lower 32 bits. The MulH family of instructions returns the upper 32 bits, enabling:
- Full 64-bit multiplication results when combined with MUL
- Overflow detection for multiplication operations
- Multi-precision arithmetic implementations

## Supported Operations

### MULH (Multiply High Signed)
- Treats both operands as signed integers
- Returns upper 32 bits of signed × signed multiplication
- Sign extends both operands to 64 bits before multiplication

### MULHSU (Multiply High Signed-Unsigned)
- First operand (rs1) treated as signed
- Second operand (rs2) treated as unsigned
- Returns upper 32 bits of signed × unsigned multiplication

### MULHU (Multiply High Unsigned)
- Treats both operands as unsigned integers
- Returns upper 32 bits of unsigned × unsigned multiplication
- Zero extends both operands to 64 bits before multiplication

## Implementation Architecture

### Limb-Based Computation
The implementation uses a limb-based approach where 32-bit values are split into multiple smaller chunks (limbs) for efficient constraint evaluation:
- Each 32-bit value is divided into `NUM_LIMBS` pieces
- Each limb contains `LIMB_BITS` bits
- Typically uses 4 limbs of 8 bits each for RV32

### Constraint System
The AIR (Algebraic Intermediate Representation) enforces:
1. Correct multiplication with carry propagation
2. Proper sign/zero extension based on opcode
3. Range checks on intermediate values
4. Bitwise checks for sign bit extraction

### Key Components

#### MulHCoreAir
- Defines the constraint system for proving correct multiplication
- Handles carry propagation between limbs
- Verifies sign extensions using bitwise lookups

#### MulHCoreChip
- Executes the multiplication operation
- Generates execution traces for proving
- Manages interactions with lookup tables

#### Column Layout
The trace uses these columns:
- Input operands (b, c)
- Output result (a - upper bits)
- Lower multiplication bits (a_mul)
- Extension values for sign handling (b_ext, c_ext)
- Opcode flags for instruction type

## Algorithm Details

### Multiplication Process
1. Compute full multiplication: `b × c = (a << LIMB_BITS) + a_mul`
2. Track carries through limb-wise multiplication
3. Apply sign/zero extension based on opcode
4. Extract upper bits as final result

### Sign Extension
- MULH: Sign extend both operands
- MULHSU: Sign extend first operand only
- MULHU: No sign extension (zero extend)

### Carry Propagation
The implementation carefully tracks carries through the multiplication:
- First computes carries for lower bits (a_mul)
- Then propagates carries to upper bits (a)
- Uses range checking to ensure carry values are valid

## Integration with OpenVM

### Memory Interface
- Reads two 32-bit operands from registers
- Writes one 32-bit result to destination register
- Uses standard RV32 register addressing

### Shared Resources
- Bitwise operation lookups for sign bit extraction
- Range tuple checker for carry validation
- Common with other arithmetic operations

### Performance Considerations
- Optimized for constraint count in zkVM
- Shares lookup tables with other components
- Efficient limb-based representation

## Testing Strategy

### Positive Tests
- Random input generation
- Verification against expected results
- Coverage of all three opcodes

### Negative Tests
- Invalid multiplication results
- Incorrect sign extensions
- Invalid carry values
- Tests constraint system robustness

### Edge Cases
- Maximum positive/negative values
- Sign bit boundaries
- Zero operands
- Overflow scenarios