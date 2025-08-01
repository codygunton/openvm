# Hintstore Quick Reference

## Opcodes

| Opcode | Description | Operands | Effect |
|--------|-------------|----------|--------|
| HINT_STOREW | Store single word from hint stream | `b`: ptr to dest addr | Stores 4 bytes at `*b` |
| HINT_BUFFER | Store multiple words from hint stream | `a`: ptr to count<br>`b`: ptr to dest addr | Stores `*a * 4` bytes starting at `*b` |

## Key Structures

### Rv32HintStoreCols
```rust
pub struct Rv32HintStoreCols<T> {
    // Operation type flags
    pub is_single: T,           // Single word operation
    pub is_buffer: T,           // Buffer operation
    pub is_buffer_start: T,     // First row of buffer op
    
    // State tracking
    pub rem_words_limbs: [T; 4], // Remaining words to process
    pub from_state: ExecutionState<T>,
    
    // Memory operations
    pub mem_ptr_ptr: T,         // Register containing memory pointer
    pub mem_ptr_limbs: [T; 4],  // Actual memory pointer value
    pub data: [T; 4],           // Data being written
    
    // Auxiliary columns for proofs
    pub mem_ptr_aux_cols: MemoryReadAuxCols<T>,
    pub write_aux: MemoryWriteAuxCols<T, 4>,
    pub num_words_aux_cols: MemoryReadAuxCols<T>,
}
```

### Rv32HintStoreChip
```rust
pub struct Rv32HintStoreChip<F> {
    air: Rv32HintStoreAir,
    records: Vec<Rv32HintStoreRecord<F>>,
    height: usize,
    offline_memory: Arc<Mutex<OfflineMemory<F>>>,
    streams: OnceLock<Arc<Mutex<Streams<F>>>>,
    bitwise_lookup_chip: SharedBitwiseOperationLookupChip<8>,
}
```

## Key Functions

### execute()
Main execution function that:
1. Reads memory pointer from register
2. Reads word count (buffer mode only)
3. Consumes hint stream data
4. Writes to memory
5. Updates execution state

### eval()
AIR constraint evaluation that enforces:
- Valid state transitions
- Memory safety
- Correct arithmetic
- Proper instruction execution

## Common Patterns

### Setting up the chip
```rust
let chip = Rv32HintStoreChip::new(
    execution_bus,
    program_bus,
    bitwise_chip,
    memory_bridge,
    offline_memory,
    pointer_max_bits,
    offset,
);
chip.set_streams(streams);
```

### Trace Generation Flow
1. Execute instructions → create records
2. Convert records → trace rows
3. Apply padding → power-of-two height
4. Generate auxiliary columns → memory proofs

## Constraints Summary

1. **Boolean constraints**: `is_single`, `is_buffer`, `is_buffer_start`
2. **Exclusivity**: Either single XOR buffer operation
3. **Buffer continuity**: Proper state transitions in multi-row ops
4. **Memory safety**: Pointer bounds checking
5. **Data validity**: All bytes in range [0, 255]
6. **Word counting**: Decrements by 1 per row in buffer mode
7. **Address increment**: +4 bytes per word in buffer mode

## Error Cases

- `HintOutOfBounds`: Not enough data in hint stream
- Memory violations: Caught by memory controller
- Invalid opcodes: Panic in `get_opcode_name()`
- Pointer overflow: Prevented by bitwise range checks