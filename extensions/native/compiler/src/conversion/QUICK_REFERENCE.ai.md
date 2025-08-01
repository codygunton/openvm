# Conversion Module Quick Reference

## Core Types

```rust
// Compiler configuration
CompilerOptions {
    word_size: usize,           // Default: 8
    enable_cycle_tracker: bool, // Default: false
}

// Address specifier
enum AS {
    Immediate = 0,  // Constant value
    Native = 4,     // Memory address
}
```

## Helper Functions

```rust
// 5-operand instruction
inst(opcode, a, b, c, d: AS, e: AS) -> Instruction<F>

// 6-operand instruction  
inst_med(opcode, a, b, c, d: AS, e: AS, f: AS) -> Instruction<F>

// Convert i32 to field element
i32_f(x: i32) -> F
```

## Common Instruction Patterns

### Memory Operations

```rust
// Load: mem[dst] ← mem[mem[src] + offset]
LoadFI(dst, src, index, size, offset) =>
    inst(LOADW, i32_f(dst), index*size+offset, i32_f(src), AS::Native, AS::Native)

// Store: mem[mem[addr] + offset] ← mem[val]
StoreFI(val, addr, index, size, offset) =>
    inst(STOREW, i32_f(val), index*size+offset, i32_f(addr), AS::Native, AS::Native)
```

### Arithmetic Operations

```rust
// Binary op: mem[dst] ← mem[lhs] op mem[rhs]
AddF(dst, lhs, rhs) =>
    inst_med(ADD, i32_f(dst), i32_f(lhs), i32_f(rhs), AS::Native, AS::Native, AS::Native)

// Immediate op: mem[dst] ← mem[lhs] op imm
AddFI(dst, lhs, imm) =>
    inst_med(ADD, i32_f(dst), i32_f(lhs), imm, AS::Native, AS::Native, AS::Immediate)
```

### Control Flow

```rust
// Jump: pc ← label, mem[dst] ← pc
Jump(dst, label) =>
    inst(JAL, i32_f(dst), labels(label)-pc, F::ZERO, AS::Native, AS::Immediate)

// Branch: if mem[lhs] != mem[rhs], pc ← label
Bne(label, lhs, rhs) =>
    inst(BNE, i32_f(lhs), i32_f(rhs), labels(label)-pc, AS::Native, AS::Native)
```

## Extension Field Operations

```rust
// Extension arithmetic (generates 1 instruction)
AddE(dst, lhs, rhs) =>
    inst(FE4ADD, i32_f(dst), i32_f(lhs), i32_f(rhs), AS::Native, AS::Native)

// Extension branch (generates EF::D instructions)
BneE(label, lhs, rhs) =>
    (0..EF::D).map(|i| 
        inst(BNE, i32_f(lhs+i), i32_f(rhs+i), 
             labels(label)-(pc+F::from(i*DEFAULT_PC_STEP)), 
             AS::Native, AS::Native))
```

## Special Instructions

### System Operations

```rust
// Halt execution
Halt => inst(TERMINATE, F::ZERO, F::ZERO, F::ZERO, AS::Immediate, AS::Immediate)

// Trap (panic)
Trap => [
    Instruction::phantom(SysPhantom::DebugPanic as u16, F::ZERO, F::ZERO, 0),
    inst(TERMINATE, F::ZERO, F::ZERO, F::ONE, AS::Immediate, AS::Immediate)
]
```

### Phantom Instructions

```rust
// Print value
PrintF(src) => 
    Instruction::phantom(NativePhantom::Print as u16, i32_f(src), F::ZERO, AS::Native as u16)

// Cycle tracking
CycleTrackerStart() => 
    Instruction::debug(SysPhantom::CtStart as u16)  // Only if enabled
```

### Cryptographic Operations

```rust
// Poseidon2 compression
Poseidon2Compress(dst, src1, src2) =>
    inst(COMP_POS2, i32_f(dst), i32_f(src1), i32_f(src2), AS::Native, AS::Native)

// FRI opening
FriReducedOpening(a, b, length, alpha, res, hint_id, is_init) =>
    Instruction { opcode: FRI_REDUCED_OPENING, a: i32_f(a), ... }
```

## Opcode Access

```rust
// Get opcode with proper offset
options.opcode_with_offset(LocalOpcode) -> VmOpcode
```

## PC and Labels

```rust
// PC increments
DEFAULT_PC_STEP = 4

// Label resolution
labels: impl Fn(F) -> F  // Maps label to absolute PC

// PC-relative offset calculation
offset = labels(label) - current_pc
```

## Quick Conversion Examples

### Simple Assignment
```rust
// ASM: r1 = 42
ImmF(1, F::from(42)) =>
    inst_med(ADD, F::ONE, F::from(42), F::ZERO, AS::Native, AS::Immediate, AS::Native)
```

### Memory Load
```rust
// ASM: r1 = mem[r2 + 8]
LoadFI(1, 2, F::ZERO, F::ONE, F::from(8)) =>
    inst(LOADW, F::ONE, F::from(8), F::from(2), AS::Native, AS::Native)
```

### Conditional Branch
```rust
// ASM: if r1 == r2 goto label
Beq(label, 1, 2) =>
    inst(BEQ, F::ONE, F::from(2), labels(label)-pc, AS::Native, AS::Native)
```

## Conversion Rules Summary

1. **Always use `i32_f()` for register numbers**
2. **Memory operands use `AS::Native`**
3. **Constants use `AS::Immediate`**
4. **Branches use PC-relative offsets**
5. **Extension ops may generate multiple instructions**
6. **Phantom instructions have no runtime effect**
7. **Debug info replicates for multi-instruction sequences**

## Common Opcodes

| Category | Opcodes |
|----------|---------|
| Memory | LOADW, STOREW, HINT_STOREW |
| Arithmetic | ADD, SUB, MUL, DIV |
| Extension | FE4ADD, FE4SUB, BBE4MUL, BBE4DIV |
| Branch | BEQ, BNE, JAL |
| System | TERMINATE, PUBLISH |
| Crypto | COMP_POS2, PERM_POS2, FRI_REDUCED_OPENING |
| Verify | VERIFY_BATCH, RANGE_CHECK |

## Field Conversion Helpers

```rust
// Safe i32 to field conversion
i32_f::<F>(x: i32) -> F {
    assert!(x < F::ORDER_U32 as i32 && x >= -(F::ORDER_U32 as i32));
    if x < 0 {
        -F::from_canonical_u32((-x) as u32)
    } else {
        F::from_canonical_u32(x as u32)
    }
}
```

## Program Structure

```rust
// Every program starts with:
mem[0] ← 0  // inst_med(ADD, F::ZERO, F::ZERO, F::ZERO, AS::Native, AS::Immediate, AS::Immediate)

// Then blocks follow with resolved labels
```