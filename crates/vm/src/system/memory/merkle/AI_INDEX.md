# Merkle Memory System Component Index

## Core Module Files

### mod.rs
**Location**: `crates/vm/src/system/memory/merkle/mod.rs`
- Main module definition and exports
- `MemoryMerkleChip` struct implementation
- Core chip logic for Merkle tree operations
- Touch node tracking and finalization

### air.rs
**Location**: `crates/vm/src/system/memory/merkle/air.rs`
- `MemoryMerkleAir` arithmetization implementation
- Constraint definitions for Merkle tree operations
- Interaction evaluation with buses
- Public value constraints

### columns.rs
**Location**: `crates/vm/src/system/memory/merkle/columns.rs`
- `MemoryMerkleCols`: Trace column layout
- `MemoryMerklePvs`: Public values structure
- Field definitions for proof generation

### trace.rs
**Location**: `crates/vm/src/system/memory/merkle/trace.rs`
- Trace generation implementation
- Tree construction and traversal logic
- `SerialReceiver` trait for hash recording
- Proof input generation

## Test Files

### tests/mod.rs
**Location**: `crates/vm/src/system/memory/merkle/tests/mod.rs`
- Comprehensive test suite
- Random test generation
- Edge case testing
- Integration with proof system

### tests/util.rs
**Location**: `crates/vm/src/system/memory/merkle/tests/util.rs`
- Test utilities and mock implementations
- `HashTestChip` for deterministic testing
- Helper functions for test setup

## Key Data Structures

### MemoryMerkleChip<CHUNK, F>
- Main chip managing Merkle operations
- Fields:
  - `air`: Arithmetization rules
  - `touched_nodes`: Set of accessed nodes
  - `num_touched_nonleaves`: Performance counter
  - `final_state`: Computed tree state
  - `overridden_height`: Optional height override

### MemoryMerkleAir<CHUNK>
- Arithmetization for Merkle operations
- Fields:
  - `memory_dimensions`: Memory layout config
  - `merkle_bus`: Memory interaction bus
  - `compression_bus`: Hash computation bus

### MemoryMerkleCols<T, CHUNK>
- Trace row structure
- Fields:
  - `expand_direction`: State indicator (-1, 0, 1)
  - `height_section`: AS vs address expansion flag
  - `parent_height`: Node level in tree
  - `is_root`: Root node indicator
  - `parent_as_label`: Address space label
  - `parent_address_label`: Address label
  - `parent_hash`: Node hash value
  - `left_child_hash`: Left child hash
  - `right_child_hash`: Right child hash
  - `left_direction_different`: Optimization flag
  - `right_direction_different`: Optimization flag

### MemoryMerklePvs<T, CHUNK>
- Public values for proof
- Fields:
  - `initial_root`: Starting Merkle root
  - `final_root`: Ending Merkle root

## Helper Structures

### FinalState<CHUNK, F>
- Internal state after finalization
- Contains trace rows and root hashes

### TreeHelper<CHUNK, F>
- Recursive tree construction helper
- Manages trace row generation
- Handles touched node tracking

## Traits and Interfaces

### SerialReceiver<T>
- Interface for receiving hash operations
- Used by hasher chips for recording

### Chip Implementation
- Implements `Chip<SC>` for proof generation
- Implements `ChipUsageGetter` for metrics

## Integration Points

### External Dependencies
- `openvm_stark_backend`: Core proving system
- `openvm_circuit_primitives_derive`: Derive macros
- Memory controller module
- Tree module for `MemoryNode`
- Hasher module for hash operations

### Bus Connections
- Merkle bus: Memory access coordination
- Compression bus: Hash computation requests

## Constants and Configuration

### CHUNK
- Default chunk size (typically 8)
- Configurable per instantiation

### Memory Dimensions
- Configured via `MemoryDimensions`
- Determines tree structure

## Testing Infrastructure

### Test Constants
- `DEFAULT_CHUNK`: Standard chunk size for tests
- `COMPRESSION_BUS`: Test bus configuration
- `MEMORY_MERKLE_BUS`: Test memory bus

### Test Functions
- `test()`: Core test harness
- `random_test()`: Randomized testing
- `memory_to_partition()`: Test helper

## Usage Patterns

### Initialization
```rust
let chip = MemoryMerkleChip::new(dimensions, merkle_bus, compression_bus);
```

### Touch and Finalize
```rust
chip.touch_range(address_space, address, length);
chip.finalize(&initial_tree, &final_memory, &mut hasher);
```

### Proof Generation
```rust
let air = chip.air();
let proof_input = chip.generate_air_proof_input();
```