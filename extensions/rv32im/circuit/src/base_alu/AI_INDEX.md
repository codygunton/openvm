# BaseAlu Component - AI Index

## Quick Navigation

### Core Implementation
- [BaseAluCoreChip](core.rs#L181-199) - Main chip implementation
- [BaseAluCoreAir](core.rs#L42-166) - AIR constraints definition
- [BaseAluCoreCols](core.rs#L28-39) - Column layout structure
- [BaseAluCoreRecord](core.rs#L169-179) - Execution record type

### Execution Functions
- [execute_instruction](core.rs#L213-250) - Main instruction execution
- [run_alu](core.rs#L273-285) - ALU operation dispatcher
- [run_add](core.rs#L287-299) - Addition with carry
- [run_subtract](core.rs#L301-318) - Subtraction with borrow
- [run_xor/or/and](core.rs#L320-339) - Bitwise operations

### Module Organization
- [Module exports](mod.rs#L1-16) - Public API and type definitions
- [Rv32BaseAluChip type](mod.rs#L12-16) - Complete chip wrapper

### Test Infrastructure
- [Positive tests](tests.rs#L50-121) - Random operation testing
- [Negative tests](tests.rs#L131-269) - Constraint violation tests
- [Sanity tests](tests.rs#L277-330) - Fixed test vectors
- [Adapter tests](tests.rs#L338-514) - Adapter-specific tests

## Key Concepts

### Opcodes
- `BaseAluOpcode::ADD` - Addition operation
- `BaseAluOpcode::SUB` - Subtraction operation  
- `BaseAluOpcode::XOR` - Bitwise XOR
- `BaseAluOpcode::OR` - Bitwise OR
- `BaseAluOpcode::AND` - Bitwise AND

### Constants
- `NUM_LIMBS` - Number of limbs per value (4 for RV32)
- `LIMB_BITS` - Bits per limb (8 for RV32)
- `RV32_CELL_BITS` - 8 bits per memory cell
- `RV32_REGISTER_NUM_LIMBS` - 4 limbs per register

### Key Structures
- `BaseAluCoreCols<T, NUM_LIMBS, LIMB_BITS>` - Trace columns
- `BaseAluCoreRecord<T, NUM_LIMBS, LIMB_BITS>` - Execution record
- `AdapterAirContext` - Adapter interface context

## Common Patterns

### Carry Computation
```rust
carry[i] = (operands + prev_carry - result) / 2^LIMB_BITS
```

### Bitwise Lookup Interaction
```rust
self.bus.send_xor(x, y, x_xor_y).eval(builder, is_valid)
```

### Opcode Flag Handling
```rust
flags.iter().zip(BaseAluOpcode::iter()).fold(...)
```

## Dependencies

- `openvm_circuit` - Core circuit framework
- `openvm_circuit_primitives` - Bitwise operation lookups
- `openvm_instructions` - Instruction definitions
- `openvm_rv32im_transpiler` - Opcode definitions
- `openvm_stark_backend` - STARK proof system

## Error Handling

- `Result<(AdapterRuntimeContext<F, I>, Self::Record)>` - Execution results
- `VerificationError::OodEvaluationMismatch` - Constraint violations
- `VerificationError::ChallengePhaseError` - Interaction failures