# OpenVM Modular Arithmetic Circuit Builder

## Overview

The `openvm-mod-circuit-builder` crate provides a modular arithmetic framework for building OpenVM circuits. It enables efficient operations on large field elements by representing them as vectors of smaller limbs and handling overflow/carry propagation automatically.

## Purpose

This crate addresses the need for:
- Efficient modular arithmetic on fields larger than the native field
- Automatic handling of overflow and carry propagation
- Symbolic expression building for complex field operations
- Integration with OpenVM's constraint system

## Architecture

### Core Components

1. **ExprBuilder** (`builder.rs`)
   - Central orchestrator for building modular arithmetic expressions
   - Manages field configuration (modulus, limb size, number of limbs)
   - Tracks variables, constraints, and intermediate computations
   - Handles overflow and carry calculations

2. **FieldVariable** (`field_variable.rs`)
   - Represents field elements in the computation
   - Tracks overflow bounds and carry information
   - Provides arithmetic operations (add, sub, mul, div)
   - Manages expression evaluation and saving

3. **SymbolicExpr** (`symbolic_expr.rs`)
   - AST representation of field arithmetic expressions
   - Supports operations: Add, Sub, Mul, Div, IntAdd, IntMul, Select
   - Handles expression evaluation to BigInt/OverflowInt
   - Provides constraint generation for the proving system

4. **FieldExpressionCoreAir** (`core_chip.rs`)
   - AIR (Algebraic Intermediate Representation) implementation
   - Integrates with OpenVM's constraint system
   - Handles opcode execution and trace generation
   - Manages interaction with range checker

### Key Design Principles

1. **Limb-based Representation**
   - Large field elements split into smaller limbs
   - Each limb fits within the native field
   - Configurable limb size (typically 8 bits for efficiency)

2. **Overflow Handling**
   - Tracks maximum absolute value for each limb
   - Computes carry bounds automatically
   - Ensures values stay within representable range

3. **Lazy Evaluation**
   - Expressions built symbolically first
   - Evaluation deferred until needed
   - Intermediate values saved to optimize constraints

4. **Modular Design**
   - Clear separation between expression building and evaluation
   - Pluggable into OpenVM's chip architecture
   - Reusable for different field implementations

## Integration with OpenVM

### Chip Architecture

The mod-builder integrates as a VmCoreChip:
- Implements custom opcodes for field operations
- Generates execution traces for the prover
- Interacts with range checker for constraint verification

### Constraint System

1. **Expression Constraints**
   - Each saved expression becomes a constraint
   - Automatic quotient and remainder computation
   - Carry propagation constraints added as needed

2. **Range Checking**
   - Integrates with VariableRangeCheckerChip
   - Ensures limbs stay within valid bounds
   - Handles both positive and negative values

### Memory Model

- Field elements stored as arrays of limbs
- Intermediate values tracked in builder state
- Efficient memory layout for trace generation

## Usage Patterns

### Basic Field Operations

```rust
// Create a field variable
let mut a = builder.new_var();
let mut b = builder.new_var();

// Perform operations
let c = &a + &b;  // Addition
let d = &a * &b;  // Multiplication
let e = &c - &d;  // Subtraction

// Save result (creates constraint)
let result_idx = e.save();
```

### Complex Expressions

```rust
// Build complex expression
let expr = (&x * &y + &z) / &w;

// Conditional selection
let result = builder.select(flag_idx, expr1, expr2);
```

### Integration Example

```rust
// Setup
let config = ExprBuilderConfig {
    modulus: prime.clone(),
    limb_bits: 8,
    num_limbs: 32,
};
let builder = ExprBuilder::new(config, range_max_bits);

// Build constraints
builder.constrain_eq(expr1, expr2);

// Generate AIR
let air = builder.generate_air(offset, opcode_indices);
```

## Performance Considerations

1. **Limb Size Selection**
   - Smaller limbs = more limbs but simpler range checks
   - Typically 8 bits provides good balance
   - Must fit multiplication results in native field

2. **Expression Complexity**
   - Deeper expressions increase carry bounds
   - Strategic saving of intermediates reduces constraints
   - Division operations are expensive

3. **Memory Usage**
   - Each saved variable requires trace columns
   - Minimize intermediate variables when possible
   - Reuse expressions where appropriate

## Testing Support

The crate provides test utilities for:
- BN254 and BLS12-381 curve arithmetic
- Random field element generation
- Limb conversion utilities
- Example implementations

## Security Notes

1. **Overflow Prevention**
   - Automatic tracking prevents silent overflows
   - Range checks ensure values stay valid
   - Carry propagation handled correctly

2. **Modular Reduction**
   - All operations performed modulo the field prime
   - Quotient computation ensures correct reduction
   - No assumptions about input ranges

3. **Constraint Completeness**
   - All intermediate values constrained
   - No unchecked operations
   - Soundness maintained throughout