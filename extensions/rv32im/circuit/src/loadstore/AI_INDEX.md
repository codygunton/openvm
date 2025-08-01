# LoadStore Component File Index

## Directory Structure
```
extensions/rv32im/circuit/src/loadstore/
├── mod.rs          # Module exports and type definitions
├── core.rs         # Core LoadStore chip implementation
└── tests.rs        # Comprehensive test suite
```

## File Descriptions

### mod.rs (14 lines)
- **Purpose**: Module organization and public API
- **Exports**: Core types via `pub use core::*`
- **Key Type**: `Rv32LoadStoreChip<F>` - Wrapper combining adapter and core chips
- **Dependencies**: 
  - `openvm_circuit::arch::VmChipWrapper`
  - `super::adapters::{Rv32LoadStoreAdapterChip, RV32_REGISTER_NUM_LIMBS}`

### core.rs (386 lines)
- **Purpose**: Core LoadStore logic and constraints
- **Key Components**:
  - `InstructionOpcode` enum: Internal opcode representation with shift encoding
  - `LoadStoreCoreCols<T>`: Trace columns for core chip
  - `LoadStoreCoreRecord<F>`: Execution record structure  
  - `LoadStoreCoreAir`: AIR constraints implementation
  - `LoadStoreCoreChip`: Main chip implementation
  - `run_write_data()`: Helper function for write data computation

- **Key Features**:
  - Flag-based opcode encoding system
  - Constraint evaluation for data transformations
  - Support for all RISC-V load/store variants
  - Shift handling for sub-word operations

### tests.rs (572 lines)
- **Purpose**: Comprehensive testing of LoadStore functionality
- **Test Categories**:
  - Random operation tests (`rand_loadstore_test`)
  - Negative constraint tests (`negative_*_tests`)
  - Sanity tests for each operation (`run_*_sanity_test`)
  - Roundtrip execution tests

- **Test Utilities**:
  - `set_and_execute()`: Helper for test setup and execution
  - `run_negative_loadstore_test()`: Framework for constraint violation tests
  
- **Coverage**:
  - All 6 load/store operations (LOADW, LOADHU, LOADBU, STOREW, STOREH, STOREB)
  - Address alignment scenarios
  - Memory space validation
  - Register edge cases (x0 handling)

## Related Files

### Parent Module (../adapters/)
- `loadstore.rs`: Adapter chip implementation
  - Handles memory interface and address calculation
  - Manages register reads/writes
  - Implements instruction preprocessing/postprocessing

### Key Dependencies
- `openvm_circuit`: Core VM circuit framework
- `openvm_instructions`: Instruction definitions
- `openvm_rv32im_transpiler`: RISC-V opcode definitions
- `openvm_stark_backend`: STARK proof system backend

## Usage Patterns

1. **Chip Creation**:
   ```rust
   let adapter = Rv32LoadStoreAdapterChip::new(...);
   let core = LoadStoreCoreChip::new(offset);
   let chip = Rv32LoadStoreChip::new(adapter, core, ...);
   ```

2. **Instruction Execution**:
   - Adapter preprocesses: calculates addresses, reads memory
   - Core executes: transforms data based on opcode
   - Adapter postprocesses: writes results to memory/registers

3. **Constraint System**:
   - Adapter constraints: memory addressing, register access
   - Core constraints: data transformation correctness
   - Combined system ensures full operation validity