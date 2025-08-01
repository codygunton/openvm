# JALR Quick Reference

## Constants

```rust
// From OpenVM instructions
const DEFAULT_PC_STEP: u32 = 4;
const PC_BITS: usize = 32;
const RV32_REGISTER_AS: u32 = 1;

// From adapters
const RV32_CELL_BITS: usize = 8;
const RV32_REGISTER_NUM_LIMBS: usize = 4;
const RV32_LIMB_MAX: u32 = 255; // (1 << RV32_CELL_BITS) - 1

// JALR specific
const JALR_OPCODE_OFFSET: usize = 0x23F;
```

## Key Types

```rust
// Core types
pub type Rv32JalrChip<F> = VmChipWrapper<F, Rv32JalrAdapterChip<F>, Rv32JalrCoreChip>;

// Column structures
pub struct Rv32JalrCoreCols<T> {
    pub imm: T,                              // 12-bit immediate
    pub rs1_data: [T; 4],                    // Source register (4 bytes)
    pub rd_data: [T; 3],                     // Dest register (3 MSB only)
    pub is_valid: T,                         // Validity flag
    pub to_pc_least_sig_bit: T,              // LSB of jump target
    pub to_pc_limbs: [T; 2],                 // Jump target / 2
    pub imm_sign: T,                         // Sign extension bit
}

// Records
pub struct Rv32JalrCoreRecord<F> {
    pub imm: F,
    pub rs1_data: [F; 4],
    pub rd_data: [F; 3],      // Only 3 MSB limbs
    pub to_pc_least_sig_bit: F,
    pub to_pc_limbs: [u32; 2],
    pub imm_sign: F,
}
```

## Core Functions

```rust
// Main execution function
pub fn run_jalr(
    opcode: Rv32JalrOpcode,
    pc: u32,
    imm: u32,
    rs1: u32,
) -> (u32, [u32; 4]) {
    let to_pc = rs1.wrapping_add(imm) & !1;  // Clear LSB
    let rd_data = decompose(pc + DEFAULT_PC_STEP);
    (to_pc, rd_data)
}

// Helper functions
fn compose(limbs: [F; 4]) -> u32;      // Convert limbs to u32
fn decompose(value: u32) -> [F; 4];    // Convert u32 to limbs
```

## Instruction Creation

```rust
// Create JALR instruction
let inst = Instruction::from_usize(
    JALR.global_opcode(),
    [
        rd,        // a: destination register
        rs1,       // b: source register  
        imm,       // c: immediate (12-bit)
        1,         // d: address space (always 1)
        0,         // e: unused
        enable,    // f: write enable (0 if rd=x0)
        sign,      // g: immediate sign bit
    ]
);
```

## Common Patterns

### Execute JALR
```rust
// In test context
tester.execute_with_pc(chip, &instruction, from_pc);

// Direct execution
let (ctx, record) = chip.execute_instruction(&inst, from_pc, reads)?;
```

### Memory Operations
```rust
// Read rs1
let (record_id, rs1_data) = memory.read::<4>(1, rs1_addr);

// Write rd (if not x0)
if rd_addr != 0 {
    memory.write(1, rd_addr, rd_data);
}
```

### Address Calculation
```rust
// Sign extend immediate
let imm_extended = imm + (imm_sign * 0xffff0000);

// Calculate jump target
let to_pc = (rs1 + imm_extended) & !1;

// Calculate return address  
let rd = pc + 4;
```

## Constraint Checks

### Range Constraints
```rust
// rd_data limbs
bitwise_lookup(rd_data[0], rd_data[1]);     // 8-bit each
range_check(rd_data[2], 8);                  // 8-bit
range_check(rd_data[3], PC_BITS - 24);       // Remaining bits

// to_pc limbs
range_check(to_pc_limbs[0], 15);             // 15-bit
range_check(to_pc_limbs[1], PC_BITS - 16);   // Remaining bits
```

### Boolean Constraints
```rust
assert_bool(is_valid);
assert_bool(imm_sign);
assert_bool(to_pc_least_sig_bit);
assert_bool(carry);  // For addition carries
```

## Error Handling

```rust
// Common errors
assert!(to_pc < (1 << PC_BITS), "PC overflow");
assert!(rd_addr < 32, "Invalid register");

// In constraints
builder.when(is_valid).assert_bool(carry);
builder.when(not(is_valid)).assert_zero(write_count);
```

## Testing Utilities

```rust
// Generate random JALR test
set_and_execute(
    &mut tester,
    &mut chip,
    &mut rng,
    JALR,
    None,    // Random immediate
    None,    // Random sign
    None,    // Random PC
    None,    // Random rs1
);

// Verify execution
assert_eq!(new_pc, expected_pc);
assert_eq!(rd_value, from_pc + 4);
```

## Debugging

```rust
// Trace JALR execution
println!("JALR: pc={:x} rs1={:x} imm={:x} -> pc={:x}",
    from_pc, 
    compose(rs1_data),
    imm_extended,
    to_pc
);

// Verify constraints manually
let least_sig_limb = from_pc + 4 - composed_rd;
assert!(least_sig_limb < 256);
```

## Performance Tips

1. **Batch range checks**: Request all ranges before execution
2. **Reuse chips**: Share bitwise/range checker across instructions  
3. **Minimize limb operations**: Use compose/decompose helpers
4. **Avoid redundant checks**: x0 writes already handled by adapter

## Common Issues

| Issue | Solution |
|-------|----------|
| PC overflow | Add range check: `assert!(to_pc < (1 << PC_BITS))` |
| Wrong rd value | Check: `rd = from_pc + 4`, not `to_pc + 4` |
| LSB not cleared | Use: `to_pc & !1` to clear bit |
| Sign extension | Use: `imm + (sign * 0xffff0000)` |
| Missing carry | Check 16-bit boundaries in address calc |