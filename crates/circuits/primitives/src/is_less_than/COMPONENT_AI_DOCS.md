# IsLessThan Component AI Documentation

## Overview
The `is_less_than` component provides a SubAir for constraining boolean outputs that equal 1 if and only if `x < y`. This is a fundamental arithmetic comparison primitive used throughout the OpenVM zkVM framework.

## Core Components

### IsLtSubAir
**Location**: `mod.rs:73-164`
**Purpose**: Main SubAir implementing the less-than comparison logic

**Key Parameters**:
- `bus: VariableRangeCheckerBus` - Bus for range check interactions
- `max_bits: usize` - Maximum bits for input values (≤ 29 for soundness)
- `decomp_limbs: usize` - Number of decomposition limbs

**Constructor**:
```rust
pub fn new(bus: VariableRangeCheckerBus, max_bits: usize) -> Self
```

### IsLessThanIo<T>
**Location**: `mod.rs:23-51`
**Purpose**: Input/output structure for the comparison operation

**Fields**:
- `x: T` - First input value
- `y: T` - Second input value  
- `out: T` - Boolean output (1 if x < y, 0 otherwise)
- `count: T` - Activation multiplicity (must be boolean)

### IsLtWhenTransitionAir
**Location**: `mod.rs:194-231`
**Purpose**: Variant that skips non-range-check constraints on the last row, for adjacent row comparisons

## Algorithm Details

The component uses a shifted difference approach:
1. Computes `y - x - 1 + 2^max_bits`
2. Decomposes the result into limbs
3. Range checks each limb
4. Constrains reconstruction to match the expected value

**Mathematical Foundation**:
- When `x < y`: Result has high bit set, `out = 1`
- When `x ≥ y`: Result fits in `max_bits`, `out = 0`

## Key Constraints

1. **Main Constraint** (`mod.rs:140`):
   ```
   condition * (lower + out * 2^max_bits - (y - x - 1 + 2^max_bits)) = 0
   ```

2. **Boolean Constraint** (`mod.rs:141`):
   ```
   out * (out - 1) = 0
   ```

3. **Range Check Constraints** (`mod.rs:145-163`):
   Each limb is range-checked with appropriate bit counts

## Implementation Requirements

### Soundness Requirements
- `max_bits ≤ 29` (enforced at construction)
- `count` must be constrained boolean by caller
- Range check interactions must be enabled for soundness

### Trace Generation
**Location**: `mod.rs:233-266`
**Function**: `generate_subrow`
**Context**: `(range_checker, x, y)`
**Output**: `(lower_decomp, out)`

## Testing Infrastructure

### IsLtTestAir
**Location**: `tests.rs:38-60`
**Purpose**: Standalone Air wrapper for testing

### IsLessThanChip
**Location**: `tests.rs:62-95`
**Purpose**: Complete chip implementation with trace generation

## Common Integration Patterns

1. **As SubAir**: Most common usage pattern
   ```rust
   let sub_air = IsLtSubAir::new(bus, max_bits);
   sub_air.eval(builder, (io, aux_cols));
   ```

2. **Transition Air**: For adjacent row comparisons
   ```rust
   let transition_air = sub_air.when_transition();
   ```

3. **Conditional Usage**: With activation flag
   ```rust
   let io = IsLessThanIo::new(x, y, out, condition);
   ```

## Performance Characteristics

- **Constraint Degree**: `deg(count) + max(1, deg(x), deg(y))`
- **Auxiliary Columns**: `max_bits.div_ceil(bus.range_max_bits)`
- **Range Check Interactions**: One per limb per activation

## Related Components
- `assert_less_than`: Similar but without output column
- `var_range`: Required for range checking functionality
- `LessThanAuxCols`: Shared auxiliary column structure

## Error Handling
- Debug assertions for bit length validation
- Construction-time soundness checks
- Test coverage for negative cases