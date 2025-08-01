# ASM Component - Implementation Guide

## Getting Started

### Basic Setup
```rust
use openvm_native_compiler::asm::{AsmBuilder, AsmCompiler, AsmConfig};
use openvm_stark_backend::p3_field::BabyBear;

type F = BabyBear;
type EF = BabyBear4; // Extension field

// Create builder
let builder = AsmBuilder::<F, EF>::default();

// Add operations (see examples below)
// ...

// Compile to assembly
let program = builder.compile_isa();
```

## Implementation Examples

### Example 1: Simple Arithmetic Operation
Implement addition of two variables:

```rust
// DSL IR operation
DslIr::AddV(dst, lhs, rhs)

// Compiler implementation in build()
DslIr::AddV(dst, lhs, rhs) => {
    // Get frame pointers
    let dst_fp = dst.fp();   // e.g., STACK_TOP - 1
    let lhs_fp = lhs.fp();   // e.g., STACK_TOP - 9  
    let rhs_fp = rhs.fp();   // e.g., STACK_TOP - 17
    
    // Generate assembly instruction
    self.push(
        AsmInstruction::AddF(dst_fp, lhs_fp, rhs_fp),
        debug_info
    );
}

// Generated assembly
add   (16777151)fp, (16777143)fp, (16777135)fp
```

### Example 2: Memory Load with Indexing
Load from array with variable index:

```rust
// DSL IR operation  
DslIr::LoadV(var, ptr, index)

// Compiler implementation
DslIr::LoadV(var, ptr, index) => {
    match index.fp() {
        // Constant index - direct load
        IndexTriple::Const(idx, offset, size) => {
            self.push(
                AsmInstruction::LoadFI(
                    var.fp(),    // destination
                    ptr.fp(),    // base address
                    idx,         // constant index
                    size,        // element size
                    offset       // byte offset
                ),
                debug_info
            );
        }
        
        // Variable index - compute address first
        IndexTriple::Var(idx_fp, offset, size) => {
            // A0 = ptr + idx * size
            self.add_scaled(A0, ptr.fp(), idx_fp, size, debug_info.clone());
            
            // Load from computed address
            self.push(
                AsmInstruction::LoadFI(
                    var.fp(),
                    A0,
                    F::ZERO,    // no index
                    F::ZERO,    // no size
                    offset      // just offset
                ),
                debug_info
            );
        }
    }
}

// Generated assembly for variable index
muli  (16777208)fp, (16777147)fp, 4    // A0 = idx * 4
add   (16777208)fp, (16777208)fp, (16777149)fp  // A0 += ptr
lwi   (16777151)fp, (16777208)fp, 0, 0, 0      // load from A0
```

### Example 3: If-Then-Else Control Flow
Conditional execution based on equality:

```rust
// DSL IR operation
DslIr::IfEq(lhs, rhs, then_block, else_block)

// Compiler implementation
DslIr::IfEq(lhs, rhs, then_block, else_block) => {
    let if_compiler = IfCompiler {
        compiler: self,
        lhs: lhs.fp(),
        rhs: ValueOrConst::Val(rhs.fp()),
        is_eq: true,
    };
    
    if_compiler.then_or_else(
        |compiler| compiler.build(then_block),
        |compiler| compiler.build(else_block),
        debug_info
    );
}

// Generated assembly structure
.L0:                          // Current block
    bne   .L2, (16777151)fp, (16777149)fp  // Branch if not equal
.L1:                          // Then block
    // then_block instructions
    j     (16777208)fp, .L3   // Jump to end
.L2:                          // Else block  
    // else_block instructions
.L3:                          // Continue
```

### Example 4: Dynamic Memory Allocation
Allocate array on heap:

```rust
// DSL IR operation
DslIr::Alloc(ptr, len, size)

// Implementation for constant length
pub fn alloc(&mut self, ptr: Ptr<F>, len: RVar<F>, size: usize) {
    match len {
        RVar::Const(len) => {
            // 1. Copy current heap pointer
            self.push(
                AsmInstruction::CopyF(ptr.fp(), HEAP_PTR),
                debug_info.clone()
            );
            
            // 2. Calculate aligned size
            let total = len.as_canonical_u32() as usize * size;
            let aligned = total.div_ceil(self.word_size) * self.word_size;
            
            // 3. Advance heap pointer
            self.push(
                AsmInstruction::AddFI(HEAP_PTR, HEAP_PTR, F::from_canonical_usize(aligned)),
                debug_info.clone()
            );
            
            // 4. Range check to prevent overflow
            let (lo_bits, hi_bits) = lo_hi_bits(MEMORY_BITS as u32);
            self.push(
                AsmInstruction::RangeCheck(HEAP_PTR, lo_bits, hi_bits),
                debug_info
            );
        }
        RVar::Val(len) => {
            // Similar but with runtime calculation
        }
    }
}

// Generated assembly for alloc(ptr, 10, 4)
copy  (16777149)fp, (16777212)    // ptr = HEAP_PTR
addi  (16777212)fp, (16777212)fp, 40  // HEAP_PTR += 40
range_check (16777212)fp, 16, 13      // Check < 2^29
```

### Example 5: Extension Field Operations
Multiply extension field by constant:

```rust
// DSL IR operation
DslIr::MulEI(dst, lhs, rhs)

// Implementation
DslIr::MulEI(dst, lhs, rhs) => {
    // First, load constant into A0 (extension)
    self.assign_exti(A0, rhs, debug_info.clone());
    
    // Then multiply extensions
    self.push(
        AsmInstruction::MulE(dst.fp(), lhs.fp(), A0),
        debug_info
    );
}

// Helper for loading extension constant
fn assign_exti(&mut self, dst: i32, imm: EF, debug_info: Option<DebugInfo>) {
    let components = imm.as_base_slice();
    for i in 0..EF::D {
        self.push(
            AsmInstruction::ImmF(dst + i as i32, components[i]),
            debug_info.clone()
        );
    }
}

// Generated assembly (for degree-4 extension)
imm   (16777208)fp, 123           // A0[0] = 123
imm   (16777209)fp, 456           // A0[1] = 456  
imm   (16777210)fp, 789           // A0[2] = 789
imm   (16777211)fp, 012           // A0[3] = 012
emul  (16777144)fp, (16777148)fp, (16777208)fp  // dst = lhs * A0
```

### Example 6: Assertion with Trap
Runtime assertion that fails on inequality:

```rust
// DSL IR operation
DslIr::AssertEqV(lhs, rhs)

// Implementation
DslIr::AssertEqV(lhs, rhs) => {
    self.assert(lhs.fp(), ValueOrConst::Val(rhs.fp()), false, debug_info)
}

// Assert implementation
pub fn assert(&mut self, lhs: i32, rhs: ValueOrConst<F, EF>, is_eq: bool) {
    let if_compiler = IfCompiler {
        compiler: self,
        lhs,
        rhs,
        is_eq: !is_eq,  // Invert condition
    };
    if_compiler.then_label(self.trap_label, debug_info);
}

// Generated assembly
bne   .L1, (16777151)fp, (16777149)fp  // Branch to trap if not equal
// .L1 contains the trap instruction
```

### Example 7: For Loop Implementation
Iterate with step size:

```rust
// DSL IR operation  
DslIr::ZipFor(starts, end0, step_sizes, loop_vars, block)

// Generated structure
// Initialize loop variables
imm   (16777151)fp, 0             // loop_var = 0

.L2:                               // Loop body start
    // Execute block operations
    // ...
    
    addi  (16777151)fp, (16777151)fp, 1  // loop_var += step
    bnei  .L2, (16777151)fp, 10          // Continue if != end

.L3:                               // Loop exit
```

## Advanced Patterns

### Pattern 1: Optimized Extension Field Addition
Add extension to field element (promoting field to extension):

```rust
fn add_ext_felt(&mut self, dst: Ext<F, EF>, lhs: Ext<F, EF>, rhs: Felt<F>) {
    // Only first component changes
    self.push(
        AsmInstruction::AddF(dst.fp(), lhs.fp(), rhs.fp()),
        debug_info.clone()
    );
    
    // Copy other components
    for i in 1..EF::D {
        self.push(
            AsmInstruction::CopyF(dst.fp() + i, lhs.fp() + i),
            debug_info.clone()
        );
    }
}
```

### Pattern 2: Efficient Address Calculation
Computing `base + index * scale + offset`:

```rust
fn add_scaled(&mut self, dst: i32, base: i32, index: i32, scale: F) {
    if scale == F::ONE {
        // Optimize for scale = 1
        self.push(AsmInstruction::AddF(dst, base, index), debug_info);
    } else {
        // General case: dst = index * scale + base
        self.push(AsmInstruction::MulFI(dst, index, scale), debug_info.clone());
        self.push(AsmInstruction::AddF(dst, dst, base), debug_info);
    }
}
```

### Pattern 3: Safe Division with Inverse
Division by immediate using multiplication:

```rust
DslIr::DivEFI(dst, lhs, rhs) => {
    // Instead of division, multiply by inverse
    let inverse = rhs.inverse();
    self.mul_ext_felti(dst, lhs, inverse, debug_info);
}
```

## Debugging Techniques

### Add Debug Prints
```rust
// Before critical operation
self.push(AsmInstruction::PrintF(value.fp()), None);

// After operation  
self.push(AsmInstruction::PrintF(result.fp()), None);
```

### Cycle Tracking
```rust
// Start tracking
self.push(
    AsmInstruction::CycleTrackerStart(),
    Some(DebugInfo { dsl_instruction: "START-LOOP".into(), trace: None })
);

// End tracking
self.push(
    AsmInstruction::CycleTrackerEnd(),
    Some(DebugInfo { dsl_instruction: "END-LOOP".into(), trace: None })
);
```

### Label Management
```rust
// Create meaningful labels
self.function_labels.insert("main_loop".to_string(), F::from_canonical_u32(5));

// Reference in assembly output
// Shows as "main_loop:" instead of ".L5:"
```

## Performance Tips

1. **Use Immediate Instructions**: `AddFI` is faster than loading constant then `AddF`
2. **Minimize Memory Access**: Keep frequently used values in low frame pointer offsets
3. **Batch Extension Operations**: Process all components together
4. **Align Allocations**: Round up to word_size for better memory access
5. **Optimize Common Patterns**: Special-case scale=1 in address calculations

## Common Mistakes to Avoid

1. **Forgetting Debug Info**: Always pass debug_info to push()
2. **Manual FP Calculation**: Use .fp() methods instead
3. **Ignoring Overflow**: Range-check all dynamic allocations
4. **Wrong Extension Layout**: Extension fields use consecutive memory
5. **Missing Basic Blocks**: Control flow requires proper block structure

## Testing Your Implementation

```rust
#[test]
fn test_new_operation() {
    let mut compiler = AsmCompiler::<F, EF>::new(4);
    let ops = vec![
        DslIr::YourNewOp(args...),
    ];
    
    compiler.build(TracedVec::new(ops));
    let code = compiler.code();
    
    // Verify assembly output
    let asm_str = format!("{}", code);
    assert!(asm_str.contains("expected_instruction"));
}
```