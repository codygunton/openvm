# Native Circuit Adapters AI Documentation Index

This directory contains AI-focused documentation for the OpenVM Native Circuit Adapters component.

## Documentation Files

### [AI_DOCS.md](./AI_DOCS.md)
High-level architectural overview of the Native Circuit Adapters component, including:
- Core adapter types and their responsibilities
- Memory access patterns and instruction handling
- Integration with the OpenVM execution framework
- Design principles for native field operations

### [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md)
Detailed implementation patterns and code examples, including:
- Creating custom adapter chips
- Memory read/write patterns
- Instruction decoding and execution
- Air constraint implementation
- Integration with core VM components

### [CLAUDE.md](./CLAUDE.md)
Instructions for AI assistants working with this component, including:
- Key principles for adapter implementation
- Common pitfalls with memory operations
- Testing requirements for adapter correctness
- Performance optimization guidelines

### [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md)
Concise reference for common operations:
- Adapter chip construction patterns
- Memory operation snippets
- Instruction handling templates
- Key traits and interfaces

## Component Source Files

### Core Adapters
- [`alu_native_adapter.rs`](./alu_native_adapter.rs) - ALU operations adapter (2 reads, 1 write, immediate support)
- [`branch_native_adapter.rs`](./branch_native_adapter.rs) - Branch operations adapter (2 reads, jump support)
- [`convert_adapter.rs`](./convert_adapter.rs) - Type conversion adapter (1 read, 1 write, variable sizes)
- [`loadstore_native_adapter.rs`](./loadstore_native_adapter.rs) - Load/store operations adapter
- [`native_vectorized_adapter.rs`](./native_vectorized_adapter.rs) - Vectorized operations adapter (2 reads, 1 write)
- [`mod.rs`](./mod.rs) - Module exports

## Related Components

The Native Circuit Adapters interact with several other components:
- `openvm_circuit::arch` - Core VM adapter interfaces and execution framework
- `openvm_circuit::system::memory` - Memory subsystem for read/write operations
- `openvm_circuit::system::program` - Program bus for instruction fetching
- Native extension chips (field arithmetic, etc.) that use these adapters

## Quick Start

For AI assistants new to this component:
1. Start with [AI_DOCS.md](./AI_DOCS.md) for architectural understanding
2. Review specific adapter files for implementation details
3. Reference [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md) for common patterns
4. Use [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md) for detailed examples
5. Follow [CLAUDE.md](./CLAUDE.md) for best practices

## Key Concepts Summary

- **VmAdapterChip**: Bridge between VM execution and specialized operations
- **ExecutionBridge**: Manages instruction execution and state transitions
- **MemoryBridge**: Handles memory read/write operations with proper verification
- **AdapterInterface**: Defines reads, writes, and instruction format
- **Air Constraints**: Algebraic constraints ensuring correct execution

## Component Responsibilities

1. **Instruction Decoding**: Parse VM instructions for native operations
2. **Memory Management**: Handle reads/writes with proper addressing
3. **State Transitions**: Update PC and execution state correctly
4. **Constraint Generation**: Produce AIR constraints for proof generation
5. **Bus Integration**: Connect to execution, program, and memory buses

## Common Use Cases

1. **Field Arithmetic**: Native field operations (add, mul, etc.)
2. **Branching Logic**: Conditional jumps and comparisons
3. **Data Movement**: Loading/storing native field elements
4. **Type Conversions**: Converting between different representations
5. **Vector Operations**: Batched operations on multiple field elements

## Adapter Types Overview

### ALU Native Adapter
- **Purpose**: Arithmetic and logic operations
- **Reads**: 2 (operands from memory or immediates)
- **Writes**: 1 (result to memory)
- **Features**: Immediate value support

### Branch Native Adapter  
- **Purpose**: Conditional branching and jumps
- **Reads**: 2 (comparison values)
- **Writes**: 0
- **Features**: PC modification, immediate support

### Convert Adapter
- **Purpose**: Type conversions and resizing
- **Reads**: 1 (source value)
- **Writes**: 1 (converted value)
- **Features**: Variable read/write sizes

### LoadStore Native Adapter
- **Purpose**: Memory load/store operations
- **Reads**: Variable based on operation
- **Writes**: Variable based on operation
- **Features**: Configurable element size

### Native Vectorized Adapter
- **Purpose**: Vector operations on multiple elements
- **Reads**: 2 (vector operands)
- **Writes**: 1 (vector result)
- **Features**: Fixed vector size N