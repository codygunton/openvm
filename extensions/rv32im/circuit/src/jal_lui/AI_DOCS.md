# JAL/LUI Component - AI Documentation

## Overview

The JAL/LUI component implements two fundamental RISC-V instructions in the OpenVM framework:
- **JAL (Jump And Link)**: Unconditional jump with return address storage
- **LUI (Load Upper Immediate)**: Load a 20-bit immediate into the upper bits of a register

### Purpose
- JAL enables function calls and unconditional branches with PC-relative addressing
- LUI constructs 32-bit constants and addresses (often paired with ADDI)
- Both instructions are essential for control flow and address generation
- Core components of the RV32I base instruction set

### Key Features
- Shared implementation for efficient resource usage
- PC-relative jump addressing with proper wraparound
- 20-bit immediate value handling
- Bitwise operation lookups for constraint verification
- Zero-knowledge proof generation for both instructions

## Architecture

### Module Structure
```
jal_lui/
├── mod.rs          # Module exports and chip type definition
├── core.rs         # Core chip implementation with AIR constraints
└── tests.rs        # Comprehensive test suite
```

### Core Components

#### 1. **Rv32JalLuiChip**
- Type alias wrapping the core chip with conditional write adapter
- Manages register writes based on destination register
- Defined as: `VmChipWrapper<F, Rv32CondRdWriteAdapterChip<F>, Rv32JalLuiCoreChip>`

#### 2. **Rv32JalLuiCoreChip**
- Implements both JAL and LUI instruction logic
- Manages bitwise operation lookups for range checking
- Processes instructions and generates execution records

#### 3. **Rv32JalLuiCoreCols**
- Column layout for the arithmetic trace
- Contains:
  - `imm`: Immediate value (different interpretation for JAL vs LUI)
  - `rd_data`: Destination register data in 4 limbs
  - `is_jal`: Boolean flag indicating JAL instruction
  - `is_lui`: Boolean flag indicating LUI instruction

### Data Representation

Values use limb-based representation with 8-bit limbs:
- 32-bit values split into 4 limbs (`RV32_REGISTER_NUM_LIMBS = 4`)
- Each limb holds values 0-255 (`RV32_CELL_BITS = 8`)
- Little-endian ordering (LSB in limb 0)
- Special PC handling for 24-bit program counter

## Implementation Details

### Instruction Execution Flow

#### JAL (Jump And Link)
1. **Save Return Address**
   - Store PC + 4 into destination register
   - Split into limbs for field representation

2. **Calculate Jump Target**
   - Add sign-extended immediate to PC
   - Handle wraparound for valid addresses

3. **Update PC**
   - Set next PC to calculated jump target

#### LUI (Load Upper Immediate)
1. **Shift Immediate**
   - Left shift immediate by 12 bits
   - Lower 12 bits become zero

2. **Store in Register**
   - Write shifted value to destination register
   - Split into limbs for field representation

3. **Increment PC**
   - Simple PC + 4 advancement

### Key Algorithms

#### JAL Return Address Calculation
```rust
// Store PC + 4 in destination register
let rd_data = array::from_fn(|i| {
    ((pc + DEFAULT_PC_STEP) >> (8 * i)) & ((1 << RV32_CELL_BITS) - 1)
});
```

#### LUI Immediate Processing
```rust
// Shift immediate left by 12 bits
let rd = (imm as u32) << 12;
let rd_data = array::from_fn(|i| {
    (rd >> (RV32_CELL_BITS * i)) & ((1 << RV32_CELL_BITS) - 1)
});
```

### Constraint System

The AIR enforces several critical constraints:

1. **Instruction Validity**
   - Exactly one of `is_jal` or `is_lui` must be true
   - Both flags must be boolean

2. **JAL Constraints**
   - Return address correctly computed from PC + 4
   - Last limb respects PC_BITS limit (typically 24 bits)
   - Proper range checking via bitwise lookups

3. **LUI Constraints**
   - First limb (LSB) must be zero (shift by 12 bits)
   - Upper limbs correctly represent immediate << 12
   - All limbs within valid ranges

4. **PC Calculation**
   - JAL: `next_pc = pc + immediate`
   - LUI: `next_pc = pc + 4`

## Usage Patterns

### Basic Usage
```rust
// Instantiate as part of RV32IM extension
let jal_lui_chip = Rv32JalLuiChip::new(
    adapter,
    Rv32JalLuiCoreChip::new(bitwise_lookup_chip),
    offline_memory
);
```

### Common Patterns

#### Function Call (JAL)
```assembly
jal ra, function_label  # Jump to function, save return address in ra
```

#### 32-bit Constant Loading
```assembly
lui t0, %hi(0x12345678)    # Load upper 20 bits
addi t0, t0, %lo(0x12345678) # Add lower 12 bits
```

### Integration Points
- Registers with VM executor for JAL and LUI opcodes
- Interacts with bitwise operation lookup tables
- Conditional register writes through adapter
- PC management with VM execution context

## Testing Strategy

### Test Categories

1. **Randomized Tests** (`rand_jal_lui_test`)
   - 100+ random valid operations
   - Tests both JAL and LUI instructions
   - Verifies trace passes all constraints

2. **Negative Tests**
   - Invalid opcode flags
   - Overflow conditions
   - Malformed immediate values
   - Verifies proper constraint violations

3. **Sanity Tests**
   - Known input/output pairs
   - Edge cases for immediates and PC values
   - Roundtrip execution verification

### Key Test Cases
- JAL with maximum positive/negative offsets
- LUI with all bits set in upper immediate
- PC wraparound behavior
- Register 0 handling (no-write cases)
- Boundary conditions for PC_BITS limit

## Performance Considerations

### Optimizations
- Shared implementation reduces code duplication
- Efficient limb-based arithmetic
- Batch range checking for limb pairs
- Minimal constraint overhead

### Trade-offs
- Combined instruction handling adds complexity
- Additional boolean flags for instruction selection
- Range checking overhead for security

## Security Considerations

### Proof Security
- All limbs are range-checked
- PC bounds enforced (24-bit limit)
- No field element overflow possible
- Sign extension handled correctly

### Implementation Security
- Explicit handling of signed immediates
- Proper wraparound arithmetic
- Comprehensive constraint coverage
- Defense against malicious traces

## Common Pitfalls

1. **Immediate Encoding**
   - JAL uses J-type format (20 bits, bit 0 implicit)
   - LUI uses U-type format (upper 20 bits)
   - Sign extension differs between instructions

2. **PC Limits**
   - PC limited to PC_BITS (24), not full 32 bits
   - Special handling for last limb in JAL

3. **Register 0**
   - Writes to x0 handled by adapter
   - Still must compute correct values

4. **Immediate Alignment**
   - JAL immediate must be even (bit 0 implicit)
   - LUI shifts by exactly 12 bits

## Related Components

- **AUIPC**: Similar immediate handling, PC-relative
- **JALR**: Register-based jump with link
- **Branch Instructions**: Conditional PC-relative jumps
- **RV32 Conditional Write Adapter**: Handles x0 special case
- **Bitwise Operation Lookup**: Range checking support

## Implementation Notes

### Why Combined Implementation?
- Both instructions share similar structure
- Efficient use of constraint system
- Reduced code duplication
- Common register write patterns

### Special Cases
- JAL to self creates infinite loop
- LUI with zero immediate clears register
- Maximum jump ranges: ±1MB for JAL

## References

- RISC-V Specification: Chapter 2.5 (JAL) and 2.4 (LUI)
- OpenVM Architecture: VmCoreChip trait documentation
- Circuit Primitives: Bitwise operation lookup documentation
- RV32IM Transpiler: Opcode definitions and encoding