# STARK Recursion Component Rules

## Component Overview

This component implements STARK proof verification within the OpenVM zkVM, enabling recursive proof composition. It is a critical security component that must maintain soundness guarantees.

## Key Principles

1. **Security First**: All changes must preserve the soundness of the verification protocol
2. **Validation Completeness**: Never skip or weaken proof validations
3. **Memory Safety**: Careful management of dynamic allocations in verification
4. **Performance**: Optimize without compromising security

## Code Standards

### Verification Logic

1. **Always validate proof shapes** before processing:
   ```rust
   // CORRECT: Validate before use
   builder.assert_usize_eq(main_trace_commits.len(), num_main_commits);
   
   // WRONG: Use without validation
   let commit = builder.get(main_trace_commits, idx); // Could panic
   ```

2. **Maintain validation tags** (T01a, T02a, etc.) when modifying code:
   ```rust
   // (T02a): `air_perm_by_height` is a valid permutation of `0..num_airs`.
   // Keep these comments when refactoring
   ```

3. **Check bounds** for all dynamic accesses:
   ```rust
   // CORRECT: Bounds checked
   builder.assert_less_than_slow_small_rhs(air_proof.log_degree, MAX_TWO_ADICITY);
   
   // WRONG: Unchecked access
   let domain = domains[log_degree]; // Could overflow
   ```

### Memory Management

1. **Use sub-builders** for large temporary computations:
   ```rust
   // CORRECT: Memory recycling
   let mut tmp_builder = builder.create_sub_builder();
   // ... do work
   tmp_builder.store_heap_ptr(old_heap_ptr);
   
   // WRONG: Accumulating in main builder
   // ... lots of temporary allocations
   ```

2. **Prefer stack allocation** for small, fixed-size data:
   ```rust
   // CORRECT: Stack allocation
   let trace_points = builder.array::<Ext<_, _>>(2);
   
   // WRONG: Heap for small fixed data
   let trace_points = builder.vec(vec![zeta, zeta_next]);
   ```

### Error Handling

1. **Panic on verification failures** (no silent failures):
   ```rust
   // CORRECT: Clear failure
   builder.assert_ext_eq(folded_constraints * sels.inv_zeroifier, quotient);
   
   // WRONG: Silent skip
   if folded_constraints * sels.inv_zeroifier != quotient { return; }
   ```

2. **Validate limitations explicitly**:
   ```rust
   // CORRECT: Clear limitation
   if m_advice.num_challenges_to_sample.len() > 1 {
       panic!("Only support 0 or 1 phase is supported");
   }
   ```

## Implementation Guidelines

### Adding New Features

1. **Extend validation tags** when adding new checks
2. **Update shape validations** for new proof components
3. **Maintain backward compatibility** for proof formats
4. **Add cycle tracking** for performance-critical sections

### Modifying Verification Flow

1. **Preserve Fiat-Shamir ordering**:
   ```rust
   // Order matters for soundness:
   // 1. Observe commitments
   // 2. Sample challenges
   // 3. Use challenges
   ```

2. **Keep constraint evaluation deterministic**
3. **Maintain proof-of-work checks** when present

### Performance Optimization

1. **Batch similar operations**:
   ```rust
   // CORRECT: Batched commitment round
   let common_main_mats = builder.array(num_common_main_traces);
   // ... fill array
   // Single round for all common mains
   ```

2. **Use static mode** when applicable:
   ```rust
   if builder.flags.static_only {
       // Compile-time optimizations
   }
   ```

## Testing Requirements

### Unit Tests

1. Test both valid and invalid proofs
2. Test edge cases (empty AIRs, maximum heights)
3. Test multi-trace configurations
4. Verify panic conditions

### Integration Tests

1. Test with real STARK proofs
2. Verify memory usage patterns
3. Check cycle counts
4. Test feature flag combinations

## Security Considerations

### Cryptographic Integrity

1. **Never weaken security parameters**
2. **Maintain challenge space size** (extension field)
3. **Preserve zero-knowledge properties** where applicable

### Input Validation

1. **Validate all external inputs** (proofs, advice)
2. **Check for integer overflows** in size calculations
3. **Verify permutation validity** completely

### Common Vulnerabilities

1. **Insufficient randomness**: Ensure challenges are properly sampled
2. **Replay attacks**: Include all commitments in Fiat-Shamir
3. **Malleability**: Validate proof shapes strictly

## Performance Patterns

### Recommended

```rust
// Parallel verification of independent checks
builder.range(0, num_airs).for_each(|i_vec, builder| {
    // Independent AIR verification
});

// Reuse computed values
let zeta_next = domain.next_point(builder, zeta);
```

### Avoid

```rust
// Sequential when could be parallel
for i in 0..num_airs {
    verify_air(i); // Could be batched
}

// Recomputing expensive operations
let zeta_next1 = compute_next(zeta);
let zeta_next2 = compute_next(zeta); // Redundant
```

## Debugging Tips

1. **Enable cycle tracking** to identify bottlenecks
2. **Use validation tags** to trace verification flow
3. **Check builder operations** for unexpected patterns
4. **Monitor memory allocation** in dynamic mode

## Code Review Checklist

- [ ] All proof shapes validated
- [ ] Bounds checks for dynamic accesses
- [ ] Fiat-Shamir ordering preserved
- [ ] Memory efficiently managed
- [ ] Security parameters unchanged
- [ ] Tests cover new code paths
- [ ] Documentation updated
- [ ] Validation tags maintained

## Future Considerations

When extending this component:

1. **Multi-phase support**: Design for >1 challenge phase
2. **Dynamic AIR selection**: Support runtime AIR choice
3. **Optimized folding**: Improve constraint evaluation
4. **Parallel verification**: Multi-core support

## Component Boundaries

This component should:
- ✅ Verify STARK proofs
- ✅ Support multi-trace proofs
- ✅ Generate circuit operations (with feature)
- ❌ Generate proofs (use prover components)
- ❌ Modify proof formats (use backend)
- ❌ Implement new constraints (use circuit DSL)