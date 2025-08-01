# SHA256 AIR Component Documentation

## Overview

The `openvm-sha256-air` crate provides a zero-knowledge implementation of the SHA256 compression function for the OpenVM framework. This component implements SHA256 hashing in an Algebraic Intermediate Representation (AIR) suitable for zero-knowledge proofs, without handling message padding.

## Architecture

### Core Components

#### 1. Sha256Air (`air.rs`)
The main AIR implementation that defines the constraints for SHA256 computation:
- **Purpose**: Implements the SHA256 compression function constraints
- **Key Features**:
  - Processes 512-bit message blocks
  - 17 rows per block (16 round rows + 1 digest row)
  - 4 SHA256 rounds per row
  - Bitwise operation lookup integration
  - Self-interaction bus for cross-block validation

#### 2. Column Structures (`columns.rs`)
Defines the trace matrix layout:
- **Sha256RoundCols**: For the first 16 rows of each block
- **Sha256DigestCols**: For the final row of each block
- **Shared fields**: flags, work_vars/hash, schedule_helper

#### 3. Trace Generation (`trace.rs`)
Handles the computation and generation of execution traces:
- **Two-pass generation**: Initial trace + missing cell completion
- **Block processing**: Converts input blocks to trace rows
- **Constraint satisfaction**: Ensures all AIR constraints are met

#### 4. Utility Functions (`utils.rs`)
Provides SHA256-specific operations and constants:
- **SHA256 functions**: Ch, Maj, Σ₀, Σ₁, σ₀, σ₁
- **Bit manipulation**: Rotation, shifting, XOR operations
- **Constants**: K values, initial hash H, buffer sizes

### Data Flow

```
Input Block (512 bits) → Message Schedule → Working Variables → Hash Output
                                    ↓
                              AIR Constraints
                                    ↓
                            Zero-Knowledge Proof
```

### Block Structure

Each SHA256 block is processed in 17 rows:
- **Rows 0-15**: Round rows (4 rounds each = 64 total rounds)
- **Row 16**: Digest row (final hash computation)

### Key Constants

- `SHA256_ROUNDS_PER_ROW`: 4 rounds per row
- `SHA256_ROWS_PER_BLOCK`: 17 rows per block
- `SHA256_HASH_WORDS`: 8 words (32-bit each)
- `SHA256_BLOCK_WORDS`: 16 words per input block

## Constraint System

### Row Constraints
- Boolean constraints on flag variables
- Bit decomposition of working variables (a, e)
- Row index encoding validation

### Transition Constraints
- Message schedule word generation (W₀-W₆₃)
- Working variable updates (a-h state)
- Cross-block hash chaining
- Proper flag transitions

### Digest Row Constraints
- Final hash computation validation
- Previous hash integration
- Block boundary handling

## Security Properties

- **Soundness**: Invalid SHA256 computations cannot produce valid proofs
- **Completeness**: Valid SHA256 computations always produce valid proofs
- **Zero-Knowledge**: Proofs reveal no information about the input beyond the hash

## Integration Points

### Dependencies
- `openvm-circuit-primitives`: Bitwise operations, utilities
- `openvm-stark-backend`: AIR framework, field operations
- `sha2`: Reference implementation for validation

### Lookup Tables
- **Bitwise Operations**: Range checks for carries and limbs
- **Self-Interactions**: Cross-block hash validation

### Bus Connections
- `bitwise_lookup_bus`: For range checks
- `bus`: Internal permutation checks

## Performance Characteristics

- **Trace Width**: Variable (max of round/digest column widths)
- **Trace Height**: 17 × number_of_blocks (padded to power of 2)
- **Constraint Degree**: Maximum degree 3
- **Memory Usage**: Linear in number of blocks

## Limitations

- **No Padding**: Input must be pre-padded to 512-bit blocks
- **Fixed Block Size**: Only processes complete 512-bit blocks
- **No Streaming**: All blocks must be known at trace generation time

## Testing

The component includes comprehensive tests:
- Random input validation
- Constraint violation detection
- Edge case handling
- Integration with OpenVM framework

## Usage Context

This component is designed for use within the OpenVM ecosystem where SHA256 hashing needs to be verified in zero-knowledge proofs. It's particularly suitable for applications requiring cryptographic hash verification without revealing the original input data.