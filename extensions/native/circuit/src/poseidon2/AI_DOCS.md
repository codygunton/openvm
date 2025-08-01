# Native Poseidon2 Component AI Documentation

## Overview
The Native Poseidon2 component implements hardware-accelerated Poseidon2 hash operations for OpenVM, with specialized support for Merkle tree batch verification (`VERIFY_BATCH`). This component is critical for efficiently verifying cryptographic commitments in zero-knowledge proofs.

## Core Architecture

### Key Files
- `chip.rs`: Main chip implementation with instruction execution logic
- `air.rs`: Arithmetic Intermediate Representation (AIR) constraints
- `columns.rs`: Column layout definitions for the trace matrix
- `trace.rs`: Trace generation for proof creation
- `tests.rs`: Comprehensive test suite

### Primary Responsibilities
1. **Poseidon2 Operations**: Native field permutation and compression
2. **Batch Verification**: Efficient Merkle proof verification for multiple matrices
3. **Memory Integration**: Seamless integration with OpenVM's memory subsystem
4. **Constraint Generation**: Sound AIR constraints for zkVM execution

## Instruction Set

### PERM_POS2 (Permutation)
Performs a Poseidon2 permutation on 16 field elements.
- **Input**: 16 field elements from memory
- **Output**: 16 permuted field elements to memory
- **Use Case**: Basic cryptographic hashing

### COMP_POS2 (Compression)
Performs Poseidon2 compression combining two 8-element states.
- **Input**: Two 8-element arrays (16 total)
- **Output**: One 8-element compressed result
- **Use Case**: Merkle tree node compression

### VERIFY_BATCH
Verifies a batch of Merkle proofs for matrices of varying heights.
- **Inputs**:
  - `dimensions`: Heights of input matrices
  - `opened_values`: Leaf values being verified
  - `proof_id`: Merkle proof siblings
  - `index_bits`: Path selector bits
  - `commit`: Expected root commitment
- **Operation**: Verifies the Merkle path from leaves to root
- **Use Case**: Batch proof verification in zkVM

## Row Types and Execution Model

The component uses different row types for efficient execution:

### 1. SimplePoseidon Rows
- Handle basic PERM_POS2 and COMP_POS2 instructions
- Single row per instruction
- Direct memory reads/writes

### 2. TopLevel Rows
Execute VERIFY_BATCH Merkle tree operations:
- **IncorporateRow**: Process matrix row hashes
- **IncorporateSibling**: Merge with Merkle siblings
- Manage the overall verification flow

### 3. InsideRow Rows
Compute rolling hashes for concatenated matrix rows:
- Process variable-length inputs
- Generate row hashes for TopLevel consumption
- Communicate via internal bus (bus 7)

### 4. Disabled Rows
Padding rows with no active computation.

## Memory Access Patterns

### Read Operations
- **Pointer Dereferencing**: Read array pointers from operands
- **Data Loading**: Load actual values from dereferenced addresses
- **Immediate Support**: Optional immediate values for efficiency

### Write Operations
- **Result Storage**: Write computation results to memory
- **Single-cycle Writes**: Atomic memory updates

### Address Space
- Uses native address space (AS::Native = 4)
- Consistent addressing for all Poseidon2 operations

## Key Design Principles

### 1. Column Reuse
The `specific` field in `NativePoseidon2Cols` is polymorphic:
- Cast to `TopLevelSpecificCols` for Merkle operations
- Cast to `InsideRowSpecificCols` for row hashing
- Cast to `SimplePoseidonSpecificCols` for basic operations
This maximizes column utilization and minimizes trace width.

### 2. Contiguous Execution Blocks
- TopLevel rows form contiguous blocks per VERIFY_BATCH
- InsideRow rows form contiguous sub-blocks
- Simplifies constraint checking and improves cache locality

### 3. Timestamp Management
- `very_first_timestamp`: Instruction start time
- `start_timestamp`: Individual row timestamp
- Enables proper ordering and execution tracking

### 4. Flexible Matrix Handling
- Supports matrices of varying heights (powers of 2)
- Dynamic concatenation based on heights
- Efficient handling of sparse Merkle trees

## Integration Points

### ExecutionBridge
- Records instruction execution
- Manages PC updates
- Tracks execution state transitions

### MemoryBridge
- Offline memory checking
- Read/write auxiliary columns
- Memory consistency verification

### VerifyBatchBus
Internal communication bus (index 7) between:
- TopLevel IncorporateRow operations
- InsideRow hash computations

### Poseidon2SubAir
Reusable Poseidon2 permutation logic:
- Shared between all row types
- Implements core cryptographic operations
- Parameterized by SBOX_REGISTERS

## Performance Considerations

### Batching
- VERIFY_BATCH processes multiple proofs in one instruction
- Reduces instruction overhead
- Amortizes setup costs

### Memory Locality
- Contiguous row blocks improve cache performance
- Minimal pointer chasing
- Efficient data layout

### Parallelization
- Independent Poseidon2 operations can parallelize
- Trace generation uses parallel iterators
- Suitable for GPU acceleration

## Security Properties

### Soundness
- AIR constraints ensure correct Poseidon2 computation
- Memory consistency checks prevent tampering
- Proper Merkle path verification

### Completeness
- All valid proofs pass verification
- Supports full range of matrix configurations
- Handles edge cases (single element, maximum height)

## Common Use Cases

### 1. Commitment Verification
Verify zkVM execution trace commitments:
```
VERIFY_BATCH with multiple trace matrices
```

### 2. State Root Updates
Compute new Merkle roots after state changes:
```
Multiple COMP_POS2 operations building tree
```

### 3. Hash Chains
Create hash chains for sequential data:
```
Repeated PERM_POS2 with chaining
```

## Implementation Notes

- **CHUNK = 8**: Fixed Poseidon2 state size
- **Bus 7**: Dedicated internal communication bus
- **Address Space 4**: Native field operations
- Maximum 7 operands per instruction (hardware limit)