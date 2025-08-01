# RV32 Adapters - Component Index

## Core Adapter Types

### Memory Access Adapters

#### `eq_mod.rs` - Equality Modulo Adapter
- **Type**: `Rv32IsEqualModAdapterChip<F, NUM_READS, BLOCKS_PER_READ, BLOCK_SIZE, TOTAL_READ_SIZE>`
- **Purpose**: Compares memory blocks for equality with modular arithmetic support
- **Key Features**:
  - Configurable number of reads (1-2)
  - Block-based memory access
  - Address range checking with bitwise operations

#### `heap.rs` - Basic Heap Adapter
- **Type**: `Rv32HeapAdapterChip<F, NUM_READS, READ_SIZE, WRITE_SIZE>`
- **Purpose**: Direct heap memory read/write operations
- **Key Features**:
  - Simple pointer-based access
  - Configurable read/write sizes
  - Register-to-heap data movement

#### `heap_branch.rs` - Branching Heap Adapter
- **Type**: `Rv32HeapBranchAdapterChip<F, NUM_READS, READ_SIZE, WRITE_SIZE>`
- **Purpose**: Conditional heap operations with branching support
- **Key Features**:
  - Branch-aware memory access
  - Conditional execution paths
  - Extended heap adapter functionality

### Vectorized Adapters

#### `vec_heap.rs` - Vector Heap Adapter
- **Type**: `Rv32VecHeapAdapterChip<F, NUM_READS, BLOCKS_PER_READ, BLOCKS_PER_WRITE, READ_SIZE, WRITE_SIZE>`
- **Purpose**: Bulk memory operations with vectorized access patterns
- **Key Features**:
  - Multiple consecutive reads/writes
  - Configurable block counts
  - Efficient bulk data processing

#### `vec_heap_two_reads.rs` - Optimized Two-Read Vector Adapter
- **Type**: `Rv32VecHeapTwoReadsAdapterChip<F, BLOCKS_PER_READ, BLOCKS_PER_WRITE, READ_SIZE, WRITE_SIZE>`
- **Purpose**: Specialized adapter for exactly two read operations
- **Key Features**:
  - Optimized for binary operations
  - Fixed two-read pattern
  - Enhanced performance

## Supporting Components

### `test_utils.rs` - Testing Utilities
- **Purpose**: Helper functions for adapter testing
- **Key Functions**:
  - `write_ptr_reg`: Write pointer to register
  - `rv32_write_heap_default`: Setup heap test data
  - `rv32_write_heap_default_with_increment`: Setup with address increment

### `lib.rs` - Module Exports
- **Purpose**: Public API surface for the adapter library
- **Exports**: All adapter types and utilities

## Type Parameters

### Common Generic Parameters
- `F`: Field type (typically `BabyBear`)
- `NUM_READS`: Number of simultaneous read operations (1-2)
- `BLOCKS_PER_READ`: Number of consecutive blocks per read
- `BLOCKS_PER_WRITE`: Number of consecutive blocks per write
- `READ_SIZE`: Size of each read operation in field elements
- `WRITE_SIZE`: Size of each write operation in field elements
- `BLOCK_SIZE`: Size of individual memory blocks
- `TOTAL_READ_SIZE`: Total read size (computed from blocks × block size)

## Memory Layout

### Address Spaces
- **Register Space (AS=1)**: RV32 register file
- **Heap Space (AS=2)**: General heap memory

### Data Organization
- Registers: 4-limb representation (RV32_REGISTER_NUM_LIMBS = 4)
- Cell bits: 8 bits per cell (RV32_CELL_BITS = 8)
- Address validation through bitwise range checks

## Integration Interfaces

### Required Buses
- `ExecutionBus`: Instruction execution coordination
- `ProgramBus`: Program counter management
- `MemoryBridge`: Memory subsystem interface
- `BitwiseOperationLookupBus`: Auxiliary bitwise operations

### Record Types
- `Rv32IsEqualModReadRecord`: Tracks equality check reads
- `Rv32VecHeapReadRecord`: Tracks vectorized read operations
- `Rv32VecHeapWriteRecord`: Tracks vectorized write operations

## Trait Implementations
- `VmAdapterChip<F>`: Core adapter functionality
- `VmAdapterAir<AB>`: AIR constraint generation
- `BaseAir<F>`: Basic AIR properties