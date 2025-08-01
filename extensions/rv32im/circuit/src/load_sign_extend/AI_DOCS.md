# Load Sign Extend Component - Detailed Documentation

## Purpose
The Load Sign Extend component implements RISC-V sign-extending load instructions (LOADB and LOADH) within the OpenVM zkVM framework. These instructions load 8-bit (byte) or 16-bit (halfword) values from memory and sign-extend them to 32-bit words, preserving the sign bit.

## Architecture Overview

### Component Structure
The component follows OpenVM's standard chip architecture pattern:
1. **Core Chip**: Implements the sign extension logic and constraints
2. **Adapter Integration**: Uses `Rv32LoadStoreAdapterChip` for memory operations
3. **Wrapper Type**: `Rv32LoadSignExtendChip` combines adapter and core functionality

### Key Design Decisions

#### Shifted Read Data
The component uses a "shifted read data" approach to handle unaligned memory access:
- Read data is pre-shifted by `(shift_amount & 2)` 
- This reduces the number of opcode flags needed
- Allows generating write_data as if shift_amount was 0 or 1

#### Opcode Flag Separation
LOADB instructions are split into two flags based on shift amount:
- `opcode_loadb_flag0`: LOADB with shift 0
- `opcode_loadb_flag1`: LOADB with shift 1
- This simplifies constraint logic for different alignment cases

## Core Implementation Details

### LoadSignExtendCoreAir

#### Constraint System
The AIR (Algebraic Intermediate Representation) enforces:

1. **Boolean Constraints**:
   - All opcode flags must be boolean
   - Sign bit indicators must be boolean
   - Exactly one opcode flag must be set

2. **Sign Bit Extraction**:
   - For LOADB: Sign bit from shifted_read_data[0] or [1]
   - For LOADH: Sign bit from shifted_read_data[NUM_CELLS/2-1]
   - Range check ensures correct bit extraction

3. **Write Data Generation**:
   ```
   - First limb: Original data (LOADH/LOADB0) or shifted (LOADB1)
   - Middle limbs: Original for LOADH, sign-extended for LOADB
   - Upper limbs: Always sign-extended
   ```

#### Memory Layout
32-bit words are stored as 4 limbs of 8 bits each:
- `NUM_CELLS = 4` (RV32_REGISTER_NUM_LIMBS)
- `LIMB_BITS = 8` (RV32_CELL_BITS)

### LoadSignExtendCoreChip

#### Execution Flow
1. **Instruction Decode**: Extract opcode and shift amount
2. **Data Shifting**: Apply read_shift = shift_amount & 2
3. **Sign Extension**: Call `run_write_data_sign_extend`
4. **Range Check**: Validate sign bit extraction
5. **Record Generation**: Create trace record with all intermediate values

#### Sign Extension Logic
```rust
// For LOADH (16-bit to 32-bit):
- Extract sign bit from bit 15
- Extend to upper 16 bits if sign bit is 1

// For LOADB (8-bit to 32-bit):  
- Extract sign bit from bit 7
- Extend to upper 24 bits if sign bit is 1
```

## Memory Access Patterns

### Alignment Requirements
- Memory addresses must be aligned to data size
- `shift = ptr_val % 4` where `ptr_val - shift` is 4-byte aligned
- This ensures `ptr_val - shift + 4 <= 2^pointer_max_bits`

### Supported Shift Values
- **LOADB**: 0, 1, 2, 3 (any byte within word)
- **LOADH**: 0, 2 (aligned halfwords only)

## Integration with OpenVM

### Adapter Interface
The component implements `VmCoreChip` trait with:
- **Reads**: Previous data and current read data
- **Writes**: Sign-extended 32-bit result
- **Instruction**: LoadStoreInstruction with shift information

### Bus Interactions
- Uses `VariableRangeCheckerBus` for sign bit validation
- Integrates with memory controller via adapter
- Maintains execution trace for proof generation

## Performance Considerations

### Optimization Strategies
1. **Pre-shifted data**: Reduces constraint complexity
2. **Separated opcode flags**: Enables efficient multiplexing
3. **Range checking**: Only on sign bit extraction

### Trace Generation
- Efficient array operations using `array::from_fn`
- Minimal field operations in hot path
- Pre-computed shift masks

## Testing Strategy

### Test Categories
1. **Randomized Tests**: 100+ random operations per opcode
2. **Negative Tests**: Invalid traces with expected errors
3. **Sanity Tests**: Known input/output pairs
4. **Edge Cases**: Boundary values and alignment

### Test Utilities
- `set_and_execute`: Configurable test execution
- `run_negative_loadstore_test`: Constraint violation testing
- Direct function testing for `run_write_data_sign_extend`

## Common Patterns

### Error Handling
- Unaligned access triggers unreachable panic
- Invalid opcodes handled by adapter layer
- Constraint violations caught during proof generation

### Debugging Support
- Comprehensive trace records
- Human-readable opcode names via `get_opcode_name`
- Detailed test assertions

## Security Considerations

### Soundness
- All sign extensions are deterministic
- Range checks prevent malicious sign bit claims
- Shift amounts are validated by adapter

### Memory Safety
- No out-of-bounds array access
- Shift operations use modulo arithmetic
- All array indices compile-time validated