# LessThan Quick Reference

## Component Overview
- **Purpose**: Implements SLT, SLTU comparisons for RV32
- **Location**: `extensions/rv32im/circuit/src/less_than/`
- **Key Trait**: `VmCoreChip<F, I>`

## Key Types

```rust
// Main chip type
type Rv32LessThanChip<F> = VmChipWrapper<F, Adapter, Core>;

// Column structure
struct LessThanCoreCols<T, NUM_LIMBS, LIMB_BITS> {
    b: [T; NUM_LIMBS],      // First operand
    c: [T; NUM_LIMBS],      // Second operand
    cmp_result: T,          // Result (0 or 1)
    opcode_slt_flag: T,     // Signed comparison
    opcode_sltu_flag: T,    // Unsigned comparison
    b_msb_f: T,            // MSB field representation
    c_msb_f: T,            // MSB field representation
    diff_marker: [T; NUM_LIMBS], // Difference position
    diff_val: T,           // Difference value
}

// Execution record
struct LessThanCoreRecord<T, NUM_LIMBS, LIMB_BITS> {
    opcode: LessThanOpcode,
    b, c: [T; NUM_LIMBS],
    cmp_result: T,
    b_msb_f, c_msb_f: T,
    diff_val: T,
    diff_idx: usize,
}
```

## Opcodes

```rust
enum LessThanOpcode {
    SLT  = 0,  // Signed less than
    SLTU = 1,  // Unsigned less than
}
```

## Key Functions

```rust
// Core comparison logic
fn run_less_than(opcode, x, y) -> (bool, usize, bool, bool)
// Returns: (result, diff_index, x_sign, y_sign)

// MSB handling
// Signed: interprets MSB as two's complement
// Unsigned: interprets MSB as positive value
```

## Comparison Algorithm

```
1. Check sign bits (for SLT only)
2. Compare limbs from MSB to LSB
3. First difference determines result
4. Equal values return false
```

## Constraint Formulas

### Difference Detection
```
For each limb i (MSB to LSB):
- If first difference: diff_marker[i] = 1
- Otherwise: diff_marker[i] = 0
- At most one marker can be 1
```

### MSB Interpretation
```
SLT (signed):
  - MSB range: [-128, 127]
  - Range check: (msb + 128)
  
SLTU (unsigned):
  - MSB range: [0, 255]
  - Range check: msb
```

### Result Computation
```
result = (b < c) considering:
- Sign bits for SLT
- First differing limb position
- Zero if b == c
```

## Usage Example

```rust
// Create chip
let bitwise_chip = SharedBitwiseOperationLookupChip::new(bus);
let lt_chip = Rv32LessThanChip::new(
    adapter,
    LessThanCoreChip::new(bitwise_chip, offset),
    memory,
);

// Execute comparison
let instruction = Instruction {
    opcode: LessThanOpcode::SLT.global_opcode(),
    // ... operands
};
lt_chip.execute_instruction(&instruction, pc, reads)?;
```

## Testing Commands

```bash
# Run all less_than tests
cargo test -p openvm-rv32im-circuit less_than

# Specific test suites
cargo test test_rv32_lt
cargo test run_less_than
```

## Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| Wrong sign handling | Check SLT vs SLTU flag usage |
| Incorrect difference | Verify limb ordering (MSB first) |
| Range check failure | Ensure MSB adjustment for signed |
| Multiple markers | Check prefix sum constraint |

## Integration Points

- **Memory**: Via `Rv32BaseAluAdapterChip`
- **Bitwise Lookup**: For MSB and diff_val range checks
- **VM**: Through `VmChipWrapper` and execution bus
- **Transpiler**: Maps RISC-V SLT/SLTU to opcodes

## Performance Notes

- Single-pass comparison algorithm
- Early exit on first difference
- Efficient MSB handling
- Minimal range checks needed