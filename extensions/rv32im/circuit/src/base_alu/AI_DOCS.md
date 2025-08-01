# BaseAlu Component - AI Documentation

## Component Overview

The BaseAlu component implements basic arithmetic and logical operations (ADD, SUB, XOR, OR, AND) for the RISC-V 32-bit instruction set within the OpenVM framework. It provides a zero-knowledge proof-friendly implementation of these fundamental ALU operations.

## Key Files

- `mod.rs` - Module exports and chip type definitions
- `core.rs` - Core ALU implementation with AIR constraints and execution logic
- `tests.rs` - Comprehensive test suite including positive, negative, and sanity tests

## Architecture

### Core Components

1. **BaseAluCoreChip** - The main execution engine that:
   - Processes ALU instructions (ADD, SUB, XOR, OR, AND)
   - Handles carry propagation for arithmetic operations
   - Interacts with the bitwise operation lookup table for range checking

2. **BaseAluCoreAir** - The AIR (Algebraic Intermediate Representation) that:
   - Defines polynomial constraints for correctness
   - Ensures proper carry behavior for ADD/SUB operations
   - Validates bitwise operations through lookup interactions

3. **Rv32BaseAluChip** - A wrapper that combines:
   - The core ALU chip
   - The RV32 adapter for memory interface
   - VM integration capabilities

### Data Flow

```
Instruction → Adapter → Core Chip → Execution → Memory Write
                ↓          ↓
            Memory Read  Bitwise
                        Lookup
```

## Key Implementation Details

### Limb-Based Arithmetic

The implementation uses a limb-based approach where 32-bit values are split into 4 limbs of 8 bits each:
- Enables efficient carry handling
- Allows for range checking via bitwise lookups
- Maintains compatibility with field arithmetic constraints

### Carry Propagation

For ADD and SUB operations:
- Carry values are computed as: `carry[i] = (operands + previous_carry - result) / 2^LIMB_BITS`
- Each carry is constrained to be boolean (0 or 1)
- This ensures correct modular arithmetic behavior

### Bitwise Operation Integration

- For XOR, OR, AND: Direct lookup table interaction validates results
- For ADD, SUB: Lookup table used for range checking operands
- Ensures all limb values stay within 8-bit bounds

## Constraint System

### Opcode Flags
- Exactly one operation flag must be active
- Flags are boolean constrained
- Maps to proper global opcode values

### Arithmetic Constraints
- ADD: `a[i] = (b[i] + c[i] + carry[i-1]) mod 2^LIMB_BITS`
- SUB: `a[i] = (b[i] - c[i] - borrow[i-1]) mod 2^LIMB_BITS`
- Carry/borrow propagation properly constrained

### Bitwise Constraints
- XOR: `a[i] = b[i] ^ c[i]`
- OR: `a[i] = b[i] | c[i]`
- AND: `a[i] = b[i] & c[i]`
- Validated through lookup table interactions

## Testing Strategy

### Positive Tests
- Random operation generation
- Verification of trace correctness
- Integration with memory and adapter

### Negative Tests
- Invalid carry values
- Out-of-range limb values
- Incorrect operation results
- Adapter constraint violations

### Sanity Tests
- Fixed test vectors for each operation
- Ensures basic correctness of execution functions

## Integration Points

1. **Memory System**: Reads operands and writes results through adapter
2. **Bitwise Lookup**: Validates operations and range checks
3. **Program Bus**: Receives instructions from VM
4. **Execution Bus**: Reports instruction execution

## Performance Considerations

- Degree-3 polynomial constraints for efficiency
- Separated ADD/SUB constraints to maintain low degree
- Efficient limb-wise processing
- Shared bitwise lookup table for all operations

## Security Notes

- All limb values are range-checked to prevent overflow attacks
- Carry values are boolean-constrained to ensure correctness
- Adapter validates memory access patterns
- No timing variations based on operand values