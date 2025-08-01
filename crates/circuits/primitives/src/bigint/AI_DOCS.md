# BigInt Primitives Component AI Documentation

## Overview
The BigInt Primitives component provides foundational building blocks for arbitrary-precision integer arithmetic in OpenVM's zkVM. It implements efficient modular arithmetic operations using an overflow representation technique that minimizes the number of range checks required during proof generation.

## Core Architecture

### Key Files
- `mod.rs`: Core `OverflowInt<T>` type and arithmetic operations
- `check_carry_to_zero.rs`: SubAir for constraining overflow integers to zero
- `check_carry_mod_to_zero.rs`: SubAir for modular equality constraints
- `utils.rs`: Utility functions for BigInt operations and common prime constants

### Primary Responsibilities
1. **Overflow Arithmetic**: Efficient representation of intermediate calculations
2. **Carry Propagation**: Converting overflow representations to canonical form
3. **Modular Constraints**: Proving modular arithmetic equations
4. **Range Checking**: Ensuring limb values stay within valid bounds

## Key Components

### OverflowInt<T>
The core type representing big integers with overflow limbs:
- **limbs**: Vector of limb values (can exceed canonical range)
- **limb_max_abs**: Maximum absolute value of any limb
- **max_overflow_bits**: Bit width needed to represent limb values

### Representation
- Canonical form: limbs in `[0, 2^limb_bits)`
- Overflow form: limbs in `[-2^overflow_bits, 2^overflow_bits)`
- Integer value: `Σ(limbs[i] * 2^(limb_bits * i))`

### SubAir Components

#### CheckCarryToZeroSubAir
Constrains that an overflow integer equals zero:
- Takes overflow limbs and carry hints
- Range checks carries are valid
- Verifies carry propagation is correct
- Ensures final carry is zero

#### CheckCarryModToZeroSubAir  
Constrains modular equality `x ≡ 0 (mod m)`:
- Proves existence of quotient `q` where `x - q*m = 0`
- Range checks quotient limbs
- Delegates to CheckCarryToZeroSubAir for remainder

## Key Design Principles

### Overflow Representation
Instead of maintaining canonical form after each operation:
- Operations accumulate overflow in limbs
- Carry propagation deferred until constraint checking
- Reduces intermediate range checks
- Improves constraint efficiency

### Limb Size Selection
- Typical `limb_bits`: 8-10 bits
- Balances between:
  - Number of limbs (affects trace width)
  - Carry complexity (affects range check costs)
  - Overflow accumulation (affects intermediate storage)

### Negative Carry Handling
- Carries can be negative due to subtraction
- Uses arithmetic right shift for correct rounding
- Range checks adjusted with offset for signed values

## Common Operations

### Arithmetic Operations
```rust
// Addition: overflow accumulates
result = a + b  // limb_max_abs increases

// Subtraction: can produce negative limbs  
result = a - b  // max_overflow_bits accounts for negatives

// Multiplication: convolution of limbs
result = a * b  // output has len = a.len + b.len - 1
```

### Modular Arithmetic Pattern
```rust
// To prove: a * b ≡ r (mod p)
// Compute: a * b - r - q * p
// Constrain: check_carry_to_zero(a * b - r - q * p)
```

## Integration Points

### Range Checker
- All carry values must be range checked
- Uses configurable decomposition parameter
- Integrates via `range_checker_bus`

### Parent AIRs
- BigInt multiplication/division AIRs use these SubAirs
- Parent manages trace columns for limbs/carries
- SubAir handles constraint generation

## Security Considerations

### Overflow Bounds
- Must ensure `overflow_bits ≤ log2(field_size) - 1`
- Prevents wraparound in field representation
- Critical for soundness of constraints

### Completeness
- All operations must track accurate overflow bounds
- Underestimating can cause constraint failures
- Overestimating wastes range check resources

### Uniqueness
- Canonical form ensures unique representation
- Overflow form allows multiple representations
- Constraints must account for non-uniqueness

## Performance Characteristics

### Trace Costs
- Limb columns: O(num_limbs)
- Carry columns: O(num_limbs) for most operations
- Multiplication carries: O(2 * num_limbs - 1)

### Constraint Costs
- Carry constraints: O(num_limbs)
- Range checks: O(num_limbs * carry_bits / decomp_bits)
- Arithmetic operations: Field operations on expressions

### Optimization Strategies
1. Batch operations before carry propagation
2. Share carry columns across multiple operations
3. Use native field operations when possible
4. Minimize limb count for specific bit widths