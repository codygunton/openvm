# SHA256 Chip Implementation Guide

## Overview

This guide explains how to work with and extend the SHA256 chip component in OpenVM. The chip provides hardware-accelerated SHA256 hashing as a custom RISC-V instruction.

## Understanding the Architecture

### Execution Flow

```
Instruction Decode → Register Read → Memory Read → SHA256 Process → Memory Write → State Update
```

1. **Instruction Reception**: The chip receives `sha256 rd, rs1, rs2` instruction
2. **Parameter Extraction**: Reads destination pointer (rd), source pointer (rs1), and length (rs2)
3. **Message Reading**: Loads input message from memory in 16-byte chunks
4. **Hash Computation**: Processes through SHA256 with automatic padding
5. **Result Storage**: Writes 32-byte digest to destination address

### Key Design Decisions

#### Memory Access Pattern
The chip reads 16 bytes per memory access (4 accesses per 64-byte block) to balance:
- Bus utilization efficiency
- Trace generation complexity
- Memory controller overhead

#### Padding Implementation
Padding is handled through a state machine with explicit flags rather than dynamic computation:
- Enables efficient constraint verification
- Supports all edge cases (message boundaries, final block)
- Minimizes non-deterministic computation

## Working with the Code

### Adding New Features

#### 1. Modifying the Instruction Format
To add parameters to the SHA256 instruction:

```rust
// In mod.rs - Update execution logic
impl<F: PrimeField32> InstructionExecutor<F> for Sha256VmChip<F> {
    fn execute(&mut self, memory: &mut MemoryController<F>, 
               instruction: &Instruction<F>, 
               from_state: ExecutionState<u32>) -> Result<ExecutionState<u32>, ExecutionError> {
        // Add new parameter handling here
        let new_param = instruction.f; // Example: using unused field
        
        // Update processing logic
    }
}
```

#### 2. Extending Memory Access
To modify how data is read/written:

```rust
// In mod.rs - Adjust read size
const SHA256_READ_SIZE: usize = 32; // Example: larger reads

// Update air.rs constraints
fn eval_reads<AB: InteractionBuilder>(&self, builder: &mut AB) {
    // Modify memory bridge interactions
}
```

#### 3. Adding Streaming Support
To support streaming hashes across multiple instructions:

```rust
// Add state tracking to Sha256VmChip
pub struct Sha256VmChip<F: PrimeField32> {
    // ... existing fields ...
    streaming_states: HashMap<u32, StreamingState>, // Track partial hashes
}

// Modify execution to handle streaming
fn execute(&mut self, /* ... */) -> Result<ExecutionState<u32>, ExecutionError> {
    if is_streaming_start(instruction) {
        self.streaming_states.insert(stream_id, initial_state);
    } else if is_streaming_continue(instruction) {
        let state = self.streaming_states.get_mut(&stream_id).unwrap();
        // Continue from previous state
    }
}
```

### Optimizing Performance

#### 1. Trace Height Reduction
Minimize rows per operation:

```rust
// In trace.rs - Batch operations where possible
impl<SC: StarkGenericConfig> Chip<SC> for Sha256VmChip<Val<SC>> {
    fn generate_air_proof_input(self) -> AirProofInput<SC> {
        // Combine related operations into single rows
        // Use auxiliary columns for complex computations
    }
}
```

#### 2. Memory Access Optimization
Reduce memory bus traffic:

```rust
// Implement caching for repeated accesses
struct CachedMemoryReader {
    cache: HashMap<u32, [u8; 16]>,
    memory: &mut MemoryController<F>,
}
```

#### 3. Parallel Processing
Enable concurrent hash operations:

```rust
// Process independent hashes in parallel
records.par_iter().map(|record| {
    process_sha256_record(record)
}).collect()
```

## Constraint System Details

### Understanding the AIR

The constraint system ensures:

1. **Correctness**: SHA256 computation follows specification
2. **Memory Safety**: All accesses are valid
3. **Padding Validity**: Message padding is correct
4. **State Consistency**: Execution state updates properly

### Key Constraint Groups

#### Padding Constraints (`eval_padding`)
```rust
// Ensures padding follows SHA256 specification
// - Single 1 bit after message
// - Zeros until final 64 bits
// - Message length in final 64 bits
```

#### Transition Constraints (`eval_transitions`)
```rust
// Maintains consistency between rows:
// - Length remains constant
// - Read pointer increments correctly
// - Timestamp advances properly
```

#### Read/Write Constraints (`eval_reads`, `eval_last_row`)
```rust
// Validates memory operations:
// - Correct addresses
// - Proper timestamps
// - Valid data transfer
```

## Testing and Debugging

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_sha256_basic() {
        let mut chip = setup_sha256_chip();
        let input = b"hello world";
        let result = chip.hash(input);
        assert_eq!(result, expected_hash);
    }
    
    #[test]
    fn test_sha256_padding_edge_cases() {
        // Test message lengths: 55, 56, 64, 119, 120 bytes
        // These hit padding boundary conditions
    }
}
```

### Debugging Tips

1. **Trace Inspection**: Use `generate_air_proof_input` to examine trace generation
2. **Constraint Violations**: Check AIR evaluation order and dependencies
3. **Memory Issues**: Verify pointer arithmetic and bounds checking
4. **Padding Errors**: Validate flag transitions and message length encoding

### Common Issues

#### Issue: Incorrect Hash Output
- Check endianness in memory reads/writes
- Verify padding calculation for message length
- Ensure block processing order is correct

#### Issue: Constraint Failures
- Validate all flag encodings are within valid ranges
- Check timestamp and pointer increment logic
- Verify memory auxiliary column generation

#### Issue: Performance Problems
- Profile trace generation for bottlenecks
- Consider batching memory operations
- Optimize lookup table usage

## Extension Examples

### 1. Adding SHA-512 Support

```rust
// Create new chip variant
pub struct Sha512VmChip<F: PrimeField32> {
    // Similar structure but with:
    // - 1024-bit blocks
    // - 64-bit words
    // - Different round constants
}
```

### 2. Implementing HMAC

```rust
// Build on top of SHA256 chip
pub struct HmacSha256Chip<F: PrimeField32> {
    sha256_chip: Sha256VmChip<F>,
    // Add key handling
    // Implement inner/outer hash logic
}
```

### 3. Batch Hashing

```rust
// Process multiple messages in single instruction
pub struct BatchSha256Chip<F: PrimeField32> {
    // Add count parameter
    // Loop over multiple messages
    // Optimize trace layout
}
```

## Best Practices

1. **Memory Safety**: Always validate pointers against `ptr_max_bits`
2. **Trace Efficiency**: Minimize auxiliary columns and row count
3. **Constraint Clarity**: Document complex constraints thoroughly
4. **Testing Coverage**: Include edge cases for padding and boundaries
5. **Error Handling**: Fail fast with clear error messages

## Integration Checklist

When integrating SHA256 chip into a larger system:

- [ ] Register opcode in transpiler extension
- [ ] Configure memory and bitwise lookup bus connections
- [ ] Set appropriate `ptr_max_bits` for address space
- [ ] Implement guest library wrapper functions
- [ ] Add chip to VM configuration
- [ ] Write integration tests with full VM
- [ ] Document memory layout requirements
- [ ] Profile and optimize trace generation