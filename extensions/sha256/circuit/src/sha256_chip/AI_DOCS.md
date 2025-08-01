# SHA256 Chip Component Documentation

## Architecture Overview

The SHA256 chip implements hardware-accelerated SHA256 hashing within the OpenVM zkVM framework. It processes variable-length messages from memory and produces 256-bit (32-byte) digests.

### Component Structure

```
sha256_chip/
├── mod.rs        # Main chip implementation
├── air.rs        # Constraint system
├── columns.rs    # Trace column definitions  
├── trace.rs      # Trace generation
└── tests.rs      # Unit tests
```

## Core Implementation (`mod.rs`)

### Sha256VmChip Structure
```rust
pub struct Sha256VmChip<F: PrimeField32> {
    pub air: Sha256VmAir,
    pub records: Vec<Sha256Record<F>>,
    pub offline_memory: Arc<Mutex<OfflineMemory<F>>>,
    pub bitwise_lookup_chip: SharedBitwiseOperationLookupChip<8>,
    offset: usize,
}
```

**Key responsibilities:**
- Execute SHA256 instructions
- Manage memory reads/writes
- Record execution traces for proving

### Instruction Execution Flow

1. **Register Reads**: Read destination pointer, source pointer, and message length from registers
2. **Message Reading**: Read input message from memory in 16-byte chunks
3. **Hashing**: Process message through SHA256 algorithm with proper padding
4. **Output Writing**: Write 32-byte digest to destination address

### Key Constants
- `SHA256_READ_SIZE = 16`: Bytes read per memory access
- `SHA256_WRITE_SIZE = 32`: Size of SHA256 digest
- `SHA256_BLOCK_CELLS = 64`: Bytes per SHA256 block
- `SHA256_NUM_READ_ROWS = 4`: Memory reads per block

## Constraint System (`air.rs`)

### Sha256VmAir Structure
Implements the arithmetic constraints ensuring correct SHA256 computation:

```rust
pub struct Sha256VmAir {
    pub execution_bridge: ExecutionBridge,
    pub memory_bridge: MemoryBridge,
    pub bitwise_lookup_bus: BitwiseOperationLookupBus,
    pub ptr_max_bits: usize,
    pub(super) sha256_subair: Sha256Air,
    pub(super) padding_encoder: Encoder,
}
```

### Padding State Machine

The padding implementation uses flags to track the padding state:

```rust
enum PaddingFlags {
    NotConsidered,      // Not in first 4 rows
    NotPadding,         // No padding needed
    FirstPadding0..15,  // First padding row with N message bytes
    FirstPadding0_LastRow..7_LastRow,  // First padding in last block
    EntirePaddingLastRow,  // Full padding in last block
    EntirePadding,      // Full padding row
}
```

### Key Constraints

1. **Padding Transitions**: Ensures padding flags transition correctly between rows
2. **Message Length**: Validates appended length matches actual message length
3. **Memory Access**: Enforces correct read/write patterns
4. **Execution Bridge**: Links instruction execution with VM state updates

## Column Layout (`columns.rs`)

### Round Columns (First 16 rows per block)
```rust
pub struct Sha256VmRoundCols<T> {
    pub control: Sha256VmControlCols<T>,  // Shared control state
    pub inner: Sha256RoundCols<T>,        // SHA256 round computation
    pub read_aux: MemoryReadAuxCols<T>,   // Memory read auxiliary
}
```

### Digest Columns (Last row per block)
```rust
pub struct Sha256VmDigestCols<T> {
    pub control: Sha256VmControlCols<T>,
    pub inner: Sha256DigestCols<T>,
    pub from_state: ExecutionState<T>,
    // Register and memory pointers
    pub rd_ptr, rs1_ptr, rs2_ptr: T,
    pub dst_ptr, src_ptr, len_data: [T; RV32_REGISTER_NUM_LIMBS],
    // Auxiliary columns for memory operations
    pub register_reads_aux: [MemoryReadAuxCols<T>; 3],
    pub writes_aux: MemoryWriteAuxCols<T, 32>,
}
```

### Control Columns (Shared)
```rust
pub struct Sha256VmControlCols<T> {
    pub len: T,                 // Message length in bytes
    pub cur_timestamp: T,       // Current memory timestamp
    pub read_ptr: T,           // Current read pointer
    pub pad_flags: [T; 6],     // Padding state encoding
    pub padding_occurred: T,    // Padding started flag
}
```

## Trace Generation (`trace.rs`)

### Trace Generation Process

1. **State Initialization**: Create initial hash state (H0-H7)
2. **Block Processing**: For each 512-bit block:
   - Generate padding based on message length
   - Update hash state through 64 rounds
   - Record memory operations
3. **Finalization**: Write final hash to memory

### Sha256State Structure
```rust
struct Sha256State {
    hash: [u32; 8],              // Current hash values
    local_block_idx: usize,      // Block index within message
    message_len: u32,            // Total message length
    block_input_message: [u8; 64],   // Raw input bytes
    block_padded_message: [u8; 64],  // Padded message block
    message_idx: usize,          // Message record index
    is_last_block: bool,         // Final block flag
}
```

## Memory Access Patterns

### Input Reading
- Reads occur in first 4 rows of each SHA256 block
- Each row reads 16 bytes from consecutive addresses
- Total of 64 bytes read per block

### Output Writing
- Single 32-byte write on the last row of the final block
- Written in big-endian format as per SHA256 specification

## Integration with OpenVM

### Instruction Encoding
- Opcode: Custom-0 (0x0b) with funct3=0b100, funct7=0x1
- Maps to `Rv32Sha256Opcode::SHA256` internally

### Execution Requirements
1. Valid memory pointers within `ptr_max_bits` range
2. Sufficient memory allocated for input and output
3. Message length < 2^30 bytes (field element limitation)

## Performance Characteristics

### Optimizations
- Batched memory operations reduce bus traffic
- Shared bitwise lookup tables across multiple chips
- Parallel trace generation for multiple blocks

### Resource Usage
- Trace height: 17 rows per SHA256 block
- Memory accesses: 4 reads per block + 3 register reads + 1 output write
- Bitwise operations: Delegated to lookup tables

## Error Handling

The chip validates:
- Pointer bounds (must fit in `ptr_max_bits`)
- Message length constraints
- Memory access permissions
- Correct instruction format

Errors result in execution failure rather than invalid proofs.