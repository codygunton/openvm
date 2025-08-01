# BigInt Primitives Component Instructions

## Component Overview
You are working with the BigInt Primitives component of OpenVM, which provides the foundational overflow integer representation and modular arithmetic constraints used throughout the zkVM for large integer operations.

## Key Principles

1. **Overflow Correctness**: Always track accurate overflow bounds - underestimating breaks soundness
2. **Carry Propagation**: Understand when carries are needed vs when overflow can accumulate  
3. **Modular Arithmetic**: Most operations ultimately prove modular equations
4. **Range Check Efficiency**: Minimize range checks by batching operations

## Working with This Component

### Before Making Changes
1. Read `README.md` for the mathematical foundation
2. Understand overflow vs canonical representations
3. Review how parent AIRs use these SubAirs
4. Consider impact on range checking costs

### Code Modification Guidelines

#### Adding New Operations
- Implement arithmetic on `OverflowInt<T>` generically
- Update `limb_max_abs` and `max_overflow_bits` correctly
- Consider both positive and negative limb values
- Add corresponding tests with edge cases

#### Modifying SubAirs
- Maintain the pattern of IO vs auxiliary columns
- Don't duplicate `is_valid` boolean checks
- Ensure constraints handle all valid overflow ranges
- Test with maximum overflow values

#### Optimizing Performance
- Prefer accumulating operations before carries
- Share carry columns when possible
- Use native field operations for small values
- Consider limb size trade-offs

### Common Pitfalls

1. **Incorrect Overflow Tracking**
   ```rust
   // WRONG: Forgetting to update overflow
   let result = OverflowInt { limbs: new_limbs, ..self };
   
   // CORRECT: Properly track overflow
   let limb_max_abs = self.limb_max_abs + other.limb_max_abs;
   let max_overflow_bits = log2_ceil_usize(limb_max_abs);
   ```

2. **Sign Handling in Carries**
   ```rust
   // WRONG: Using regular division
   carry = val / (1 << limb_bits);
   
   // CORRECT: Using arithmetic right shift
   carry = val >> limb_bits;
   ```

3. **Range Check Bounds**
   ```rust
   // WRONG: Range checking without offset
   range_check(carry, carry_bits);
   
   // CORRECT: Offset for signed values
   range_check(carry + (1 << (carry_bits - 1)), carry_bits);
   ```

### Testing Requirements

#### Correctness Tests
- Test with maximum limb values
- Test negative limb scenarios
- Verify carry propagation edge cases
- Check modular arithmetic identities

#### Performance Tests
- Measure constraint count vs limb size
- Profile range check usage
- Benchmark vs native field operations

### Security Checklist

- [ ] Overflow bounds respect field size limit
- [ ] Carries are range checked appropriately
- [ ] Negative values handled correctly
- [ ] No assumptions about input canonicality
- [ ] Constraints complete for all overflow ranges

## Integration Patterns

### Using in Parent AIRs
```rust
// 1. Define columns for limbs and carries
// 2. Compute overflow expression
// 3. Call SubAir eval with proper context
// 4. Manage auxiliary column allocation
```

### Typical Workflow
1. Construct OverflowInt from inputs
2. Perform arithmetic operations
3. Generate carry hints if needed
4. Apply SubAir constraints
5. Extract results

## Performance Targets

- Limb size: 8-10 bits typical
- Overflow accumulation: 2-4 operations
- Range check calls: O(num_limbs)
- Constraint complexity: Linear in limbs

## Debugging Tips

1. **Constraint Failures**
   - Check overflow bound calculations
   - Verify carry generation matches constraints
   - Ensure range check bits are sufficient

2. **Performance Issues**
   - Profile range check usage
   - Consider larger limb sizes
   - Batch operations before carries

3. **Incorrect Results**
   - Verify limb ordering (little-endian)
   - Check sign handling in operations
   - Validate modular reduction logic

## Future Considerations

When extending this component:
- Maintain generic type support
- Consider SIMD optimizations
- Keep SubAir pattern modular
- Document overflow characteristics
- Add comprehensive test coverage

Remember: This component is foundational - changes impact many higher-level operations. Always consider the full stack implications of modifications.