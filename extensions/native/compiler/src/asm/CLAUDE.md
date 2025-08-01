# ASM Component - Claude Instructions

## Component Overview
You are working with the ASM (Assembly) component of the OpenVM native compiler. This component translates high-level DSL IR operations into low-level assembly instructions. It is a critical part of the compilation pipeline that bridges the gap between abstract operations and executable code.

## Key Responsibilities
When working with this component, you must:

1. **Maintain Memory Safety**: Always ensure heap allocations are range-checked and frame pointer calculations don't overflow
2. **Preserve Debug Information**: Never discard debug info when translating operations
3. **Follow Frame Pointer Convention**: Use the established fp() methods for variable addressing
4. **Handle Extension Fields Correctly**: Use specialized methods for extension field operations
5. **Implement Proper Error Handling**: Use trap mechanism for assertions and runtime failures

## Critical Files to Understand

### compiler.rs
- Core compilation logic - READ THIS FIRST
- Contains memory layout constants and frame pointer calculations
- Implements the main `build()` method that processes DslIr operations
- Has specialized control flow compilers (IfCompiler, ZipForCompiler)

### instruction.rs  
- Defines all assembly instruction types
- Contains formatting logic for human-readable output
- Critical for understanding available operations

### code.rs
- Defines BasicBlock and AssemblyCode structures
- Important for understanding program organization

## Memory Model Rules

### Address Space
- Total address space: 2^29 bytes (512MB)
- Heap starts at: 2^24 (HEAP_START_ADDRESS)
- Stack top at: HEAP_START_ADDRESS - 64
- Never allow addresses to exceed MEMORY_TOP

### Frame Pointer Layout
```
Vars:  1, 2, 9, 10, 17, 18... (8n + 1, 8n + 2)
Felts: 3, 4, 11, 12, 19, 20... (8n + 3, 8n + 4)  
Exts:  5-8, 13-16, 21-24... (8n + 5 through 8n + 8)
```

## Code Patterns to Follow

### Adding New Instructions
1. Add variant to `AsmInstruction` enum in instruction.rs
2. Add formatting in the `fmt()` method
3. Add translation case in compiler.rs `build()` method
4. Ensure debug info is preserved

### Implementing IR Operations
```rust
DslIr::NewOperation(args...) => {
    // 1. Calculate frame pointers
    let dst_fp = dst.fp();
    
    // 2. Generate instruction(s)
    self.push(AsmInstruction::NewInstr(dst_fp, ...), debug_info);
    
    // 3. Handle special cases (e.g., variable indexing)
    match index.fp() {
        IndexTriple::Const(...) => { /* constant case */ }
        IndexTriple::Var(...) => { /* variable case */ }
    }
}
```

### Extension Field Operations
Always use the specialized methods:
```rust
// DON'T manually emit instructions for each component
// DO use the helper methods
self.add_ext_exti(dst, lhs, rhs, debug_info);
self.mul_ext_felt(dst, lhs, rhs, debug_info);
```

## Common Pitfalls to Avoid

1. **Memory Overflow**: Always range-check heap allocations
2. **Wrong FP Calculation**: Use the provided fp() methods, don't calculate manually
3. **Missing Debug Info**: Always pass debug_info to push() calls
4. **Inefficient Extension Ops**: Use specialized methods instead of component-wise
5. **Incorrect Control Flow**: Use IfCompiler/ZipForCompiler for structured flow

## Testing Considerations

When modifying the ASM component:
1. Test with both constant and variable memory indices
2. Verify heap allocation doesn't overflow
3. Check that debug info appears in output
4. Test control flow with nested conditions
5. Verify extension field operations produce correct results

## Performance Guidelines

1. **Minimize Instructions**: Use immediate variants when possible
2. **Avoid Redundant Loads**: Cache frequently accessed values
3. **Optimize Extension Fields**: Use specialized compound operations
4. **Align Memory**: Respect word_size for allocations
5. **Place Trap Early**: Unlikely assertion failures should branch to early trap

## Integration Points

### From IR Layer
- Receives: TracedVec<DslIr<AsmConfig<F, EF>>>
- Must handle: All DslIr variants

### To Conversion Layer
- Produces: AssemblyCode<F, EF>
- Via: convert_program() in conversion module

### Debug Output
- Assembly can be printed with Display trait
- Labels are resolved to readable names

## Security Considerations

1. **Range Checks**: Critical for preventing memory corruption
2. **Overflow Prevention**: Check arithmetic operations won't overflow
3. **Trap on Failure**: Assertions must fail safely
4. **No Undefined Behavior**: Handle all edge cases explicitly

## When Adding Features

1. Consider memory layout impact
2. Maintain backward compatibility with existing IR
3. Document new instructions thoroughly
4. Add appropriate debug information
5. Test with various field types (F and EF)
6. Ensure formatting produces readable assembly

Remember: This component is performance-critical and security-sensitive. Every instruction matters, and correctness is paramount.