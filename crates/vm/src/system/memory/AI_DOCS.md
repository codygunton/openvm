# OpenVM Memory System Documentation

## Overview

The OpenVM memory system is a sophisticated zero-knowledge memory implementation that provides efficient memory access with cryptographic verification. It supports multiple address spaces, timestamped memory operations, and Merkle tree-based memory verification.

## Architecture

The memory system consists of several key components:

### Core Types

- **`MemoryAddress<S, T>`**: Full pointer to a memory location containing an address space and pointer
- **`OpType`**: Memory operation types (Read/Write)
- **`HeapAddress<S, T>`**: Specialized address type for heap operations

### Memory Implementations

#### 1. Online Memory (`online.rs`)
- Simple memory structure for execution
- Maintains a log of memory accesses
- Uses `AddressMap` for efficient storage
- Tracks timestamps for each operation

#### 2. Offline Memory (`offline.rs`)
- Complex memory system for proof generation
- Implements timestamped block-based memory management
- Supports memory access adapters for different block sizes
- Maintains memory records for verification

Key features:
- **Block Management**: Dynamic block splitting and merging
- **Timestamp Tracking**: Every memory access is timestamped
- **Access Adapters**: Handles memory accesses of sizes 2, 4, 8, 16, 32
- **Equipartition**: Finalizes memory into fixed-size blocks for verification

#### 3. Persistent Memory (`persistent.rs`)
- Handles memory that persists across execution boundaries
- Implements boundary constraints for initial and final memory states
- Integrates with Merkle tree verification
- Uses compression bus for hash computation

#### 4. Volatile Memory (`volatile/`)
- Memory that doesn't persist between executions
- Sorted address verification
- Simpler than persistent memory (no Merkle tree integration)

### Supporting Components

#### Memory Controller (`controller/`)
- Already documented in separate AI_DOCS.md
- Manages memory operations and routing

#### Merkle Tree (`merkle/`)
- Already documented in separate AI_DOCS.md
- Provides cryptographic verification of memory state

#### Access Adapters (`adapter/`)
- Handles memory accesses of different sizes (2, 4, 8, 16, 32)
- Manages split and merge operations for block alignment
- Generates proof traces for memory operations

#### Offline Checker (`offline_checker/`)
- **Memory Bus**: Interaction bus for memory operations
- **Memory Bridge**: Connects memory operations with range checking
- Verifies memory access patterns and timestamps

#### Tree Module (`tree/`)
- Constructs Merkle trees from memory images
- Supports uniform tree construction
- Handles sparse memory efficiently

## Key Concepts

### 1. Timestamped Memory
Every memory operation has an associated timestamp:
- Initial timestamp is 0
- Timestamps increment with each operation
- Used to verify correct ordering of memory accesses

### 2. Block-Based Management
Memory is managed in power-of-two sized blocks:
- Initial block size is configurable (typically 1, 2, 4, or 8)
- Blocks can be split or merged as needed
- Ensures efficient memory access patterns

### 3. Address Spaces
- Address space 0 is special (identity mapping)
- Each address space has independent memory
- Configured via `MemoryConfig`

### 4. Memory Image
- Represents the initial state of memory
- Can be loaded from external sources
- Converted to paged vectors for efficient access

### 5. Equipartition
- Final memory state must be in fixed-size blocks
- Required for Merkle tree construction
- Enables efficient verification

## Memory Access Flow

1. **Read/Write Request**: Application requests memory access
2. **Online Memory**: Updates internal state and logs operation
3. **Offline Memory**: 
   - Finds or creates appropriate block
   - Updates timestamps
   - Records operation for proof generation
4. **Access Adapters**: Handle block splitting/merging if needed
5. **Memory Bus**: Broadcasts operation for verification
6. **Proof Generation**: Creates execution trace from records

## Proof Generation

The memory system generates proofs through several chips:

1. **Persistent Boundary Chip**: Proves initial/final memory states
2. **Volatile Boundary Chip**: Proves volatile memory consistency
3. **Access Adapter Chips**: Prove block operations
4. **Offline Checker Chip**: Verifies memory bus consistency

## Configuration

Memory system is configured via `MemoryConfig`:
- `as_offset`: Offset for address space numbering
- `clk_max_bits`: Maximum bits for timestamps
- `access_capacity`: Expected number of memory accesses
- `max_access_adapter_n`: Maximum adapter size (2, 4, 8, 16, or 32)

## Usage Patterns

### Basic Memory Access
```rust
// Online memory (execution)
let (record_id, values) = memory.read::<4>(address_space, pointer);
let (record_id, prev_values) = memory.write::<4>(address_space, pointer, new_values);

// Offline memory (proof generation)
offline_memory.read(address_space, pointer, len, &mut adapters);
offline_memory.write(address_space, pointer, values, &mut adapters);
```

### Memory Initialization
```rust
let initial_memory = MemoryImage::new(/* ... */);
let memory = Memory::from_image(initial_memory, access_capacity);
```

### Finalization for Proofs
```rust
let final_memory = offline_memory.finalize::<CHUNK_SIZE>(&mut adapters);
persistent_chip.finalize(&initial_memory, &final_memory, &mut hasher);
```

## Performance Considerations

1. **Block Size Selection**: Larger initial blocks reduce adapter overhead
2. **Access Patterns**: Sequential accesses are more efficient
3. **Address Space Usage**: Minimize number of active address spaces
4. **Timestamp Management**: Batch operations when possible

## Security Properties

1. **Timestamp Monotonicity**: Timestamps always increase
2. **Memory Consistency**: Reads return last written value
3. **Merkle Tree Binding**: Memory state is cryptographically committed
4. **Access Pattern Privacy**: Proof doesn't reveal access patterns

## Integration Points

- **VM Execution**: Via `Memory` struct in execution engine
- **Proof System**: Through various memory chips
- **Hasher**: For Merkle tree construction
- **Range Checker**: For value range verification