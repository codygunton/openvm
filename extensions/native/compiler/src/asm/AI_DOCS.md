# ASM Component - Detailed Documentation

## Overview
The ASM component is the assembly generation layer of the OpenVM native compiler. It translates high-level DSL IR operations into low-level assembly instructions that can be executed by the OpenVM runtime. This component handles memory management, control flow, arithmetic operations, and cryptographic primitives.

## Core Concepts

### Assembly Instructions
The assembly language supports a rich set of instructions categorized into:

1. **Memory Operations**
   - `LoadFI/LoadEI`: Load field/extension elements from memory
   - `StoreFI/StoreEI`: Store field/extension elements to memory
   - Indexed addressing with offset support

2. **Arithmetic Operations**
   - Field operations: AddF, SubF, MulF, DivF
   - Extension field operations: AddE, SubE, MulE, DivE
   - Immediate variants for constants

3. **Control Flow**
   - Unconditional jump: `Jump`
   - Conditional branches: `Beq/Bne` (equal/not equal)
   - Support for both field and extension field comparisons

4. **Cryptographic Operations**
   - `Poseidon2Permute`: State permutation
   - `Poseidon2Compress`: 2-to-1 compression
   - `FriReducedOpening`: FRI protocol support
   - `VerifyBatch`: Batch verification for commitments

5. **Special Operations**
   - `Trap`: Assertion failure handler
   - `Halt`: Program termination
   - `RangeCheck`: Bit range verification
   - Print operations for debugging

### Memory Model

The memory is organized as a 29-bit address space (512MB) with the following layout:

```
Address Space (2^29 bytes)
┌─────────────────────────┐ MEMORY_TOP (2^29 - 4)
│                         │
│         Heap           │ Dynamic allocation
│     (grows upward)     │
│                         │
├─────────────────────────┤ HEAP_START_ADDRESS (2^24)
│     HEAP_PTR (-4)      │ Current heap pointer
├─────────────────────────┤
│       A0 (-8)          │ Utility register
├─────────────────────────┤
│                         │
│        Stack           │ Local variables
│    (fixed layout)      │
│                         │
└─────────────────────────┘ STACK_TOP (-64)
```

### Frame Pointer System

The compiler uses a frame pointer (fp) system for efficient variable access:

- **Var<F>**: Basic variables stored at positions 1, 2, 9, 10, 17, 18...
  - Formula: `fp = STACK_TOP - (8 * (index / 2) + 1 + (index % 2))`

- **Felt<F>**: Field elements at positions 3, 4, 11, 12, 19, 20...
  - Formula: `fp = STACK_TOP - ((index >> 1) << 3) + 3 + (index & 1))`

- **Ext<F, EF>**: Extension elements occupy 4 consecutive positions
  - Positions: 5-8, 13-16, 21-24...
  - Formula: `fp = STACK_TOP - 8 * index`

## Compilation Process

### 1. Initialization
```rust
// Create compiler with word size
let mut compiler = AsmCompiler::new(word_size);

// Initialize heap pointer
compiler.push(AsmInstruction::ImmF(HEAP_PTR, HEAP_START_ADDRESS));

// Set up trap handler for assertions
compiler.push(AsmInstruction::j(trap_label + 1));
compiler.basic_block();
compiler.push(AsmInstruction::Trap);
```

### 2. IR Translation
Each `DslIr` operation is translated to one or more assembly instructions:

```rust
match op {
    DslIr::AddV(dst, lhs, rhs) => {
        compiler.push(AsmInstruction::AddF(dst.fp(), lhs.fp(), rhs.fp()));
    }
    DslIr::LoadV(var, ptr, index) => {
        // Handle constant vs variable indexing
        match index.fp() {
            IndexTriple::Const(idx, off, size) => {
                compiler.push(AsmInstruction::LoadFI(var.fp(), ptr.fp(), idx, size, off));
            }
            IndexTriple::Var(idx, off, size) => {
                compiler.add_scaled(A0, ptr.fp(), idx, size);
                compiler.push(AsmInstruction::LoadFI(var.fp(), A0, 0, 0, off));
            }
        }
    }
}
```

### 3. Control Flow Compilation

#### If-Then-Else
The `IfCompiler` manages conditional execution:

```rust
// If-then-else structure
current_block:
    branch_instruction (condition) else_block
then_block:
    then_instructions...
    jump main_flow
else_block:
    else_instructions...
main_flow:
    continue...
```

#### For Loops
The `ZipForCompiler` handles parallel iteration:

```rust
// Initialize loop variables
loop_init:
    set loop_vars to start values
loop_body:
    execute loop body
    increment loop_vars by step_sizes
    branch_if_not_done loop_body
loop_exit:
    continue...
```

### 4. Memory Allocation
Dynamic allocation uses a bump allocator:

```rust
pub fn alloc(&mut self, ptr: Ptr<F>, len: RVar<F>, size: usize) {
    // Copy current heap pointer
    self.push(AsmInstruction::CopyF(ptr.fp(), HEAP_PTR));
    
    // Calculate allocation size (aligned to word_size)
    let aligned_size = size.div_ceil(word_size) * word_size;
    
    // Advance heap pointer
    self.push(AsmInstruction::AddFI(HEAP_PTR, HEAP_PTR, aligned_size));
    
    // Range check to prevent overflow
    self.push(AsmInstruction::RangeCheck(HEAP_PTR, lo_bits, hi_bits));
}
```

## Extension Field Operations

The compiler includes specialized methods for extension field arithmetic:

### Addition with Extension Immediate
```rust
fn add_ext_exti(&mut self, dst: Ext<F, EF>, lhs: Ext<F, EF>, rhs: EF) {
    let rhs_components = rhs.as_base_slice();
    for i in 0..EF::D {
        self.push(AsmInstruction::AddFI(
            dst.fp() + i, 
            lhs.fp() + i, 
            rhs_components[i]
        ));
    }
}
```

### Multiplication with Field Element
```rust
fn mul_ext_felt(&mut self, dst: Ext<F, EF>, lhs: Ext<F, EF>, rhs: Felt<F>) {
    // Multiply each component
    for i in 0..EF::D {
        self.push(AsmInstruction::MulF(
            dst.fp() + i,
            lhs.fp() + i,
            rhs.fp()
        ));
    }
}
```

## Debug Support

### Debug Information
Each instruction can carry debug information:
```rust
pub struct DebugInfo {
    pub dsl_instruction: String,  // Original DSL operation
    pub trace: Option<String>,    // Stack trace or source location
}
```

### Cycle Tracking
Performance profiling with cycle trackers:
```rust
DslIr::CycleTrackerStart(name) => {
    compiler.push(AsmInstruction::CycleTrackerStart(), 
                  DebugInfo { dsl_instruction: format!("CT-{}", name), ... });
}
```

## Error Handling

### Assertions
Runtime assertions use the trap mechanism:
```rust
pub fn assert(&mut self, lhs: i32, rhs: ValueOrConst<F, EF>, is_eq: bool) {
    // Generate conditional branch to trap block
    let if_compiler = IfCompiler { lhs, rhs, is_eq: !is_eq };
    if_compiler.then_label(self.trap_label);
}
```

### Range Checking
Bit decomposition verification:
```rust
fn lo_hi_bits(bits: u32) -> (i32, i32) {
    let lo_bits = bits.min(16);
    let hi_bits = bits.max(16) - 16;
    (lo_bits as i32, hi_bits as i32)
}
```

## Assembly Output

The compiler produces human-readable assembly:
```asm
.L0:
    imm   (16777212)fp, (16777216)
    j     (16777208)fp, .L2
.L1:
    trap
.L2:
    addi  (16777149)fp, (16777151)fp, 0
    lwi   (16777147)fp, (16777149)fp, 0, 0, 0
    beqi  .L4, (16777147)fp, 0
```

## Performance Considerations

1. **Register Allocation**: Uses stack-based allocation with fixed frame pointer offsets
2. **Instruction Selection**: Immediate variants reduce memory accesses
3. **Extension Field Optimization**: Component-wise operations avoid temporary allocations
4. **Memory Alignment**: Heap allocations aligned to word size
5. **Branch Prediction**: Trap block placed early for unlikely assertion failures

## Integration with OpenVM

The ASM component integrates with:
- **IR Layer**: Receives DslIr operations
- **Conversion Layer**: Produces Program<F> via convert_program()
- **Circuit Layer**: Uses instruction definitions
- **Runtime**: Assembly executed by OpenVM interpreter

## Best Practices

1. **Memory Safety**: Always range-check heap allocations
2. **Debug Info**: Preserve source operation information
3. **Control Flow**: Use structured if/loop compilers
4. **Extension Fields**: Use specialized methods for efficiency
5. **Error Handling**: Fail fast with trap on assertions