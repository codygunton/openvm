# OpenVM Pairing Circuit Extension - Index

## Module Structure

### Root Modules
- `lib.rs` - Main library exports
- `config.rs` - Configuration types and builders
- `pairing_extension.rs` - Core extension implementation
- `fp12.rs` - Fp12 field extension type

### Pairing Chips (`pairing_chip/`)
- `mod.rs` - Module exports
- `miller_double_step.rs` - Miller loop doubling step
- `miller_double_and_add_step.rs` - Combined double-and-add operation

#### Line Evaluation (`pairing_chip/line/`)
- `mod.rs` - Line module exports
- `evaluate_line.rs` - Line function evaluation chip

##### D-Type Pairings (`pairing_chip/line/d_type/`)
- `mod.rs` - D-type exports
- `mul_013_by_013.rs` - Multiplication of sparse Fp12 elements
- `mul_by_01234.rs` - Multiplication by 5-sparse element
- `tests.rs` - D-type pairing tests

##### M-Type Pairings (`pairing_chip/line/m_type/`)
- `mod.rs` - M-type exports  
- `mul_023_by_023.rs` - Multiplication of sparse Fp12 elements
- `mul_by_02345.rs` - Multiplication by 5-sparse element
- `tests.rs` - M-type pairing tests

### Fp12 Chips (`fp12_chip/`)
- `mod.rs` - Fp12 chip exports
- `add.rs` - Fp12 addition chip
- `sub.rs` - Fp12 subtraction chip
- `mul.rs` - Fp12 multiplication chip
- `tests.rs` - Fp12 arithmetic tests

## Key Types

### Enums
- `PairingCurve` - Supported pairing curves (BN254, BLS12-381)
- `PairingExtensionExecutor<F>` - Executor variants for different curves
- `PairingExtensionPeriphery<F>` - Peripheral components

### Structs
- `PairingExtension` - Main extension configuration
- `Rv32PairingConfig` - Complete VM configuration with pairing
- `Fp12` - 12-degree field extension type
- `PairingHintSubEx` - Phantom sub-executor for final exponentiation

### Chips
- `MillerDoubleStepChip<F, INPUT_BLOCKS, OUTPUT_BLOCKS, BLOCK_SIZE>`
- `MillerDoubleAndAddStepChip<F, INPUT_BLOCKS, OUTPUT_BLOCKS, BLOCK_SIZE>`
- `EvaluateLineChip<F, INPUT_BLOCKS1, INPUT_BLOCKS2, OUTPUT_BLOCKS, BLOCK_SIZE>`
- `Fp12AddChip<F, INPUT_BLOCKS, OUTPUT_BLOCKS, BLOCK_SIZE>`
- `Fp12SubChip<F, INPUT_BLOCKS, OUTPUT_BLOCKS, BLOCK_SIZE>`
- `Fp12MulChip<F, INPUT_BLOCKS, OUTPUT_BLOCKS, BLOCK_SIZE>`
- `Fp12Mul013By013Chip<F, INPUT_BLOCKS, OUTPUT_BLOCKS, BLOCK_SIZE>`
- `Fp12MulBy01234Chip<F, INPUT_BLOCKS, OUTPUT_BLOCKS, BLOCK_SIZE>`
- `Fp12Mul023By023Chip<F, INPUT_BLOCKS, OUTPUT_BLOCKS, BLOCK_SIZE>`
- `Fp12MulBy02345Chip<F, INPUT_BLOCKS, OUTPUT_BLOCKS, BLOCK_SIZE>`

## Constants

### BN254
- `BN254_NUM_LIMBS` = 32
- `BN254_LIMB_BITS` = 8
- Block size = 32

### BLS12-381
- `BLS12_381_NUM_LIMBS` = 48
- `BLS12_381_LIMB_BITS` = 8
- Block size = 16

## Functions

### Configuration
- `PairingCurve::curve_config()` - Get curve configuration
- `PairingCurve::xi()` - Get xi parameter for curve
- `Rv32PairingConfig::new()` - Create new VM configuration

### Fp12 Operations
- `Fp12::new()` - Create new Fp12 element
- `Fp12::add()` - Addition in Fp12
- `Fp12::sub()` - Subtraction in Fp12
- `Fp12::mul()` - Full multiplication in Fp12
- `Fp12::mul_by_01234()` - Multiply by sparse element
- `Fp12::mul_by_02345()` - Multiply by sparse element

### Circuit Building
- Expression builders for:
  - `miller_double_step_expr()`
  - `miller_double_and_add_step_expr()`
  - `evaluate_line_expr()`
  - `fp12_add_expr()`
  - `fp12_sub_expr()`
  - `fp12_mul_expr()`
  - `mul_013_by_013_expr()`
  - `mul_by_01234_expr()`
  - `mul_023_by_023_expr()`
  - `mul_by_02345_expr()`

### Phantom Execution
- `hint_pairing()` - Generate final exponentiation hints
- `read_fp()` - Read field element from memory

## Traits Implemented

### Extension Traits
- `VmExtension<F>` for `PairingExtension`

### Chip Traits
- `Chip` - Basic chip functionality
- `ChipUsageGetter` - Resource usage tracking
- `InstructionExecutor` - Instruction execution

### Other Traits
- Standard derives: `Clone`, `Debug`, `Serialize`, `Deserialize`
- `From`, `FromRepr` for enum conversions