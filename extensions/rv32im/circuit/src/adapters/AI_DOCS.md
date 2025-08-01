# RV32IM Circuit Adapters Component AI Documentation

## Overview
The RV32IM Circuit Adapters component provides the bridge between OpenVM's execution framework and the RISC-V 32-bit integer instruction set (RV32IM). These adapters translate RISC-V instructions into circuit operations while maintaining register consistency, memory coherence, and proper program flow.

## Core Architecture

### Key Files
- `alu.rs`: Arithmetic/logic unit adapter for register operations
- `branch.rs`: Conditional branching adapter
- `jalr.rs`: Jump and link register adapter
- `loadstore.rs`: Memory load/store operations adapter
- `mul.rs`: Multiplication and division adapter
- `rdwrite.rs`: Direct register write adapter
- `mod.rs`: Utility functions and exports

### Primary Responsibilities
1. **RISC-V Instruction Execution**: Implement RV32I base integer instructions
2. **Register File Management**: Handle 32 general-purpose registers
3. **Memory Operations**: Support aligned and unaligned memory access
4. **Program Counter Updates**: Manage sequential and branch execution
5. **Constraint Generation**: Produce sound AIR constraints

## Key Components

### Register Representation
RISC-V 32-bit registers are stored as 4 limbs of 8 bits each:
```rust
pub const RV32_REGISTER_NUM_LIMBS: usize = 4;
pub const RV32_CELL_BITS: usize = 8;
```

### Address Spaces
- **Register Space** (`RV32_REGISTER_AS = 1`): For general-purpose registers
- **Immediate Space** (`RV32_IMM_AS = 0`): For immediate values

### Utility Functions
- `compose`: Convert 4-limb representation to u32
- `decompose`: Convert u32 to 4-limb representation
- `read_rv32_register`: Read register with memory tracking
- `abstract_compose`: Generic composition for symbolic execution

## Adapter Types Deep Dive

### ALU Adapter (`alu.rs`)
**Purpose**: Arithmetic and logic operations on registers

**Supported Operations**:
- Arithmetic: ADD, SUB, ADDI
- Logic: AND, OR, XOR, ANDI, ORI, XORI
- Shifts: SLL, SRL, SRA, SLLI, SRLI, SRAI
- Comparisons: SLT, SLTU, SLTI, SLTIU

**Interface**:
- Reads: 2 (rs1 always from register, rs2 from register or immediate)
- Writes: 1 (rd to register)
- Immediate support: Yes (for I-type instructions)

**Key Design Features**:
- Integrates with bitwise operation lookup tables for efficiency
- Supports both register-register and register-immediate operations
- Handles sign extension for immediate values

### Branch Adapter (`branch.rs`)
**Purpose**: Conditional branching based on register comparisons

**Supported Operations**:
- BEQ: Branch if equal
- BNE: Branch if not equal
- BLT: Branch if less than (signed)
- BGE: Branch if greater or equal (signed)
- BLTU: Branch if less than (unsigned)
- BGEU: Branch if greater or equal (unsigned)

**Interface**:
- Reads: 2 (rs1 and rs2 from registers)
- Writes: 0
- PC modification: Yes (adds immediate offset on branch taken)

**Key Design Features**:
- No register writes, only control flow changes
- Immediate offset encoded in instruction
- PC updated based on comparison result

### JALR Adapter (`jalr.rs`)
**Purpose**: Jump and link register for function calls

**Supported Operations**:
- JALR: Jump to address in rs1 + immediate, save return address

**Interface**:
- Reads: 1 (rs1 from register)
- Writes: 1 (rd with PC+4, skipped if rd=x0)
- PC modification: Yes (sets to rs1 + immediate)

**Key Design Features**:
- Supports function call and return patterns
- Optional write (no write when rd=x0)
- Handles PC+4 calculation for return address

### LoadStore Adapter (`loadstore.rs`)
**Purpose**: Memory load and store operations

**Supported Operations**:
- Word: LW, SW (4 bytes)
- Halfword: LH, LHU, SH (2 bytes)  
- Byte: LB, LBU, SB (1 byte)

**Interface**:
- Reads: Variable (register for address, memory for loads)
- Writes: Variable (register for loads, memory for stores)
- Immediate support: Yes (address offset)

**Key Design Features**:
- Handles aligned and unaligned memory access
- Batch reads/writes 4 bytes for efficiency
- Manages shift amounts for sub-word operations
- Sign/zero extension for loads
- Complex adapter with specialized interface

### Mul Adapter (`mul.rs`)
**Purpose**: Multiplication and division operations (M extension)

**Supported Operations**:
- MUL: Lower 32 bits of product
- MULH: Upper 32 bits of signed × signed
- MULHSU: Upper 32 bits of signed × unsigned
- MULHU: Upper 32 bits of unsigned × unsigned
- DIV: Signed division
- DIVU: Unsigned division
- REM: Signed remainder
- REMU: Unsigned remainder

**Interface**:
- Reads: 2 (rs1 and rs2 from registers)
- Writes: 1 (rd to register)

**Key Design Features**:
- Handles both multiplication and division
- Supports signed and unsigned variants
- Manages overflow and division by zero

### RdWrite Adapter (`rdwrite.rs`)
**Purpose**: Direct register writes for immediate loads

**Supported Operations**:
- LUI: Load upper immediate
- AUIPC: Add upper immediate to PC
- MV: Move immediate to register

**Interface**:
- Reads: 0
- Writes: 1 (rd to register)
- Immediate support: Yes

**Key Design Features**:
- No memory reads required
- Direct immediate to register transfer
- Supports PC-relative addressing (AUIPC)

## Memory Model

### Register File
- 32 general-purpose registers (x0-x31)
- x0 is hardwired to zero
- Each register is 32 bits (4 limbs of 8 bits)
- Stored in address space 1

### Memory Operations
- Support for byte, halfword, and word access
- Alignment handled internally
- Little-endian byte ordering
- Shift operations for unaligned access

## Execution Flow

### Instruction Cycle
1. Fetch instruction from program bus
2. Decode operands and operation type
3. Read source registers/immediates
4. Perform operation
5. Write results to destination
6. Update program counter

### State Transitions
- `from_state`: Current execution state (PC, timestamp)
- `to_state`: Next execution state after instruction
- PC increment: Usually `DEFAULT_PC_STEP` (4 bytes)
- Branch/jump: PC modified by offset or register value

## Integration Points

### ExecutionBridge
- Connects to execution bus for instruction records
- Connects to program bus for instruction fetch
- Manages state transition verification

### MemoryBridge
- Handles all register reads and writes
- Ensures memory consistency
- Provides offline checking auxiliaries

### Bitwise Operations
- ALU adapter uses shared bitwise lookup tables
- Efficient implementation of AND, OR, XOR
- Reduces constraint complexity

### Range Checking
- LoadStore adapter uses variable range checker
- Ensures offsets are within bounds
- Critical for soundness

## Design Principles

### 1. **Correctness First**
- Every operation fully constrained
- No trusted computations
- Complete state verification

### 2. **Efficiency**
- Batch operations where possible
- Share lookup tables
- Minimize memory accesses

### 3. **Modularity**
- Each adapter handles specific instruction types
- Clean interfaces between components
- Reusable patterns

### 4. **RISC-V Compliance**
- Follows RISC-V ISA specification
- Handles all corner cases
- Proper sign extension

## Common Patterns

### Register Read Pattern
```rust
let (record_id, value) = read_rv32_register(
    memory,
    RV32_REGISTER_AS,
    register_pointer
);
```

### Immediate Handling
- I-type: 12-bit sign-extended
- S-type: 12-bit sign-extended (split encoding)
- B-type: 13-bit sign-extended (even values)
- J-type: 21-bit sign-extended (even values)

### Memory Access Pattern
1. Calculate effective address (base + offset)
2. Determine shift for alignment
3. Read/write 4-byte aligned chunk
4. Extract/insert target bytes
5. Handle sign/zero extension for loads

## Performance Considerations

### Optimization Strategies
1. **Batched Operations**: LoadStore adapter batches 4-byte accesses
2. **Lookup Tables**: Bitwise operations use precomputed tables
3. **Conditional Writes**: JALR skips write when rd=x0
4. **Immediate Support**: Reduces memory traffic for constants

### Circuit Complexity
- ALU operations: O(1) constraints per operation
- Memory operations: Higher complexity due to alignment
- Branch operations: Minimal constraints (comparison only)
- Multiplication: May use specialized multiplication gates

## Testing Requirements

### Instruction Coverage
- All RV32I base instructions
- All RV32M multiplication instructions
- Edge cases (x0 writes, overflow, alignment)
- Immediate value ranges

### State Verification
- PC updates (sequential and jumps)
- Register value integrity
- Memory consistency
- Execution trace validity

### Compliance Testing
- RISC-V compliance test suite
- Random instruction sequences
- Boundary conditions
- Error handling