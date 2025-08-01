# Branch Less Than Component AI Documentation

## Overview
The branch_lt component implements RISC-V branching instructions that compare two register values for less-than relationships. It supports both signed and unsigned comparisons for branch-if-less-than (BLT/BLTU) and branch-if-greater-or-equal (BGE/BGEU) operations.

## Key Files
- `mod.rs`: Module entry point, defines the main chip type alias
- `core.rs`: Core implementation with AIR constraints and execution logic
- `tests.rs`: Comprehensive test suite

## Architecture

### Main Components

1. **Rv32BranchLessThanChip<F>**
   - Type alias wrapping the core chip with RV32 adapter
   - Uses 8 limbs of 8 bits each for 32-bit register values

2. **BranchLessThanCoreChip**
   - Core chip implementing branch comparison logic
   - Integrates with bitwise operation lookup for range checking
   - Handles signed/unsigned comparisons

3. **BranchLessThanCoreAir**
   - AIR (Algebraic Intermediate Representation) defining constraints
   - Validates comparison operations
   - Ensures correct branching behavior

### Column Structure (BranchLessThanCoreCols)
- `a`, `b`: Input register values (as limb arrays)
- `cmp_result`: Boolean result of comparison
- `imm`: Immediate value for branch offset
- `opcode_*_flag`: Flags for each opcode type (BLT, BLTU, BGE, BGEU)
- `a_msb_f`, `b_msb_f`: Most significant bytes as field elements
- `cmp_lt`: Whether a < b
- `diff_marker`: Marks position of first difference
- `diff_val`: Value of difference at marked position

## Supported Operations

### Opcodes
1. **BLT** (Branch Less Than - signed)
   - Branches if rs1 < rs2 (signed comparison)
   
2. **BLTU** (Branch Less Than Unsigned)
   - Branches if rs1 < rs2 (unsigned comparison)
   
3. **BGE** (Branch Greater or Equal - signed)
   - Branches if rs1 >= rs2 (signed comparison)
   
4. **BGEU** (Branch Greater or Equal Unsigned)
   - Branches if rs1 >= rs2 (unsigned comparison)

## Implementation Details

### Comparison Algorithm
1. Compares register values limb by limb from MSB to LSB
2. Tracks first position where values differ
3. Handles signed vs unsigned by interpreting MSB differently
4. For signed comparisons, MSB determines sign (two's complement)

### Constraint System
- Validates exactly one opcode flag is set
- Ensures comparison result matches opcode semantics
- Range checks MSB values for signed/unsigned bounds
- Verifies difference marker consistency
- Constrains PC update based on comparison result

### Execution Flow
1. Decode instruction opcode and operands
2. Perform comparison based on opcode type
3. Calculate branch target PC if condition met
4. Generate trace record with all intermediate values
5. Request range checks via bitwise lookup chip

## Integration Points

### Dependencies
- `BitwiseOperationLookupChip`: For range checking operations
- `Rv32BranchAdapterChip`: Adapter for RV32 instruction format
- OpenVM instruction execution framework

### Bus Communication
- Sends range check requests to bitwise operation bus
- Validates MSB values are in correct range for signed/unsigned

## Testing Strategy
- Comprehensive random testing of all opcodes
- Edge case testing for boundary values
- Constraint verification tests
- Negative tests for invalid operations

## Performance Considerations
- Efficient limb-wise comparison
- Minimal constraint overhead
- Shared bitwise lookup for range checks
- Optimized for 32-bit RISC-V semantics