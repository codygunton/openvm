# Shift Component - Quick Reference

## Key Constants
```rust
RV32_REGISTER_NUM_LIMBS = 4  // 32-bit word as 4 bytes
RV32_CELL_BITS = 8           // Each limb is 8 bits
MAX_SHIFT = 31               // Maximum shift amount (5 bits used)
```

## Shift Operations
| Opcode | Operation | Description |
|--------|-----------|-------------|
| SLL | Shift Left Logical | `a = b << c`, fill with zeros |
| SRL | Shift Right Logical | `a = b >> c`, fill with zeros |
| SRA | Shift Right Arithmetic | `a = b >> c`, fill with sign bit |

## Core Types
```rust
// Column layout for trace
ShiftCoreCols<T, NUM_LIMBS, LIMB_BITS>

// AIR constraints
ShiftCoreAir<NUM_LIMBS, LIMB_BITS>

// Execution record
ShiftCoreRecord<T, NUM_LIMBS, LIMB_BITS>

// Main chip
ShiftCoreChip<NUM_LIMBS, LIMB_BITS>

// RV32 wrapper
Rv32ShiftChip<F> = VmChipWrapper<F, Adapter, Core>
```

## Key Functions
```rust
// Execute shift operation
run_shift(opcode, x, y) -> (result, limb_shift, bit_shift)

// Extract shift amounts from y
get_shift(y) -> (limb_shift, bit_shift)
  limb_shift = (y[0] % 32) / 8
  bit_shift = (y[0] % 32) % 8

// Shift implementations
run_shift_left(x, y) -> (result, limb_shift, bit_shift)
run_shift_right(x, y, logical) -> (result, limb_shift, bit_shift)
```

## Constraint Checklist
- [ ] Opcode flags are boolean and sum to 1
- [ ] Bit shift markers are boolean and sum to 1
- [ ] Limb shift markers are boolean and sum to 1
- [ ] Bit multipliers match shift amount (2^bit_shift)
- [ ] Carry values are range-checked
- [ ] Sign bit extraction uses XOR lookup
- [ ] All limb values are range-checked

## Testing Patterns
```rust
// Positive test with random inputs
run_rv32_shift_rand_test(opcode, num_tests)

// Negative test with pranked trace
run_rv32_shift_negative_test(opcode, a, b, c, prank_vals, is_interaction_error)

// Prank specific values
ShiftPrankValues {
    bit_shift: Option<u32>,
    bit_multiplier_left: Option<u32>,
    bit_multiplier_right: Option<u32>,
    b_sign: Option<u32>,
    bit_shift_marker: Option<[u32; LIMB_BITS]>,
    limb_shift_marker: Option<[u32; NUM_LIMBS]>,
    bit_shift_carry: Option<[u32; NUM_LIMBS]>,
}
```

## Common Formulas

### Left Shift (SLL)
```
a[j] = b[j-i] << bit_shift + carry[j-i-1]
carry[i] = b[i] >> (8 - bit_shift)
```

### Right Shift (SRL/SRA)
```
a[j] = b[j+i] >> bit_shift + (b[j+i+1] << (8 - bit_shift))
carry[i] = b[i] & ((1 << bit_shift) - 1)
```

### Sign Extension (SRA only)
```
sign = b[3] >> 7
fill = sign ? 0xFF : 0x00
```

## Integration Points
- **Memory Bridge**: Register reads/writes
- **Bitwise Lookup**: XOR for sign bit, range checks
- **Range Checker**: Verify shift amounts and carries
- **Execution Bus**: Instruction dispatch

## Quick Debugging Tips
1. **Wrong result**: Check shift amount calculation and carry logic
2. **Constraint failure**: Verify boolean flags and marker sums
3. **Range check error**: Check bit_shift < 8, carries are valid
4. **Sign extension bug**: Verify XOR lookup and fill value
5. **Off-by-one**: Check limb/bit shift boundary conditions