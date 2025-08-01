# AUIPC Component - AI Index

## Quick Navigation

### Core Files
- [`mod.rs`](./mod.rs) - Module exports and Rv32AuipcChip type definition
- [`core.rs`](./core.rs) - Core implementation with AIR constraints and execution logic
- [`tests.rs`](./tests.rs) - Comprehensive test suite

### Key Types
- `Rv32AuipcChip<F>` - Main chip type with adapter wrapper
- `Rv32AuipcCoreChip` - Core execution and constraint logic
- `Rv32AuipcCoreCols<T>` - Trace column layout
- `Rv32AuipcCoreAir` - AIR constraint implementation
- `Rv32AuipcCoreRecord<F>` - Execution record type

### Key Functions
- `run_auipc()` - Core execution logic (line 294)
- `execute_instruction()` - VM instruction processor (line 220)
- `eval()` - AIR constraint evaluator (line 60)

### Constants
- `RV32_CELL_BITS = 8` - Bits per limb
- `RV32_REGISTER_NUM_LIMBS = 4` - Limbs per 32-bit value
- `RV32_LIMB_MAX = 255` - Maximum limb value

### Test Entry Points
- `rand_auipc_test()` - Random positive tests (line 60)
- `invalid_limb_negative_tests()` - Constraint violation tests (line 159)
- `execute_roundtrip_sanity_test()` - Execution verification (line 255)

## Component Summary

**Purpose**: Implements RISC-V AUIPC (Add Upper Immediate to PC) instruction

**Computation**: `rd = pc + (imm << 12)`

**Key Features**:
- 32-bit addition with carry propagation
- Limb-based field arithmetic (4x8-bit limbs)
- Integrated range checking via bitwise lookups
- Zero-knowledge proof generation

**Integration**:
- Part of RV32IM extension
- Uses RV32 write adapter for register updates
- Shares bitwise lookup tables with other components

## Usage Example
```rust
// Instantiation
let core = Rv32AuipcCoreChip::new(bitwise_lookup_chip);
let chip = Rv32AuipcChip::new(adapter, core, offline_memory);

// Execution
// AUIPC rd, imm => rd = pc + (imm << 12)
```

## Related Documentation
- [AI_DOCS.md](./AI_DOCS.md) - Comprehensive documentation
- [QUICK_REFERENCE.ai.md](./QUICK_REFERENCE.ai.md) - Quick reference guide
- [IMPLEMENTATION_GUIDE.ai.md](./IMPLEMENTATION_GUIDE.ai.md) - Implementation details