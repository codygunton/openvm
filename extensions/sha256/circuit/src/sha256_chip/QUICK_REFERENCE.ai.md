# SHA256 Chip Quick Reference

## Component Overview
Hardware-accelerated SHA256 hashing in OpenVM zkVM via custom RISC-V instruction.

## File Structure
```
sha256_chip/
├── mod.rs        # Main implementation & instruction executor
├── air.rs        # Constraints & padding logic
├── columns.rs    # Trace column layouts
├── trace.rs      # Trace generation
└── tests.rs      # Unit tests
```

## Key Types

### Main Structures
```rust
Sha256VmChip<F>         // Main chip implementing InstructionExecutor
Sha256VmAir             // Constraint system
Sha256Record<F>         // Execution record for trace generation
```

### Column Types
```rust
Sha256VmRoundCols<T>    // Round computation columns (rows 0-15)
Sha256VmDigestCols<T>   // Final digest columns (row 16)
Sha256VmControlCols<T>  // Shared control state
```

## Constants
```rust
SHA256_READ_SIZE = 16        // Bytes per memory read
SHA256_WRITE_SIZE = 32       // SHA256 digest size
SHA256_BLOCK_CELLS = 64      // Bytes per SHA256 block
SHA256_NUM_READ_ROWS = 4     // Reads per block
SHA256_REGISTER_READS = 3    // Register reads (dst, src, len)
```

## Instruction Format
```assembly
sha256 rd, rs1, rs2
```
- `rd`: Destination pointer for 32-byte output
- `rs1`: Source pointer for input message  
- `rs2`: Message length in bytes

## Memory Layout

### Input Reading Pattern
```
Block 0: [16 bytes][16 bytes][16 bytes][16 bytes] = 64 bytes
Block 1: [16 bytes][16 bytes][16 bytes][16 bytes] = 64 bytes
...
```

### Output Format
32-byte digest written in big-endian format

## Padding States
```rust
NotConsidered     // Not in first 4 rows
NotPadding        // No padding needed
FirstPadding0-15  // First padding with N message bytes
FirstPadding0-7_LastRow  // First padding in final block
EntirePadding     // Full padding row
EntirePaddingLastRow     // Full padding in final block
```

## Key Functions

### Instruction Execution
```rust
impl InstructionExecutor<F> for Sha256VmChip<F> {
    fn execute(&mut self, memory: &mut MemoryController<F>, 
               instruction: &Instruction<F>,
               from_state: ExecutionState<u32>) -> Result<ExecutionState<u32>, ExecutionError>
}
```

### Trace Generation
```rust
impl Chip<SC> for Sha256VmChip<Val<SC>> {
    fn generate_air_proof_input(self) -> AirProofInput<SC>
}
```

## Constraints

### Main Constraint Functions
- `eval_padding()` - Padding state machine
- `eval_transitions()` - Row-to-row consistency
- `eval_reads()` - Memory read validation
- `eval_last_row()` - Final output & state update

## Usage Example

### Guest Code
```rust
extern "C" {
    fn zkvm_sha256_impl(input: *const u8, len: usize, output: *mut u8);
}

let hash = [0u8; 32];
zkvm_sha256_impl(data.as_ptr(), data.len(), hash.as_mut_ptr());
```

### Integration
```rust
// Create chip
let sha256_chip = Sha256VmChip::new(
    system_port,
    24,  // ptr_max_bits
    bitwise_lookup_chip,
    bus_idx,
    offset,
    offline_memory
);

// Register with VM
vm.add_extension(sha256_chip);
```

## Common Patterns

### Processing Variable Length Input
```rust
let num_blocks = ((len << 3) + 1 + 64).div_ceil(512);
// Process num_blocks * 64 bytes with padding
```

### Memory Pointer Validation
```rust
assert!(ptr < (1 << ptr_max_bits));
```

### Endianness Conversion
```rust
// Memory is little-endian, SHA256 needs big-endian
let word_be = word_le.reverse();
```

## Performance Notes

- 17 rows per SHA256 block in trace
- 4 memory reads + 1 write per block
- Parallel trace generation supported
- Shared bitwise lookup tables

## Error Cases

1. **Invalid Pointers**: > `ptr_max_bits`
2. **Message Too Long**: > 2^30 bytes  
3. **Memory Access**: Invalid addresses
4. **Instruction Format**: Wrong opcode/funct fields

## Quick Debugging

### Check Trace
```rust
let trace = chip.generate_air_proof_input();
// Inspect specific rows/columns
```

### Verify Padding
```rust
// Message length L → padding starts at byte L
// Append: 0x80, zeros, 8-byte length (big-endian)
```

### Common Issues
- Wrong endianness in I/O
- Padding calculation errors
- Timestamp misalignment
- Pointer arithmetic overflow