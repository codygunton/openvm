# Modular Chip AI Documentation Index

This directory contains AI-focused documentation for the OpenVM Modular Chip component.

## Documentation Files

### [AI_DOCS.md](./AI_DOCS.md)
High-level architectural overview of the Modular Chip component, including:
- Core architecture and modular arithmetic strategy
- Key components (ModularIsEqualChip, ModularAddSubChip, ModularMulDivChip)
- Design principles and constraint systems
- Security considerations and soundness proofs

### [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md)
Detailed implementation patterns and code examples, including:
- Chip construction patterns
- Field expression builder usage
- Adapter integration patterns
- Constraint implementation examples
- Common modular arithmetic patterns

### [CLAUDE.md](./CLAUDE.md)
Instructions for AI assistants working with this component, including:
- Key principles for modular arithmetic operations
- Common pitfalls with modular constraints
- Testing requirements for arithmetic correctness
- Performance optimization guidelines

### [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md)
Concise reference for common operations:
- Chip instantiation snippets
- Basic arithmetic operations (ADD, SUB, MUL, DIV)
- Equality check patterns
- Setup operation usage

## Component Source Files

### Core Implementation
- [`mod.rs`](./mod.rs) - Module exports and type aliases
- [`addsub.rs`](./addsub.rs) - Addition and subtraction chip implementation
- [`muldiv.rs`](./muldiv.rs) - Multiplication and division chip implementation
- [`is_eq.rs`](./is_eq.rs) - Modular equality checking chip
- [`tests.rs`](./tests.rs) - Unit tests for modular operations
- [`README.md`](./README.md) - Mathematical proof of soundness for ModularIsEqualCoreChip

## Related Components

The Modular Chip interacts with several other components:
- `openvm_mod_circuit_builder` - Field expression building framework
- `openvm_rv32_adapters` - RISC-V 32-bit adapters for instruction execution
- `openvm_circuit_primitives` - Variable range checking and bitwise operations
- `openvm_algebra_transpiler` - Instruction opcodes and transpiler extensions

## Key Concepts

### Modular Arithmetic Operations
The component implements efficient modular arithmetic for arbitrary moduli:
- **ADD/SUB**: Addition and subtraction modulo N
- **MUL/DIV**: Multiplication and division modulo N  
- **IS_EQ**: Equality checking with modular constraint verification

### Setup Operations
Each arithmetic operation has a corresponding SETUP variant:
- SETUP_ADDSUB: Prepares modulus for addition/subtraction
- SETUP_MULDIV: Prepares modulus for multiplication/division
- SETUP_ISEQ: Prepares modulus for equality checking

### Constraint System
The chips use sophisticated constraint systems to ensure:
- All operations are performed modulo the specified modulus
- Intermediate values are properly range-checked
- Soundness is maintained through mathematical proofs