# BigInt Circuit Quick Reference

## Setup and Configuration

### Basic VM Configuration
```rust
use openvm_bigint_circuit::Int256Rv32Config;

let config = Int256Rv32Config::default();
```

### Custom Configuration
```rust
let config = Int256Rv32Config {
    system: SystemConfig::default().with_continuations(),
    rv32i: Rv32I,
    rv32m: Rv32M::default(),
    io: Rv32Io,
    bigint: Int256 {
        range_tuple_checker_sizes: [256, 8192], // Custom sizes
    },
};
```

## Available Operations

### Arithmetic Operations
| Operation | Guest Function | Opcode | Description |
|-----------|---------------|---------|-------------|
| Addition | `zkvm_u256_wrapping_add_impl` | `0x400` | Wrapping addition |
| Subtraction | `zkvm_u256_wrapping_sub_impl` | `0x401` | Wrapping subtraction |
| Multiplication | `zkvm_u256_wrapping_mul_impl` | `0x450` | Lower 256 bits of product |

### Bitwise Operations
| Operation | Guest Function | Opcode | Description |
|-----------|---------------|---------|-------------|
| AND | `zkvm_u256_bitand_impl` | `0x404` | Bitwise AND |
| OR | `zkvm_u256_bitor_impl` | `0x403` | Bitwise OR |
| XOR | `zkvm_u256_bitxor_impl` | `0x402` | Bitwise XOR |

### Shift Operations
| Operation | Guest Function | Opcode | Description |
|-----------|---------------|---------|-------------|
| Left Shift | `zkvm_u256_wrapping_shl_impl` | `0x405` | Logical left shift |
| Right Shift | `zkvm_u256_wrapping_shr_impl` | `0x406` | Logical right shift |
| Arithmetic Right | `zkvm_u256_arithmetic_shr_impl` | `0x407` | Sign-extending shift |

### Comparison Operations
| Operation | Guest Function | Opcode | Description |
|-----------|---------------|---------|-------------|
| Less Than | SLT instruction | `0x408` | Signed comparison |
| Less Than Unsigned | SLTU instruction | `0x409` | Unsigned comparison |
| Equality | `zkvm_u256_eq_impl` | Branch | Returns bool |
| Compare | `zkvm_u256_cmp_impl` | Multiple | Returns Ordering |

## Guest Program Usage

### Using with U256 Type
```rust
use zkvm_u256::U256;

let a = U256::from_u32(12345);
let b = U256::from_u32(67890);

// Arithmetic
let sum = a.wrapping_add(b);
let diff = a.wrapping_sub(b);
let product = a.wrapping_mul(b);

// Bitwise
let and_result = a & b;
let or_result = a | b;
let xor_result = a ^ b;

// Shifts
let left_shifted = a << 4;
let right_shifted = a >> 4;

// Comparison
if a < b {
    // Handle less than
}
if a == b {
    // Handle equality
}
```

### Direct Extern Usage
```rust
unsafe {
    let mut result = [0u8; 32];
    let a = [1u8; 32];
    let b = [2u8; 32];
    
    zkvm_u256_wrapping_add_impl(
        result.as_mut_ptr(),
        a.as_ptr(),
        b.as_ptr()
    );
}
```

### Memory Layout
```rust
// 256-bit value representation (little-endian)
// Byte array: [LSB...MSB]
let value: [u8; 32] = [
    0x01, 0x00, 0x00, 0x00, // Limb 0 (bits 0-31)
    0x00, 0x00, 0x00, 0x00, // Limb 1 (bits 32-63)
    // ... continues for 8 limbs total
];
```

## Integration Examples

### Adding BigInt Support to Existing VM
```rust
use openvm_bigint_circuit::Int256;
use openvm_circuit::arch::{VmConfig, VmExtension};

#[derive(VmConfig)]
struct MyVmConfig {
    #[system]
    system: SystemConfig,
    
    #[extension]
    rv32i: Rv32I,
    
    #[extension]
    bigint: Int256, // Add this
}
```

### Custom Instruction in Assembly
```asm
# ADD256: rd = rs1 + rs2
.insn r 0x0b, 0x5, 0x00, rd, rs1, rs2

# Branch if equal (256-bit)
.insn b 0x0b, 0x6, rs1, rs2, offset
```

## Testing Patterns

### Unit Test Template
```rust
#[test]
fn test_bigint_operation() {
    let mut builder = VmChipTestBuilder::new();
    let chip = create_bigint_chip();
    
    // Setup test values
    let a = [0xFF; 32]; // All 1s
    let b = [0x01, 0x00, /* 30 zeros */]; // 1
    
    // Write to memory
    builder.write_heap_bytes(0x1000, &a);
    builder.write_heap_bytes(0x1020, &b);
    
    // Execute operation
    chip.execute(opcode, 0x1040, 0x1000, 0x1020);
    
    // Read result
    let result = builder.read_heap_bytes(0x1040, 32);
    assert_eq!(result, expected);
}
```

### Integration Test Pattern
```rust
#[test]
fn test_complex_calculation() {
    let program = Program::new(vec![
        // Load values
        Instruction::new(LOAD, ...),
        // Perform 256-bit multiplication
        Instruction::new(MUL256, ...),
        // Branch on result
        Instruction::new(BEQ256, ...),
    ]);
    
    let result = run_program(program);
    assert!(result.success);
}
```

## Performance Tips

### 1. Batch Operations
```rust
// Process multiple operations together
for (a, b, result_addr) in operations {
    execute_add256(result_addr, a, b);
}
```

### 2. Reuse Loaded Values
```rust
// Good: Load once, use multiple times
let a = load_u256(addr_a);
let sum = a + b;
let product = a * c;

// Bad: Multiple loads
let sum = load_u256(addr_a) + b;
let product = load_u256(addr_a) * c;
```

### 3. Alignment Optimization
```rust
// Ensure 256-bit values are 32-byte aligned
#[repr(align(32))]
struct AlignedU256([u8; 32]);
```

## Common Patterns

### Safe Arithmetic with Overflow Check
```rust
fn safe_add(a: U256, b: U256) -> Option<U256> {
    let (result, overflow) = a.overflowing_add(b);
    if overflow {
        None
    } else {
        Some(result)
    }
}
```

### Modular Arithmetic
```rust
fn mod_mul(a: U256, b: U256, modulus: U256) -> U256 {
    let product = a.wrapping_mul(b);
    product % modulus // Requires division extension
}
```

### Byte Array Conversion
```rust
// U256 to bytes
let bytes: [u8; 32] = u256_value.to_le_bytes();

// Bytes to U256
let u256_value = U256::from_le_bytes(bytes);
```

## Debugging

### Print 256-bit Value
```rust
#[cfg(debug)]
fn debug_print_u256(label: &str, value: &[u8; 32]) {
    print!("{}: 0x", label);
    for byte in value.iter().rev() {
        print!("{:02x}", byte);
    }
    println!();
}
```

### Verify Constraints
```rust
// In tests, verify operation correctness
fn verify_add(a: &[u32; 8], b: &[u32; 8], result: &[u32; 8]) {
    let mut carry = 0u64;
    for i in 0..8 {
        let sum = a[i] as u64 + b[i] as u64 + carry;
        assert_eq!(result[i], sum as u32);
        carry = sum >> 32;
    }
}
```

## Chip Type Reference

```rust
// Type aliases for all operations
type BaseAlu256<F> = Rv32BaseAlu256Chip<F>;
type LessThan256<F> = Rv32LessThan256Chip<F>;
type Mul256<F> = Rv32Multiplication256Chip<F>;
type Shift256<F> = Rv32Shift256Chip<F>;
type BranchEq256<F> = Rv32BranchEqual256Chip<F>;
type BranchLt256<F> = Rv32BranchLessThan256Chip<F>;
```