# JALR Component - AI Documentation Index

## Overview
This directory contains the RISC-V JALR (Jump And Link Register) instruction implementation for the OpenVM zkVM. JALR performs indirect jumps with return address saving.

## Documentation Files

### 📚 [AI_DOCS.md](./AI_DOCS.md)
Comprehensive technical documentation covering:
- Architecture and component structure
- Instruction format and execution semantics
- Implementation details (address calculation, limb decomposition)
- Constraint system and validation rules
- Testing strategy and integration points

### 🛠️ [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md)
Practical implementation guide with:
- Common patterns for extending JALR functionality
- Integration examples with memory system
- Debugging techniques and constraint verification
- Performance optimization strategies

### ⚡ [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md)
Quick lookup for:
- Key constants and types
- Common code patterns
- API reference
- Troubleshooting tips

### 🤖 [CLAUDE.md](./CLAUDE.md)
AI assistant instructions for:
- Component-specific guidelines
- Code modification patterns
- Testing requirements
- Common tasks and workflows

## Source Files

### Core Implementation
- **[core.rs](./core.rs)**: Main JALR execution logic and constraints
  - `Rv32JalrCoreChip`: Core computation chip
  - `Rv32JalrCoreAir`: Constraint system
  - `run_jalr()`: Execution function

### Module Definition
- **[mod.rs](./mod.rs)**: Module exports and chip composition
  - Type alias for complete JALR chip
  - Public API surface

### Testing
- **[tests.rs](./tests.rs)**: Comprehensive test suite
  - Positive tests with random inputs
  - Negative tests for constraint violations
  - Sanity tests with known values

## Key Concepts

### JALR Instruction
- **Purpose**: Indirect jump with return address save
- **Format**: I-type instruction (rd, rs1, imm)
- **Operation**: `pc = (rs1 + imm) & ~1; rd = pc + 4`

### Architecture Highlights
1. **Two-chip design**: Core logic + Memory adapter
2. **Optimized trace**: 3 limbs stored, 1 derived
3. **Address alignment**: LSB clearing for 2-byte alignment
4. **Overflow protection**: Range checks on PC values

### Integration Requirements
- Memory controller for register access
- Execution bus for state updates
- Bitwise lookup tables for range checks
- Program bus for instruction fetch

## Quick Start

To understand JALR implementation:
1. Read [AI_DOCS.md](./AI_DOCS.md) for complete overview
2. Check [core.rs](./core.rs) for execution logic
3. Review [tests.rs](./tests.rs) for usage examples
4. See [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md) for API details

## Related Components
- **Adapter System**: `../adapters/jalr.rs` - Memory interface
- **JAL/LUI**: `../jal_lui/` - Related jump instructions
- **Base ALU**: `../base_alu/` - Register operations
- **Branch**: `../branch_lt/` - Conditional jumps