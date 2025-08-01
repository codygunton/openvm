# OpenVM BigInt Circuit Extension

## Overview

The OpenVM BigInt Circuit Extension provides native 256-bit integer arithmetic operations within the zkVM framework. This extension enables efficient computation on large integers that are commonly used in cryptographic applications, blockchain protocols, and scientific computing where 256-bit precision is required.

## Architecture

### Component Structure

The bigint circuit extension follows OpenVM's modular chip architecture:

```
bigint-circuit/
├── src/
│   ├── lib.rs         # Public API and chip type definitions
│   ├── extension.rs   # VM extension implementation and configuration
│   └── tests.rs       # Integration tests
```

### Key Design Principles

1. **Chip-Based Architecture**: Each arithmetic operation is implemented as a dedicated chip that integrates seamlessly with the OpenVM framework
2. **Adapter Pattern**: Uses `Rv32HeapAdapterChip` and `Rv32HeapBranchAdapterChip` to handle memory operations and translate between RISC-V instructions and circuit operations
3. **Core Logic Separation**: Leverages core arithmetic chips from `openvm-rv32im-circuit` with 256-bit configurations
4. **Bus-Based Communication**: Utilizes OpenVM's bus system for inter-chip communication and constraint verification

## Supported Operations

### Arithmetic Operations
- **Addition** (`ADD`): 256-bit unsigned addition with wrapping
- **Subtraction** (`SUB`): 256-bit unsigned subtraction with wrapping
- **Multiplication** (`MUL`): 256-bit unsigned multiplication (lower 256 bits of result)

### Bitwise Operations
- **AND**: Bitwise AND operation
- **OR**: Bitwise OR operation
- **XOR**: Bitwise XOR operation

### Shift Operations
- **SLL**: Logical left shift
- **SRL**: Logical right shift
- **SRA**: Arithmetic right shift (sign-extending)

### Comparison Operations
- **SLT**: Set less than (signed comparison)
- **SLTU**: Set less than unsigned
- **BEQ**: Branch if equal (256-bit comparison)
- **BLT/BLTU**: Branch if less than (signed/unsigned)

## Implementation Details

### Chip Types

The extension defines six main chip types:

1. **`Rv32BaseAlu256Chip<F>`**: Handles basic ALU operations (ADD, SUB, AND, OR, XOR)
2. **`Rv32LessThan256Chip<F>`**: Implements comparison operations (SLT, SLTU)
3. **`Rv32Multiplication256Chip<F>`**: Handles 256-bit multiplication
4. **`Rv32Shift256Chip<F>`**: Implements shift operations
5. **`Rv32BranchEqual256Chip<F>`**: Branch on equality comparison
6. **`Rv32BranchLessThan256Chip<F>`**: Branch on less-than comparison

### Memory Layout

- 256-bit integers are stored as 8 limbs of 32 bits each
- Little-endian representation in memory
- Heap-based addressing for operands and results

### Instruction Encoding

The extension uses custom RISC-V instruction encoding:
- **Opcode**: `0x0b` (custom-0 in RISC-V spec)
- **Funct3**: 
  - `0b101` for arithmetic/logic operations
  - `0b110` for branch operations
- **Funct7**: Distinguishes between specific operations

### Opcode Offsets

Each operation class has a dedicated opcode offset:
- Base ALU: `0x400`
- Shift: `0x405`
- Less Than: `0x408`
- Branch Equal: `0x420`
- Branch Less Than: `0x425`
- Multiplication: `0x450`

## Integration with OpenVM

### VM Extension Configuration

The `Int256` extension integrates into the VM through:

```rust
pub struct Int256Rv32Config {
    pub system: SystemConfig,
    pub rv32i: Rv32I,
    pub rv32m: Rv32M,
    pub io: Rv32Io,
    pub bigint: Int256,
}
```

### Dependency Management

The extension requires:
- **Bitwise Operation Lookup**: Shared chip for efficient bitwise operations
- **Range Tuple Checker**: For multiplication overflow checking
- **Memory Bridge**: For heap access coordination
- **System Buses**: Execution and program buses for instruction flow

### Peripheral Chips

Two main peripheral chips support the arithmetic operations:
1. **`SharedBitwiseOperationLookupChip<8>`**: Provides lookup tables for 8-bit bitwise operations
2. **`SharedRangeTupleCheckerChip<2>`**: Validates range constraints for multiplication

## Performance Considerations

### Optimization Strategies

1. **Lookup Tables**: Bitwise operations use precomputed lookup tables for efficiency
2. **Shared Resources**: Multiple chips share lookup tables and range checkers to minimize circuit size
3. **Configurable Sizes**: Range checker sizes can be tuned based on workload

### Default Configuration

```rust
range_tuple_checker_sizes: [1 << 8, 32 * (1 << 8)]
```

This provides efficient support for typical 256-bit operations while maintaining reasonable proof generation times.

## Usage in Guest Programs

Guest programs access 256-bit operations through inline assembly or external functions:

```rust
// Example: 256-bit addition
unsafe extern "C" fn zkvm_u256_wrapping_add_impl(
    result: *mut u8,
    a: *const u8,
    b: *const u8
) {
    // Custom instruction execution
}
```

## Testing

The extension includes comprehensive tests covering:
- Random operation testing with property verification
- Edge cases (overflow, underflow, zero operands)
- Branch target calculation correctness
- Integration with the broader VM system

## Security Considerations

1. **Constant-Time Operations**: All operations execute in constant time relative to input values
2. **No Secret-Dependent Branching**: Branch operations reveal comparison results but not intermediate values
3. **Memory Safety**: Heap adapters ensure proper bounds checking

## Future Enhancements

Potential areas for extension:
- Support for signed 256-bit integers (int256)
- Division and modulo operations
- Extended precision arithmetic (512-bit, 1024-bit)
- Optimized square and exponentiation operations