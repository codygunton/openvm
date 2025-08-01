# Conversion Module Instructions for Claude

## Module Context
You are working with the conversion module in the OpenVM native compiler. This module is the final stage that transforms assembly instructions into executable VM instructions.

## Key Principles

### 1. Instruction Encoding
- Every ASM instruction maps to one or more VM instructions
- Use the `inst` helper for 5-operand instructions
- Use the `inst_med` helper for 6-operand instructions
- Always specify address specifiers (AS::Native or AS::Immediate)

### 2. Address Specifier Rules
- **AS::Native (4)**: Used for memory operands (register values)
- **AS::Immediate (0)**: Used for constant values
- The choice affects how the VM interprets operands

### 3. PC and Label Management
- PC increments by DEFAULT_PC_STEP (4) per instruction
- Labels must be resolved to PC-relative offsets
- Extension field branches MUST account for multi-instruction sequences

### 4. Extension Field Operations
- Extension fields have dimension EF::D (typically 4)
- Operations on extension fields generate D instructions
- Branch instructions must handle all D components correctly

## Common Patterns

### Memory Access Pattern
```rust
// Load: mem[dst] ← mem[mem[src] + index * size + offset]
inst(
    options.opcode_with_offset(NativeLoadStoreOpcode::LOADW),
    i32_f(dst),
    index * size + offset,
    i32_f(src),
    AS::Native,
    AS::Native,
)
```

### Branch Pattern
```rust
// Branch: if condition, pc ← labels[label]
inst(
    options.opcode_with_offset(BranchOpcode),
    i32_f(operand1),
    operand2_or_immediate,
    labels(label) - pc,  // PC-relative offset
    AS::Native,
    AS::Native_or_Immediate,
)
```

### Arithmetic Pattern
```rust
// Binary op: mem[dst] ← mem[lhs] op mem[rhs]
inst_med(
    options.opcode_with_offset(OpCode),
    i32_f(dst),
    i32_f(lhs),
    i32_f(rhs),
    AS::Native,
    AS::Native,
    AS::Native,
)
```

## Critical Requirements

### 1. Field Conversion
Always use `i32_f` to convert i32 values to field elements:
```rust
i32_f(value)  // Handles negative values correctly
```

### 2. Extension Field Branches
For BneE/BeqE instructions:
- Generate exactly EF::D comparison instructions
- Adjust PC offset for each instruction in the sequence
- Use proper early-exit logic for efficiency

### 3. Debug Information
- Preserve debug_info through conversion
- Replicate debug_info for multi-instruction expansions

### 4. Phantom Instructions
Use correct discriminants:
```rust
PhantomDiscriminant(NativePhantom::Print as u16)
PhantomDiscriminant(SysPhantom::CtStart as u16)
```

## Common Mistakes to Avoid

1. **Wrong AS Usage**: Using AS::Immediate for memory operands
2. **PC Calculation**: Forgetting to account for multi-instruction sequences
3. **Field Overflow**: Not checking bounds in `i32_f` conversion
4. **Missing Initialization**: Forgetting to initialize register 0
5. **Label Resolution**: Using absolute instead of relative addresses

## Testing Considerations

When modifying conversion logic:
1. Test with both base field and extension field operations
2. Verify branch targets are correct
3. Check edge cases (negative offsets, zero operands)
4. Ensure debug information is preserved
5. Validate cycle tracker insertion when enabled

## Performance Guidelines

1. Minimize instruction expansion where possible
2. Use compile-time calculations for offsets
3. Avoid unnecessary field conversions
4. Leverage immediate mode for constants
5. Consider instruction ordering for cache efficiency

## Integration Notes

### Input Validation
- Assembly instructions should already be validated
- Focus on correct encoding, not semantic validation

### Output Guarantees
- All instructions have valid opcodes
- All field elements are within range
- Debug information is complete
- Program starts with register 0 initialization

## Extension Points

When adding new instruction types:
1. Add the ASM instruction variant
2. Add the conversion case in `convert_instruction`
3. Determine single vs multi-instruction expansion
4. Choose appropriate addressing modes
5. Test with both field types

## Debugging Tips

1. Use debug prints to verify PC calculations
2. Check instruction encoding with VM decoder
3. Verify label resolution with small test programs
4. Use cycle tracker to identify performance issues
5. Compare generated code with hand-written examples