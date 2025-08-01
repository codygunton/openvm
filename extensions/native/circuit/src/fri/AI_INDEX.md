# FRI (Fast Reed-Solomon Interactive Oracle Proof) AI Documentation Index

This directory contains AI-focused documentation for the OpenVM FRI component.

## Documentation Files

### [AI_DOCS.md](./AI_DOCS.md)
High-level architectural overview of the FRI component, including:
- Core FRI reduction algorithm implementation
- Rolling hash computation for polynomial evaluation
- Three-phase execution model (workload, instruction1, instruction2)
- Integration with OpenVM's execution and memory systems

### [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md)
Detailed implementation patterns and code examples, including:
- FRI reduced opening instruction execution
- Trace generation and column layout
- Memory access patterns for a/b value processing
- AIR constraint implementation details
- Integration with hint streaming system

### [CLAUDE.md](./CLAUDE.md)
Instructions for AI assistants working with this component, including:
- Key principles for FRI implementation
- Common pitfalls with rolling hash computation
- Testing requirements for correctness
- Performance optimization guidelines

### [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md)
Concise reference for common operations:
- FRI instruction format and operands
- Column structure definitions
- Memory operation patterns
- Key constants and parameters

## Component Source Files

### Core Implementation
- [`mod.rs`](./mod.rs) - Main FRI reduced opening chip implementation
- [`tests.rs`](./tests.rs) - Comprehensive test suite

## Related Components

The FRI component interacts with several other components:
- `openvm_circuit::arch` - Core execution framework and memory management
- `openvm_circuit::system::memory` - Offline memory checking for reads/writes
- `field_extension` - Field extension arithmetic (degree 4)
- `openvm_native_compiler` - FRI opcodes and instruction encoding

## Quick Start

For AI assistants new to this component:
1. Start with [AI_DOCS.md](./AI_DOCS.md) for architectural understanding
2. Review [`mod.rs`](./mod.rs) for implementation details
3. Reference [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md) for common patterns
4. Use [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md) for detailed examples
5. Follow [CLAUDE.md](./CLAUDE.md) for best practices

## Key Concepts Summary

- **FRI Reduced Opening**: Core operation computing polynomial evaluation at a point
- **Rolling Hash**: Incremental computation of `sum(alpha^i * (b_i - a_i))`
- **Three-Phase Execution**: Workload rows → Instruction1 → Instruction2
- **Field Extension**: Operations over degree-4 extension field
- **Hint Streaming**: Optional initialization of a-values from hint stream

## Component Responsibilities

1. **Polynomial Evaluation**: Compute FRI reduced opening over field extension
2. **Memory Management**: Read/write a and b values with proper addressing
3. **State Tracking**: Maintain rolling hash result across workload rows
4. **Instruction Execution**: Handle FRI_REDUCED_OPENING opcode
5. **Trace Generation**: Produce valid execution trace for proving

## Common Use Cases

1. **FRI Protocol**: Core component of Fast Reed-Solomon IOP
2. **Polynomial Commitment**: Evaluate committed polynomial at query point
3. **STARK Verification**: Part of STARK proof generation/verification
4. **Field Extension Operations**: Batched operations over extension field

## Performance Characteristics

- **Height**: 2 + length (instruction rows + workload rows)
- **Width**: 27 columns maximum
- **Memory Accesses**: 5 reads for setup + 2*length for data
- **Computation**: O(length) field extension operations