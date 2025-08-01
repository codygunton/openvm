# AUIPC Component - Quick Reference

## What It Does
Implements RISC-V AUIPC instruction: `rd = pc + (immediate << 12)`

## Key Files
- `core.rs` - Main implementation
- `mod.rs` - Public API
- `tests.rs` - Test suite

## Core Types
```rust
// Main chip type
type Rv32AuipcChip<F> = VmChipWrapper<F, Rv32RdWriteAdapterChip<F>, Rv32AuipcCoreChip>;

// Trace columns
struct Rv32AuipcCoreCols<T> {
    is_valid: T,
    imm_limbs: [T; 3],    // Upper 3 limbs of immediate
    pc_limbs: [T; 2],     // Middle 2 limbs of PC
    rd_data: [T; 4],      // All 4 limbs of result
}
```

## Key Implementation
```rust
// Core execution (line 294)
pub fn run_auipc(pc: u32, imm: u32) -> [u32; 4] {
    let rd = pc.wrapping_add(imm << 12);
    // Split into 8-bit limbs
    array::from_fn(|i| (rd >> (8 * i)) & 0xFF)
}
```

## Limb Representation
- 32-bit values → 4 limbs of 8 bits each
- Little-endian order (LSB first)
- Immediate LSB always 0 (not stored)
- PC MSB and LSB handled separately

## Constraints Enforced
1. Valid carry propagation in addition
2. All limbs within 8-bit range
3. PC MSB within PC_BITS limit
4. Correct opcode matching

## Usage in VM
```rust
// Instantiate
let chip = Rv32AuipcChip::new(adapter, core, memory);

// Register with executor
executor.register(chip, [AUIPC_OPCODE]);
```

## Common Values
- `RV32_CELL_BITS = 8`
- `RV32_REGISTER_NUM_LIMBS = 4`
- `PC_BITS = 24` (typically)

## Testing Commands
```bash
# Run AUIPC tests
cargo test -p openvm-rv32im-circuit auipc

# Run with logs
RUST_LOG=debug cargo test -p openvm-rv32im-circuit auipc
```