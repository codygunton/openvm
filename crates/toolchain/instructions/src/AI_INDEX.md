# AI_INDEX: OpenVM Instructions Component

## Overview
This crate provides the core instruction representation system for the OpenVM zkVM framework. It defines instruction formats, opcodes, program structures, and executable representations used throughout the virtual machine.

## Core Components

### 1. Instruction System (`instruction.rs`)
- **`Instruction<F>`**: Core instruction type with 7 operands (a-g) plus opcode
- **`NUM_OPERANDS`**: Constant defining 7 operands per instruction
- **`DebugInfo`**: Instruction debugging information with traces and DSL representation
- Key constructors:
  - `from_isize()`: Create from signed integers (5 operands)
  - `from_usize()`: Create from unsigned integers (N operands)
  - `large_from_isize()`: Create with all 7 operands
  - `phantom()`: Create phantom instructions
  - `debug()`: Create debug phantom instructions

### 2. Opcode System (`lib.rs`)
- **`VmOpcode`**: Global opcode type wrapping usize
- **`LocalOpcode` trait**: Interface for local opcodes with class offset
- **Built-in opcodes**:
  - `SystemOpcode`: TERMINATE, PHANTOM (offset 0x0)
  - `PublishOpcode`: PUBLISH (offset 0x020)
  - `UnimplementedOpcode`: REPLACE_ME (offset 0xdeadaf)
- Opcode conversion utilities between local/global

### 3. Program Representation (`program.rs`)
- **`Program<F>`**: Container for instruction sequences
- **PC management**:
  - `PC_BITS`: 30-bit program counter
  - `DEFAULT_PC_STEP`: 4 (RISC-V compatible)
  - `MAX_ALLOWED_PC`: (1 << 30) - 1
- **Key features**:
  - Sparse instruction storage with optional entries
  - Debug info association
  - PC-based enumeration
  - Serialization support

### 4. Executable Format (`exe.rs`)
- **`VmExe<F>`**: Complete executable representation
- **`MemoryImage<F>`**: BTreeMap<(u32, u32), F> for initial memory
- **`FnBounds`**: Function boundary tracking
- **`FnBound`**: Individual function metadata (start, end, name)

### 5. Phantom Instructions (`phantom.rs`)
- **`PhantomDiscriminant`**: u16 wrapper for phantom types
- **`SysPhantom` enum**:
  - `Nop`: No-operation (0)
  - `DebugPanic`: Runtime panic trigger
  - `CtStart`: Start tracing
  - `CtEnd`: End tracing

### 6. RISC-V Support (`riscv.rs`)
- **Constants**:
  - `RV32_REGISTER_NUM_LIMBS`: 4 (32-bit as 4x8-bit)
  - `RV32_CELL_BITS`: 8
  - `RV32_IMM_AS`: 0 (immediate address space)
  - `RV32_REGISTER_AS`: 1 (register address space)  
  - `RV32_MEMORY_AS`: 2 (memory address space)

### 7. Utilities (`utils.rs`)
- **`parse_biguint_auto()`**: Parse BigUint from hex/binary/decimal strings
- **`isize_to_field()`**: Convert signed integers to field elements

## Key Traits

### LocalOpcode
```rust
pub trait LocalOpcode {
    const CLASS_OFFSET: usize;
    fn from_usize(value: usize) -> Self;
    fn local_usize(&self) -> usize;
    fn global_opcode(&self) -> VmOpcode;
}
```

## Dependencies
- `openvm-stark-backend`: Field traits and backend
- `openvm-instructions-derive`: Derive macros for opcodes
- `serde`: Serialization support
- `num-bigint`, `num-traits`: BigInteger operations
- `strum_macros`: Enum utilities
- `itertools`: Iterator extensions
- `backtrace`: Debug trace capture

## Type Parameters
- `F`: Field type parameter used throughout for instruction operands and memory values

## Design Patterns
1. **Phantom Pattern**: Special instructions for debugging/tracing
2. **Sparse Program Storage**: Optional instruction slots for efficient memory
3. **Debug Info Separation**: Optional debug information attached to instructions
4. **RISC-V Compatibility**: 4-byte PC step, 32-bit register model
5. **Address Space Model**: Separate spaces for immediates, registers, memory