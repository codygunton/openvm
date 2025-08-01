# AI_DOCS: OpenVM Instructions Component

## Component Purpose
The `openvm-instructions` crate is the foundational layer of the OpenVM zkVM instruction system. It provides the core abstractions for representing VM instructions, opcodes, programs, and executables. This crate is designed for host-side usage, including procedural macros, and forms the basis for the entire OpenVM instruction architecture.

## Architecture Overview

### Instruction Model
OpenVM uses a RISC-inspired instruction format with a fixed number of operands:
- **1 opcode**: Identifies the operation
- **7 operands (a-g)**: Field elements used as instruction parameters
- **Fixed format**: All instructions have the same structure for simplicity

This design allows for:
- Uniform instruction decoding
- Flexible operand usage per instruction type
- Efficient serialization and deserialization
- Easy extension with new instruction types

### Opcode Architecture
The opcode system uses a two-level hierarchy:
1. **Local Opcodes**: Enum variants within a specific instruction class
2. **Global Opcodes**: Unique identifiers across the entire VM

Each instruction class has a `CLASS_OFFSET` that positions its local opcodes in the global opcode space. This prevents conflicts and allows modular extension.

### Program Counter Model
The PC system is designed for RISC-V compatibility:
- **30-bit PC**: Supports programs up to ~1 billion instructions
- **4-byte step**: Matches RISC-V 32-bit instruction size
- **Configurable base**: Allows programs to start at any address
- **Sparse storage**: Programs can have gaps between instructions

## Key Concepts

### 1. Instruction Representation
```rust
pub struct Instruction<F> {
    pub opcode: VmOpcode,
    pub a: F, pub b: F, pub c: F, pub d: F,
    pub e: F, pub f: F, pub g: F,
}
```
Instructions are generic over field type `F`, allowing use with different finite fields.

### 2. Program Structure
Programs are collections of instructions with:
- **Optional slots**: Not every PC value needs an instruction
- **Debug information**: Optional debugging data per instruction
- **PC mapping**: Instructions are addressed by PC = base + (index × step)

### 3. Memory Model
The VM uses a three-tier address space model:
- **Address Space 0**: Immediate values (RV32_IMM_AS)
- **Address Space 1**: Registers (RV32_REGISTER_AS)
- **Address Space 2**: Main memory (RV32_MEMORY_AS)

### 4. Executable Format
A complete executable (`VmExe`) contains:
- **Program**: The instruction sequence
- **PC start**: Initial program counter value
- **Initial memory**: Pre-loaded memory contents
- **Function bounds**: Metadata about function locations

## Design Principles

### 1. Host-Side Focus
This crate is explicitly designed for host-side usage:
- No guest-side dependencies
- Suitable for use in procedural macros
- Efficient serialization for program transfer

### 2. Extensibility
The opcode system allows easy addition of new instruction types:
- Define new `LocalOpcode` enums with unique offsets
- Implement the `LocalOpcode` trait
- Instructions automatically integrate with the VM

### 3. Debugging Support
Rich debugging features are built-in:
- Optional debug info per instruction
- Backtrace capture support
- DSL instruction representation
- Phantom instructions for debugging

### 4. RISC-V Compatibility
Design choices align with RISC-V conventions:
- 4-byte PC step (32-bit instructions)
- 32-bit registers as 4×8-bit limbs
- Similar memory addressing model

## Usage Patterns

### Creating Instructions
```rust
// From signed integers (5 operands)
let inst = Instruction::from_isize(opcode, 1, 2, 3, 4, 5);

// From unsigned integers with variable operand count
let inst = Instruction::from_usize(opcode, [10, 20, 30]);

// All 7 operands
let inst = Instruction::large_from_isize(opcode, 1, 2, 3, 4, 5, 6, 7);

// Phantom instruction for debugging
let inst = Instruction::phantom(discriminant, a, b, c_upper);
```

### Building Programs
```rust
// Create empty program
let mut program = Program::new_empty(step, pc_base);

// Add instructions
program.push_instruction(instruction);
program.push_instruction_and_debug_info(instruction, Some(debug_info));

// Create from instruction list
let program = Program::from_instructions(&instructions);

// Enumerate with PC values
for (pc, instruction, debug_info) in program.enumerate_by_pc() {
    // Process instruction at specific PC
}
```

### Creating Executables
```rust
// Basic executable
let exe = VmExe::new(program)
    .with_pc_start(0x1000)
    .with_init_memory(memory_image);

// With function bounds
exe.fn_bounds.insert(0x1000, FnBound {
    start: 0x1000,
    end: 0x2000,
    name: "main".to_string(),
});
```

## Serialization
The crate provides efficient serialization:
- Programs serialize only defined instructions (sparse format)
- Debug info is excluded from serialization (host-only)
- Memory images use BTreeMap for sorted, efficient access
- Bitcode format for compact binary representation

## Integration Points

### With Compiler (`openvm-compiler`)
- Compiler generates `Instruction` and `Program` instances
- Uses local opcode enums for type safety
- Produces `VmExe` as final output

### With VM (`openvm-circuit`)
- VM loads and executes `VmExe` format
- Interprets opcodes and operands
- Uses PC model for instruction fetch

### With Extensions
- Extensions define new `LocalOpcode` enums
- Register unique `CLASS_OFFSET` values
- Integrate seamlessly with existing instruction system

## Error Handling
The crate uses safe patterns:
- Optional instruction slots prevent out-of-bounds access
- Field conversions handle negative values correctly
- Serialization preserves program structure

## Performance Considerations
- Sparse program storage minimizes memory usage
- BTreeMap for memory provides O(log n) access
- Instruction format optimized for field arithmetic
- Serialization focused on space efficiency