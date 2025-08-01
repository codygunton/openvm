# JAL/LUI Quick Reference

## Component Overview
Implements RISC-V JAL (Jump And Link) and LUI (Load Upper Immediate) instructions.

## Key Files
- `core.rs` - Main implementation
- `mod.rs` - Type definitions
- `tests.rs` - Test suite

## Core Types
```rust
// Main chip type
type Rv32JalLuiChip<F> = VmChipWrapper<F, Rv32CondRdWriteAdapterChip<F>, Rv32JalLuiCoreChip>;

// Trace columns
struct Rv32JalLuiCoreCols<T> {
    imm: T,                              // Immediate value
    rd_data: [T; 4],                     // Result register (4 limbs)
    is_jal: T,                           // JAL instruction flag
    is_lui: T,                           // LUI instruction flag
}
```

## Instruction Behavior

### JAL (Jump And Link)
- **Operation**: `rd = PC + 4; PC = PC + sext(immediate)`
- **Immediate**: 21-bit signed, bit 0 implicit
- **Use case**: Function calls, unconditional jumps

### LUI (Load Upper Immediate)
- **Operation**: `rd = immediate << 12; PC = PC + 4`
- **Immediate**: 20-bit unsigned
- **Use case**: Building 32-bit constants

## Key Constants
```rust
RV32_REGISTER_NUM_LIMBS = 4  // Limbs per register
RV32_CELL_BITS = 8           // Bits per limb
PC_BITS = 24                 // Program counter width
DEFAULT_PC_STEP = 4          // Standard PC increment
```

## Common Patterns

### JAL Function Call
```assembly
jal ra, function    # Jump to function, save return in ra (x1)
```

### 32-bit Constant Loading
```assembly
lui t0, %hi(0x12345678)      # Load upper 20 bits
addi t0, t0, %lo(0x12345678) # Add lower 12 bits
```

## Testing Commands
```bash
# Run all JAL/LUI tests
cargo test -p openvm-rv32im-circuit jal_lui

# Run with debug output
RUST_LOG=debug cargo test -p openvm-rv32im-circuit jal_lui
```

## Quick Debugging

### Check Immediate Encoding
```rust
// JAL: Sign-extended 21-bit (bit 0 implicit)
let signed_imm = (imm + (1 << 20)) as i32 - (1 << 20);

// LUI: Unsigned 20-bit
let unsigned_imm = imm & 0xFFFFF;
```

### Verify Limb Decomposition
```rust
// Any 32-bit value to limbs
let limbs = array::from_fn(|i| (value >> (8 * i)) & 0xFF);
```

### Common Constraint Violations
1. Both `is_jal` and `is_lui` set
2. Limb value > 255
3. LUI with non-zero first limb
4. JAL with invalid PC bits

## Performance Tips
- Batch range checks in pairs
- Minimize bitwise lookups
- Use wrapping arithmetic for PC

## Security Checklist
- [ ] Range check all limbs
- [ ] Validate boolean flags
- [ ] Handle PC overflow
- [ ] Check immediate bounds
- [ ] Verify sign extension