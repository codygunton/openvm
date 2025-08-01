# AI Index - OpenVM Native Compiler IR Component

## Component Overview
The IR (Intermediate Representation) component provides a domain-specific language (DSL) for implementing programs that can compile to both recursive zkVM and R1CS/Plonk-ish circuits. It offers a flexible abstraction layer for cryptographic computations with support for native field elements, emulated field arithmetic, and extension field operations.

## Module Structure

### Core Modules
- **mod.rs** - Main module exposing public API and defining the central `Config` trait
- **types.rs** - Core type definitions for variables and witness structures
- **builder.rs** - Program construction API with control flow abstractions
- **instructions.rs** - Complete instruction set enumeration (`DslIr`)
- **var.rs** - Variable traits and memory interfaces

### Variable Types
- **Var<N>** - Native field element variables (for counters, loops)
- **Felt<F>** - Emulated field element variables (for recursive verification)
- **Ext<F,EF>** - Extension field element variables
- **Usize<N>** - Compile-time or runtime size variables

### Collections & Memory
- **collections.rs** - Array abstractions with compile/runtime variants
- **ptr.rs** - Pointer types for memory operations
- **symbolic.rs** - Symbolic representations for lazy evaluation

### Specialized Operations
- **bits.rs** - Bit manipulation operations
- **fri.rs** - FRI (Fast Reed-Solomon IOP) specific operations
- **poseidon.rs** - Poseidon hash function support
- **select.rs** - Conditional selection trait
- **verify_batch.rs** - Batch verification utilities

### Utilities
- **utils.rs** - Constants and helper functions (LIMB_BITS, NUM_LIMBS)

## Key Types

### Config Trait
```rust
pub trait Config: Clone + Default {
    type N: PrimeField;        // Native field
    type F: PrimeField32 + TwoAdicField;  // Target field
    type EF: ExtensionField<Self::F> + TwoAdicField; // Extension field
}
```

### Builder
- Main API for constructing IR programs
- Provides control flow (if/else, loops)
- Memory allocation and management
- Instruction emission with optional tracing

### DslIr Enum
Comprehensive instruction set including:
- Immediate assignments
- Arithmetic operations (add, sub, mul, div)
- Memory operations (load, store)
- Control flow (break, continue)
- Specialized operations (Poseidon, FRI)
- Debugging (print, assert)

## Design Patterns

### Variable Management
- All variables are identified by unique 32-bit IDs
- Three separate namespaces: Var, Felt, Ext
- Type safety through phantom types

### Memory Model
- Static allocation with compile-time sizes
- Dynamic allocation with runtime sizes
- Pointer-based access patterns

### Symbolic Execution
- Lazy evaluation through symbolic types
- Constant folding optimization
- Expression simplification

## Usage Context
This IR serves as the compilation target for higher-level OpenVM native programs, providing:
1. Type-safe field arithmetic operations
2. Memory management abstractions
3. Control flow primitives
4. Cryptographic operation support
5. Cross-compilation to different proof systems

## Dependencies
- openvm_stark_backend - Field definitions and backend traits
- Standard Rust collections and iterators
- Serde for serialization support