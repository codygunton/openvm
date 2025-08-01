# DivRem Component - AI Documentation

## Overview

The DivRem component implements integer division and remainder operations for the RISC-V 32-bit integer instruction set (RV32IM). It handles both signed and unsigned division/remainder operations while maintaining correct behavior for edge cases like division by zero and signed overflow.

## Architecture

### Core Components

1. **DivRemCoreChip**: The main chip that implements the division and remainder operations
   - Handles the core arithmetic logic: `b = c * q + r` where `0 <= |r| < |c|`
   - Manages special cases (zero divisor, signed overflow)
   - Enforces range constraints and sign correctness

2. **Rv32DivRemChip**: A wrapper that adapts the core chip for RV32IM usage
   - Type alias combining VmChipWrapper with Rv32MultAdapterChip and DivRemCoreChip
   - Integrates with the VM architecture

### Supported Operations

The component implements four RISC-V division/remainder opcodes:
- `DIV`: Signed division
- `DIVU`: Unsigned division  
- `REM`: Signed remainder
- `REMU`: Unsigned remainder

## Key Implementation Details

### Algorithm

The division algorithm follows the standard mathematical definition:
- For division: `b ÷ c = q` with remainder `r`
- Constraint: `b = c * q + r` where `0 <= |r| < |c|`
- Sign rules:
  - `sign(q) = sign(b) XOR sign(c)` (when q ≠ 0)
  - `sign(r) = sign(b)` (when r ≠ 0)

### Special Cases

1. **Zero Divisor**: When divisor is 0
   - Quotient set to all 1s (0xFFFFFFFF for RV32)
   - Remainder equals dividend

2. **Signed Overflow**: When dividing minimum signed value by -1
   - Only occurs for: `-2^31 ÷ -1` in RV32
   - Quotient equals dividend
   - Remainder is 0

### Constraint System

The component uses several auxiliary columns to enforce correctness:
- `zero_divisor`: Flag for division by zero
- `r_zero`: Flag for zero remainder
- `b_sign`, `c_sign`, `q_sign`: Sign tracking
- `r_prime`: Absolute value of remainder for comparison
- `lt_marker`, `lt_diff`: For enforcing `|r| < |c|`

### External Dependencies

- **BitwiseOperationLookupChip**: For sign checking operations
- **RangeTupleCheckerChip**: For range checking quotient and remainder values
- Uses limb-based representation with configurable `NUM_LIMBS` and `LIMB_BITS`

## Security Considerations

- All edge cases are explicitly handled to prevent undefined behavior
- Range checks ensure values stay within valid bounds
- Sign handling prevents integer overflow vulnerabilities
- Extensive test coverage for both positive and negative cases

## Testing

The component includes comprehensive tests in `tests.rs`:
- Random operation tests
- Special case coverage (zero divisor, overflow, etc.)
- Negative tests to verify constraint violations are caught
- Sanity tests for algorithm correctness

## Integration Notes

- Integrates with the RV32IM extension system via opcode offset 0x254
- Uses the standard VM adapter interface for reads/writes
- Supports both 32-bit and potentially larger integer sizes through parameterization