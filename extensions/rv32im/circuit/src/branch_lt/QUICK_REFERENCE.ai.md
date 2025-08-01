# Branch Less Than Quick Reference

## Opcodes
```rust
BranchLessThanOpcode::BLT   // Branch if rs1 < rs2 (signed)
BranchLessThanOpcode::BLTU  // Branch if rs1 < rs2 (unsigned)
BranchLessThanOpcode::BGE   // Branch if rs1 >= rs2 (signed)
BranchLessThanOpcode::BGEU  // Branch if rs1 >= rs2 (unsigned)
```

## Key Types
```rust
// Main chip type
type Rv32BranchLessThanChip<F> = VmChipWrapper<F, 
    Rv32BranchAdapterChip<F>,
    BranchLessThanCoreChip<8, 8>
>;

// Core columns
struct BranchLessThanCoreCols<T, NUM_LIMBS, LIMB_BITS> {
    a: [T; NUM_LIMBS],           // First operand
    b: [T; NUM_LIMBS],           // Second operand
    cmp_result: T,               // Branch taken?
    cmp_lt: T,                   // Is a < b?
    imm: T,                      // Branch offset
    opcode_*_flag: T,            // One flag per opcode
    diff_marker: [T; NUM_LIMBS], // First difference position
}
```

## Core Functions
```rust
// Comparison logic
fn run_cmp(opcode, a, b) -> (take_branch, diff_idx, a_sign, b_sign)

// Instruction execution
fn execute_instruction(instruction, from_pc, reads) -> (context, record)

// AIR constraint evaluation
fn eval(builder, cols, from_pc) -> AdapterAirContext
```

## Constants
- `RV32_REGISTER_NUM_LIMBS = 8` - 32-bit values as 8 bytes
- `RV32_CELL_BITS = 8` - 8 bits per limb
- `DEFAULT_PC_STEP = 4` - Standard PC increment

## Usage Example
```rust
// Create chip
let bitwise = SharedBitwiseOperationLookupChip::new(bus);
let core = BranchLessThanCoreChip::new(bitwise, offset);
let chip = Rv32BranchLessThanChip::new(adapter, core);

// Execute instruction
let instruction = Instruction {
    opcode: BLT_OPCODE,
    a: rs1_addr,
    b: rs2_addr, 
    c: branch_offset,
};
let (context, record) = chip.execute_instruction(&instruction, pc, [rs1_val, rs2_val])?;
```

## Comparison Rules
| Opcode | Condition | Signed | Branch If |
|--------|-----------|--------|-----------|
| BLT    | a < b     | Yes    | True      |
| BLTU   | a < b     | No     | True      |
| BGE    | a >= b    | Yes    | True      |
| BGEU   | a >= b    | No     | True      |

## MSB Handling
- **Signed**: MSB = 1 means negative (two's complement)
- **Unsigned**: MSB is magnitude bit
- Range checks: signed [-128, 127], unsigned [0, 255]

## PC Update Logic
```rust
if comparison_result {
    next_pc = current_pc + immediate  // Branch taken
} else {
    next_pc = current_pc + 4          // Next instruction
}
```

## Common Patterns
```rust
// Check if signed operation
let signed = matches!(opcode, BLT | BGE);

// Check if >= operation  
let ge_op = matches!(opcode, BGE | BGEU);

// Convert MSB to field element (signed)
let msb_f = if sign_bit {
    -F::from(256 - msb_value)
} else {
    F::from(msb_value)
};
```