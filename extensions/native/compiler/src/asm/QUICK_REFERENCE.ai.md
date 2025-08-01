# ASM Component - Quick Reference

## Memory Constants
```rust
MEMORY_BITS: 29                    // 2^29 = 512MB address space
MEMORY_TOP: (1 << 29) - 4         // Highest valid address
HEAP_START_ADDRESS: 1 << 24       // 16MB - heap start
HEAP_PTR: HEAP_START_ADDRESS - 4  // Heap pointer location
A0: HEAP_START_ADDRESS - 8        // Utility register
STACK_TOP: HEAP_START_ADDRESS - 64 // Stack begins here
```

## Frame Pointer Formulas
```rust
// Variables (1, 2, 9, 10, 17, 18...)
Var::fp() = STACK_TOP - (8 * (index / 2) + 1 + (index % 2))

// Field elements (3, 4, 11, 12, 19, 20...)  
Felt::fp() = STACK_TOP - ((index >> 1) << 3) + 3 + (index & 1))

// Extension elements (5-8, 13-16, 21-24...)
Ext::fp() = STACK_TOP - 8 * index

// Pointers use their address variable's fp
Ptr::fp() = address.fp()
```

## Key Assembly Instructions

### Memory Operations
```asm
lwi   (dst)fp, (src)fp, index, size, offset   // Load word
lei   (dst)fp, (src)fp, index, size, offset   // Load extension
swi   (val)fp, (src)fp, index, size, offset   // Store word
sei   (val)fp, (src)fp, index, size, offset   // Store extension
```

### Arithmetic - Field
```asm
add   (dst)fp, (lhs)fp, (rhs)fp    // Addition
addi  (dst)fp, (lhs)fp, imm        // Add immediate
sub   (dst)fp, (lhs)fp, (rhs)fp    // Subtraction
subi  (dst)fp, (lhs)fp, imm        // Subtract immediate
mul   (dst)fp, (lhs)fp, (rhs)fp    // Multiplication
muli  (dst)fp, (lhs)fp, imm        // Multiply immediate
div   (dst)fp, (lhs)fp, (rhs)fp    // Division
divi  (dst)fp, (lhs)fp, imm        // Divide immediate
```

### Arithmetic - Extension Field
```asm
eadd  (dst)fp, (lhs)fp, (rhs)fp    // Extension add
esub  (dst)fp, (lhs)fp, (rhs)fp    // Extension subtract
emul  (dst)fp, (lhs)fp, (rhs)fp    // Extension multiply
ediv  (dst)fp, (lhs)fp, (rhs)fp    // Extension divide
```

### Control Flow
```asm
j     (addr)fp, label              // Jump
beq   label, (lhs)fp, (rhs)fp     // Branch if equal
bne   label, (lhs)fp, (rhs)fp     // Branch if not equal
beqi  label, (lhs)fp, imm         // Branch if equal immediate
bnei  label, (lhs)fp, imm         // Branch if not equal immediate
```

### Special Operations
```asm
trap                               // Assertion failure
halt                               // Stop execution
range_check (v)fp, lo_bits, hi_bits // Verify bit range
poseidon2_permute (dst)fp, (src)fp // Crypto permutation
```

## Common Patterns

### Heap Allocation
```rust
// 1. Copy heap pointer to dst
self.push(AsmInstruction::CopyF(ptr.fp(), HEAP_PTR));

// 2. Advance heap pointer by aligned size
let aligned = size.div_ceil(word_size) * word_size;
self.push(AsmInstruction::AddFI(HEAP_PTR, HEAP_PTR, aligned));

// 3. Range check new heap pointer
self.push(AsmInstruction::RangeCheck(HEAP_PTR, lo_bits, hi_bits));
```

### Variable Indexing
```rust
match index.fp() {
    IndexTriple::Const(idx, off, size) => {
        // Direct indexed load
        self.push(AsmInstruction::LoadFI(dst.fp(), src.fp(), idx, size, off));
    }
    IndexTriple::Var(idx_fp, off, size) => {
        // Calculate address first
        self.add_scaled(A0, src.fp(), idx_fp, size);
        self.push(AsmInstruction::LoadFI(dst.fp(), A0, 0, 0, off));
    }
}
```

### If-Then-Else Structure
```asm
.Lcurrent:
    bnei  .Lelse, (lhs)fp, rhs    // Branch to else if condition false
.Lthen:
    // then block code
    j     (A0)fp, .Lend           // Jump to end
.Lelse:
    // else block code
.Lend:
    // continue
```

### Loop Structure
```asm
.Linit:
    imm   (loop_var)fp, start      // Initialize loop variable
.Lloop:
    // loop body
    addi  (loop_var)fp, (loop_var)fp, step  // Increment
    bnei  .Lloop, (loop_var)fp, end         // Continue if not done
.Lexit:
    // continue after loop
```

## Extension Field Helpers

### Component-wise Operations
```rust
// Addition with immediate
for i in 0..EF::D {
    self.push(AsmInstruction::AddFI(dst.fp() + i, lhs.fp() + i, rhs[i]));
}

// Multiplication with field element
for i in 0..EF::D {
    self.push(AsmInstruction::MulF(dst.fp() + i, lhs.fp() + i, rhs.fp()));
}
```

## DslIr to Assembly Mapping

| DslIr Operation | Assembly Instructions |
|----------------|----------------------|
| `ImmV(dst, val)` | `imm (dst)fp, val` |
| `AddV(dst, lhs, rhs)` | `add (dst)fp, (lhs)fp, (rhs)fp` |
| `LoadV(dst, ptr, idx)` | `lwi (dst)fp, (ptr)fp, idx, size, off` |
| `IfEq(lhs, rhs, then, else)` | `bne` + blocks + `j` |
| `AssertEqV(lhs, rhs)` | `bne trap_label, (lhs)fp, (rhs)fp` |
| `Alloc(ptr, len, size)` | `copy` + `addi` + `range_check` |

## Quick Debugging

### Print Values
```rust
DslIr::PrintF(val) => self.push(AsmInstruction::PrintF(val.fp()))
DslIr::PrintV(val) => self.push(AsmInstruction::PrintV(val.fp()))
DslIr::PrintE(val) => self.push(AsmInstruction::PrintE(val.fp()))
```

### Cycle Tracking
```rust
DslIr::CycleTrackerStart(name) => self.push(AsmInstruction::CycleTrackerStart())
DslIr::CycleTrackerEnd(name) => self.push(AsmInstruction::CycleTrackerEnd())
```

## Important Notes
- Always range-check heap allocations
- Use A0 as scratch register for address calculations
- Trap block is always at label 1
- Extension fields use D consecutive memory locations
- Debug info should be preserved through compilation