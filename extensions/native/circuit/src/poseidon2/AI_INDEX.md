# Native Poseidon2 AI Documentation Index

This directory contains AI-focused documentation for the OpenVM Native Poseidon2 component.

## Documentation Files

### [AI_DOCS.md](./AI_DOCS.md)
High-level architectural overview of the Native Poseidon2 component, including:
- Core instruction set (PERM_POS2, COMP_POS2, VERIFY_BATCH)
- Row types and execution model
- Memory access patterns and integration points
- Performance and security considerations

### [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md)
Detailed implementation patterns and code examples, including:
- Adding new Poseidon2-based instructions
- Implementing custom row types
- Memory operation patterns
- AIR constraint development
- Testing strategies for cryptographic operations

### [CLAUDE.md](./CLAUDE.md)
Instructions for AI assistants working with this component, including:
- Critical correctness requirements for cryptographic code
- Memory consistency invariants
- Performance optimization guidelines
- Common implementation pitfalls
- Testing requirements

### [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md)
Concise reference for common operations:
- Instruction encoding formats
- Row type selection logic
- Memory access helpers
- Constraint patterns
- Test utilities

### [README.md](./README.md)
Technical specification document describing:
- VERIFY_BATCH instruction specification
- Merkle tree verification algorithm
- Row layout and column usage
- Implementation details

## Component Source Files

### Core Implementation
- [`chip.rs`](./chip.rs) - Main chip implementation with instruction executors
- [`air.rs`](./air.rs) - AIR constraints for correctness
- [`columns.rs`](./columns.rs) - Trace column definitions
- [`trace.rs`](./trace.rs) - Trace generation logic
- [`mod.rs`](./mod.rs) - Module exports and constants
- [`tests.rs`](./tests.rs) - Comprehensive test suite

### Key Data Structures

#### Record Types
- `VerifyBatchRecord` - Full VERIFY_BATCH execution record
- `SimplePoseidonRecord` - PERM/COMP operation record
- `TopLevelRecord` - Merkle tree operation record
- `InsideRowRecord` - Row hashing operation record

#### Column Structures
- `NativePoseidon2Cols` - Main column layout
- `TopLevelSpecificCols` - Merkle-specific columns
- `InsideRowSpecificCols` - Row hash columns
- `SimplePoseidonSpecificCols` - Basic operation columns

## Related Components

The Native Poseidon2 component interacts with:
- `openvm_poseidon2_air` - Core Poseidon2 cryptographic implementation
- `openvm_circuit::arch` - Execution framework and instruction handling
- `openvm_circuit::system::memory` - Memory subsystem integration
- `openvm_native_compiler` - Instruction opcodes and compilation

## Key Concepts

### Execution Flow
1. Instruction fetch via ExecutionBridge
2. Operand dereferencing from memory
3. Row-specific computation (Simple/TopLevel/InsideRow)
4. Result writing to memory
5. PC update and state transition

### VERIFY_BATCH Algorithm
1. Initialize with first opened value hash
2. For each height level:
   - Concatenate and hash opened values
   - Compress with current node
   - Incorporate Merkle sibling based on path bit
3. Verify final node equals commitment

### Performance Patterns
- Batch verification amortizes overhead
- Contiguous row blocks for cache efficiency
- Internal bus minimizes memory traffic
- Polymorphic columns reduce trace width