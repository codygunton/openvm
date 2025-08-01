# BaseAlu Quick Reference

## Component Overview
- **Purpose**: Implements ADD, SUB, XOR, OR, AND for RV32
- **Location**: `extensions/rv32im/circuit/src/base_alu/`
- **Key Trait**: `VmCoreChip<F, I>`

## Key Types

```rust
// Main chip type
type Rv32BaseAluChip<F> = VmChipWrapper<F, Adapter, Core>;

// Column structure
struct BaseAluCoreCols<T, NUM_LIMBS, LIMB_BITS> {
    a: [T; NUM_LIMBS],  // Result
    b: [T; NUM_LIMBS],  // Operand 1
    c: [T; NUM_LIMBS],  // Operand 2
    opcode_*_flag: T,   // Operation flags
}

// Execution record
struct BaseAluCoreRecord<T, NUM_LIMBS, LIMB_BITS> {
    opcode: BaseAluOpcode,
    a, b, c: [T; NUM_LIMBS],
}
```

## Opcodes

```rust
enum BaseAluOpcode {
    ADD = 0x200,
    SUB = 0x201,
    XOR = 0x202,
    OR  = 0x203,
    AND = 0x204,
}
```

## Key Functions

```rust
// Execute ALU operation
fn run_alu(opcode, x, y) -> [u32; NUM_LIMBS]

// Individual operations
fn run_add(x, y) -> [u32; NUM_LIMBS]    // With carry
fn run_subtract(x, y) -> [u32; NUM_LIMBS] // With borrow
fn run_xor(x, y) -> [u32; NUM_LIMBS]     // Bitwise
fn run_or(x, y) -> [u32; NUM_LIMBS]      // Bitwise
fn run_and(x, y) -> [u32; NUM_LIMBS]     // Bitwise
```

## Constraint Formulas

### Arithmetic Operations
```
ADD: a[i] = (b[i] + c[i] + carry[i-1]) mod 2^LIMB_BITS
SUB: a[i] = (b[i] - c[i] - borrow[i-1]) mod 2^LIMB_BITS
```

### Bitwise Operations
```
XOR: a[i] = b[i] ^ c[i]
OR:  a[i] = b[i] | c[i]  
AND: a[i] = b[i] & c[i]
```

### Bitwise Lookup Encoding
```
XOR result: x_xor_y = a[i]
OR result:  x_xor_y = 2*a[i] - b[i] - c[i]
AND result: x_xor_y = b[i] + c[i] - 2*a[i]
```

## Usage Example

```rust
// Create chip
let bitwise_chip = SharedBitwiseOperationLookupChip::new(bus);
let alu_chip = Rv32BaseAluChip::new(
    adapter,
    BaseAluCoreChip::new(bitwise_chip, offset),
    memory,
);

// Execute instruction
let instruction = Instruction {
    opcode: BaseAluOpcode::ADD.global_opcode(),
    // ... operands
};
alu_chip.execute_instruction(&instruction, pc, reads)?;
```

## Testing Commands

```bash
# Run all base_alu tests
cargo test -p openvm-rv32im-circuit base_alu

# Specific test suites
cargo test rv32_alu_add_rand_test
cargo test rv32_alu_negative_test
cargo test run_add_sanity_test
```

## Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| Carry overflow | Ensure carry values are boolean constrained |
| Range violations | Check all limbs are < 2^LIMB_BITS |
| Wrong opcode | Verify offset alignment with transpiler |
| Interaction errors | Ensure bitwise lookups are requested |

## Integration Points

- **Memory**: Via `Rv32BaseAluAdapterChip`
- **Bitwise Lookup**: Via `SharedBitwiseOperationLookupChip`
- **VM**: Through `VmChipWrapper` and execution bus
- **Transpiler**: Maps RISC-V instructions to opcodes

## Performance Notes

- Constraint degree: 3 (optimized)
- Limb size: 8 bits (for RV32)
- Number of limbs: 4 (32 bits total)
- Lookup batching: Enabled for efficiency