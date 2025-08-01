# Branch Less Than Component Index

## Core Types
- `Rv32BranchLessThanChip<F>` - Main chip type for RV32 branch less-than operations
- `BranchLessThanCoreChip<NUM_LIMBS, LIMB_BITS>` - Generic core implementation
- `BranchLessThanCoreAir<NUM_LIMBS, LIMB_BITS>` - AIR constraints definition
- `BranchLessThanCoreCols<T, NUM_LIMBS, LIMB_BITS>` - Trace column structure
- `BranchLessThanCoreRecord<T, NUM_LIMBS, LIMB_BITS>` - Execution record

## Key Functions
- `run_cmp()` - Core comparison logic for signed/unsigned operations
- `execute_instruction()` - Instruction execution implementation
- `eval()` - AIR constraint evaluation
- `generate_trace_row()` - Trace generation from execution record

## Opcodes
- `BranchLessThanOpcode::BLT` - Branch if less than (signed)
- `BranchLessThanOpcode::BLTU` - Branch if less than (unsigned)
- `BranchLessThanOpcode::BGE` - Branch if greater or equal (signed)
- `BranchLessThanOpcode::BGEU` - Branch if greater or equal (unsigned)

## Configuration Constants
- `RV32_REGISTER_NUM_LIMBS = 8` - Number of limbs for 32-bit values
- `RV32_CELL_BITS = 8` - Bits per limb
- `DEFAULT_PC_STEP = 4` - Default PC increment

## Integration Points
- Bitwise operation lookup bus for range checking
- RV32 branch adapter for instruction format
- VmCoreChip trait implementation
- VmCoreAir trait implementation

## Test Utilities
- Random test generation for all opcodes
- Constraint satisfaction verification
- Edge case validation
- Performance benchmarking support