# BigInt Primitives AI Documentation Index

This directory contains AI-focused documentation for the OpenVM BigInt Primitives component.

## Documentation Files

### [AI_DOCS.md](./AI_DOCS.md)
High-level architectural overview of the BigInt Primitives component, including:
- Core architecture and overflow representation strategy
- Key components (OverflowInt, SubAirs)
- Design principles and trade-offs
- Security considerations and performance characteristics

### [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md)
Detailed implementation patterns and code examples, including:
- OverflowInt construction patterns
- Arithmetic operation implementations
- SubAir integration patterns
- Carry propagation algorithms
- Common modular arithmetic patterns

### [CLAUDE.md](./CLAUDE.md)
Instructions for AI assistants working with this component, including:
- Key principles for BigInt operations
- Common pitfalls with overflow representations
- Testing requirements for arithmetic correctness
- Performance optimization guidelines

### [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md)
Concise reference for common operations:
- OverflowInt construction snippets
- Basic arithmetic operations
- Modular constraint patterns
- Key constants and utility functions

## Component Source Files

### Core Implementation
- [`mod.rs`](./mod.rs) - OverflowInt type and arithmetic operations
- [`check_carry_to_zero.rs`](./check_carry_to_zero.rs) - Zero constraint SubAir
- [`check_carry_mod_to_zero.rs`](./check_carry_mod_to_zero.rs) - Modular constraint SubAir
- [`utils.rs`](./utils.rs) - Helper functions and constants
- [`README.md`](./README.md) - Technical documentation on the algorithm

## Related Components

The BigInt Primitives interact with several other components:
- `../../range/` - Range checking for carry values
- `../../var_range/` - Variable range checking bus
- BigInt arithmetic chips (multiplication, division) that use these primitives
- Field extension chips that build on BigInt operations

## Quick Start

For AI assistants new to this component:
1. Start with [AI_DOCS.md](./AI_DOCS.md) for architectural understanding
2. Read [README.md](./README.md) for detailed algorithm explanation
3. Reference [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md) for common operations
4. Use [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md) for detailed patterns
5. Follow [CLAUDE.md](./CLAUDE.md) for best practices

## Key Concepts Summary

- **OverflowInt**: Representation allowing limbs to exceed canonical bounds
- **Carry Propagation**: Converting overflow form to canonical form
- **Modular Constraints**: Proving equations modulo a prime
- **SubAir Pattern**: Reusable constraint components
- **Range Checking**: Ensuring values fit in specified bit widths
- **Limb-based Arithmetic**: Breaking large integers into smaller chunks

## Component Responsibilities

1. **Overflow Representation**: Efficient intermediate calculation format
2. **Arithmetic Operations**: Add, subtract, multiply with overflow tracking
3. **Constraint Generation**: Zero and modular equality constraints
4. **Carry Management**: Propagating carries to canonical form
5. **Range Integration**: Interfacing with range check systems

## Common Use Cases

1. **RSA Operations**: Large modular exponentiations
2. **Elliptic Curves**: Field arithmetic for curve operations
3. **Pairing Operations**: Extension field arithmetic
4. **Hash Functions**: Large integer operations in hash algorithms
5. **Signature Verification**: Modular arithmetic in signature schemes