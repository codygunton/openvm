# Load Sign Extend Component - AI Index

## Component Overview
The Load Sign Extend component implements RISC-V signed load instructions (LOADB, LOADH) that load byte/halfword values from memory and sign-extend them to 32-bit words.

## File Structure
```
load_sign_extend/
├── mod.rs           # Module exports and chip type alias definition
├── core.rs          # Core chip implementation with sign extension logic
└── tests.rs         # Comprehensive test suite
```

## Key Components

### LoadSignExtendCoreChip
- **Purpose**: Handles sign extension for byte (LOADB) and halfword (LOADH) load operations
- **Location**: `core.rs:181-279`
- **Key Features**:
  - Sign extension from 8-bit (byte) to 32-bit
  - Sign extension from 16-bit (halfword) to 32-bit
  - Handles shifted read data for unaligned memory access

### LoadSignExtendCoreAir
- **Purpose**: Defines constraints for sign extension operations
- **Location**: `core.rs:63-179`
- **Key Constraints**:
  - Validates opcode flags (loadb0, loadb1, loadh)
  - Enforces sign bit extraction and extension
  - Manages shifted data for alignment handling

### Rv32LoadSignExtendChip
- **Purpose**: Wrapper combining adapter and core functionality
- **Location**: `mod.rs:12-16`
- **Type**: `VmChipWrapper<F, Rv32LoadStoreAdapterChip<F>, LoadSignExtendCoreChip>`

## Data Structures

### LoadSignExtendCoreCols
- **Purpose**: Column layout for constraint system
- **Location**: `core.rs:34-47`
- **Fields**:
  - Opcode flags for different shift amounts
  - Sign bit indicators
  - Shifted read data array
  - Previous data array

### LoadSignExtendCoreRecord
- **Purpose**: Execution trace record
- **Location**: `core.rs:50-60`
- **Contains**: Shifted data, opcode, shift amount, sign bit

## Key Functions

### run_write_data_sign_extend
- **Purpose**: Core sign extension logic
- **Location**: `core.rs:281-319`
- **Handles**:
  - LOADH: Sign extends 16-bit to 32-bit
  - LOADB: Sign extends 8-bit to 32-bit
  - Shift-based alignment (0-3 for bytes, 0/2 for halfwords)

## Integration Points
- Uses `RV32_REGISTER_NUM_LIMBS` (4) and `RV32_CELL_BITS` (8)
- Integrates with `Rv32LoadStoreAdapterChip` for memory operations
- Uses `VariableRangeCheckerBus` for constraint validation
- Part of the RV32IM instruction set implementation

## Opcodes Handled
- `LOADB` (0x216): Load byte with sign extension
- `LOADH` (0x217): Load halfword with sign extension