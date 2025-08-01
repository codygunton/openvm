# Claude Assistant Instructions - OpenVM Native Compiler IR

## Component Overview
You are working with the OpenVM Native Compiler IR (Intermediate Representation), a sophisticated DSL for cryptographic computations. This IR serves as a compilation target that can generate both recursive zkVM bytecode and R1CS/Plonk constraints.

## Key Concepts to Remember

### Type System
- **Three distinct variable types**: `Var<N>` (native), `Felt<F>` (field), `Ext<F,EF>` (extension)
- These types are NOT interchangeable - mixing them is a compile error
- Use phantom types to ensure type safety at compile time

### Builder Pattern
- Always use `Builder<C>` as the primary API
- Call `builder.eval()` to evaluate symbolic expressions
- Variables must be initialized before use

### Memory Model
- Prefer static allocation (`Array::Fixed`) when sizes are known
- Use dynamic allocation (`Array::Dyn`) only when necessary
- Memory access is bounds-checked in debug mode

## Common Tasks

### When Asked to Implement Field Arithmetic
1. Identify the field type needed (Native, Target, or Extension)
2. Use appropriate variable type (`Var`, `Felt`, or `Ext`)
3. Remember division is only available for field elements
4. Consider using symbolic execution for complex expressions

### When Asked to Implement Algorithms
1. Start with clear variable initialization
2. Use appropriate control flow abstractions
3. Consider early exit patterns for searches
4. Test with small inputs first

### When Working with Arrays
1. Choose between static and dynamic based on use case
2. Use `builder.array()` for known values
3. Use `builder.alloc()` + `Array::Dyn` for runtime sizes
4. Always check bounds for dynamic arrays

## Code Generation Guidelines

### Variable Naming
```rust
// Good
let sum: Var<C::N> = builder.eval(0);
let field_elem: Felt<C::F> = builder.eval(C::F::ONE);

// Avoid
let x = builder.eval(0); // Type unclear
let temp = builder.eval(value); // Non-descriptive
```

### Control Flow
```rust
// Prefer high-level abstractions
builder.if_eq(x, y).then(|builder| { /* ... */ });

// Over manual control flow tracking
```

### Error Handling
- Always validate inputs when implementing division
- Check array bounds for dynamic access
- Use assertions for invariants that must hold

## Performance Considerations

### Optimization Opportunities
1. Use symbolic execution to batch operations
2. Minimize the number of variables allocated
3. Prefer static loops over dynamic when possible
4. Group similar operations together

### Anti-patterns to Avoid
- Creating unnecessary intermediate variables
- Using dynamic arrays for fixed-size data
- Nested loops with high iteration counts
- Forgetting to evaluate symbolic expressions

## Integration Points

### When Extending the IR
1. New operations go in the `DslIr` enum
2. Add corresponding builder methods
3. Implement backend execution logic
4. Update witness generation

### When Debugging
1. Use `builder.print_*` for value inspection
2. Enable backtrace collection for instruction tracking
3. Add cycle trackers around expensive operations
4. Verify witness values in tests

## Safety Requirements

### Type Safety
- Never cast between incompatible field types
- Ensure Config trait bounds are satisfied
- Use phantom types consistently

### Memory Safety
- Always initialize variables before use
- Check bounds for dynamic array access
- Free large allocations when done

## Common Pitfalls

1. **Forgetting to evaluate symbolic expressions**
   ```rust
   let x = SymbolicVar::from(a) + SymbolicVar::from(b);
   // Must call builder.eval(x) to get Var<N>
   ```

2. **Mixing variable types**
   ```rust
   let native: Var<N> = ...;
   let field: Felt<F> = ...;
   // Cannot do: native + field
   ```

3. **Uninitialized variables**
   ```rust
   let x: Var<N> = builder.uninit();
   // Must assign before use
   ```

## Testing Recommendations

1. Test each operation in isolation first
2. Verify witness generation is deterministic
3. Check edge cases (zero, max values)
4. Benchmark constraint counts
5. Test both static and dynamic variants

## Documentation Standards

When documenting IR code:
1. Explain the algorithm, not just the implementation
2. Document field type requirements
3. Note any assumptions about input ranges
4. Include complexity analysis for loops
5. Provide usage examples

## File Organization

- Core types go in `types.rs`
- Builder API in `builder.rs`
- Instruction definitions in `instructions.rs`
- Specialized operations in their own modules
- Keep public API surface minimal

Remember: The IR is a low-level abstraction. When helping users, guide them toward using the high-level builder methods rather than constructing instructions manually.