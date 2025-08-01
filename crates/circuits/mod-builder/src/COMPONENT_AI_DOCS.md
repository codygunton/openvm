# OpenVM Modular Arithmetic Circuit Builder - AI Component Documentation

## Component Overview

The `openvm-mod-circuit-builder` is a specialized framework for building arithmetic circuits over large prime fields within the OpenVM ecosystem. It provides a high-level symbolic expression system that automatically manages limb-based arithmetic, overflow tracking, and constraint generation for cryptographic operations.

## Core Architecture

### Key Components

1. **ExprBuilder**: The central orchestrator that manages symbolic expressions, variables, and constraints
   - Tracks limb-based arithmetic with automatic overflow management
   - Manages constraint generation and range checking integration
   - Handles modular reduction and field arithmetic operations

2. **FieldVariable**: Represents field elements with associated arithmetic operations
   - Wraps symbolic expressions with metadata about limb structure
   - Provides overloaded operators for natural field arithmetic syntax
   - Implements lazy evaluation with automatic saving mechanisms

3. **SymbolicExpr**: Abstract syntax tree for field arithmetic expressions
   - Supports basic operations: Add, Sub, Mul, Div, IntAdd, IntMul
   - Handles conditional selection with Select variant
   - Maintains expression structure for optimization passes

4. **FieldExpressionCoreAir**: AIR implementation for constraint verification
   - Converts symbolic expressions to AIR constraints
   - Integrates with OpenVM's instruction execution model
   - Supports multi-opcode chips with setup operations

## Technical Implementation Details

### Limb-Based Arithmetic System

The component operates on a limb-based representation where large field elements are decomposed into smaller limbs:

```rust
pub struct ExprBuilderConfig {
    pub modulus: BigUint,      // The prime field modulus
    pub num_limbs: usize,      // Number of limbs per field element
    pub limb_bits: usize,      // Bits per limb (typically 8)
}
```

**Key Properties:**
- Automatic overflow tracking across all operations
- Lazy carry propagation for optimization
- Range checking integration for constraint satisfaction
- Support for negative intermediate values during computation

### Expression Management

The system uses a two-phase approach:
1. **Construction Phase**: Build symbolic expressions without evaluation
2. **Constraint Phase**: Generate AIR constraints from saved expressions

**Expression Lifecycle:**
- Expressions start as symbolic trees
- `.save()` converts expressions to variables and adds constraints
- Saved variables can be reused in subsequent expressions
- Final constraints ensure modular arithmetic correctness

### Overflow and Range Management

The builder automatically tracks overflow bounds:
- Each `FieldVariable` maintains `limb_max_abs` bounds
- Overflow bits are computed automatically: `max_overflow_bits = log2_ceil(limb_max_abs)`
- Range checker integration ensures all values stay within bounds
- Carry propagation handled transparently during constraint generation

## API Design Patterns

### Natural Arithmetic Syntax

```rust
// Standard field operations using overloaded operators
let result = a + b * c - d;  // Creates symbolic expression
let saved_result = result.save();  // Converts to constrained variable
```

### Conditional Operations

```rust
// Select one of two expressions based on a flag
let conditional = FieldVariable::select(flag_id, expr_true, expr_false);
```

### Division Handling

```rust
// Division automatically creates inverse constraints
let quotient = numerator / denominator;  // Panics if denominator is zero
```

## Integration Points

### With OpenVM Core
- Generates `VmCoreAir` implementations for instruction execution
- Integrates with opcode dispatch and trace generation
- Supports setup operations for stateful computations

### With Range Checker
- Automatic integration with `VariableRangeCheckerChip`
- Batched range checking for efficiency
- Configurable range decomposition parameters

### With Other Components
- Coordinate with other arithmetic components for consistent limb sizes
- Share modulus configuration across related operations
- Integrate with memory management for trace generation

## Performance Characteristics

### Optimization Features
- Lazy evaluation prevents unnecessary intermediate constraints
- Expression tree flattening for reduced constraint count
- Automatic subexpression sharing via `.save()`
- Batched range checking for improved efficiency

### Scalability Considerations
- Constraint count scales linearly with saved variables
- Memory usage proportional to expression tree depth
- Range checking cost amortized across operations
- Limb size affects both memory and constraint efficiency

## Security Properties

### Correctness Guarantees
- All field operations are properly constrained
- Overflow prevention through automatic bound tracking
- Modular reduction enforced at constraint level
- No unchecked arithmetic operations possible

### Attack Resistance
- Division by zero detection and prevention
- Overflow bounds verified through range checking
- Constraint completeness ensures no unconstrained values
- Modular arithmetic properties preserved throughout

## Common Usage Patterns

### Elliptic Curve Operations
- Point addition and doubling with slope calculations
- Coordinate transformations and projective arithmetic
- Batch processing of multiple curve operations

### Cryptographic Primitives
- Modular exponentiation with efficient squaring
- Inverse calculations with proper zero handling
- Hash function field arithmetic components

### Pairing Computations
- Miller loop line evaluations
- Final exponentiation steps
- Tower field arithmetic over extension fields

## Error Handling and Debugging

### Runtime Assertions
- Configuration validity checking in constructors
- Division by zero detection with panic
- Constraint satisfaction verification in debug builds

### Debugging Support
- Expression tree visualization through Display trait
- Overflow bound tracking for constraint debugging
- Range checking failure diagnostics

### Common Pitfalls
- Forgetting to save expressions before constraint generation
- Mixing incompatible limb configurations
- Assuming positive limb values during intermediate computation
- Manual carry computation instead of relying on automatic handling

## Configuration Guidelines

### Limb Size Selection
- 8-bit limbs optimal for most cryptographic applications
- Balance between constraint count and range checking efficiency
- Consider target field size when choosing limb count

### Range Checker Configuration
- Decomposition parameter affects constraint efficiency
- Bus allocation must coordinate with other components
- Maximum range bits must accommodate overflow bounds

### Performance Tuning
- Minimize division operations where possible
- Use `.save()` strategically for subexpression reuse
- Batch similar operations for improved efficiency
- Consider expression tree depth for optimization

## Component Dependencies

### Required Dependencies
- `openvm-stark-backend`: Core AIR and constraint system
- `openvm-circuit-primitives`: Range checking and bigint utilities
- `openvm-circuit`: VM integration and instruction handling
- `num-bigint`, `num-traits`: Big integer arithmetic support

### Optional Dependencies
- `halo2curves-axiom`: Standard elliptic curve implementations
- `openvm-pairing-guest`: Pairing-specific field operations

### Development Dependencies
- Test utilities for BN254 and BLS12-381 curves
- Property-based testing framework integration
- Benchmark suite for performance measurement

This component serves as the foundation for all modular arithmetic operations in OpenVM, providing a secure, efficient, and ergonomic interface for cryptographic circuit construction while maintaining the flexibility needed for custom field operations and optimizations.