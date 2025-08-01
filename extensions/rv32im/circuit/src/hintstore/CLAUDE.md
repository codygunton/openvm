# Hintstore Component Guidelines for Claude

## Component Overview

The hintstore component is a critical part of the OpenVM zkVM that enables guest programs to consume external witness data (hints). This component implements custom RISC-V extension opcodes for secure and efficient hint injection into VM memory.

## Key Concepts to Understand

1. **Hints**: External witness data provided to the VM at runtime (non-deterministic inputs)
2. **Two Operation Modes**:
   - HINT_STOREW: Single 4-byte word store
   - HINT_BUFFER: Multi-word sequential store
3. **Multi-row Traces**: Buffer operations span multiple trace rows with careful state management

## Important Implementation Details

### Opcode Handling
- HINT_STOREW: Simple operation, single trace row
- HINT_BUFFER: Complex operation, multiple trace rows with state machine

### Memory Safety
- All pointers must be bounds-checked using bitwise operations
- Memory addresses increment by 4 (word-aligned)
- Overflow prevention through range checking on MSBs

### Trace Structure
```rust
// Key trace columns:
is_single       // 1 for HINT_STOREW
is_buffer       // 1 for HINT_BUFFER rows
is_buffer_start // 1 for first row of HINT_BUFFER
rem_words       // Remaining words to process (decrements)
mem_ptr         // Current memory pointer (increments by 4)
data            // 4-byte data being written
```

## Common Patterns

### Execution Flow
1. Read memory pointer from register
2. (Buffer only) Read word count from register  
3. Check hint stream has enough data
4. Consume hints and write to memory
5. Update execution state

### Constraint Patterns
- Boolean constraints: `assert_bool(flag)`
- Conditional constraints: `when(condition).assert_eq(a, b)`
- State transitions: Validate next row based on current row

## Critical Invariants

1. **Hint Stream**: Never read beyond available hints
2. **Memory Alignment**: All addresses are 4-byte aligned
3. **State Consistency**: Buffer operations maintain valid state across rows
4. **Timestamp Ordering**: Proper increment for each memory operation

## Common Modifications

### Adding New Hint Operations
1. Define new opcode in `instructions.rs`
2. Add new column flags if needed
3. Implement execution logic
4. Add constraints for new operation
5. Update trace generation
6. Write comprehensive tests

### Optimizing Performance
- Minimize trace height by batching operations
- Reduce constraint complexity where possible
- Consider specialized operations for common patterns

## Testing Requirements

Always include:
1. Positive tests with random inputs
2. Negative tests that violate constraints
3. Edge cases (empty buffer, maximum size, etc.)
4. Integration tests with full VM

## Security Considerations

1. **Pointer Validation**: Always check bounds
2. **Data Validation**: Ensure hints are valid bytes (0-255)
3. **Stream Safety**: Check availability before consumption
4. **No Information Leakage**: Constraints must not reveal hint values

## Code Style

- Use descriptive variable names
- Comment complex constraint logic
- Follow existing patterns for consistency
- Add debug assertions in execution code
- Document any deviations from standard patterns

## Common Pitfalls to Avoid

1. Forgetting to increment timestamps for memory operations
2. Incorrect state transitions in buffer mode
3. Not checking hint stream length
4. Pointer arithmetic overflow
5. Mixing up limb ordering (little-endian)