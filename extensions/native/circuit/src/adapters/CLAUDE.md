# Native Circuit Adapters - Claude AI Assistant Instructions

## Overview
When working with the Native Circuit Adapters component, you are dealing with the critical bridge between OpenVM's execution framework and specialized native field operations. These adapters must be implemented with extreme care to maintain soundness and efficiency.

## Key Principles

### 1. Memory Safety First
- **NEVER** access memory directly - always use MemoryBridge
- All reads must go through `memory_bridge.read()` or `memory_bridge.read_or_immediate()`
- All writes must go through `memory_bridge.write()`
- Memory operations must be properly constrained in the AIR

### 2. Execution State Consistency
- Always properly update the PC (program counter)
- Use `DEFAULT_PC_STEP` for normal advancement
- Ensure `from_state` and `to_state` are consistent
- Never skip state transition constraints

### 3. Instruction Decoding
- Operands follow consistent patterns:
  - operands[0:2] typically encode first read address
  - operands[2:4] typically encode second read address  
  - operands[4:6] typically encode write address
- Always validate operand count matches expected

### 4. Constraint Completeness
- Every operation must be fully constrained
- Range check all values that need bounds
- Verify all memory operations
- No "trust me" assumptions

## Common Implementation Patterns

### When Implementing a New Adapter

1. **Start with the Interface**:
   - Determine number of reads/writes
   - Decide on immediate support
   - Check if jumps are needed

2. **Define the Columns Structure**:
   - Include `ExecutionState<T>` for from_state
   - Add read auxiliary columns for each read
   - Add write auxiliary columns for each write
   - Include any operation-specific columns

3. **Implement the AIR**:
   - Handle instruction decoding
   - Implement operation constraints
   - Add memory interactions
   - Update execution state

4. **Write the Runtime**:
   - Implement preprocess for memory operations
   - Generate correct trace rows
   - Handle postprocessing

## Testing Requirements

### Unit Tests Must Verify:
1. **Correct Operation**: Result matches expected
2. **Memory Consistency**: Reads/writes are valid
3. **State Transitions**: PC updates correctly
4. **Edge Cases**: Zero values, max values, etc.

### Integration Tests Must Check:
1. **Interaction with Core Chips**: Adapter + chip work together
2. **Program Flow**: Multiple instructions execute correctly
3. **Memory Conflicts**: No interference between operations

## Performance Guidelines

### Do:
- Use immediate values for constants
- Batch similar operations when possible
- Access memory sequentially when feasible
- Minimize auxiliary column count

### Don't:
- Add unnecessary memory operations
- Create redundant constraints
- Use complex addressing when simple works
- Over-generalize at the cost of efficiency

## Common Pitfalls to Avoid

### 1. Forgetting Constraints
```rust
// WRONG: Operation without constraints
let result = a * b;

// CORRECT: With proper constraints
let result = a * b;
builder.assert_eq(cols.result, result);
```

### 2. Incorrect Memory Addressing
```rust
// WRONG: Hardcoded address space
let addr = MemoryAddress { 
    address_space: 0.into(), // Don't hardcode!
    pointer: ptr 
};

// CORRECT: Use instruction-provided address space
let addr = MemoryAddress {
    address_space: ctx.reads[0].address_space.clone(),
    pointer: ctx.reads[0].pointer.clone(),
};
```

### 3. Missing Range Checks
```rust
// WRONG: Unchecked carry value
let carry = (a + b) >> LIMB_BITS;

// CORRECT: With range constraint
let carry = (a + b) >> LIMB_BITS;
self.range_checker.range_check(builder, carry, CARRY_BITS);
```

## Code Review Checklist

When reviewing adapter code, verify:

- [ ] All memory operations use MemoryBridge
- [ ] PC is updated correctly (advancement or jump)
- [ ] All arithmetic operations are constrained
- [ ] Range checks are applied where needed
- [ ] Immediate values are handled properly
- [ ] Test coverage includes edge cases
- [ ] No direct memory access
- [ ] Consistent operand decoding
- [ ] Proper error handling in runtime

## Debugging Tips

### When Constraints Fail:
1. Check trace generation matches AIR expectations
2. Verify memory read/write indices
3. Ensure all columns are populated
4. Look for off-by-one errors in indexing

### When Tests Fail:
1. Print instruction operands
2. Trace memory operations
3. Verify address calculations
4. Check immediate value handling

### Performance Issues:
1. Profile constraint count
2. Check for redundant operations
3. Verify memory access patterns
4. Look for unnecessary columns

## Integration Guidelines

### With Field Arithmetic:
- ALU adapter connects to arithmetic chips
- Ensure consistent field element handling
- Verify overflow/underflow behavior

### With Control Flow:
- Branch adapter modifies PC
- Ensure proper jump targets
- Handle conditional vs unconditional

### With Memory System:
- Respect address space boundaries
- Handle multi-word values correctly
- Ensure proper synchronization

## Security Considerations

### Always:
- Validate all inputs
- Range check intermediate values
- Prevent integer overflow in addressing
- Ensure deterministic execution

### Never:
- Trust unchecked values
- Skip constraint verification
- Allow unbounded operations
- Expose internal state incorrectly

## Final Notes

The adapter layer is critical for OpenVM's correctness. When in doubt:
1. Add more constraints rather than fewer
2. Test edge cases thoroughly
3. Document non-obvious design choices
4. Ask for review on security-critical changes

Remember: A sound adapter is better than an optimal but broken one.