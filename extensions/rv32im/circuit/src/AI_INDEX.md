# RV32IM Circuit Extension - File Index

## Core Module Files

### Main Entry Points
- `lib.rs` - Module exports and public API
- `extension.rs` - Extension configurations and VmExtension implementations
- `test_utils.rs` - Testing utilities for RV32IM circuits

### Documentation
- `README.md` - Detailed design documentation and circuit statements
- `AI_DOCS.md` - Component overview for AI assistance
- `AI_INDEX.md` - This file catalog
- `IMPLEMENTATION_GUIDE.ai.md` - Implementation patterns and concepts
- `QUICK_REFERENCE.ai.md` - Common tasks and code snippets
- `CLAUDE.md` - Component-specific AI instructions

## Instruction Implementation Modules

### Adapters (VM Interface Layer)
Located in `adapters/`:
- `mod.rs` - Adapter module exports
- `alu.rs` - ALU adapter for register operations
- `branch.rs` - Branch adapter for conditional jumps
- `jalr.rs` - JALR adapter for jump and link register
- `loadstore.rs` - Load/store adapter for memory operations
- `mul.rs` - Multiplication adapter
- `rdwrite.rs` - Register write adapter

### Core Instruction Implementations

Each instruction type has its own directory with:
- `mod.rs` - Module definition and chip structure
- `core.rs` - Core instruction logic and constraints
- `tests.rs` - Unit tests (where applicable)

**Arithmetic & Logic:**
- `base_alu/` - Basic ALU operations (ADD, SUB, AND, OR, XOR)
- `less_than/` - Comparison operations (SLT, SLTU)
- `shift/` - Shift operations (SLL, SRL, SRA)

**Multiplication & Division:**
- `mul/` - Basic multiplication (MUL)
- `mulh/` - High multiplication (MULH, MULHU, MULHSU)
- `divrem/` - Division and remainder (DIV, DIVU, REM, REMU)

**Control Flow:**
- `branch_eq/` - Branch equal/not equal (BEQ, BNE)
- `branch_lt/` - Branch less than (BLT, BLTU, BGE, BGEU)
- `jal_lui/` - Jump and link / Load upper immediate
- `jalr/` - Jump and link register
- `auipc/` - Add upper immediate to PC

**Memory Operations:**
- `loadstore/` - Basic load/store operations
- `load_sign_extend/` - Load with sign extension (LB, LH)

**Special Operations:**
- `hintstore/` - Hint and I/O operations (includes README.md)

## Key Patterns

### Chip Structure
Each instruction typically has:
1. An adapter chip in `adapters/`
2. A core chip in its instruction directory
3. A combined chip that connects adapter and core

### Module Organization
- Public exports in `lib.rs`
- Extension configuration in `extension.rs`
- Instruction-specific logic isolated in subdirectories
- Shared utilities in parent module

### Testing
- Unit tests colocated with implementations
- Shared test utilities in `test_utils.rs`
- Integration through OpenVM's test framework