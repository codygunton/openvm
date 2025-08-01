# LoadStore Quick Reference

## Component Overview
- **Purpose**: Handles RISC-V load/store memory operations
- **Architecture**: Two-chip design (Core + Adapter)
- **Key Feature**: 4-byte aligned memory access with sub-word support

## Supported Operations

| Opcode | Instruction | Size | Description |
|--------|-------------|------|-------------|
| LOADW  | lw rd, imm(rs1) | 4 bytes | Load word |
| LOADHU | lhu rd, imm(rs1) | 2 bytes | Load halfword unsigned |
| LOADBU | lbu rd, imm(rs1) | 1 byte | Load byte unsigned |
| STOREW | sw rs2, imm(rs1) | 4 bytes | Store word |
| STOREH | sh rs2, imm(rs1) | 2 bytes | Store halfword |
| STOREB | sb rs2, imm(rs1) | 1 byte | Store byte |

## Key Constants
```rust
const RV32_REGISTER_NUM_LIMBS: usize = 4;  // 32-bit registers as 4 bytes
const RV32_CELL_BITS: usize = 8;           // 8 bits per limb
const RV32_REGISTER_AS: u32 = 1;           // Register address space
const DEFAULT_PC_STEP: u32 = 4;            // Default PC increment
```

## Core Data Structures

### LoadStoreCoreCols
```rust
pub struct LoadStoreCoreCols<T, const NUM_CELLS: usize> {
    pub flags: [T; 4],              // Opcode encoding
    pub is_valid: T,                // Validity flag
    pub is_load: T,                 // Load/store indicator
    pub read_data: [T; NUM_CELLS],  // Input data
    pub prev_data: [T; NUM_CELLS],  // Previous memory contents
    pub write_data: [T; NUM_CELLS], // Output data
}
```

### LoadStoreInstruction (Adapter Interface)
```rust
pub struct LoadStoreInstruction<T> {
    pub is_valid: T,
    pub opcode: T,
    pub is_load: T,
    pub load_shift_amount: T,
    pub store_shift_amount: T,
}
```

## Address Calculation
```rust
ptr_val = rs1 + sign_extend(imm)
shift = ptr_val % 4
aligned_ptr = ptr_val - shift
```

## Flag Encoding Table

| Operation | flags[0] | flags[1] | flags[2] | flags[3] |
|-----------|----------|----------|----------|----------|
| LOADW,0   | 2 | 0 | 0 | 0 |
| LOADHU,0  | 0 | 2 | 0 | 0 |
| LOADHU,2  | 0 | 0 | 2 | 0 |
| LOADBU,0  | 0 | 0 | 0 | 2 |
| LOADBU,1  | 1 | 0 | 0 | 0 |
| LOADBU,2  | 0 | 1 | 0 | 0 |
| LOADBU,3  | 0 | 0 | 1 | 0 |
| STOREW,0  | 0 | 0 | 0 | 1 |
| STOREH,0  | 1 | 1 | 0 | 0 |
| STOREH,2  | 1 | 0 | 1 | 0 |
| STOREB,0  | 1 | 0 | 0 | 1 |
| STOREB,1  | 0 | 1 | 1 | 0 |
| STOREB,2  | 0 | 1 | 0 | 1 |
| STOREB,3  | 0 | 0 | 1 | 1 |

## Common Usage Patterns

### Creating the Chip
```rust
let adapter = Rv32LoadStoreAdapterChip::new(
    execution_bus,
    program_bus,
    memory_bridge,
    pointer_max_bits,
    range_checker_chip,
);
let core = LoadStoreCoreChip::new(offset);
let chip = Rv32LoadStoreChip::new(adapter, core, memory_mutex);
```

### Executing an Instruction
```rust
let instruction = Instruction::from_usize(
    opcode.global_opcode(),
    [rd_rs2, rs1, imm, reg_as, mem_as, enabled, imm_sign],
);
tester.execute(&mut chip, &instruction);
```

### Testing Write Data Computation
```rust
let write_data = run_write_data(
    opcode,
    read_data,
    prev_data,
    shift_amount
);
```

## Memory Access Rules

### Load Operations
- Read from: `memory[aligned_ptr]` in address space `mem_as`
- Write to: `register[rd]` in address space 1
- Special case: No write if `rd == x0`

### Store Operations
- Read from: `register[rs2]` in address space 1
- Write to: `memory[aligned_ptr]` in address space `mem_as`
- Merge with previous memory contents for sub-word stores

## Error Conditions
- `ptr_val >= 2^pointer_max_bits`: Address overflow
- Invalid opcode/shift combination
- Wrong address space for operation type
- Constraint violations in trace

## Performance Tips
1. Use word operations when possible (LOADW/STOREW)
2. Align data structures to 4-byte boundaries
3. Batch related memory operations
4. Minimize x0 destination loads

## Quick Debugging

### Check Alignment
```rust
assert_eq!(ptr_val % 4, shift_amount);
assert_eq!(aligned_ptr % 4, 0);
```

### Verify Flag Encoding
```rust
let sum = flags.iter().sum();
assert!(sum == 0 || sum == 1 || sum == 2);
assert!(flags.iter().all(|&f| f == 0 || f == 1 || f == 2));
```

### Validate Write Data
```rust
// For loads
assert_eq!(write_data[0], read_data[shift]);  // LOADBU
// For stores  
assert_eq!(write_data[shift], read_data[0]);  // STOREB
assert_eq!(write_data[other], prev_data[other]); // Unchanged
```

## Common Issues and Solutions

| Issue | Cause | Solution |
|-------|-------|----------|
| Wrong data loaded | Incorrect shift | Check `ptr_val % 4` calculation |
| Store overwrites | Missing prev_data | Ensure prev_data is read correctly |
| x0 modified | Missing check | Verify `needs_write = 0` when `rd = x0` |
| Address overflow | Large pointer | Check pointer_max_bits configuration |
| Constraint failure | Invalid trace | Verify flag encoding matches opcode |