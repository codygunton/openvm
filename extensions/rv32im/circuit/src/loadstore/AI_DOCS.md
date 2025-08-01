# LoadStore Component AI Documentation

## Overview

The LoadStore component in OpenVM's RV32IM extension handles RISC-V memory load and store operations. It manages byte, halfword, and word-aligned memory accesses with appropriate sign/zero extensions and data alignment. The component consists of two main chips working in tandem: the core chip that handles data transformations and the adapter chip that interfaces with the VM's memory system.

## Architecture

### Component Structure

1. **LoadStoreCoreChip**: Handles the core logic for load/store operations
   - Manages byte/halfword to word conversions
   - Handles unsigned extends and sign extensions
   - Processes shifting for unaligned accesses (within 4-byte boundaries)
   - Treats each (opcode, shift) pair as a separate instruction

2. **Rv32LoadStoreAdapterChip**: Interfaces between the core chip and VM memory system
   - Manages memory and register operations
   - Calculates memory addresses from rs1 + immediate
   - Handles 4-byte aligned batch reads/writes
   - Routes data between registers and memory

### Supported Operations

| Opcode | Operation | Description |
|--------|-----------|-------------|
| LOADW  | Load Word | Loads 4-byte word from memory |
| LOADHU | Load Halfword Unsigned | Loads 2-byte halfword with zero extension |
| LOADBU | Load Byte Unsigned | Loads 1-byte with zero extension |
| STOREW | Store Word | Stores 4-byte word to memory |
| STOREH | Store Halfword | Stores 2-byte halfword to memory |
| STOREB | Store Byte | Stores 1-byte to memory |

## Key Concepts

### Memory Alignment

The adapter always performs 4-byte aligned memory accesses. For sub-word operations:
- Calculates shift amount: `shift = ptr_val % 4`
- Adjusts pointer: `aligned_ptr = ptr_val - shift`
- Core chip handles data shifting based on the shift amount

### Data Flow

#### Load Operations
1. Adapter reads rs1 register
2. Calculates memory address: `ptr_val = rs1 + sign_extended_immediate`
3. Performs aligned read from memory
4. Core chip processes read data based on opcode and shift
5. Result written to rd register (unless rd = x0)

#### Store Operations
1. Adapter reads rs1 for address calculation
2. Reads rs2 for data to store
3. Reads current memory contents (prev_data)
4. Core chip merges new data with prev_data based on opcode and shift
5. Writes merged result to aligned memory address

### Constraint System

The core chip enforces several key constraints:
- Valid flag encoding using 4 flag bits
- Proper opcode determination from flag combinations
- Correct data transformation based on opcode
- Load/store type matching the operation

## Implementation Details

### Core Chip Columns
```rust
pub struct LoadStoreCoreCols<T, const NUM_CELLS: usize> {
    pub flags: [T; 4],        // Encodes opcode and shift
    pub is_valid: T,          // Boolean validity flag
    pub is_load: T,           // Boolean load indicator
    pub read_data: [T; NUM_CELLS],  // Data read from memory/register
    pub prev_data: [T; NUM_CELLS],  // Previous memory contents
    pub write_data: [T; NUM_CELLS], // Computed write data
}
```

### Adapter Chip Columns
```rust
pub struct Rv32LoadStoreAdapterCols<T> {
    pub from_state: ExecutionState<T>,
    pub rs1_ptr: T,
    pub rs1_data: [T; RV32_REGISTER_NUM_LIMBS],
    pub rd_rs2_ptr: T,
    pub imm: T,
    pub imm_sign: T,
    pub mem_ptr_limbs: [T; 2],
    pub mem_as: T,
    pub needs_write: T,
    // ... auxiliary columns for memory operations
}
```

### Write Data Computation

The core chip computes write_data based on the operation type:

- **LOADW**: Direct copy of read_data
- **LOADBU**: Places selected byte in first limb, zeros others
- **LOADHU**: Places selected halfword in first two limbs, zeros others
- **STOREW**: Direct copy of read_data
- **STOREB**: Merges single byte from read_data into prev_data
- **STOREH**: Merges halfword from read_data into prev_data

## Testing

The component includes comprehensive tests:
- Random operation tests with 100+ iterations
- Negative tests for constraint validation
- Sanity tests for each operation type
- Address alignment verification
- Memory space validation

## Security Considerations

1. **Pointer Bounds**: Enforces pointer_max_bits limit to prevent overflow
2. **Address Space Validation**: Ensures operations use correct address spaces
3. **Alignment Requirements**: Maintains 4-byte alignment for all memory accesses
4. **Register Protection**: x0 register remains read-only (writes ignored)

## Performance Notes

- All memory operations are batched in 4-byte chunks for efficiency
- Shift operations are handled in the constraint system without additional cycles
- Flag encoding allows efficient opcode determination
- Separate read/write paths optimize for common cases

## Integration Points

The LoadStore component integrates with:
- **ExecutionBus**: For instruction dispatch
- **MemoryBridge**: For memory read/write operations
- **ProgramBus**: For program counter updates
- **RangeChecker**: For address validation
- **OfflineMemory**: For trace generation