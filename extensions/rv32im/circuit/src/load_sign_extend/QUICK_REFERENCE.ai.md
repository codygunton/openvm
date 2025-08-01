# Load Sign Extend - Quick Reference

## Component Purpose
Sign-extends 8-bit (LOADB) and 16-bit (LOADH) values to 32-bit words in RISC-V

## Key Files
- `mod.rs` - Module exports and chip alias
- `core.rs` - Sign extension logic and constraints  
- `tests.rs` - Test suite

## Main Types
```rust
// Wrapper chip combining adapter + core
type Rv32LoadSignExtendChip<F> = VmChipWrapper<F, Adapter, Core>

// Core chip with sign extension logic
LoadSignExtendCoreChip<NUM_CELLS=4, LIMB_BITS=8>

// Column layout for constraints
LoadSignExtendCoreCols<T, NUM_CELLS> {
    opcode_loadb_flag0: T,  // LOADB shift 0
    opcode_loadb_flag1: T,  // LOADB shift 1  
    opcode_loadh_flag: T,   // LOADH any shift
    shift_most_sig_bit: T,  // (shift & 2) >> 1
    data_most_sig_bit: T,   // Sign bit value
    shifted_read_data: [T; NUM_CELLS],
    prev_data: [T; NUM_CELLS],
}
```

## Opcodes
- `LOADB` (0x216) - Load byte, sign extend to 32-bit
- `LOADH` (0x217) - Load halfword, sign extend to 32-bit

## Key Function
```rust
run_write_data_sign_extend<F, NUM_CELLS, LIMB_BITS>(
    opcode: Rv32LoadStoreOpcode,
    read_data: [F; NUM_CELLS],  
    prev_data: [F; NUM_CELLS],  // unused
    shift: u32,
) -> [F; NUM_CELLS]
```

## Sign Extension Rules
- **LOADB**: Extend bit 7 to bits 8-31
- **LOADH**: Extend bit 15 to bits 16-31
- Sign bit = 1 → Fill with 0xFF limbs
- Sign bit = 0 → Fill with 0x00 limbs

## Shift Handling
- LOADB supports shifts 0-3 (any byte)
- LOADH supports shifts 0,2 (aligned only)
- Pre-shift by `shift & 2` for efficiency
- Actual shift in constraints: 0 or 1

## Memory Layout
- 32-bit word = 4 limbs × 8 bits
- Little-endian byte order
- Aligned access required

## Quick Test
```bash
cargo test -p openvm-rv32im-circuit load_sign_extend
```

## Common Issues
1. **Wrong shift flag**: Check `shift & 1` for LOADB flags
2. **Sign bit wrong**: Verify extraction position
3. **Alignment error**: Ensure halfwords on even bytes

## Integration Example
```rust
// Create chip
let core = LoadSignExtendCoreChip::new(range_checker);
let chip = Rv32LoadSignExtendChip::new(adapter, core, mutex);

// Execute LOADB at address with shift 1
let inst = Instruction::from_usize(
    LOADB.global_opcode(),
    [dst, src, imm, ...] 
);
chip.execute_instruction(&inst, pc, reads)?;
```