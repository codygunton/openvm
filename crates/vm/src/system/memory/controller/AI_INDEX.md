# Memory Controller AI Documentation Index

This directory contains AI-focused documentation for the OpenVM Memory Controller component.

## Documentation Files

### [AI_DOCS.md](./AI_DOCS.md)
High-level architectural overview of the Memory Controller component, including:
- Core architecture and responsibilities
- Key components and their interactions
- Memory modes (volatile vs persistent)
- Integration points with other systems

### [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md)
Detailed implementation patterns and code examples, including:
- Construction patterns for different memory modes
- Memory access patterns and best practices
- State machine model and transitions
- Performance optimization techniques
- Common integration patterns

### [CLAUDE.md](./CLAUDE.md)
Instructions for AI assistants working with this component, including:
- Key principles and guidelines
- Common pitfalls to avoid
- Testing requirements and security checklist
- Performance targets and debugging tips

### [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md)
Concise reference for common operations:
- Construction snippets
- Basic read/write operations
- Proof generation flow
- Key types and constants

## Component Source Files

### Core Implementation
- [`mod.rs`](./mod.rs) - Main MemoryController implementation
- [`interface.rs`](./interface.rs) - Memory interface abstraction
- [`dimensions.rs`](./dimensions.rs) - Memory dimension calculations

## Related Components

The Memory Controller interacts with several other components:
- `../offline/` - Offline memory for proof generation
- `../online/` - Online memory for execution
- `../adapter/` - Access pattern optimizations
- `../merkle/` - Merkle tree for persistent memory
- `../volatile/` - Volatile memory boundary conditions
- `../persistent/` - Persistent memory boundary conditions

## Quick Start

For AI assistants new to this component:
1. Start with [AI_DOCS.md](./AI_DOCS.md) for architectural understanding
2. Reference [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md) for common operations
3. Use [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md) for detailed patterns
4. Follow [CLAUDE.md](./CLAUDE.md) for best practices

## Key Concepts Summary

- **Memory Controller**: Central coordinator for all memory operations in OpenVM
- **Volatile vs Persistent**: Two modes supporting different use cases
- **Record IDs**: Track memory operations for proof generation
- **Timestamps**: Ensure causal ordering of memory accesses
- **Access Adapters**: Optimize proof size for common access patterns
- **Finalization**: Required step before proof generation

## Component Responsibilities

1. **Memory Management**: Read/write operations with validation
2. **Proof Generation**: Create AIR proofs for memory accesses
3. **State Tracking**: Maintain consistency between online/offline memory
4. **Performance**: Optimize access patterns and proof generation
5. **Security**: Ensure all accesses are authenticated and bounded