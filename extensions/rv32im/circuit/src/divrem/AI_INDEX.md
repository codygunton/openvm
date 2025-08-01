# DivRem Component - File Index

## Component Structure

```
divrem/
├── mod.rs      # Module definition and public exports
├── core.rs     # Core implementation of division/remainder operations
└── tests.rs    # Comprehensive test suite
```

## File Descriptions

### mod.rs (16 lines)
**Purpose**: Module entry point and type definitions

**Key Elements**:
- Imports required dependencies from the VM architecture
- Re-exports all items from `core.rs`
- Defines `Rv32DivRemChip<F>` type alias that wraps the core chip with VM adapter

**Dependencies**:
- `openvm_circuit::arch::VmChipWrapper`
- `super::adapters::{Rv32MultAdapterChip, RV32_CELL_BITS, RV32_REGISTER_NUM_LIMBS}`

### core.rs (724 lines)
**Purpose**: Core implementation of division and remainder operations

**Key Structures**:
- `DivRemCoreCols<T, NUM_LIMBS, LIMB_BITS>`: Column layout for constraint system
  - Division components: `b`, `c`, `q`, `r` (dividend, divisor, quotient, remainder)
  - Special case flags: `zero_divisor`, `r_zero`
  - Sign tracking: `b_sign`, `c_sign`, `q_sign`, `sign_xor`
  - Auxiliary columns for constraints
  
- `DivRemCoreAir<NUM_LIMBS, LIMB_BITS>`: AIR (Algebraic Intermediate Representation) implementation
  - Defines constraint evaluation logic
  - Manages interaction with lookup chips

- `DivRemCoreChip<NUM_LIMBS, LIMB_BITS>`: Main chip implementation
  - Executes division/remainder instructions
  - Generates execution traces
  - Integrates with bitwise and range checking chips

**Key Functions**:
- `run_divrem()`: Core algorithm for division/remainder calculation
- `run_sltu_diff_idx()`: Helper for unsigned less-than comparison
- `run_mul_carries()`: Computes carry values for multiplication verification
- `limbs_to_biguint()` / `biguint_to_limbs()`: Conversion utilities
- `negate()`: Two's complement negation for signed operations

**Opcodes Handled**:
- `DivRemOpcode::DIV` - Signed division
- `DivRemOpcode::DIVU` - Unsigned division
- `DivRemOpcode::REM` - Signed remainder
- `DivRemOpcode::REMU` - Unsigned remainder

### tests.rs (718 lines)
**Purpose**: Comprehensive test coverage for the divrem component

**Test Categories**:

1. **Positive Tests** (Lines 44-209):
   - `run_rv32_divrem_rand_test()`: Random operation testing
   - Individual tests for DIV, DIVU, REM, REMU opcodes
   - Special case coverage (zero divisor, signed overflow, zero remainder)

2. **Negative Tests** (Lines 211-549):
   - Tests that verify constraint violations are properly caught
   - Wrong quotient/remainder values
   - Incorrect flag settings
   - Sign handling errors
   - Uses `DivRemPrankValues` to inject faulty values

3. **Sanity Tests** (Lines 551-717):
   - Direct algorithm verification
   - Tests for `run_divrem()`, `run_sltu_diff_idx()`, `run_mul_carries()`
   - Edge case validation (zero divisor, signed overflow, minimum dividend)

**Test Infrastructure**:
- Uses `VmChipTestBuilder` for chip testing
- Integrates with bitwise and range checker chips
- Verifies both execution correctness and constraint satisfaction

## Key Design Patterns

1. **Limb-based Arithmetic**: Operations work on arrays of limbs for potential big integer support
2. **Constraint-based Verification**: Uses algebraic constraints to ensure correctness
3. **Special Case Handling**: Explicit handling of edge cases with dedicated flags
4. **Sign Management**: Careful tracking and verification of signs for signed operations
5. **Modular Architecture**: Core logic separated from VM integration layer

## Integration Points

- Integrates with RV32IM instruction set at opcode offset 0x254
- Uses standard VM adapter interfaces for memory operations
- Leverages shared lookup chips for auxiliary operations
- Follows OpenVM chip architecture patterns