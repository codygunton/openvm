# RV32 Adapters - AI Documentation

## Overview
The rv32-adapters extension provides specialized memory access adapter chips for the OpenVM RISC-V 32-bit implementation. These adapters enable efficient data movement between RISC-V registers and heap memory with support for bulk operations and vectorized memory access patterns.

## Purpose
This component bridges the gap between RISC-V register operations and complex memory access patterns required by cryptographic and computational workloads. It provides optimized pathways for:
- Bulk memory reads/writes from heap storage
- Vectorized memory operations with configurable block sizes
- Pointer-based memory access from register values
- Efficient data comparison and equality checking

## Architecture
The adapters follow OpenVM's modular chip architecture:
- **AIR (Algebraic Intermediate Representation)**: Defines constraints for memory operations
- **Chip Implementation**: Handles runtime execution and trace generation
- **Memory Bridge Integration**: Seamlessly connects with OpenVM's memory subsystem

## Key Components

### 1. Eq Mod Adapter (`eq_mod.rs`)
- Compares data blocks from memory for equality
- Supports configurable read sizes and block counts
- Implements efficient modular equality checking

### 2. Heap Adapter (`heap.rs`)
- Basic heap memory read/write operations
- Supports up to 2 simultaneous read pointers
- Configurable read and write sizes

### 3. Heap Branch Adapter (`heap_branch.rs`)
- Conditional memory operations based on branching logic
- Extends heap adapter with branch-aware memory access

### 4. Vec Heap Adapter (`vec_heap.rs`)
- Vectorized memory operations for bulk data processing
- Supports multiple consecutive reads/writes
- Configurable block sizes for both read and write operations

### 5. Vec Heap Two Reads Adapter (`vec_heap_two_reads.rs`)
- Specialized variant optimized for exactly two read operations
- Enhanced performance for common two-operand patterns

## Integration Points
- **Memory Controller**: Manages address space separation (registers vs heap)
- **Execution Bridge**: Handles instruction execution and PC updates
- **Bitwise Operation Lookup**: Provides auxiliary operations for address calculations
- **RV32 Circuit Integration**: Seamlessly extends base RV32 instruction set

## Usage Context
These adapters are typically used when:
- Implementing cryptographic primitives requiring bulk memory access
- Processing large data structures in guest programs
- Optimizing memory-intensive algorithms
- Building custom instructions that operate on memory regions