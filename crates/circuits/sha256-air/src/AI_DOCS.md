# OpenVM SHA256-AIR Documentation

## Overview

The `openvm-sha256-air` crate provides a highly optimized implementation of the SHA-256 compression function as an Arithmetic Intermediate Representation (AIR) for zero-knowledge proof systems. This implementation is specifically designed for integration within OpenVM's modular zkVM architecture.

### Key Features
- **No-padding design**: Processes pre-padded 512-bit message blocks
- **Efficient constraint system**: Minimizes constraint degree through careful algebraic design
- **Bitwise operation integration**: Leverages OpenVM's lookup tables for efficient bit operations
- **Multi-message support**: Handles multiple concatenated messages with proper state management
- **Parallel trace generation**: Supports concurrent processing of independent blocks

## Architecture

### Trace Matrix Structure

The SHA-256 computation is organized into a two-dimensional trace matrix where:
- Each SHA-256 block occupies 17 rows
- Rows 0-15 are "round rows" processing 4 rounds each (64 total)
- Row 16 is the "digest row" for block finalization
- Padding rows fill the trace to the next power of 2

### Column Architecture

The design uses two column structures that share common fields:

```rust
// Common fields (polymorphic design)
struct CommonFields<T> {
    flags: Sha256FlagsCols<T>,
    work_vars_or_hash: Sha256WorkVarsCols<T>,
    schedule_helper: Sha256MessageHelperCols<T>,
}

// Round-specific fields
struct Sha256RoundCols<T> {
    ...CommonFields,
    message_schedule: Sha256MessageScheduleCols<T>,
}

// Digest-specific fields  
struct Sha256DigestCols<T> {
    ...CommonFields,
    final_hash: [[T; 4]; 8],
    prev_hash: [[T; 2]; 8],
}
```

## Core Components

### 1. Sha256Air

The main AIR implementation containing all constraint logic:

```rust
pub struct Sha256Air {
    pub bitwise_lookup_bus: BitwiseOperationLookupBus,
    pub row_idx_encoder: Encoder,
    bus: PermutationCheckBus,
}
```

Key methods:
- `eval_row`: Single-row constraints (flags, boolean values)
- `eval_transitions`: Cross-row constraints (message schedule, work variables)
- `eval_digest_row`: Block finalization constraints
- `eval_prev_hash`: Inter-block hash chaining

### 2. Message Schedule Implementation

The message schedule W[0..63] is computed as:
- W[0..15]: Direct from input message
- W[16..63]: W[t] = σ1(W[t-2]) + W[t-7] + σ0(W[t-15]) + W[t-16]

Due to constraint degree limitations, intermediate values propagate partial computations:

```
intermed_4[i] = W[i-4] + σ0(W[i-3])  
intermed_8[i] = intermed_4[i-4]
intermed_12[i] = intermed_8[i-4]
```

### 3. Working Variables Update

Each round updates 8 working variables (a-h) where only a and e are explicitly stored:

```rust
T1 = h + Σ1(e) + Ch(e,f,g) + K[round] + W[round]
T2 = Σ0(a) + Maj(a,b,c)
new_a = T1 + T2
new_e = d + T1
```

Other variables are derived through rotation:
- b through d are previous values of a
- f through h are previous values of e

### 4. Bitwise Operations

All bitwise operations use field-compatible implementations:

```rust
// Choose function: Ch(x,y,z) = (x AND y) XOR (NOT x AND z)
ch_field(x, y, z) = select(x, y, z) for each bit

// Majority function: Maj(x,y,z) = (x AND y) XOR (x AND z) XOR (y AND z)
maj_field(x, y, z) = xy + xz + yz - 2xyz for each bit

// Rotation and shift operations on bit arrays
rotr(bits, n) = rotate right by n positions
shr(bits, n) = shift right by n positions
```

## Constraint System

### Row-Level Constraints

1. **Flag constraints**: Ensure valid flag combinations
2. **Boolean constraints**: Verify bit values are 0 or 1
3. **Index constraints**: Validate row indices are in range [0, 17]

### Transition Constraints

1. **Message schedule**: Verify W[t] computation with carry propagation
2. **Working variables**: Check a and e updates with 16-bit limb arithmetic
3. **State transitions**: Ensure proper row sequencing and block boundaries

### Interaction Constraints

1. **Bitwise lookups**: Range checks for carries and hash bytes
2. **Self-interaction bus**: Hash chaining between consecutive blocks

## Trace Generation

### Two-Pass Algorithm

**First Pass**: Main computation
1. Compute message schedule words
2. Update working variables
3. Track carries and intermediate values
4. Generate digest row with final hash

**Second Pass**: Constraint satisfaction
1. Fill dummy values for intermed_12 (rows 0-3, 15-16)
2. Fill dummy values for intermed_4 (rows 0, 16)
3. Fill dummy carries for digest rows
4. Generate padding rows

### Memory Layout

```rust
// Trace memory organization
Block 0: [Round0..Round15, Digest0]
Block 1: [Round16..Round31, Digest1]
...
Block N: [RoundM..RoundM+15, DigestN]
Padding: [Padding rows to power of 2]
```

## Usage Examples

### Basic Usage

```rust
use openvm_sha256_air::{Sha256Air, generate_trace};

// Initialize with lookup tables
let bitwise_bus = BitwiseOperationLookupBus::new(bus_idx);
let sha256_air = Sha256Air::new(bitwise_bus, self_bus_idx);

// Prepare message blocks (must be 512 bits each)
let blocks = vec![
    (first_block, false),   // Not last block
    (second_block, false),  // Not last block
    (final_block, true),    // Last block of message
];

// Generate execution trace
let trace = generate_trace(&sha256_air, bitwise_chip, blocks);
```

### Multi-Message Processing

```rust
// Process multiple independent messages
let messages = vec![
    vec![(block1, false), (block2, true)],  // First message
    vec![(block3, false), (block4, true)],  // Second message
];

let mut all_blocks = Vec::new();
for message in messages {
    all_blocks.extend(message);
}

let trace = generate_trace(&sha256_air, bitwise_chip, all_blocks);
```

### Custom Block Processing

```rust
// Manual trace generation for custom scenarios
let mut trace = vec![F::ZERO; height * width];

// Process individual blocks with custom parameters
sha256_air.generate_block_trace(
    &mut trace[offset..],
    width,
    start_col,
    &input_words,
    bitwise_chip.clone(),
    &prev_hash,
    is_last_block,
    global_block_idx,
    local_block_idx,
    &buffer_values,
);

// Fill missing cells
sha256_air.generate_missing_cells(&mut trace[offset..], width, start_col);
```

## Performance Considerations

### Optimization Strategies

1. **Parallel Block Processing**: Independent blocks can be traced concurrently
2. **Batched Lookups**: Bitwise operations are batched for efficiency
3. **Precomputed Values**: Constants like invalid carries are precomputed
4. **Minimal Constraints**: Careful design minimizes constraint evaluations

### Memory Usage

- Trace size: `O(num_blocks * 17 * column_width)`
- Lookup table requests: `O(num_blocks * rounds_per_block)`
- Intermediate storage: Minimal, mostly stack-allocated

## Security Analysis

### Soundness Properties

1. **Complete Coverage**: All 64 SHA-256 rounds are fully constrained
2. **Padding Security**: Trace padding prevents boundary attacks
3. **Hash Chaining**: Self-interaction bus ensures proper block sequencing
4. **Carry Bounds**: Arithmetic design prevents overflow attacks

### Assumptions

1. Input blocks are properly padded according to SHA-256 specification
2. Bitwise lookup tables are correctly initialized
3. Field size is sufficient for all arithmetic operations

## Integration Guide

### With OpenVM Circuits

```rust
use openvm_circuit_primitives::SubAir;
use openvm_stark_backend::AirBuilder;

impl MyCircuit {
    fn eval<AB: AirBuilder>(&self, builder: &mut AB) {
        // Evaluate SHA256 sub-air
        self.sha256_air.eval(builder, start_column);
        
        // Use SHA256 results in larger circuit
        // ...
    }
}
```

### With Bitwise Operations

```rust
// Initialize shared lookup chip
let bitwise_chip = SharedBitwiseOperationLookupChip::new(bus);

// SHA256 automatically requests necessary lookups
generate_trace(&sha256_air, bitwise_chip.clone(), blocks);

// Finalize lookups after all requests
bitwise_chip.finalize();
```

## Troubleshooting

### Common Issues

1. **Trace length not power of 2**: Ensure padding rows are added
2. **Constraint failures**: Check input block padding and initialization
3. **Lookup mismatches**: Verify bitwise chip initialization
4. **Hash chain breaks**: Ensure proper is_last_block flags

### Debugging Tips

1. Use `SHA256_ROUNDS_PER_ROW = 1` for step-by-step debugging
2. Verify intermediate values against reference implementation
3. Check flag consistency across row transitions
4. Validate carry values are within expected bounds

## References

- [NIST FIPS 180-4](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf): SHA-256 specification
- OpenVM documentation: Integration examples
- `sha2` crate: Reference implementation used for testing