# Keccak-256 Circuit Component Index

## File Structure

### Core Implementation
- `lib.rs` - Main chip implementation and instruction executor
  - `KeccakVmChip` - Main Keccak-256 VM chip
  - `KeccakRecord` - Execution record for each Keccak instruction
  - `KeccakInputBlock` - Input block data structure
  - Constants: `KECCAK_WIDTH_BYTES`, `KECCAK_RATE_BYTES`, etc.

### AIR (Arithmetic Intermediate Representation)
- `air.rs` - Constraint definitions and interactions
  - `KeccakVmAir` - Main AIR implementation
  - `eval_keccak_f()` - Keccak-f permutation constraints
  - `constrain_padding()` - Padding rule enforcement
  - `constrain_absorb()` - State absorption constraints
  - `eval_instruction()` - Instruction execution constraints
  - `constrain_input_read()` - Memory read constraints
  - `constrain_output_write()` - Memory write constraints

### Column Definitions
- `columns.rs` - Trace column structures
  - `KeccakVmCols<T>` - Main trace columns
  - `KeccakInstructionCols<T>` - Instruction-related columns
  - `KeccakSpongeCols<T>` - Sponge construction columns
  - `KeccakMemoryCols<T>` - Memory auxiliary columns
  - Column count constants

### Trace Generation
- `trace.rs` - Execution trace builder
  - `generate_air_proof_input()` - Main trace generation
  - State management and transitions
  - Memory interaction handling
  - Integration with p3-keccak-air

### Extension Integration
- `extension.rs` - VM extension configuration
  - `Keccak256Rv32Config` - Configuration for RV32 with Keccak
  - `Keccak256` - Extension trait implementation
  - `Keccak256Executor` - Executor enum
  - `Keccak256Periphery` - Periphery chips

### Utilities
- `utils.rs` - Helper functions
  - `keccak_f()` - Keccak-f permutation wrapper
  - `keccak256()` - Standard Keccak-256 hash function
  - `num_keccak_f()` - Calculate required permutations

### Tests
- `tests.rs` - Unit and integration tests
  - Single block tests
  - Multi-block tests
  - Edge case testing
  - Integration with VM

## Key Data Structures

### KeccakVmChip<F>
```rust
pub struct KeccakVmChip<F: PrimeField32> {
    pub air: KeccakVmAir,
    pub records: Vec<KeccakRecord<F>>,
    pub bitwise_lookup_chip: SharedBitwiseOperationLookupChip<8>,
    offset: usize,
    offline_memory: Arc<Mutex<OfflineMemory<F>>>,
}
```

### KeccakRecord<F>
```rust
pub struct KeccakRecord<F> {
    pub pc: F,
    pub dst_read: RecordId,
    pub src_read: RecordId,
    pub len_read: RecordId,
    pub input_blocks: Vec<KeccakInputBlock>,
    pub digest_writes: [RecordId; KECCAK_DIGEST_WRITES],
}
```

### KeccakVmCols<T>
```rust
pub struct KeccakVmCols<T> {
    pub inner: KeccakPermCols<T>,      // Keccak-f permutation columns
    pub sponge: KeccakSpongeCols<T>,   // Sponge construction
    pub instruction: KeccakInstructionCols<T>, // Instruction data
    pub mem_oc: KeccakMemoryCols<T>,   // Memory auxiliary
}
```

## Important Constants

### Keccak Parameters
- `KECCAK_WIDTH_BYTES = 200` - Total state size in bytes
- `KECCAK_RATE_BYTES = 136` - Rate portion in bytes
- `KECCAK_CAPACITY_BYTES = 64` - Capacity portion in bytes
- `KECCAK_DIGEST_BYTES = 32` - Output digest size

### Memory Access
- `KECCAK_REGISTER_READS = 3` - Register reads per instruction
- `KECCAK_WORD_SIZE = 4` - Bytes per memory access
- `KECCAK_ABSORB_READS = 34` - Memory reads per input block
- `KECCAK_DIGEST_WRITES = 8` - Memory writes for output

### Derived Constants
- `KECCAK_WIDTH_U16S = 100` - State size in 16-bit limbs
- `KECCAK_RATE_U16S = 68` - Rate size in 16-bit limbs
- `NUM_ABSORB_ROUNDS = 17` - Rate size in 64-bit words
- `KECCAK_DIGEST_U64S = 4` - Digest size in 64-bit words

## Integration Points

### Instruction Handling
- Implements `InstructionExecutor<F>` trait
- Handles `Rv32KeccakOpcode::KECCAK256` opcode
- Updates PC by `DEFAULT_PC_STEP`

### Memory Interface
- Uses `MemoryController` for all memory operations
- Tracks timestamps for memory consistency
- Supports offline memory checking

### Bus Connections
- Execution bus for instruction flow
- Memory bridge for read/write operations
- Bitwise lookup bus for XOR operations

## Proving Architecture

### AIR Structure
1. Keccak-f permutation (using p3-keccak-air)
2. Padding constraints
3. State absorption via XOR
4. Memory read/write constraints
5. Instruction execution constraints

### Interaction Types
- Send interactions: XOR operations, range checks
- Receive interactions: Instruction fetch
- Memory interactions: Read/write with timestamps

### Constraint Degrees
- Linear: Timestamp transitions, pointer arithmetic
- Quadratic: Most state transitions, memory addressing
- Cubic: XOR lookup interactions, conditional constraints