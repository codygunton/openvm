# DivRem Component - Implementation Guidelines

## Component Overview

The DivRem component implements RISC-V integer division and remainder operations within the OpenVM zkVM framework. It handles both signed and unsigned operations with careful attention to edge cases and constraint-based verification.

## Critical Implementation Rules

### 1. Mathematical Invariants

**ALWAYS** maintain the fundamental division equation:
```
b = c * q + r
```
Where:
- `b` = dividend
- `c` = divisor  
- `q` = quotient
- `r` = remainder
- Constraint: `0 <= |r| < |c|`

### 2. Special Case Handling

**NEVER** forget to handle these special cases:

1. **Zero Divisor** (`c = 0`):
   - Set `zero_divisor` flag to 1
   - Quotient = all 1s (e.g., 0xFFFFFFFF for 32-bit)
   - Remainder = dividend

2. **Signed Overflow** (only for signed operations):
   - Occurs when: minimum signed value ÷ -1
   - For RV32: -2³¹ ÷ -1
   - Quotient = dividend
   - Remainder = 0

### 3. Sign Rules

For signed operations, **ALWAYS** apply these rules:
- `sign(q) = sign(b) XOR sign(c)` when q ≠ 0
- `sign(r) = sign(b)` when r ≠ 0
- When r = 0, use `r_zero` flag to handle special case

### 4. Constraint System Design

When modifying constraints:
- **ENSURE** all auxiliary columns have proper inverse checks
- **VERIFY** range checks cover all limbs of q and r
- **MAINTAIN** the lt (less than) comparison logic for |r| < |c|

### 5. Testing Requirements

**ALWAYS** include tests for:
- Random operations with various input patterns
- All special cases (zero divisor, overflow)
- Sign combinations (++, +-, -+, --)
- Boundary values (0, 1, -1, MAX, MIN)
- Negative tests that verify constraints catch violations

## Code Patterns to Follow

### Limb-Based Operations

```rust
// Always work with limb arrays
pub b: [T; NUM_LIMBS],
pub c: [T; NUM_LIMBS],
pub q: [T; NUM_LIMBS],
pub r: [T; NUM_LIMBS],
```

### Flag Management

```rust
// Special case flags must be boolean
builder.assert_bool(cols.zero_divisor);
builder.assert_bool(cols.r_zero);

// Exactly one special case at a time
let special_case = cols.zero_divisor + cols.r_zero;
builder.assert_bool(special_case);
```

### Range Checking Pattern

```rust
// Always range check both quotient and remainder
for (q, carry) in q.iter().zip(carry.iter()) {
    self.range_tuple_bus
        .send(vec![(*q).into(), carry.clone()])
        .eval(builder, is_valid.clone());
}
```

## Common Pitfalls to Avoid

1. **Forgetting carry propagation** in multiplication verification
2. **Incorrect sign extension** for signed operations
3. **Missing range checks** on auxiliary values
4. **Not handling the r = 0 case** properly for sign determination
5. **Assuming 32-bit only** - keep the implementation generic over NUM_LIMBS

## Performance Considerations

- Use lookup tables (BitwiseOperationLookup) for sign checks
- Batch range checks through RangeTupleChecker
- Minimize field inversions by caching when possible
- Keep hot paths (common cases) efficient

## Integration Checklist

When integrating with other components:
- [ ] Verify opcode offset matches transpiler definition (0x254)
- [ ] Ensure adapter chip properly handles reads/writes
- [ ] Check that NUM_LIMBS and LIMB_BITS match system configuration
- [ ] Validate interaction with shared lookup chips
- [ ] Test with both VmChipTestBuilder and full system tests

## Debugging Tips

1. **Constraint failures**: Check special case flags first
2. **Wrong results**: Verify sign handling and carry propagation
3. **Range check errors**: Ensure all limbs are properly bounded
4. **Integration issues**: Confirm opcode mapping and adapter setup

## Security Notes

- All inputs must be range-checked before use
- Special cases must be exhaustively handled
- Sign overflow conditions need explicit checking
- Test coverage should include adversarial inputs