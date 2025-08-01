# LoadStore Implementation Guide

## Overview

This guide provides detailed implementation instructions for working with the LoadStore component in OpenVM's RV32IM extension. The LoadStore system handles all RISC-V memory load and store operations through a two-chip architecture.

## Architecture Overview

### Two-Chip Design

1. **LoadStoreCoreChip**: Handles data transformation logic
   - Processes read_data, prev_data → write_data transformations
   - Manages shift operations for sub-word accesses
   - Enforces constraint system for correctness

2. **Rv32LoadStoreAdapterChip**: Handles VM integration
   - Manages memory addressing and register access
   - Interfaces with ExecutionBus and MemoryBridge
   - Performs address calculation and alignment

## Implementation Steps

### 1. Understanding the Instruction Flow

```rust
// Instruction format for LoadStore operations
Instruction {
    opcode,     // Global opcode for the operation
    a,          // rd (destination) for loads, rs2 (source) for stores  
    b,          // rs1 (base address register)
    c,          // immediate value (lower 16 bits)
    d,          // register address space (always RV32_REGISTER_AS)
    e,          // memory address space
    f,          // enabled flag (0 to disable writes to x0)
    g,          // immediate sign extension bit
}
```

### 2. Address Calculation

The adapter performs address calculation:
```rust
// Get base address from rs1
let rs1_val = compose(rs1_data);

// Sign extend immediate
let imm_extended = imm + imm_sign * 0xffff0000;

// Calculate memory pointer
let ptr_val = rs1_val.wrapping_add(imm_extended);

// Calculate shift for alignment
let shift_amount = ptr_val % 4;

// Get aligned pointer
let aligned_ptr = ptr_val - shift_amount;
```

### 3. Core Operation Logic

The core chip processes data based on opcode and shift:

#### Load Operations
```rust
match (opcode, shift) {
    (LOADW, 0) => {
        // Direct copy of read_data
        write_data = read_data;
    }
    (LOADBU, 0..=3) => {
        // Extract single byte to first limb
        write_data[0] = read_data[shift];
        write_data[1..] = [0; NUM_CELLS-1];
    }
    (LOADHU, 0 | 2) => {
        // Extract halfword to first two limbs
        write_data[0..NUM_CELLS/2] = read_data[shift..shift+NUM_CELLS/2];
        write_data[NUM_CELLS/2..] = [0; NUM_CELLS/2];
    }
}
```

#### Store Operations
```rust
match (opcode, shift) {
    (STOREW, 0) => {
        // Direct copy of read_data
        write_data = read_data;
    }
    (STOREB, 0..=3) => {
        // Merge single byte into prev_data
        write_data = prev_data;
        write_data[shift] = read_data[0];
    }
    (STOREH, 0 | 2) => {
        // Merge halfword into prev_data
        write_data = prev_data;
        write_data[shift..shift+NUM_CELLS/2] = read_data[0..NUM_CELLS/2];
    }
}
```

### 4. Constraint Implementation

#### Flag Encoding System
The core uses 4 flag bits to encode 14 different (opcode, shift) combinations:
```rust
// Flag patterns for each operation
match (opcode, shift) {
    (LOADW, 0)  => flags = [2, 0, 0, 0],
    (LOADHU, 0) => flags = [0, 2, 0, 0],
    (LOADHU, 2) => flags = [0, 0, 2, 0],
    (LOADBU, 0) => flags = [0, 0, 0, 2],
    (LOADBU, 1) => flags = [1, 0, 0, 0],
    (LOADBU, 2) => flags = [0, 1, 0, 0],
    (LOADBU, 3) => flags = [0, 0, 1, 0],
    (STOREW, 0) => flags = [0, 0, 0, 1],
    (STOREH, 0) => flags = [1, 1, 0, 0],
    (STOREH, 2) => flags = [1, 0, 1, 0],
    (STOREB, 0) => flags = [1, 0, 0, 1],
    (STOREB, 1) => flags = [0, 1, 1, 0],
    (STOREB, 2) => flags = [0, 1, 0, 1],
    (STOREB, 3) => flags = [0, 0, 1, 1],
}
```

#### Constraint Verification
```rust
// Each flag must be 0, 1, or 2
builder.assert_zero(flag * (flag - 1) * (flag - 2));

// Sum of flags must be 0, 1, or 2
let sum = flags.iter().sum();
builder.assert_zero(sum * (sum - 1) * (sum - 2));

// is_valid must be 0 when sum is 0
builder.when((sum - 1) * (sum - 2)).assert_zero(is_valid);
```

### 5. Memory Interface

#### Read Operations
```rust
// For loads: read from memory at aligned address
let read_record = memory.read::<RV32_REGISTER_NUM_LIMBS>(
    mem_as,                    // Memory address space
    aligned_ptr                // 4-byte aligned address
);

// For stores: read rs2 register
let read_record = memory.read::<RV32_REGISTER_NUM_LIMBS>(
    RV32_REGISTER_AS,         // Register address space
    rs2_ptr                   // Register number
);
```

#### Write Operations
```rust
// For loads: write to rd register (unless rd = x0)
if rd != 0 && is_load {
    memory.write(RV32_REGISTER_AS, rd, write_data);
}

// For stores: write to memory
if is_store {
    memory.write(mem_as, aligned_ptr, write_data);
}
```

## Common Implementation Patterns

### 1. Adding New Load/Store Variant

To add a new load/store variant:

1. Add to `Rv32LoadStoreOpcode` enum
2. Add flag pattern to `generate_trace_row`
3. Implement data transformation in `run_write_data`
4. Update constraint system in `eval` if needed
5. Add test cases

### 2. Modifying Address Calculation

Address calculation modifications should be made in the adapter:
```rust
// In preprocess()
let custom_offset = /* your calculation */;
let ptr_val = rs1_val.wrapping_add(imm_extended).wrapping_add(custom_offset);
```

### 3. Custom Memory Spaces

To support custom memory spaces:
```rust
// Modify memory space validation in adapter eval()
builder.when(is_load).assert_in_range(mem_as, MIN_AS, MAX_LOAD_AS);
builder.when(is_store).assert_in_range(mem_as, MIN_STORE_AS, MAX_STORE_AS);
```

## Testing Guidelines

### 1. Unit Tests
```rust
#[test]
fn test_custom_operation() {
    let mut tester = VmChipTestBuilder::default();
    let mut chip = /* create chip */;
    
    // Setup test case
    set_and_execute(
        &mut tester,
        &mut chip,
        &mut rng,
        YOUR_OPCODE,
        Some(rs1_value),      // Optional rs1 override
        Some(imm),            // Optional immediate override
        Some(imm_sign),       // Optional sign bit override
        Some(mem_as),         // Optional memory space override
    );
    
    // Verify results
    assert_eq!(expected, tester.read(/* address */));
}
```

### 2. Constraint Tests
```rust
#[test]
fn test_invalid_constraint() {
    run_negative_loadstore_test(
        OPCODE,
        Some(read_data),      // Override read_data
        Some(prev_data),      // Override prev_data  
        Some(write_data),     // Override write_data (invalid)
        None,                 // Keep original flags
        None,                 // Keep original is_load
        None, None, None, None,
        VerificationError::OodEvaluationMismatch,
    );
}
```

## Performance Optimization

### 1. Batch Operations
The system always processes 4 bytes at a time. Optimize by:
- Aligning data structures to 4-byte boundaries
- Minimizing shift operations when possible
- Using word operations (LOADW/STOREW) when applicable

### 2. Constraint Optimization
- Flag encoding minimizes constraint degree
- Shift amounts are computed separately for loads/stores to keep degree low
- Use of helper expressions reduces repeated calculations

### 3. Memory Access Patterns
- Sequential accesses benefit from memory locality
- Consider prefetching for predictable access patterns
- Minimize register spills by careful register allocation

## Debugging Tips

### 1. Trace Analysis
```rust
// In generate_trace_row, add debug output
println!("LoadStore trace: opcode={:?}, shift={}, flags={:?}", 
         opcode, shift, flags);
println!("  read_data={:?}", read_data);
println!("  prev_data={:?}", prev_data);
println!("  write_data={:?}", write_data);
```

### 2. Constraint Verification
```rust
// Manually verify constraints
let expected_write = run_write_data(opcode, read_data, prev_data, shift);
assert_eq!(write_data, expected_write, 
          "Write data mismatch for {:?}", opcode);
```

### 3. Common Issues
- **Alignment errors**: Check shift calculation and ptr alignment
- **Wrong data**: Verify opcode flag encoding matches operation
- **Memory errors**: Ensure address spaces are correct
- **x0 writes**: Verify enabled flag is 0 for x0 destination

## Integration Checklist

When integrating LoadStore into a new system:

- [ ] Configure pointer_max_bits for address space
- [ ] Set up RangeChecker with sufficient range
- [ ] Connect ExecutionBus and MemoryBridge
- [ ] Register opcodes with instruction decoder
- [ ] Configure memory address spaces
- [ ] Add LoadStore chip to VM chip set
- [ ] Test all supported operations
- [ ] Verify constraint satisfaction
- [ ] Check performance metrics