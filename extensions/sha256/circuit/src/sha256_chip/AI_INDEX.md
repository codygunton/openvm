# SHA256 Chip Component AI Index

## Purpose
This component implements a SHA256 hash function chip for the OpenVM zkVM. It provides hardware-accelerated SHA256 hashing with automatic padding and memory integration.

## Component Overview
The SHA256 chip handles full SHA256 hashing with padding for variable-length inputs read from VM memory. It integrates with the OpenVM execution bus and memory system to provide a seamless hashing operation as a custom instruction.

## Key Files

### Core Implementation
- `mod.rs` - Main chip implementation containing `Sha256VmChip` struct and instruction executor
- `air.rs` - Arithmetic Intermediate Representation (AIR) constraints for SHA256 operations
- `columns.rs` - Column layout definitions for the trace matrix
- `trace.rs` - Trace generation logic for proving SHA256 computations

### Parent Integration
- `../lib.rs` - Module exports and extension integration

## Main Components

### Sha256VmChip
The main chip struct that:
- Implements the `InstructionExecutor` trait
- Handles memory reads/writes for input/output
- Manages padding according to SHA256 specification
- Integrates with bitwise lookup tables for efficiency

### Sha256VmAir
The AIR (constraint system) that:
- Enforces correct SHA256 computation
- Handles message padding constraints
- Manages memory access patterns
- Integrates with the SHA256 core computation subair

### Column Structure
- `Sha256VmRoundCols` - Columns for SHA256 round computations
- `Sha256VmDigestCols` - Columns for final digest computation
- `Sha256VmControlCols` - Shared control columns for state management

## Key Features

### Memory Integration
- Reads input message from memory in 16-byte chunks
- Writes 32-byte digest output to memory
- Supports arbitrary message lengths with automatic padding

### Padding Implementation
- Implements full SHA256 padding (append 1 bit, zeros, and 64-bit length)
- Handles edge cases for messages spanning multiple blocks
- Efficient flag-based encoding for padding states

### Performance Optimizations
- Batched memory reads (16 bytes at a time)
- Shared bitwise operation lookup tables
- Parallelized trace generation

## Integration Points

### Instruction Format
- Custom RISC-V instruction: `sha256 rd, rs1, rs2`
  - `rd`: Destination pointer for 32-byte hash output
  - `rs1`: Source pointer for input message
  - `rs2`: Length of input message in bytes

### Memory Access Pattern
1. Read 3 registers (dst, src, len)
2. Read input message in 16-byte chunks
3. Process through SHA256 rounds
4. Write 32-byte digest to destination

### Constraints
- Maximum pointer size determined by `ptr_max_bits` (minimum 24 bits)
- Message length must fit in a field element (< 2^30 bytes)
- Addresses must be valid within the memory space

## Usage Example
```rust
// In guest code:
zkvm_sha256_impl(input_ptr, input_len, output_ptr);

// Transpiles to:
// sha256 output_ptr, input_ptr, input_len
```

## Dependencies
- `openvm-sha256-air` - Core SHA256 computation logic
- `openvm-circuit-primitives` - Bitwise operations and encoders
- `openvm-rv32im-circuit` - RISC-V register adapters
- `sha2` - Reference implementation for testing

## Architecture Notes
- Integrates as an extension to the OpenVM instruction set
- Uses no-CPU architecture allowing direct memory access
- Proves correct computation through STARK-based constraints
- Supports concurrent execution with other VM operations