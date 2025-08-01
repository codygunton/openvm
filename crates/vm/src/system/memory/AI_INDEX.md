# OpenVM Memory System Index

## Core Module Structure

### Root Module (`mod.rs`)
- **Types**: `OpType`, `MemoryAddress`, `HeapAddress`
- **Re-exports**: Controller, offline memory, paged vectors
- **Location**: `crates/vm/src/system/memory/mod.rs`

### Memory Implementations

#### Online Memory (`online.rs`)
- **Struct**: `Memory<F>` - Runtime memory with logging
- **Enum**: `MemoryLogEntry<T>` - Log entry types
- **Key Methods**: `read()`, `write()`, `increment_timestamp_by()`
- **Location**: `crates/vm/src/system/memory/online.rs`

#### Offline Memory (`offline.rs`)
- **Struct**: `OfflineMemory<F>` - Proof generation memory
- **Struct**: `MemoryRecord<T>` - Memory access records
- **Struct**: `BlockMap` - Block management system
- **Constants**: `INITIAL_TIMESTAMP`
- **Location**: `crates/vm/src/system/memory/offline.rs`

### Memory Types

#### Persistent Memory (`persistent.rs`)
- **Struct**: `PersistentBoundaryChip<F, CHUNK>` - Persistent memory proof chip
- **Struct**: `PersistentBoundaryCols<T, CHUNK>` - Trace columns
- **Air**: `PersistentBoundaryAir<CHUNK>` - AIR constraints
- **Location**: `crates/vm/src/system/memory/persistent.rs`

#### Volatile Memory (`volatile/mod.rs`)
- **Struct**: `VolatileBoundaryChip<F>` - Volatile memory proof chip
- **Struct**: `VolatileBoundaryCols<T>` - Trace columns
- **Air**: `VolatileBoundaryAir` - AIR constraints
- **Tests**: `volatile/tests.rs`
- **Location**: `crates/vm/src/system/memory/volatile/`

### Supporting Components

#### Access Adapters (`adapter/`)
- **Struct**: `AccessAdapterInventory<F>` - Manages all adapters
- **Struct**: `AccessAdapterChip<F, N>` - Adapter for size N
- **Enum**: `AccessAdapterRecordKind` - Split/Merge operations
- **Files**:
  - `adapter/mod.rs` - Main adapter logic
  - `adapter/air.rs` - AIR constraints
  - `adapter/columns.rs` - Trace column definitions
  - `adapter/tests.rs` - Unit tests

#### Memory Controller (`controller/`)
- **Note**: Has separate AI documentation
- **Files**:
  - `controller/mod.rs`
  - `controller/dimensions.rs`
  - `controller/interface.rs`
  - Controller AI docs: `AI_DOCS.md`, `AI_INDEX.md`, etc.

#### Merkle Tree (`merkle/`)
- **Note**: Has separate AI documentation
- **Files**:
  - `merkle/mod.rs`
  - `merkle/air.rs`
  - `merkle/columns.rs`
  - `merkle/trace.rs`
  - `merkle/tests/`
  - Merkle AI docs: `AI_DOCS.md`, `AI_INDEX.md`, etc.

#### Offline Checker (`offline_checker/`)
- **Struct**: `MemoryBus` - Memory operation bus
- **Struct**: `MemoryBridge` - Bridge to range checker
- **Files**:
  - `offline_checker/mod.rs`
  - `offline_checker/bridge.rs`
  - `offline_checker/bus.rs`
  - `offline_checker/columns.rs`

#### Paged Vectors (`paged_vec.rs`)
- **Struct**: `PagedVec<T, PAGE_SIZE>` - Paged storage
- **Struct**: `AddressMap<T, PAGE_SIZE>` - Address-indexed storage
- **Constant**: `PAGE_SIZE = 1024`
- **Location**: `crates/vm/src/system/memory/paged_vec.rs`

#### Memory Tree (`tree/`)
- **Enum**: `MemoryNode<CHUNK, F>` - Tree node types
- **Module**: `public_values` - Public value handling
- **Files**:
  - `tree/mod.rs` - Tree construction
  - `tree/public_values.rs` - Public values

### Test Files
- `tests.rs` - Integration tests
- `adapter/tests.rs` - Adapter unit tests
- `volatile/tests.rs` - Volatile memory tests
- `merkle/tests/` - Merkle tree tests

## Key Interfaces

### Memory Trait Hierarchy
```
Memory Operations
├── Online Memory (Memory<F>)
│   ├── read<N>()
│   ├── write<N>()
│   └── increment_timestamp_by()
├── Offline Memory (OfflineMemory<F>)
│   ├── read()
│   ├── write()
│   ├── finalize<N>()
│   └── increment_timestamp()
└── Memory Image (MemoryImage<F>)
    ├── insert()
    ├── get()
    └── items()
```

### Chip Hierarchy
```
Memory Chips
├── PersistentBoundaryChip
├── VolatileBoundaryChip
├── AccessAdapterChip<N>
├── OfflineCheckerChip
├── MemoryControllerChip (in controller/)
└── MerkleChip (in merkle/)
```

## Configuration Types

### MemoryConfig (from arch module)
- `as_offset`: Address space offset
- `clk_max_bits`: Clock/timestamp bits
- `access_capacity`: Expected accesses
- `max_access_adapter_n`: Max adapter size

### MemoryDimensions (in controller/)
- Address space and pointer dimensions
- Tree height calculations

## Common Patterns

### Memory Access Pattern
1. Request → Online Memory → Log
2. Request → Offline Memory → Block Access → Adapter → Record
3. Finalize → Equipartition → Merkle Tree

### Proof Generation Pattern
1. Collect memory records
2. Generate adapter traces
3. Generate boundary traces
4. Verify with offline checker

## Dependencies

### Internal Dependencies
- `openvm_circuit_primitives`: Basic circuits
- `openvm_stark_backend`: STARK proof system
- `crate::arch`: Architecture definitions

### External Dependencies
- `rustc_hash`: Fast hashing
- `itertools`: Iterator utilities
- `static_assertions`: Compile-time checks