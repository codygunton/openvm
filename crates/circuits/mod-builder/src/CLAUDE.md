# Claude Instructions for openvm-mod-circuit-builder

## Component Overview

You are working with the OpenVM modular arithmetic circuit builder, which provides a framework for building arithmetic circuits over large prime fields. This component is critical for implementing cryptographic operations like elliptic curve arithmetic and pairings.

## Key Principles

1. **Limb-Based Arithmetic**: All field operations work on limb representations. Never assume values fit in a single field element.

2. **Overflow Tracking**: The system automatically tracks overflow bounds. Trust the overflow calculations and don't try to manually manage them.

3. **Lazy Evaluation**: Expressions are built symbolically and evaluated only when needed. This is intentional for optimization.

4. **Reference Counting**: The builder uses `Rc<RefCell<>>` for shared ownership. Always use `.borrow()` or `.borrow_mut()` appropriately.

## Common Tasks

### When implementing new field operations:
1. Use the existing arithmetic operators (`+`, `-`, `*`, `/`) on `FieldVariable`
2. Call `.save()` on intermediate results that will be reused
3. Add constraints with `constrain_eq()` for the final result
4. Don't forget to handle edge cases (zero divisor, point at infinity, etc.)

### When debugging constraint failures:
1. Check overflow bounds - values might be exceeding limb capacity
2. Verify modular reduction is happening correctly
3. Ensure all expressions are properly saved before use in constraints
4. Look for sign issues in subtraction operations

### When optimizing performance:
1. Minimize division operations - they're expensive
2. Save and reuse common subexpressions
3. Consider the depth of expression trees
4. Batch similar operations together

## Code Style Guidelines

### Naming Conventions
- Use descriptive names for field variables (e.g., `lambda` for EC slope, not `l`)
- Prefix test utilities with curve name (e.g., `bn254_add`, `bls12_381_double`)
- Use `_limbs` suffix for limb arrays

### Error Handling
- Panic on division by zero with clear message
- Assert configuration validity in constructors
- Use `expect()` with descriptive messages for Option/Result

### Testing
- Always test with multiple field sizes
- Include edge cases (zero, one, modulus-1)
- Verify constraint satisfaction, not just computation correctness
- Use the provided test utilities for standard curves

## Common Pitfalls to Avoid

1. **Don't manually compute carries** - The system handles this automatically
2. **Don't assume limb values are positive** - They can be negative during computation
3. **Don't forget to save expressions** - Unsaved expressions can't be used in constraints
4. **Don't mix limb sizes** - Ensure consistent limb configuration throughout

## Integration Notes

### With OpenVM Core
- The mod-builder generates `VmCoreAir` implementations
- Opcodes must be registered with the VM
- Trace generation happens automatically

### With Other Components
- Range checker must be properly configured
- Coordinate limb sizes with other arithmetic components
- Share modulus configuration across related chips

## Performance Considerations

1. **Limb Size**: 8 bits is optimal for most use cases
2. **Constraint Count**: Each saved variable adds constraints
3. **Range Checks**: Batched automatically but still have cost
4. **Memory Layout**: Consecutive variables improve cache performance

## Security Critical Points

1. **Modular Reduction**: Always verify values are properly reduced
2. **Overflow Prevention**: Rely on the automatic overflow tracking
3. **Constraint Completeness**: Every computation must be constrained
4. **No Unchecked Operations**: All arithmetic must go through the builder

## When Modifying This Component

1. **Maintain Backward Compatibility**: Many components depend on this
2. **Update Test Utilities**: Keep curve implementations in sync
3. **Document Overflow Changes**: Any changes to overflow calculation must be clearly documented
4. **Benchmark Changes**: Performance is critical for this component

## Debugging Tips

1. **Print Expressions**: Use the Display impl to see expression structure
2. **Check Limb Bounds**: Verify `limb_max_abs` values are reasonable
3. **Trace Evaluation**: Step through `evaluate()` for complex expressions
4. **Verify Range Checks**: Ensure range checker bus is properly connected

Remember: This component is foundational for OpenVM's cryptographic operations. Changes here affect the entire system's correctness and performance.