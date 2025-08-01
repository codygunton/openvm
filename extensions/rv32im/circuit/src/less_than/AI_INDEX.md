# LessThan Component - AI Index

## Quick Navigation

### Core Implementation
- [LessThanCoreChip](core.rs#L182-200) - Main chip implementation
- [LessThanCoreAir](core.rs#L48-164) - AIR constraints definition
- [LessThanCoreCols](core.rs#L27-46) - Column layout structure
- [LessThanCoreRecord](core.rs#L166-180) - Execution record type

### Execution Functions
- [execute_instruction](core.rs#L211-291) - Main instruction execution
- [run_less_than](core.rs#L316-329) - Core comparison logic
- [generate_trace_row](core.rs#L297-309) - Trace generation

### Module Organization
- [Module exports](mod.rs#L1-16) - Public API and type definitions
- [Rv32LessThanChip type](mod.rs#L11-15) - Complete chip wrapper

### Test Infrastructure
- [Test module](tests.rs) - Comprehensive test suite

## Key Concepts

### Opcodes
- `LessThanOpcode::SLT` - Signed less than
- `LessThanOpcode::SLTU` - Unsigned less than

### Constants
- `NUM_LIMBS` - Number of limbs per value (4 for RV32)
- `LIMB_BITS` - Bits per limb (8 for RV32)
- `RV32_CELL_BITS` - 8 bits per memory cell
- `RV32_REGISTER_NUM_LIMBS` - 4 limbs per register

### Key Structures
- `LessThanCoreCols<T, NUM_LIMBS, LIMB_BITS>` - Trace columns
  - `b, c`: Input operands
  - `cmp_result`: Comparison result (0 or 1)
  - `opcode_slt_flag, opcode_sltu_flag`: Operation flags
  - `b_msb_f, c_msb_f`: MSB field representations
  - `diff_marker`: Marks first differing limb
  - `diff_val`: Absolute difference at marker position
- `LessThanCoreRecord<T, NUM_LIMBS, LIMB_BITS>` - Execution record
- `AdapterAirContext` - Adapter interface context

## Common Patterns

### MSB Sign Handling
```rust
// For signed comparison
msb_f = -F::from_canonical_u32((1 << LIMB_BITS) - value)
// For unsigned comparison  
msb_f = F::from_canonical_u32(value)
```

### Difference Detection
```rust
for i in (0..NUM_LIMBS).rev() {
    if x[i] != y[i] {
        return ((x[i] < y[i]) ^ x_sign ^ y_sign, i, x_sign, y_sign);
    }
}
```

### Range Check Pattern
```rust
// MSB range check with sign adjustment
self.bus.send_range(
    msb + (1 << (LIMB_BITS - 1)) * slt_flag,
    ...
)
```

## Dependencies

- `openvm_circuit` - Core circuit framework
- `openvm_circuit_primitives` - Bitwise operation lookups
- `openvm_instructions` - Instruction definitions
- `openvm_rv32im_transpiler` - Opcode definitions
- `openvm_stark_backend` - STARK proof system

## Error Handling

- `Result<(AdapterRuntimeContext<F, I>, Self::Record)>` - Execution results
- Constraint violations detected during proof generation
- Range check failures for invalid MSB values