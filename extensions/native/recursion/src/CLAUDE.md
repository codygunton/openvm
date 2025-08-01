# OpenVM Native Recursion Extension - AI Assistant Instructions

## Component Overview

You are working with the OpenVM Native Recursion Extension, which implements STARK proof verification within the zkVM. This enables recursive proof composition and aggregation.

## Key Concepts to Remember

1. **Two Configuration Types**:
   - **Inner Config**: BabyBear field (31-bit prime) for recursion within OpenVM
   - **Outer Config**: Bn254 field for final proofs compatible with Ethereum

2. **Verification Flow**:
   - Build verification program with `VerifierProgram`
   - Program reads proof from input using witness
   - Initialize FRI PCS configuration
   - Run verification with `StarkVerifier`

3. **Multi-STARK Support**: The verifier can handle multiple AIRs with different trace heights in a single proof

## Common Tasks and Solutions

### Building a Verifier Program

```rust
// Standard approach
let advice = new_from_inner_multi_vk(&vk);
let program = VerifierProgram::<InnerConfig>::build(advice, &fri_params);
```

### Working with Builders

Always use the Builder pattern for constructing verification logic:
```rust
let mut builder = Builder::<InnerConfig>::default();
// Build verification logic
let program = builder.compile_isa_with_options(options);
```

### Memory Management

For non-static mode, use sub-builders to manage memory:
```rust
if !builder.flags.static_only {
    let mut tmp_builder = builder.create_sub_builder();
    let old_heap_ptr = tmp_builder.load_heap_ptr();
    // ... work ...
    tmp_builder.store_heap_ptr(old_heap_ptr);
}
```

## Important Implementation Details

1. **Proof Structure**: The `StarkProofVariable` contains commitments, openings, per-AIR data, and permutation information

2. **Challenge Generation**: Uses duplex sponge construction with Poseidon2

3. **FRI Protocol**: Implements two-adic FRI with batched openings

4. **Constraint Evaluation**: Uses symbolic expression DAGs with folding

## Common Patterns

### Array Iteration
```rust
builder.range(0, size).for_each(|i_vec, builder| {
    let i = i_vec[0];
    // Process element at index i
});
```

### Conditional Execution
```rust
builder.if_eq(a, b).then(|builder| {
    // Execute if equal
});
```

### Performance Tracking
```rust
builder.cycle_tracker_start("OperationName");
// ... operation ...
builder.cycle_tracker_end("OperationName");
```

## Feature Flags

- `static-verifier`: Enables Halo2 integration
- `evm-prove`: EVM proof generation
- `test-utils`: Testing utilities
- `parallel`: Parallel computation support

## Security Considerations

1. Always validate proof components before use
2. Check trace height constraints
3. Verify proof-of-work when required
4. Ensure proper challenge ordering

## Common Pitfalls to Avoid

1. **Not witnessing the proof**: Always call `Proof::witness()` before verification
2. **Incorrect array bounds**: Verify sizes match expectations
3. **Challenge order**: Maintain correct Fiat-Shamir transcript
4. **Domain compatibility**: Ensure log_degree fits within MAX_TWO_ADICITY

## Testing and Debugging

1. Use `testing_utils` for generating test data
2. Enable cycle tracking for performance analysis
3. Use static mode for deterministic testing
4. Check feature flags for required functionality

## Integration Points

1. **With OpenVM Core**: Uses native compiler IR and circuit abstractions
2. **With STARK Backend**: Leverages proof structures and verifier logic
3. **With Halo2** (optional): Provides aggregation circuits

## File Organization

- Core verification: `stark/mod.rs`
- Type definitions: `types.rs`, `vars.rs`
- Challenge generation: `challenger/`
- FRI implementation: `fri/`
- Configuration: `config/`
- Utilities: `utils.rs`, `helper.rs`, `hints.rs`

## When Implementing New Features

1. Follow existing patterns for variable types
2. Use the Builder abstraction consistently
3. Add cycle tracking for performance-critical sections
4. Include proper error handling with assertions
5. Document security assumptions

## Performance Optimization

1. Use hints for witness generation
2. Batch operations when possible
3. Leverage sub-builders for memory management
4. Precompute static configurations
5. Enable parallel features when appropriate