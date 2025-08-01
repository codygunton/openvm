# Conversion Module Implementation Guide

## Overview
This guide provides detailed implementation guidance for working with the conversion module, which transforms assembly instructions into VM-executable format.

## Architecture Deep Dive

### Module Structure
```
conversion/
└── mod.rs          # All conversion logic in single file
```

### Key Design Decisions

1. **Single-File Architecture**: All conversion logic is contained in `mod.rs` for simplicity
2. **Functional Design**: Pure functions with no mutable state
3. **Type Safety**: Generic over field types (F, EF)
4. **Zero-Copy**: Minimal allocations during conversion

## Implementation Patterns

### 1. Adding New Instructions

When implementing a new instruction type:

```rust
// Step 1: Add to AsmInstruction enum (in asm module)
pub enum AsmInstruction<F, EF> {
    // ... existing variants
    YourNewInstruction(i32, i32, F),  // (dst, src, immediate)
}

// Step 2: Add conversion case
match instruction {
    // ... existing cases
    AsmInstruction::YourNewInstruction(dst, src, imm) => vec![
        inst_med(
            options.opcode_with_offset(YourOpcode::NEW_OP),
            i32_f(dst),
            i32_f(src),
            imm,
            AS::Native,
            AS::Native,
            AS::Immediate,
        ),
    ],
}
```

### 2. Multi-Instruction Expansion

For operations that require multiple VM instructions:

```rust
AsmInstruction::ComplexOp(args) => {
    let mut instructions = vec![];
    
    // Generate instruction sequence
    for i in 0..count {
        instructions.push(inst(
            opcode,
            // Calculate operands based on iteration
            operand_a + F::from_canonical_usize(i),
            operand_b,
            operand_c,
            AS::Native,
            AS::Native,
        ));
    }
    
    instructions
}
```

### 3. PC-Relative Addressing

For branch instructions with PC-relative offsets:

```rust
AsmInstruction::Branch(label, condition) => vec![
    inst(
        options.opcode_with_offset(BranchOpcode),
        condition,
        F::ZERO,
        labels(label) - pc,  // Calculate relative offset
        AS::Native,
        AS::Immediate,
    ),
]
```

### 4. Extension Field Handling

Extension field operations require special care:

```rust
// For extension field comparison (BneE)
AsmInstruction::BneE(label, lhs, rhs) => (0..EF::D)
    .map(|i| {
        let current_pc = pc + F::from_canonical_usize(i * DEFAULT_PC_STEP as usize);
        inst(
            options.opcode_with_offset(NativeBranchEqualOpcode(BranchEqualOpcode::BNE)),
            i32_f(lhs + (i as i32)),
            i32_f(rhs + (i as i32)),
            labels(label) - current_pc,  // Adjust for instruction position
            AS::Native,
            AS::Native,
        )
    })
    .collect()
```

## Common Implementation Tasks

### 1. Implementing Memory Operations

```rust
// Load with indexing
AsmInstruction::LoadIndexed(dst, base, index, scale) => vec![
    inst(
        options.opcode_with_offset(NativeLoadStoreOpcode::LOADW),
        i32_f(dst),
        index * scale,  // Compile-time calculation
        i32_f(base),
        AS::Native,
        AS::Native,
    ),
]
```

### 2. Implementing Arithmetic Operations

```rust
// Three-address arithmetic
AsmInstruction::ArithOp(dst, src1, src2) => vec![
    inst_med(
        options.opcode_with_offset(FieldArithmeticOpcode::OP),
        i32_f(dst),
        i32_f(src1),
        i32_f(src2),
        AS::Native,
        AS::Native,
        AS::Native,
    ),
]
```

### 3. Implementing System Calls

```rust
// System operation with phantom
AsmInstruction::SysCall(syscall_id) => vec![
    Instruction::phantom(
        PhantomDiscriminant(syscall_id as u16),
        F::ZERO,
        F::ZERO,
        0,
    ),
    // Follow with actual system instruction if needed
]
```

## Advanced Techniques

### 1. Conditional Compilation

```rust
if options.enable_feature {
    vec![/* feature-specific instructions */]
} else {
    vec![]  // No-op when disabled
}
```

### 2. Optimization Opportunities

```rust
// Combine multiple operations when possible
match (current_inst, next_inst) {
    (AsmInstruction::LoadFI(..), AsmInstruction::AddFI(..)) => {
        // Potentially combine into single fused instruction
    }
    _ => // Normal conversion
}
```

### 3. Debug Information Management

```rust
let instructions = convert_instruction(...);
let debug_infos = vec![debug_info.clone(); instructions.len()];
```

## Testing Strategies

### 1. Unit Tests

```rust
#[test]
fn test_instruction_conversion() {
    let options = CompilerOptions::default();
    let inst = AsmInstruction::AddF(1, 2, 3);
    let program = convert_instruction(
        inst,
        None,
        F::ZERO,
        |l| l,
        &options,
    );
    assert_eq!(program.instructions.len(), 1);
    assert_eq!(program.instructions[0].opcode, 
               options.opcode_with_offset(FieldArithmeticOpcode::ADD));
}
```

### 2. Round-Trip Tests

```rust
#[test]
fn test_round_trip() {
    // Convert ASM → VM → Execute → Verify result
    let asm_program = create_test_program();
    let vm_program = convert_program(asm_program, options);
    let result = execute_program(vm_program);
    assert_eq!(result, expected);
}
```

### 3. Property-Based Tests

```rust
#[quickcheck]
fn prop_conversion_preserves_semantics(program: AssemblyCode<F, EF>) {
    let converted = convert_program(program.clone(), CompilerOptions::default());
    // Verify semantic equivalence
}
```

## Performance Optimization

### 1. Minimize Allocations

```rust
// Pre-allocate when size is known
let mut instructions = Vec::with_capacity(expected_size);
```

### 2. Compile-Time Calculations

```rust
// Do calculations at compile time
const OFFSET: usize = SIZE * COUNT + BASE;
inst(..., F::from_canonical_usize(OFFSET), ...)
```

### 3. Avoid Redundant Conversions

```rust
// Cache common field conversions
let zero = F::ZERO;
let one = F::ONE;
```

## Error Handling

### 1. Validation

```rust
// Add assertions for invariants
assert!(x_bit <= 16, "x_bit out of range");
assert!(y_bit <= 14, "y_bit out of range");
```

### 2. Graceful Degradation

```rust
match instruction {
    AsmInstruction::Unknown => {
        // Emit trap instruction instead of panicking
        vec![create_trap_instruction()]
    }
}
```

## Integration Checklist

When integrating new features:

- [ ] Add instruction variant to AsmInstruction
- [ ] Implement conversion in convert_instruction
- [ ] Add necessary opcodes to instruction set
- [ ] Update CompilerOptions if needed
- [ ] Write unit tests
- [ ] Update documentation
- [ ] Verify debug info preservation
- [ ] Check performance impact
- [ ] Test with both field types (F and EF)

## Debugging Techniques

### 1. Instruction Tracing

```rust
#[cfg(debug_assertions)]
eprintln!("Converting: {:?} at PC {}", instruction, pc);
```

### 2. Label Resolution Verification

```rust
let resolved = labels(label);
assert!(resolved >= pc, "Backward branch to {}", resolved);
```

### 3. Field Range Checking

```rust
debug_assert!(value.as_canonical_u64() < F::ORDER);
```

## Common Pitfalls and Solutions

### Pitfall 1: Incorrect PC Calculation
**Solution**: Always account for instruction position in multi-instruction sequences

### Pitfall 2: Wrong AS Usage
**Solution**: Use AS::Native for memory, AS::Immediate for constants

### Pitfall 3: Extension Field Misalignment
**Solution**: Ensure all components are handled in correct order

### Pitfall 4: Debug Info Loss
**Solution**: Replicate debug info for each generated instruction

### Pitfall 5: Label Resolution Errors
**Solution**: Test with programs containing forward and backward branches