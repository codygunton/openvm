# BigInt Circuit Component Index

## Type Definitions

### Main Chip Types
- `Rv32BaseAlu256Chip<F>` - Base ALU operations chip for 256-bit integers [lib.rs:15]
- `Rv32LessThan256Chip<F>` - Less-than comparison chip [lib.rs:21]
- `Rv32Multiplication256Chip<F>` - 256-bit multiplication chip [lib.rs:27]
- `Rv32Shift256Chip<F>` - Shift operations chip [lib.rs:33]
- `Rv32BranchEqual256Chip<F>` - Branch-on-equal chip [lib.rs:39]
- `Rv32BranchLessThan256Chip<F>` - Branch-on-less-than chip [lib.rs:45]

### Configuration Types
- `Int256Rv32Config` - Main configuration struct for Int256 VM [extension.rs:29]
- `Int256` - Extension configuration parameters [extension.rs:59]

### Executor and Periphery
- `Int256Executor<F>` - Enum containing all executor chips [extension.rs:77]
- `Int256Periphery<F>` - Enum containing all periphery chips [extension.rs:87]

## Constants and Defaults

### Instruction Encoding
- `OPCODE: u8 = 0x0b` - Custom-0 RISC-V opcode [guest/lib.rs:6]
- `INT256_FUNCT3: u8 = 0b101` - Function code for arithmetic ops [guest/lib.rs:7]
- `BEQ256_FUNCT3: u8 = 0b110` - Function code for branch ops [guest/lib.rs:8]

### Opcode Offsets
- `Rv32BaseAlu256Opcode::CLASS_OFFSET = 0x400` [transpiler/lib.rs:20]
- `Rv32Shift256Opcode::CLASS_OFFSET = 0x405` [transpiler/lib.rs:30]
- `Rv32LessThan256Opcode::CLASS_OFFSET = 0x408` [transpiler/lib.rs:40]
- `Rv32BranchEqual256Opcode::CLASS_OFFSET = 0x420` [transpiler/lib.rs:50]
- `Rv32BranchLessThan256Opcode::CLASS_OFFSET = 0x425` [transpiler/lib.rs:60]
- `Rv32Mul256Opcode::CLASS_OFFSET = 0x450` [transpiler/lib.rs:70]

### Configuration Defaults
- `default_range_tuple_checker_sizes() -> [u32; 2]` - Returns `[256, 8192]` [extension.rs:72]

## Key Functions

### Extension Building
- `Int256::build()` - Builds VM inventory with all chips [extension.rs:98]

### Initialization
- `Int256Rv32Config::default()` - Creates default configuration [extension.rs:46]
- `Int256::default()` - Creates default Int256 extension [extension.rs:65]

## Enums

### `Int256Funct7`
Instruction function codes [guest/lib.rs:13]:
- `Add = 0` - Addition
- `Sub = 1` - Subtraction  
- `Xor = 2` - Bitwise XOR
- `Or = 3` - Bitwise OR
- `And = 4` - Bitwise AND
- `Sll = 5` - Logical left shift
- `Srl = 6` - Logical right shift
- `Sra = 7` - Arithmetic right shift
- `Slt = 8` - Set less than (signed)
- `Sltu = 9` - Set less than unsigned
- `Mul = 10` - Multiplication

## Guest Functions (externs)

### Arithmetic Operations
- `zkvm_u256_wrapping_add_impl()` - 256-bit addition [guest/externs.rs:8]
- `zkvm_u256_wrapping_sub_impl()` - 256-bit subtraction [guest/externs.rs:20]
- `zkvm_u256_wrapping_mul_impl()` - 256-bit multiplication [guest/externs.rs:32]

### Bitwise Operations
- `zkvm_u256_bitxor_impl()` - Bitwise XOR [guest/externs.rs:44]
- `zkvm_u256_bitand_impl()` - Bitwise AND [guest/externs.rs:56]
- `zkvm_u256_bitor_impl()` - Bitwise OR [guest/externs.rs:68]

### Shift Operations
- `zkvm_u256_wrapping_shl_impl()` - Left shift [guest/externs.rs:80]
- `zkvm_u256_wrapping_shr_impl()` - Logical right shift [guest/externs.rs:92]
- `zkvm_u256_arithmetic_shr_impl()` - Arithmetic right shift [guest/externs.rs:104]

### Comparison Operations
- `zkvm_u256_eq_impl()` - Equality comparison [guest/externs.rs:116]
- `zkvm_u256_cmp_impl()` - Three-way comparison [guest/externs.rs:131]

### Utility Functions
- `zkvm_u256_clone_impl()` - Clone operation [guest/externs.rs:160]

## Transpiler

### Main Type
- `Int256TranspilerExtension` - Transpiler for Int256 instructions [transpiler/lib.rs:80]

### Key Method
- `Int256TranspilerExtension::process_custom()` - Processes custom instructions [transpiler/lib.rs:83]

## Test Utilities

- `run_int_256_rand_execute()` - Random operation testing helper [tests.rs:41]

## Dependencies

### Circuit Dependencies
- `openvm-circuit` - Core circuit framework
- `openvm-circuit-primitives` - Bitwise and range checking primitives
- `openvm-rv32im-circuit` - Core arithmetic chips
- `openvm-rv32-adapters` - Heap memory adapters

### Transpiler Dependencies  
- `openvm-bigint-transpiler` - Opcode definitions
- `openvm-rv32im-transpiler` - Base opcode types
- `openvm-instructions` - Instruction encoding