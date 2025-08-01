# Native Poseidon2 Component - AI Assistant Instructions

## Critical Safety Requirements

### Cryptographic Correctness
- **NEVER** modify Poseidon2 permutation logic without deep understanding
- **ALWAYS** preserve exact field arithmetic operations
- **VERIFY** all changes maintain algebraic soundness
- **TEST** extensively with known test vectors

### Memory Consistency
- **ENSURE** all memory reads complete before writes
- **MAINTAIN** proper timestamp ordering
- **VERIFY** address calculations stay within bounds
- **CHECK** pointer dereferencing validity

## Component Principles

### 1. Row Type Discipline
Each row type has specific responsibilities:
- **SimplePoseidon**: One instruction, one row, direct execution
- **TopLevel**: Orchestrates VERIFY_BATCH, manages state
- **InsideRow**: Computes hashes, reports to TopLevel
- **NEVER** mix responsibilities between row types

### 2. Column Polymorphism
The `specific` field enables column reuse:
```rust
// Cast safely based on row type
let specific_cols: &TopLevelSpecificCols<F> = cols.specific.borrow();
```
- **ALWAYS** use correct cast for row type
- **NEVER** access wrong column interpretation
- **DOCUMENT** which columns are active per row type

### 3. Bus Communication
Internal bus (index 7) connects InsideRow to TopLevel:
- **InsideRow** sends: computed hash result
- **TopLevel** receives: hash for incorporation
- **MAINTAIN** bus consistency across all rows
- **VERIFY** send/receive balance

## Common Implementation Patterns

### Adding New Instructions
1. Define opcode in `openvm_native_compiler`
2. Add instruction executor in `chip.rs`
3. Implement trace generation in `trace.rs`
4. Add AIR constraints in `air.rs`
5. Write comprehensive tests

### Memory Access Pattern
```rust
// Standard memory read pattern
let (value, aux) = mem.read(timestamp, address, field_address_space);
// Record auxiliary data for constraints
record.read_aux = aux;
```

### Timestamp Management
```rust
// Increment timestamp after each memory operation
state.timestamp += 1;
// Preserve instruction start time
record.very_first_timestamp = instruction_start;
```

## Testing Requirements

### Correctness Tests
- **Unit tests**: Each instruction type independently
- **Integration tests**: Full VERIFY_BATCH scenarios
- **Edge cases**: Empty inputs, maximum sizes, single elements
- **Adversarial**: Invalid proofs must fail

### Performance Tests
- **Batch sizes**: Verify scaling behavior
- **Matrix dimensions**: Test various height combinations
- **Memory patterns**: Ensure efficient access

### Example Test Pattern
```rust
#[test]
fn test_verify_batch_basic() {
    // Setup
    let chip = setup_chip();
    let (dimensions, opened_values, proof, index_bits, commit) = generate_test_case();
    
    // Execute
    let record = chip.execute_verify_batch(...);
    
    // Verify
    assert_eq!(record.final_hash, commit);
    assert!(verify_constraints(&record));
}
```

## Performance Optimization

### Do's
- ✓ Batch related operations in single instruction
- ✓ Use contiguous memory layouts
- ✓ Minimize pointer dereferencing
- ✓ Reuse computed values across rows

### Don'ts
- ✗ Add unnecessary memory operations
- ✗ Break row contiguity without reason
- ✗ Duplicate Poseidon2 computations
- ✗ Use dynamic allocation in hot paths

## Debugging Tips

### Trace Inspection
1. Check row type flags are mutually exclusive
2. Verify timestamp progression
3. Confirm memory read/write ordering
4. Validate bus communication balance

### Common Issues
- **Wrong operand count**: VERIFY_BATCH uses all 7 operands
- **Bus imbalance**: Ensure every send has receive
- **Column casting**: Match cast type to row type
- **Memory alignment**: Field vs extension field sizing

## Integration Guidelines

### With Memory Subsystem
- Use `MemoryBridge` for all accesses
- Record auxiliary columns for constraints
- Respect address space assignments

### With Execution Framework
- Implement `InstructionExecutor` trait
- Update PC correctly
- Record execution state transitions

### With Poseidon2SubAir
- Reuse existing permutation logic
- Don't duplicate cryptographic code
- Maintain consistent parameters

## Code Review Checklist

Before submitting changes:
- [ ] All tests pass including adversarial cases
- [ ] Memory access follows established patterns
- [ ] Row types maintain clear separation
- [ ] Bus operations balance correctly
- [ ] Timestamps increment properly
- [ ] No cryptographic logic modifications
- [ ] Performance characteristics preserved
- [ ] Documentation updated

## Security Considerations

### Soundness
- Every computation must be constrained
- No unchecked assumptions
- Adversarial inputs handled correctly

### Side Channels
- Constant-time operations where applicable
- No branching on secret data
- Memory access patterns independent of secrets

### Audit Trail
- Document all changes to cryptographic paths
- Maintain test coverage above 95%
- Review with cryptography expert if unsure