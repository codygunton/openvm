# FRI Component - Claude AI Assistant Instructions

## Overview
When working with the FRI (Fast Reed-Solomon Interactive Oracle Proof) component, you are implementing a critical cryptographic primitive for STARK proof systems. This component requires careful attention to field arithmetic, memory access patterns, and constraint completeness.

## Key Principles

### 1. Rolling Hash Correctness
- **ALWAYS** compute in reverse order: process elements from length-1 down to 0
- The formula is: `result = result * alpha + (b[i] - a[i])`
- Field extension operations must be properly constrained
- Never skip intermediate state verification

### 2. Three-Phase Execution Model
- **Workload Rows**: Actual computation happens here
- **Instruction1 Row**: Setup and configuration
- **Instruction2 Row**: Finalization and result writing
- Each phase has specific responsibilities - don't mix them

### 3. Memory Access Discipline
- All memory operations through MemoryBridge
- Respect the timestamp ordering
- a-values can be read OR written (never both in same instruction)
- b-values are always read-only

### 4. Field Extension Handling
- Extension degree is fixed at 4 (EXT_DEG)
- Elements stored as arrays of 4 field elements
- Use FieldExtension module for arithmetic
- Ensure all 4 components are constrained

## Common Implementation Patterns

### When Modifying FRI Implementation

1. **Trace Generation Order**:
   ```rust
   // Process in reverse for correct rolling hash
   for (i, (&a_id, &b_id)) in record.a_rws.iter()
       .rev()  // CRITICAL: reverse iteration
       .zip_eq(record.b_reads.iter().rev())
       .enumerate()
   ```

2. **Timestamp Management**:
   ```rust
   // Workload row timestamps decrease
   timestamp: start_timestamp + F::from_canonical_usize((length - i) * 2)
   
   // Each row consumes 2 time units
   ```

3. **Pointer Arithmetic**:
   ```rust
   // a_ptr increments by 1
   a_ptr: a_ptr + F::from_canonical_usize(length - i)
   
   // b_ptr increments by EXT_DEG (4)
   b_ptr: b_ptr + F::from_canonical_usize((length - i) * EXT_DEG)
   ```

## Testing Requirements

### Unit Tests Must Verify:
1. **Correct Result**: Matches expected polynomial evaluation
2. **Memory Consistency**: All reads/writes are valid
3. **State Transitions**: Proper phase sequencing
4. **Edge Cases**: Length 0, length 1, maximum length
5. **Field Extension**: All 4 components correct

### Integration Tests Must Check:
1. **Hint Streaming**: Both init and non-init modes
2. **Address Spaces**: Different memory regions
3. **Instruction Sequencing**: Multiple FRI ops in succession
4. **Boundary Conditions**: Full trace utilization

## Performance Guidelines

### Do:
- Process elements in batches when possible
- Use sequential memory access patterns
- Minimize column count (current: 27)
- Reuse computation across phases

### Don't:
- Add unnecessary intermediate columns
- Perform redundant memory operations
- Over-generalize beyond current needs
- Break the three-phase structure

## Common Pitfalls to Avoid

### 1. Incorrect Rolling Hash Order
```rust
// WRONG: Forward iteration
for i in 0..length {
    result = result * alpha + (b[i] - a[i])
}

// CORRECT: Reverse iteration
for i in (0..length).rev() {
    result = result * alpha + (b[i] - a[i])
}
```

### 2. Timestamp Calculation Errors
```rust
// WRONG: Increasing timestamps
timestamp: start_timestamp + F::from_canonical_usize(i * 2)

// CORRECT: Decreasing timestamps
timestamp: start_timestamp + F::from_canonical_usize((length - i) * 2)
```

### 3. Field Extension Component Handling
```rust
// WRONG: Only constraining first component
builder.assert_eq(result[0], expected[0]);

// CORRECT: Constraining all components
assert_array_eq(builder, result, expected);
```

### 4. Phase Flag Confusion
```rust
// WRONG: Mixing phase indicators
is_workload_row && is_ins_row  // Never both true

// CORRECT: Exclusive phases
is_workload_row XOR is_ins_row XOR disabled
```

## Code Review Checklist

When reviewing FRI code, verify:

- [ ] Rolling hash computed in reverse order
- [ ] All 4 field extension components constrained
- [ ] Proper three-phase execution structure
- [ ] Timestamps decrease within workload
- [ ] Memory operations use correct timestamps
- [ ] Pointer arithmetic matches data layout
- [ ] Phase transitions properly constrained
- [ ] Edge cases (length 0/1) handled correctly
- [ ] Hint streaming works for both modes
- [ ] No missing constraints on intermediate values

## Debugging Tips

### When Results Are Wrong:
1. Check iteration order (must be reverse)
2. Verify field extension arithmetic
3. Print intermediate rolling hash values
4. Ensure a and b indices match

### When Constraints Fail:
1. Check phase flag consistency
2. Verify timestamp calculations
3. Ensure pointer increments are correct
4. Look for off-by-one in indices

### Performance Issues:
1. Profile constraint count
2. Check memory access patterns
3. Verify no redundant operations
4. Ensure optimal column usage

## Integration Guidelines

### With Memory System:
- Respect offline checking protocol
- Use correct address spaces
- Handle multi-word values properly
- Maintain timestamp ordering

### With Field Extension:
- Use provided arithmetic functions
- Maintain 4-element representation
- Ensure proper constraint coverage
- Handle zero elements correctly

### With Execution Framework:
- Update PC by DEFAULT_PC_STEP
- Maintain execution state consistency
- Handle instruction completion properly
- Integrate with program bus correctly

## Security Considerations

### Always:
- Verify complete polynomial evaluation
- Range check all indices
- Validate memory addresses
- Ensure deterministic execution

### Never:
- Skip constraints on any component
- Allow unbounded computations
- Trust unchecked values
- Expose intermediate states

## Optimization Opportunities

### Current Optimizations:
1. Unified column structure (27 total)
2. Efficient three-phase execution
3. Minimal auxiliary columns
4. Batched memory operations

### Future Considerations:
1. Multiple FRI ops in parallel
2. Specialized layouts for common lengths
3. Optimized field arithmetic
4. Cache-friendly access patterns

## Final Notes

The FRI component is cryptographically critical. When implementing:
1. Prioritize correctness over optimization
2. Test thoroughly with known test vectors
3. Document any deviations from standard FRI
4. Consider proof system implications

Remember: A sound FRI implementation is essential for STARK security. Every constraint matters.