# MulH Component Quick Reference

## Creating a MulH Chip

```rust
use openvm_circuit_primitives::{
    bitwise_op_lookup::SharedBitwiseOperationLookupChip,
    range_tuple::SharedRangeTupleCheckerChip,
};
use openvm_rv32im_circuit::{
    adapters::Rv32MultAdapterChip,
    mulh::{MulHCoreChip, Rv32MulHChip},
};

// Create shared lookup tables
let bitwise_chip = SharedBitwiseOperationLookupChip::<8>::new(bitwise_bus);
let range_checker = SharedRangeTupleCheckerChip::new(range_tuple_bus);

// Create the MulH chip
let mulh_chip = Rv32MulHChip::new(
    Rv32MultAdapterChip::new(exec_bus, program_bus, memory_bridge),
    MulHCoreChip::new(bitwise_chip.clone(), range_checker.clone()),
    offline_memory_mutex,
);
```

## Executing MulH Instructions

```rust
use openvm_instructions::{instruction::Instruction, LocalOpcode};
use openvm_rv32im_transpiler::MulHOpcode;

// MULH: rd = (rs1 * rs2) >> 32 (signed)
let mulh_insn = Instruction::from_usize(
    MulHOpcode::MULH.global_opcode(),
    [rd, rs1, rs2, 1, 0]
);

// MULHSU: rd = (rs1 * rs2) >> 32 (signed × unsigned)
let mulhsu_insn = Instruction::from_usize(
    MulHOpcode::MULHSU.global_opcode(),
    [rd, rs1, rs2, 1, 0]
);

// MULHU: rd = (rs1 * rs2) >> 32 (unsigned)
let mulhu_insn = Instruction::from_usize(
    MulHOpcode::MULHU.global_opcode(),
    [rd, rs1, rs2, 1, 0]
);

// Execute the instruction
chip.execute_instruction(&mulh_insn);
```

## Assembly Examples

```asm
# Full 64-bit multiplication (signed)
mul  a0, a1, a2    # Lower 32 bits
mulh a3, a1, a2    # Upper 32 bits
# Result: a3:a0 contains full 64-bit product

# Check for multiplication overflow
mul  t0, a0, a1    # Lower bits (result we care about)
mulh t1, a0, a1    # Upper bits (should be 0 or -1 for no overflow)
srai t2, t0, 31    # Sign extend bit 31 of result
bne  t1, t2, overflow_handler

# Unsigned 64-bit multiplication
mul   a0, a1, a2   # Lower 32 bits
mulhu a3, a1, a2   # Upper 32 bits

# Mixed signed/unsigned multiplication
mulhsu a0, a1, a2  # a1 is signed, a2 is unsigned
```

## Direct Usage of Core Functions

```rust
use openvm_rv32im_circuit::mulh::core::run_mulh;
use openvm_rv32im_transpiler::MulHOpcode;

// Compute high multiplication directly
let x: [u32; 4] = [0x12, 0x34, 0x56, 0x78];  // Little-endian limbs
let y: [u32; 4] = [0x9A, 0xBC, 0xDE, 0xF0];

let (high_bits, low_bits, carries, x_ext, y_ext) = 
    run_mulh::<4, 8>(MulHOpcode::MULH, &x, &y);

// high_bits contains the upper 32 bits of the multiplication
```

## Common Patterns

### 64-bit Result from 32-bit Multiplication
```rust
fn mul64(a: u32, b: u32) -> u64 {
    let low = mul(a, b);
    let high = mulhu(a, b);
    ((high as u64) << 32) | (low as u64)
}
```

### Overflow Detection
```rust
fn checked_mul_i32(a: i32, b: i32) -> Option<i32> {
    let result = mul(a, b);
    let high = mulh(a, b);
    let expected_high = (result >> 31) as i32; // Sign extension
    
    if high == expected_high {
        Some(result)
    } else {
        None // Overflow occurred
    }
}
```

### Multi-Precision Multiplication
```rust
// Multiply two 64-bit numbers on 32-bit system
fn mul64x64(a_hi: u32, a_lo: u32, b_hi: u32, b_lo: u32) -> (u32, u32, u32, u32) {
    let ll = mulhu(a_lo, b_lo);
    let lh = mulhu(a_lo, b_hi);
    let hl = mulhu(a_hi, b_lo);
    let hh = mulhu(a_hi, b_hi);
    
    // Combine partial products...
}
```

## Testing the MulH Component

```rust
use openvm_circuit::arch::testing::VmChipTestBuilder;

// Setup test environment
let mut tester = VmChipTestBuilder::default();

// Write operands to memory
tester.write::<4>(1, rs1_addr, operand1);
tester.write::<4>(1, rs2_addr, operand2);

// Execute MULH instruction
tester.execute(&mut chip, &mulh_instruction);

// Read and verify result
let result = tester.read::<4>(1, rd_addr);
assert_eq!(result, expected_high_bits);
```

## Performance Notes

- MulH operations share lookup tables with other arithmetic operations
- All three variants (MULH, MULHSU, MULHU) have similar performance
- Constraint count scales with number of limbs (typically 4 for RV32)
- Sign extension checks only applied when needed (not for MULHU)

## Common Issues

1. **Sign Extension**: Ensure correct opcode for signed vs unsigned
2. **Limb Ordering**: Results are in little-endian limb format
3. **Overflow**: MulH helps detect but doesn't prevent overflow
4. **Instruction Format**: Always use `[rd, rs1, rs2, 1, 0]` operand format