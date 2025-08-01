# RV32 Adapters - Quick Reference

## Available Adapters

### Equality Check Adapter
```rust
use openvm_rv32_adapters::Rv32IsEqualModAdapterChip;

// Compare two 256-bit values from heap
type EqAdapter = Rv32IsEqualModAdapterChip<F, 2, 1, 32, 32>;
```

### Basic Heap Adapter
```rust
use openvm_rv32_adapters::Rv32HeapAdapterChip;

// Read 32 bytes, write 32 bytes
type HeapAdapter = Rv32HeapAdapterChip<F, 1, 32, 32>;
```

### Vector Heap Adapter
```rust
use openvm_rv32_adapters::Rv32VecHeapAdapterChip;

// Read 2 pointers, 4 blocks each, write 1 block
type VecAdapter = Rv32VecHeapAdapterChip<F, 2, 4, 1, 32, 32>;
```

## Common Usage Patterns

### Creating an Adapter Instance
```rust
let adapter = Rv32HeapAdapterChip::<F, 2, 32, 32>::new(
    execution_bus,
    program_bus,
    memory_bridge,
    24, // address_bits
    bitwise_chip,
);
```

### Instruction Format
```rust
// Heap read/write instruction
let instruction = Instruction::from_isize(
    opcode,         // Custom opcode for adapter
    rd as isize,    // Destination register
    rs1 as isize,   // Source register 1 (pointer)
    rs2 as isize,   // Source register 2 (pointer or 0)
    1,              // Register address space
    2,              // Heap address space
);
```

## Memory Access Examples

### Single Pointer Read
```rust
// Read 32 bytes from heap[rs1] into rd
const READ_32_OPCODE: usize = 0x100;
let inst = Instruction::from_isize(
    VmOpcode::from_usize(READ_32_OPCODE),
    rd, rs1, 0, 1, 2
);
```

### Dual Pointer Read
```rust
// Read from heap[rs1] and heap[rs2], process, write to rd
const DUAL_READ_OPCODE: usize = 0x200;
let inst = Instruction::from_isize(
    VmOpcode::from_usize(DUAL_READ_OPCODE),
    rd, rs1, rs2, 1, 2
);
```

### Vectorized Read/Write
```rust
// Read 4 blocks from heap[rs1], write 2 blocks to heap[rd]
const VEC_RW_OPCODE: usize = 0x300;
let inst = Instruction::from_isize(
    VmOpcode::from_usize(VEC_RW_OPCODE),
    rd, rs1, 0, 1, 2
);
```

## Testing Utilities

### Setup Test Memory
```rust
use openvm_rv32_adapters::test_utils::*;

// Write test data to heap
let data = vec![
    [F::from_canonical_u32(0x11111111); 8],
    [F::from_canonical_u32(0x22222222); 8],
];
let inst = rv32_write_heap_default(&mut tester, data, vec![], opcode);
```

### Write Pointer to Register
```rust
write_ptr_reg(&mut tester, 1, 5, 0x1000); // Write 0x1000 to register 5
```

## Parameter Selection Guide

### Block Sizes
- **8**: Single 64-bit word
- **16**: 128-bit values (common for cryptography)
- **32**: 256-bit values (hashes, field elements)
- **64**: 512-bit values (extended precision)

### Number of Reads
- **1**: Unary operations (copy, transform)
- **2**: Binary operations (add, compare, combine)

### Blocks per Read
- **1**: Single value access
- **2-4**: Small arrays or structures
- **8+**: Bulk data processing

## Integration Example

```rust
// Complete adapter setup for 256-bit operations
pub fn setup_crypto_adapters(builder: &mut VmBuilder) {
    // Equality check for 256-bit values
    let eq_adapter = Rv32IsEqualModAdapterChip::<F, 2, 1, 32, 32>::new(
        builder.execution_bus(),
        builder.program_bus(),
        builder.memory_bridge(),
        24,
        builder.bitwise_chip(),
    );
    builder.add_adapter(0x100, eq_adapter);
    
    // Bulk copy for arrays of 256-bit values
    let vec_adapter = Rv32VecHeapAdapterChip::<F, 1, 8, 8, 32, 32>::new(
        builder.execution_bus(),
        builder.program_bus(),
        builder.memory_bridge(),
        24,
        builder.bitwise_chip(),
    );
    builder.add_adapter(0x200, vec_adapter);
}
```

## Performance Tips

1. **Align block sizes** with your data structures
2. **Minimize adapter switches** by batching similar operations
3. **Use vectorized adapters** for bulk operations
4. **Pre-calculate pointers** in registers when possible
5. **Choose appropriate address bits** for your memory space

## Debugging

### Common Issues
- **Invalid address**: Check address < 2^address_bits
- **Wrong address space**: Verify AS constants (1 for registers, 2 for heap)
- **Timestamp mismatch**: Ensure consistent increment pattern
- **Size mismatch**: Verify READ_SIZE and WRITE_SIZE parameters

### Trace Inspection
```rust
// Enable debug logging
std::env::set_var("RUST_LOG", "debug");

// Check adapter execution
println!("Instruction: {:?}", instruction);
println!("Read data: {:?}", read_record);
println!("Write result: {:?}", write_record);
```