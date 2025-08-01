# Algebra Circuit Extension - Structure Index

## Directory Structure
```
extensions/algebra/circuit/src/
├── lib.rs                      # Main library exports
├── config.rs                   # VM configuration types
├── modular_extension.rs        # Modular arithmetic extension implementation
├── fp2_extension.rs            # Fp2 field extension implementation
├── fp2.rs                      # Fp2 expression builder integration
├── modular_chip/              # Modular arithmetic chip implementations
│   ├── mod.rs                 # Module exports and type aliases
│   ├── addsub.rs              # Add/subtract operations
│   ├── muldiv.rs              # Multiply/divide operations
│   ├── is_eq.rs               # Equality checking
│   └── tests.rs               # Unit tests
└── fp2_chip/                  # Fp2 arithmetic chip implementations
    ├── mod.rs                 # Module exports
    ├── addsub.rs              # Fp2 add/subtract operations
    └── muldiv.rs              # Fp2 multiply/divide operations
```

## Module Relationships

### Core Extensions
- `ModularExtension` - Base extension for modular arithmetic
- `Fp2Extension` - Quadratic extension built on modular arithmetic

### Configuration Hierarchy
```
SystemConfig
└── Rv32ModularConfig
    ├── Rv32I (base ISA)
    ├── Rv32M (multiply extension)
    ├── Rv32Io (I/O extension)
    └── ModularExtension

SystemConfig
└── Rv32ModularWithFp2Config
    ├── Rv32I
    ├── Rv32M
    ├── Rv32Io
    ├── ModularExtension
    └── Fp2Extension
```

### Chip Types

#### Modular Arithmetic Chips
- `ModularAddSubChip<F, NUM_LANES, LANE_SIZE>` - Addition and subtraction
- `ModularMulDivChip<F, NUM_LANES, LANE_SIZE>` - Multiplication and division
- `ModularIsEqualChip<F, NUM_LANES, LANE_SIZE, TOTAL_LIMBS>` - Equality testing

#### Fp2 Arithmetic Chips
- `Fp2AddSubChip<F, NUM_LANES, LANE_SIZE>` - Fp2 addition and subtraction
- `Fp2MulDivChip<F, NUM_LANES, LANE_SIZE>` - Fp2 multiplication and division

### Key Traits and Derives
- `VmExtension<F>` - Core trait for VM extensions
- `VmConfig` - Configuration trait with automatic field detection
- `ChipUsageGetter` - Resource usage tracking
- `InstructionExecutor` - Instruction execution interface
- `AnyEnum` - Enum variant handling

## Import Patterns

### External Dependencies
```rust
// Numeric operations
use num_bigint::{BigUint, RandBigInt};
use num_traits::{FromPrimitive, One};

// OpenVM core
use openvm_circuit::{arch::*, system::phantom::*};
use openvm_circuit_primitives::bitwise_op_lookup::*;

// Instruction definitions
use openvm_instructions::{LocalOpcode, PhantomDiscriminant, VmOpcode};
use openvm_algebra_transpiler::{ModularPhantom, Rv32ModularArithmeticOpcode, Fp2Opcode};

// Circuit building
use openvm_mod_circuit_builder::{ExprBuilder, ExprBuilderConfig, FieldVariable};

// Adapters
use openvm_rv32_adapters::{Rv32IsEqualModAdapterChip, Rv32VecHeapAdapterChip};
```

### Internal Modules
```rust
// From lib.rs
pub use modular_chip::*;
pub use fp2_chip::*;
pub use modular_extension::*;
pub use fp2_extension::*;
pub use fp2::*;
pub use config::*;
```

## Configuration Flow

1. **User Creates Config**
   ```rust
   Rv32ModularConfig::new(vec![modulus1, modulus2])
   ```

2. **Extension Registration**
   - Config fields marked with `#[extension]` auto-register
   - Extensions receive `VmInventoryBuilder` in `build()`

3. **Chip Instantiation**
   - Extension creates chips based on moduli sizes
   - Chips registered with appropriate opcode ranges

4. **Guest Initialization**
   - `generate_init_file_contents()` creates guest setup code
   - Moduli indices map to opcode offsets

## Type Parameters

### Field Type
- `F: PrimeField32` - The base field for the VM (typically BabyBear)

### Chip Parameters
- `NUM_LANES` - Number of parallel processing lanes
- `LANE_SIZE` - Size of each lane in limbs
- `TOTAL_LIMBS` - Total number of limbs (NUM_LANES × LANE_SIZE)

### Common Configurations
- 32-byte fields: `NUM_LANES=1, LANE_SIZE=32`
- 48-byte fields: `NUM_LANES=3, LANE_SIZE=16`

## Testing Infrastructure

### Test Utilities
- `openvm_mod_circuit_builder::test_utils` - Circuit testing helpers
- BN254 curve for realistic field parameters
- Comparison against `halo2curves_axiom` implementations

### Test Pattern
1. Create expression builder
2. Build Fp2 operations
3. Generate trace with test inputs
4. Verify against reference implementation
5. Run STARK prover for soundness check