# Keccak-256 Circuit Component Documentation

## Overview

The Keccak-256 circuit component implements a stateful Keccak-256 hasher for the OpenVM zkVM framework. It handles the full Keccak sponge construction (padding, absorbing, and squeezing) on variable-length inputs read from VM memory. The implementation is optimized for proving efficiency and integrates seamlessly with the OpenVM execution environment.

## Architecture

### Core Components

1. **KeccakVmChip** (`lib.rs`): The main chip that implements the Keccak-256 instruction executor
   - Manages state transitions between Keccak-f permutations
   - Handles memory reads for input data and writes for output digest
   - Integrates with the bitwise operation lookup chip for XOR operations

2. **KeccakVmAir** (`air.rs`): The Arithmetic Intermediate Representation (AIR) defining constraints
   - Enforces Keccak-f permutation constraints
   - Validates padding rules (10*1 padding)
   - Manages state transitions between rounds and blocks
   - Handles memory interactions and timestamp consistency

3. **Column Structure** (`columns.rs`): Defines the trace layout
   - `KeccakVmCols`: Main columns containing permutation state, sponge data, and instruction info
   - `KeccakInstructionCols`: Instruction execution and register access columns
   - `KeccakSpongeCols`: Sponge construction and padding columns
   - `KeccakMemoryCols`: Memory access auxiliary columns

4. **Trace Generation** (`trace.rs`): Builds the execution trace for proving
   - Generates Keccak-f permutation traces using p3-keccak-air
   - Manages state transitions between input blocks
   - Handles memory auxiliary column generation

## Key Features

### Variable-Length Input Handling
- Processes inputs of any length by breaking them into rate-sized blocks (136 bytes)
- Efficiently handles partial blocks with proper padding
- Maintains state across multiple Keccak-f permutations for long inputs

### Memory Integration
- Reads input data from VM memory in 4-byte chunks
- Writes 32-byte digest output to memory
- Tracks memory access timestamps for consistency

### Padding Implementation
- Implements standard Keccak 10*1 padding rule
- Handles edge cases (single byte padding when input fills rate)
- Validates padding constraints in the AIR

### State Management
- Maintains 1600-bit Keccak state (25 × 64-bit words)
- Splits state into rate (136 bytes) and capacity (64 bytes) portions
- Resets state for new hash computations

## Integration Points

### Instruction Format
```rust
KECCAK256 dst, src, len
```
- `dst`: Destination address for 32-byte hash output
- `src`: Source address of input data
- `len`: Length of input in bytes

### Bus Interactions
1. **Execution Bus**: Receives instruction and updates PC/timestamp
2. **Memory Bridge**: Handles all memory reads and writes
3. **Bitwise Lookup Bus**: Performs XOR operations for state updates

### Extension Configuration
The component integrates as a VM extension through `Keccak256Rv32Config`, which:
- Registers the Keccak opcode handler
- Sets up the bitwise operation lookup chip
- Configures memory access parameters

## Performance Characteristics

### Trace Height
- Each Keccak-f permutation requires 24 rounds
- Total height = `num_blocks × 24` where `num_blocks = ceil(len / 136)`

### Memory Accesses
- Register reads: 3 (dst, src, len)
- Input reads: up to 34 per block (136 bytes / 4 bytes per read)
- Output writes: 8 (32 bytes / 4 bytes per write)

### Constraint Degree
- Most constraints are degree 2-3
- XOR lookup interactions can reach degree 3
- Optimized for efficient proving

## Implementation Details

### State Representation
- Keccak state stored as 5×5 matrix of 64-bit words
- Each word decomposed into 4 16-bit limbs for p3-keccak-air compatibility
- High bytes tracked separately for efficient XOR operations

### Round Processing
1. **First Round**: Reset state if new computation, read input block
2. **Middle Rounds**: Continue Keccak-f permutation
3. **Last Round**: Complete permutation, prepare for next block or output

### Block Transitions
- Smooth transition between input blocks for multi-block hashes
- Timestamp and pointer updates maintain consistency
- Padding detection triggers final block processing

## Security Considerations

- All memory accesses are bounds-checked against configured pointer limits
- Input lengths are validated to prevent overflow
- Padding rules strictly enforced to match Keccak specification
- State isolation between different hash computations

## Usage Example

```rust
// In VM guest code
let input = b"Hello, World!";
let mut output = [0u8; 32];
keccak256(input.as_ptr(), input.len(), output.as_mut_ptr());
```

This compiles to a single `KECCAK256` instruction that the circuit handles efficiently.