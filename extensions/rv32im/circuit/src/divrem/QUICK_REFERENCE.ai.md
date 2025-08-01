# DivRem Component - Quick Reference

## Component Location
```
openvm/extensions/rv32im/circuit/src/divrem/
```

## Supported Operations

| Opcode | Operation | Description | Opcode Value |
|--------|-----------|-------------|--------------|
| DIV    | Signed division | `rd = rs1 ÷ rs2` | 0x254 |
| DIVU   | Unsigned division | `rd = rs1 ÷ rs2` | 0x255 |
| REM    | Signed remainder | `rd = rs1 % rs2` | 0x256 |
| REMU   | Unsigned remainder | `rd = rs1 % rs2` | 0x257 |

## Key Types

```rust
// Main chip type
type Rv32DivRemChip<F> = VmChipWrapper<
    F,
    Rv32MultAdapterChip<F>,
    DivRemCoreChip<RV32_REGISTER_NUM_LIMBS, RV32_CELL_BITS>
>;

// Constants for RV32
const RV32_REGISTER_NUM_LIMBS: usize = 4;  // 32-bit = 4 × 8-bit limbs
const RV32_CELL_BITS: usize = 8;           // 8 bits per limb
```

## Mathematical Formula
```
b = c × q + r
```
Where:
- `b` = dividend (rs1)
- `c` = divisor (rs2)
- `q` = quotient (result for DIV/DIVU)
- `r` = remainder (result for REM/REMU)
- Constraint: `0 ≤ |r| < |c|`

## Special Cases

### 1. Division by Zero
- Condition: `c = 0`
- Result: `q = 0xFFFFFFFF`, `r = b`

### 2. Signed Overflow
- Condition: `-2³¹ ÷ -1` (only for signed ops)
- Result: `q = -2³¹`, `r = 0`

## Sign Rules

For signed operations:
- `sign(q) = sign(b) ⊕ sign(c)` (when q ≠ 0)
- `sign(r) = sign(b)` (when r ≠ 0)

## Key Functions

```rust
// Core division algorithm
fn run_divrem<const NUM_LIMBS: usize, const LIMB_BITS: usize>(
    signed: bool,
    x: &[u32; NUM_LIMBS],  // dividend
    y: &[u32; NUM_LIMBS],  // divisor
) -> ([u32; NUM_LIMBS], [u32; NUM_LIMBS], bool, bool, bool, DivRemCoreSpecialCase)
// Returns: (quotient, remainder, x_sign, y_sign, q_sign, special_case)

// Two's complement negation
fn negate<const NUM_LIMBS: usize, const LIMB_BITS: usize>(
    x: &[u32; NUM_LIMBS]
) -> [u32; NUM_LIMBS]

// Unsigned comparison helper
fn run_sltu_diff_idx<const NUM_LIMBS: usize>(
    x: &[u32; NUM_LIMBS],
    y: &[u32; NUM_LIMBS],
    cmp: bool
) -> usize
```

## Constraint Columns

| Column | Purpose |
|--------|---------|
| `b[NUM_LIMBS]` | Dividend input |
| `c[NUM_LIMBS]` | Divisor input |
| `q[NUM_LIMBS]` | Quotient output |
| `r[NUM_LIMBS]` | Remainder output |
| `zero_divisor` | Flag: c = 0 |
| `r_zero` | Flag: r = 0 (non-zero divisor) |
| `b_sign`, `c_sign`, `q_sign` | Sign bits |
| `r_prime[NUM_LIMBS]` | Absolute value of r |
| `lt_marker[NUM_LIMBS]` | Less-than comparison markers |

## Integration Example

```rust
// Create the chip
let divrem_chip = Rv32DivRemChip::<F>::new(
    adapter,
    DivRemCoreChip::new(
        bitwise_chip,
        range_checker,
        DivRemOpcode::CLASS_OFFSET  // 0x254
    ),
    memory_controller
);

// Execute division
let instruction = Instruction::from_usize(
    DivRemOpcode::DIV.global_opcode(),  // or DIVU, REM, REMU
    [rd, rs1, rs2, 1, 0]
);
```

## Testing Patterns

```rust
// Test special cases
run_test(opcode, [98, 188, 163, 127], [0, 0, 0, 0]);     // Zero divisor
run_test(opcode, [0, 0, 0, 128], [255, 255, 255, 255]); // Signed overflow
run_test(opcode, [0, 0, 1, 0], [0, 0, 1, 0]);           // Zero remainder

// Negative test with injected errors
let prank_vals = DivRemPrankValues {
    q: Some([wrong_quotient]),
    r: Some([wrong_remainder]),
    ..Default::default()
};
```

## Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| Wrong sign in result | Check sign_xor calculation and q_sign constraints |
| Range check failure | Verify all carries are properly bounded |
| Special case not handled | Ensure zero_divisor and r_zero flags are set correctly |
| Constraint violation | Check lt_marker logic for |r| < |c| enforcement |

## Performance Notes

- Uses lookup tables for sign checking (BitwiseOperationLookup)
- Batches range checks through RangeTupleChecker
- Caches field inversions (c_sum_inv, r_sum_inv)
- Optimized for common case (non-special divisions)

## Debug Commands

```rust
// Print trace columns for debugging
println!("b={:?} c={:?} q={:?} r={:?}", cols.b, cols.c, cols.q, cols.r);
println!("zero_divisor={} r_zero={}", cols.zero_divisor, cols.r_zero);
println!("signs: b={} c={} q={} xor={}", 
    cols.b_sign, cols.c_sign, cols.q_sign, cols.sign_xor);
```