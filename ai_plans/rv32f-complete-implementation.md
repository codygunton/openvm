# RV32F Complete Implementation Plan

## Executive Summary

### Problem Statement
OpenVM currently implements only 6 out of 32 RV32F (single-precision floating-point) instructions:
- **Implemented**: FLW, FSW, FADD.S, FSUB.S, FMUL.S, FDIV.S
- **Missing**: 26 instructions including square root, fused multiply-add, comparisons, conversions, sign manipulation, min/max, classification, and register moves

This limits the usability of the float extension for real-world programs that require comprehensive IEEE 754 single-precision support.

### Proposed Solution
Systematically implement all remaining RV32F instructions by:
1. Adding transpiler support to recognize and decode each instruction category
2. Creating or extending executors to handle new instruction types
3. Wiring executors to the existing `_zisk_float` handler (which already supports all operations via SoftFloat)
4. Expanding test coverage to verify each instruction works correctly

### Technical Approach

**Key Insight**: The hard work is already done! The `float.c` handler at `extensions/floats/guest/vendor/zisk/lib-float/c/src/float/float.c` already implements full RISC-V F extension semantics using the SoftFloat library. Our task is simply to wire the transpiler and executors to this existing handler.

**Architecture Pattern** (already established):
```
RISC-V Instruction → Transpiler (decode) → OpenVM Instruction → Executor → JALR to _zisk_float → Handler
```

For each instruction category, we need to:
1. **Transpiler**: Decode RISC-V encoding and create OpenVM instruction with appropriate operands
2. **Executor**: Extract operands, reconstruct RISC-V instruction, write to `FLOAT_INST_ADDR`, JALR to handler
3. **Handler**: Already implemented - reads instruction, dispatches to SoftFloat, writes result

### Instruction Categories

| Category | Count | Examples | Complexity |
|----------|-------|----------|------------|
| Arithmetic (sqrt) | 1 | FSQRT.S | Low - extends existing float_alu |
| Fused Multiply-Add | 4 | FMADD.S, FMSUB.S, FNMSUB.S, FNMADD.S | Medium - R4-type (4 operands) |
| Sign Injection | 3 | FSGNJ.S, FSGNJN.S, FSGNJX.S | Low - similar to float_alu |
| Min/Max | 2 | FMIN.S, FMAX.S | Low - similar to float_alu |
| Float→Int Conversion | 2 | FCVT.W.S, FCVT.WU.S | Medium - writes to integer register |
| Int→Float Conversion | 2 | FCVT.S.W, FCVT.S.WU | Medium - reads from integer register |
| Register Move | 2 | FMV.X.W, FMV.W.X | Medium - crosses register files |
| Comparison | 3 | FEQ.S, FLT.S, FLE.S | Medium - writes to integer register |
| Classification | 1 | FCLASS.S | Low - writes to integer register |

### Expected Outcomes
- Complete RV32F instruction set support (32/32 instructions)
- Comprehensive test suite covering all instruction categories
- Full IEEE 754 single-precision compliance via SoftFloat
- Foundation for future RV32D (double-precision) support

---

## Goals & Objectives

### Primary Goals
- **Complete RV32F Coverage**: Implement all 26 remaining instructions to achieve 100% RV32F compliance
- **Maintain Existing Quality**: All new instructions follow the established transpiler→executor→handler pattern
- **Comprehensive Testing**: Expand test.S to verify correct behavior of all instructions including edge cases

### Secondary Objectives
- **Code Reuse**: Maximize reuse of existing executor patterns (float_alu can handle most R-type instructions)
- **Clear Documentation**: Each instruction category has clear examples in test.S
- **Performance**: Maintain efficient JALR-based dispatch to external handler
- **Foundation for RV64F/RV32D**: Design with extensibility in mind for future precision variants

---

## Solution Overview

### Approach

The implementation follows a **category-based parallelization strategy**:
1. Group instructions by type (R-type, R4-type, conversions, etc.)
2. Implement each category in parallel
3. Each category includes: transpiler decoder + executor logic + comprehensive tests
4. Final integration validates all categories work together

### Key Components

1. **Transpiler Extensions** (`extensions/floats/transpiler/src/lib.rs`)
   - Add decoders for new instruction formats (R4-type for fused multiply-add)
   - Extend existing decoders for new funct7/funct3 variants
   - Map RISC-V opcodes to OpenVM float opcodes (0x300-0x3XX range)

2. **New Executors** (create in `extensions/floats/circuit/src/`)
   - `float_fma/` - Handles R4-type fused multiply-add instructions (FMADD.S, FMSUB.S, etc.)
   - `float_convert/` - Handles float↔integer conversions (FCVT.W.S, FCVT.S.W, etc.)
   - `float_compare/` - Handles comparisons that write to integer registers (FEQ.S, FLT.S, FLE.S)
   - `float_move/` - Handles register moves between float and integer files (FMV.X.W, FMV.W.X)
   - `float_class/` - Handles FCLASS.S classification instruction

3. **Extended Executors** (modify existing)
   - `float_alu/` - Add FSQRT.S, FMIN.S, FMAX.S, sign injection (FSGNJ.S, FSGNJN.S, FSGNJX.S)

4. **Test Expansion** (`examples/floats/test.S`)
   - Add test cases for each instruction category
   - Include edge cases: NaN, ±∞, ±0, subnormals, rounding modes
   - Verify correct results and exception flags

### Architecture Diagram

```
RISC-V Instructions (32 total)
    ↓
┌───────────────────────────────────────────────────────────┐
│  Transpiler (extensions/floats/transpiler/src/lib.rs)     │
│  - Decodes RISC-V F instructions                          │
│  - Extracts rd, rs1, rs2, rs3, funct7, funct3, rm         │
│  - Creates OpenVM instructions (opcodes 0x300-0x30A)      │
└───────────────────────────────────────────────────────────┘
    ↓
┌───────────────────────────────────────────────────────────┐
│  Executors (extensions/floats/circuit/src/)               │
│  ┌─────────────┬─────────────┬──────────────┬──────────┐  │
│  │ float_load  │ float_store │  float_alu   │float_fma │  │
│  │  (FLW)      │  (FSW)      │ (FADD, FSUB, │(FMADD.S) │  │
│  │             │             │  FMUL, FDIV, │(FMSUB.S) │  │
│  │             │             │  FSQRT, etc) │  etc     │  │
│  └─────────────┴─────────────┴──────────────┴──────────┘  │
│  ┌─────────────┬─────────────┬──────────────┬──────────┐  │
│  │float_convert│float_compare│  float_move  │float_cls │  │
│  │ (FCVT.W.S)  │  (FEQ.S,    │  (FMV.X.W,   │(FCLASS.S)│  │
│  │ (FCVT.S.W)  │   FLT.S,    │   FMV.W.X)   │          │  │
│  │             │   FLE.S)    │              │          │  │
│  └─────────────┴─────────────┴──────────────┴──────────┘  │
│                                                             │
│  All executors:                                            │
│  1. Read operands from float/int register files            │
│  2. Reconstruct RISC-V instruction encoding                │
│  3. Write instruction to FLOAT_INST_ADDR (0x1F001108)      │
│  4. JALR to _zisk_float handler (address at 0x0001EC60)    │
└───────────────────────────────────────────────────────────┘
    ↓
┌───────────────────────────────────────────────────────────┐
│  Float Handler (_zisk_float in float.c)                   │
│  - Reads instruction from FLOAT_INST_ADDR                  │
│  - Decodes opcode/funct7/funct3                           │
│  - Calls SoftFloat library (f32_add, f32_mul, etc.)       │
│  - Handles IEEE 754 edge cases (NaN, ∞, ±0)               │
│  - Writes result to float or integer register file         │
│  - Returns via RET (PC ← x1 + 4)                           │
└───────────────────────────────────────────────────────────┘
```

### Data Flow Example (FMADD.S)

```
Assembly:  fmadd.s f2, f0, f1, f3  # f2 = (f0 * f1) + f3

Step 1: Transpiler
  - Decodes R4-type: opcode=0x43, rd=2, rs1=0, rs2=1, rs3=3, rm=0, funct2=0
  - Creates OpenVM instruction: opcode=0x306, a=2, b=0, c=1, d=3, e=0

Step 2: Executor (float_fma)
  - Reads rd=2, rs1=0, rs2=1, rs3=3, rm=0
  - Reconstructs RISC-V: 0b00011_00_00001_00000_000_00010_1000011
  - Writes to FLOAT_INST_ADDR
  - Reads handler address from 0x0001EC60 → 0x000100f8
  - Stores return address in x1
  - Jumps to 0x000100f8

Step 3: Handler (_zisk_float)
  - Reads instruction from FLOAT_INST_ADDR
  - Decodes opcode=0x43 (FMADD)
  - Extracts rd=2, rs1=0, rs2=1, rs3=3
  - Calls f32_mulAdd(fregs[0], fregs[1], fregs[3])
  - Writes result to fregs[2]
  - Returns (RET)

Result: f2 now contains (f0 * f1) + f3 with IEEE 754 rounding
```

---

## Implementation Tasks

### CRITICAL IMPLEMENTATION RULES
1. **NO PLACEHOLDER CODE**: Every implementation must be production-ready
2. **FOLLOW EXISTING PATTERNS**: Use float_alu/float_load/float_store as templates
3. **COMPLETE IMPLEMENTATIONS**: Each task includes transpiler + executor + tests
4. **EXACT RISC-V ENCODING**: Reconstruct instruction bits exactly per RISC-V spec
5. **HANDLER IS READY**: float.c already works - just wire to it correctly

### Visual Dependency Tree

```
extensions/floats/
├── transpiler/src/
│   └── lib.rs (Tasks #0, #1, #2, #3, #4, #5, #6, #7, #8)
│       - Add decoders for all instruction categories
│
├── circuit/src/
│   ├── float_alu/
│   │   ├── core.rs (Task #1: Add FSQRT opcode support)
│   │   └── execution.rs (Task #1: Handle FSQRT, FMIN, FMAX, FSGNJ*)
│   │
│   ├── float_fma/ (Task #2: NEW - R4-type fused multiply-add)
│   │   ├── mod.rs
│   │   ├── core.rs
│   │   └── execution.rs
│   │
│   ├── float_convert/ (Task #3: NEW - float↔int conversions)
│   │   ├── mod.rs
│   │   ├── core.rs
│   │   └── execution.rs
│   │
│   ├── float_compare/ (Task #4: NEW - comparisons to int reg)
│   │   ├── mod.rs
│   │   ├── core.rs
│   │   └── execution.rs
│   │
│   ├── float_move/ (Task #5: NEW - register file transfers)
│   │   ├── mod.rs
│   │   ├── core.rs
│   │   └── execution.rs
│   │
│   ├── float_class/ (Task #6: NEW - FCLASS.S)
│   │   ├── mod.rs
│   │   ├── core.rs
│   │   └── execution.rs
│   │
│   ├── lib.rs (Task #9: Register all new executors)
│   └── Cargo.toml (Task #9: Add executor modules)
│
└── guest/src/lib.rs (Task #10: Export float intrinsics)

examples/floats/
└── test.S (Tasks #11-#17: Add comprehensive tests)
    - Basic arithmetic tests
    - FMA tests
    - Conversion tests
    - Comparison tests
    - Edge case tests (NaN, ±∞, ±0)
    - Rounding mode tests
```

### Execution Plan

#### Group A: Foundation & Transpiler (Execute all in parallel)

- [ ] **Task #0**: Add transpiler support for extended float_alu instructions
  - File: `extensions/floats/transpiler/src/lib.rs`
  - Implements: Decoder for FSQRT.S, FMIN.S, FMAX.S, FSGNJ.S, FSGNJN.S, FSGNJX.S
  - RISC-V encoding:
    - FSQRT.S: opcode=0x53, funct7=0x2C, rs2=0, funct3=rm
    - FMIN.S: opcode=0x53, funct7=0x14, funct3=0
    - FMAX.S: opcode=0x53, funct7=0x14, funct3=1
    - FSGNJ.S: opcode=0x53, funct7=0x10, funct3=0
    - FSGNJN.S: opcode=0x53, funct7=0x10, funct3=1
    - FSGNJX.S: opcode=0x53, funct7=0x10, funct3=2
  - OpenVM opcodes: Extend 0x302-0x305 range to include 0x306 (FSQRT), 0x307 (FMIN/FMAX), 0x308 (FSGNJ*)
  - Operand encoding: Same as existing float_alu (rd, rs1, rs2, opcode_variant)
  - Integration: Extends existing `FloatTranspilerExtension::process_custom` match on funct7
  - Context: These are R-type instructions similar to FADD/FSUB/FMUL/FDIV already implemented

- [x] **Task #1**: Extend float_alu executor for new instructions
  - Files:
    - `extensions/floats/circuit/src/float_alu/core.rs`
    - `extensions/floats/circuit/src/float_alu/execution.rs`
  - Imports: No new imports needed
  - Implements:
    - In `core.rs`: Add opcode constants for FSQRT (6), FMIN/FMAX (7), FSGNJ* (8)
    - In `execution.rs` `execute_e12_impl()`:
      - Extend funct7 match to handle:
        - `0x2C` → FSQRT.S
        - `0x14` → FMIN.S (funct3=0) / FMAX.S (funct3=1)
        - `0x10` → FSGNJ.S/FSGNJN.S/FSGNJX.S (funct3=0/1/2)
      - Reconstruct RISC-V encoding with correct funct7, rs2 (0 for FSQRT), funct3
  - RISC-V instruction reconstruction:
    ```rust
    let (funct7, funct3) = match pre_compute.opcode {
        6 => (0x2C, 0), // FSQRT
        7 => (0x14, if is_max { 1 } else { 0 }), // FMIN/FMAX
        8 => (0x10, sign_op_type), // FSGNJ variants
        _ => (existing_opcodes...)
    };
    let riscv_inst = (funct7 << 25) | ((pre_compute.rs2 as u32) << 20) |
                     ((pre_compute.rs1 as u32) << 15) | (funct3 << 12) |
                     ((pre_compute.rd as u32) << 7) | 0x53;
    ```
  - Exports: No new exports (extends existing FloatAluExecutor)
  - Integration: Existing JALR pattern to _zisk_float handler
  - Note: Handler already supports all these operations in float.c

- [ ] **Task #2**: Add transpiler support for R4-type fused multiply-add instructions
  - File: `extensions/floats/transpiler/src/lib.rs`
  - Implements: Decoder for FMADD.S, FMSUB.S, FNMSUB.S, FNMADD.S
  - RISC-V encoding (R4-type):
    - FMADD.S: opcode=0x43, funct2=0, rs3[31:27], rs2[24:20], rs1[19:15], rm[14:12], rd[11:7]
    - FMSUB.S: opcode=0x47, funct2=0
    - FNMSUB.S: opcode=0x4B, funct2=0
    - FNMADD.S: opcode=0x4F, funct2=0
  - Decoder logic:
    ```rust
    fn decode_r4_type(&self, instruction: u32) -> OpenVMInstruction {
        let opcode = instruction & 0x7F;
        let rd = (instruction >> 7) & 0x1F;
        let rm = (instruction >> 12) & 0x7;
        let rs1 = (instruction >> 15) & 0x1F;
        let rs2 = (instruction >> 20) & 0x1F;
        let rs3 = (instruction >> 27) & 0x1F;
        let funct2 = (instruction >> 25) & 0x3;

        // Map to OpenVM opcode based on RISC-V opcode
        let openvm_opcode = match opcode {
            0x43 => 0x309, // FMADD.S
            0x47 => 0x30A, // FMSUB.S
            0x4B => 0x30B, // FNMSUB.S
            0x4F => 0x30C, // FNMADD.S
            _ => panic!("Invalid R4-type opcode")
        };

        OpenVMInstruction::new(openvm_opcode, rd, rs1, rs2, rs3, rm)
    }
    ```
  - OpenVM opcodes: 0x309 (FMADD), 0x30A (FMSUB), 0x30B (FNMSUB), 0x30C (FNMADD)
  - Operand encoding: a=rd, b=rs1, c=rs2, d=rs3, e=rm (5 operands for R4-type)
  - Integration: Add new match arms in `process_custom()` for opcodes 0x43/0x47/0x4B/0x4F
  - Context: R4-type is unique format with 4 register operands (rd, rs1, rs2, rs3)

- [ ] **Task #3**: Add transpiler support for float↔int conversion instructions
  - File: `extensions/floats/transpiler/src/lib.rs`
  - Implements: Decoder for FCVT.W.S, FCVT.WU.S, FCVT.S.W, FCVT.S.WU
  - RISC-V encoding (R-type with rs2 field as conversion type):
    - FCVT.W.S: opcode=0x53, funct7=0x60, rs2=0, funct3=rm
    - FCVT.WU.S: opcode=0x53, funct7=0x60, rs2=1, funct3=rm
    - FCVT.S.W: opcode=0x53, funct7=0x68, rs2=0, funct3=rm
    - FCVT.S.WU: opcode=0x53, funct7=0x68, rs2=1, funct3=rm
  - Decoder logic:
    ```rust
    // In match for opcode 0x53 (FP_OPCODE)
    0x60 => { // Float to Int
        let rs2 = (instruction >> 20) & 0x1F;
        let unsigned = rs2 == 1;
        OpenVMInstruction::new(0x30D, rd, rs1, unsigned as u8, rm)
    }
    0x68 => { // Int to Float
        let rs2 = (instruction >> 20) & 0x1F;
        let unsigned = rs2 == 1;
        OpenVMInstruction::new(0x30E, rd, rs1, unsigned as u8, rm)
    }
    ```
  - OpenVM opcodes: 0x30D (FCVT.W/WU.S), 0x30E (FCVT.S.W/WU)
  - Operand encoding: a=rd, b=rs1, c=unsigned_flag, d=rm
  - Integration: Add funct7 cases 0x60 and 0x68 in FP_OPCODE handler
  - Context: These cross register files (float ↔ integer)

- [ ] **Task #4**: Add transpiler support for comparison instructions
  - File: `extensions/floats/transpiler/src/lib.rs`
  - Implements: Decoder for FEQ.S, FLT.S, FLE.S
  - RISC-V encoding (R-type, writes to integer register):
    - FEQ.S: opcode=0x53, funct7=0x50, funct3=2
    - FLT.S: opcode=0x53, funct7=0x50, funct3=1
    - FLE.S: opcode=0x53, funct7=0x50, funct3=0
  - Decoder logic:
    ```rust
    0x50 => { // Comparisons
        let funct3 = (instruction >> 12) & 0x7;
        let comp_type = match funct3 {
            0 => 0, // FLE
            1 => 1, // FLT
            2 => 2, // FEQ
            _ => panic!("Invalid comparison funct3")
        };
        OpenVMInstruction::new(0x30F, rd, rs1, rs2, comp_type)
    }
    ```
  - OpenVM opcode: 0x30F (all comparisons)
  - Operand encoding: a=rd (int reg), b=rs1 (float), c=rs2 (float), d=comp_type
  - Integration: Add funct7 case 0x50 in FP_OPCODE handler
  - Context: Reads from float registers, writes boolean result to integer register

- [x] **Task #5**: Add transpiler support for register move instructions
  - File: `extensions/floats/transpiler/src/lib.rs`
  - Implements: Decoder for FMV.X.W, FMV.W.X
  - RISC-V encoding:
    - FMV.X.W: opcode=0x53, funct7=0x70, rs2=0, funct3=0 (float → int)
    - FMV.W.X: opcode=0x53, funct7=0x78, rs2=0, funct3=0 (int → float)
  - Decoder logic:
    ```rust
    0x70 => { // FMV.X.W (float to int)
        OpenVMInstruction::new(0x310, rd, rs1, 0, 0)
    }
    0x78 => { // FMV.W.X (int to float)
        OpenVMInstruction::new(0x311, rd, rs1, 0, 0)
    }
    ```
  - OpenVM opcodes: 0x310 (FMV.X.W), 0x311 (FMV.W.X)
  - Operand encoding: a=rd, b=rs1 (simple 2-operand)
  - Integration: Add funct7 cases 0x70 and 0x78 in FP_OPCODE handler
  - Context: Bitwise copy between register files without interpretation

- [ ] **Task #6**: Add transpiler support for FCLASS.S instruction
  - File: `extensions/floats/transpiler/src/lib.rs`
  - Implements: Decoder for FCLASS.S
  - RISC-V encoding:
    - FCLASS.S: opcode=0x53, funct7=0x70, rs2=0, funct3=1
  - Decoder logic:
    ```rust
    // Extend funct7=0x70 case to check funct3
    0x70 => {
        let funct3 = (instruction >> 12) & 0x7;
        match funct3 {
            0 => OpenVMInstruction::new(0x310, rd, rs1, 0, 0), // FMV.X.W
            1 => OpenVMInstruction::new(0x312, rd, rs1, 0, 0), // FCLASS.S
            _ => panic!("Invalid funct3 for funct7=0x70")
        }
    }
    ```
  - OpenVM opcode: 0x312 (FCLASS)
  - Operand encoding: a=rd (int reg), b=rs1 (float reg)
  - Integration: Extends funct7=0x70 case to distinguish FMV.X.W vs FCLASS.S via funct3
  - Context: Examines float value, writes 10-bit classification mask to integer register

---

#### Group B: New Executors (Execute all in parallel after Group A)

- [ ] **Task #7**: Create float_fma executor for fused multiply-add instructions
  - Folder: `extensions/floats/circuit/src/float_fma/`
  - Files to create: `mod.rs`, `core.rs`, `execution.rs`
  - Imports:
    ```rust
    // core.rs
    use openvm_circuit_derive::*;
    use openvm_stark_backend::p3_field::PrimeField32;

    // execution.rs
    use std::borrow::{Borrow, BorrowMut};
    use std::mem::size_of;
    use openvm_circuit::arch::*;
    use openvm_circuit::system::memory::online::GuestMemory;
    use openvm_circuit_primitives_derive::AlignedBytesBorrow;
    use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
    use openvm_stark_backend::p3_field::PrimeField32;
    use crate::constants::*;
    use super::core::FloatFmaExecutor;
    ```
  - Implements:
    - `mod.rs`: Module exports
    - `core.rs`:
      ```rust
      #[derive(Debug, Clone, AlignedBorrow)]
      pub struct FloatFmaExecutor;

      impl FloatFmaExecutor {
          pub fn new() -> Self {
              Self
          }
      }
      ```
    - `execution.rs`:
      - `FloatFmaPreCompute` struct: rd, rs1, rs2, rs3, rm, opcode (6 fields for R4-type)
      - `pre_compute_impl()`: Extract operands from instruction
      - `execute_e12_impl()`:
        1. Reconstruct RISC-V R4-type encoding
        2. Write to FLOAT_INST_ADDR
        3. Load handler address
        4. Store return address in x1
        5. JALR to handler
  - RISC-V instruction reconstruction:
    ```rust
    let opcode = match pre_compute.opcode {
        0 => 0x43, // FMADD.S
        1 => 0x47, // FMSUB.S
        2 => 0x4B, // FNMSUB.S
        3 => 0x4F, // FNMADD.S
        _ => unreachable!()
    };
    let riscv_inst = ((pre_compute.rs3 as u32) << 27) | (0 << 25) | // funct2=0 for .S
                     ((pre_compute.rs2 as u32) << 20) |
                     ((pre_compute.rs1 as u32) << 15) |
                     ((pre_compute.rm as u32) << 12) |
                     ((pre_compute.rd as u32) << 7) |
                     opcode;
    ```
  - Exports: FloatFmaExecutor implementing Executor<F> and MeteredExecutor<F>
  - Integration: Registered in circuit/src/lib.rs, called by transpiler for opcodes 0x309-0x30C
  - Example: `fmadd.s f2, f0, f1, f3` → handler computes (f0 * f1) + f3 → result in f2

- [ ] **Task #8**: Create float_convert executor for float↔int conversions
  - Folder: `extensions/floats/circuit/src/float_convert/`
  - Files to create: `mod.rs`, `core.rs`, `execution.rs`
  - Imports: Same base imports as Task #7
  - Implements:
    - `FloatConvertPreCompute`: rd, rs1, unsigned_flag, rm, direction (float→int or int→float)
    - `execute_e12_impl()`:
      - For FCVT.W/WU.S (float → int):
        1. Read float value from float register rs1
        2. Reconstruct RISC-V instruction with funct7=0x60, rs2=(unsigned ? 1 : 0)
        3. JALR to handler
        4. Handler writes result to integer register rd
      - For FCVT.S.W/WU (int → float):
        1. Read int value from integer register rs1
        2. Reconstruct RISC-V instruction with funct7=0x68, rs2=(unsigned ? 1 : 0)
        3. JALR to handler
        4. Handler writes result to float register rd
  - RISC-V instruction reconstruction:
    ```rust
    let (funct7, rs2_val) = match (pre_compute.direction, pre_compute.unsigned_flag) {
        (0, false) => (0x60, 0), // FCVT.W.S
        (0, true)  => (0x60, 1), // FCVT.WU.S
        (1, false) => (0x68, 0), // FCVT.S.W
        (1, true)  => (0x68, 1), // FCVT.S.WU
        _ => unreachable!()
    };
    let riscv_inst = (funct7 << 25) | (rs2_val << 20) |
                     ((pre_compute.rs1 as u32) << 15) |
                     ((pre_compute.rm as u32) << 12) |
                     ((pre_compute.rd as u32) << 7) | 0x53;
    ```
  - Exports: FloatConvertExecutor
  - Integration: Handles register file boundary crossing
  - Note: Handler performs actual IEEE 754 conversion with rounding and overflow checking

- [ ] **Task #9**: Create float_compare executor for comparison instructions
  - Folder: `extensions/floats/circuit/src/float_compare/`
  - Files to create: `mod.rs`, `core.rs`, `execution.rs`
  - Imports: Same base imports as Task #7
  - Implements:
    - `FloatComparePreCompute`: rd (int reg), rs1 (float), rs2 (float), comp_type (0=FLE, 1=FLT, 2=FEQ)
    - `execute_e12_impl()`:
      1. Read float values from rs1 and rs2
      2. Reconstruct RISC-V instruction with funct7=0x50, funct3=comp_type
      3. JALR to handler
      4. Handler performs comparison, writes 0 or 1 to integer register rd
  - RISC-V instruction reconstruction:
    ```rust
    let funct3 = match pre_compute.comp_type {
        0 => 0, // FLE.S
        1 => 1, // FLT.S
        2 => 2, // FEQ.S
        _ => unreachable!()
    };
    let riscv_inst = (0x50 << 25) | ((pre_compute.rs2 as u32) << 20) |
                     ((pre_compute.rs1 as u32) << 15) | (funct3 << 12) |
                     ((pre_compute.rd as u32) << 7) | 0x53;
    ```
  - Exports: FloatCompareExecutor
  - Integration: Writes boolean result to integer register (not float register)
  - Note: Handler implements IEEE 754 comparison semantics including NaN handling

- [ ] **Task #10**: Create float_move executor for register transfer instructions
  - Folder: `extensions/floats/circuit/src/float_move/`
  - Files to create: `mod.rs`, `core.rs`, `execution.rs`
  - Imports: Same base imports as Task #7
  - Implements:
    - `FloatMovePreCompute`: rd, rs1, direction (0=float→int, 1=int→float)
    - `execute_e12_impl()`:
      - For FMV.X.W (float → int):
        1. Read 32-bit value from float register rs1
        2. Reconstruct RISC-V instruction with funct7=0x70, rs2=0, funct3=0
        3. JALR to handler
        4. Handler writes bit pattern to integer register rd
      - For FMV.W.X (int → float):
        1. Read 32-bit value from integer register rs1
        2. Reconstruct RISC-V instruction with funct7=0x78, rs2=0, funct3=0
        3. JALR to handler
        4. Handler writes bit pattern to float register rd
  - RISC-V instruction reconstruction:
    ```rust
    let funct7 = if pre_compute.direction == 0 { 0x70 } else { 0x78 };
    let riscv_inst = (funct7 << 25) | (0 << 20) | // rs2=0
                     ((pre_compute.rs1 as u32) << 15) | (0 << 12) | // funct3=0
                     ((pre_compute.rd as u32) << 7) | 0x53;
    ```
  - Exports: FloatMoveExecutor
  - Integration: Bitwise transfer without format conversion
  - Note: Preserves NaN payloads and all bit patterns (no IEEE 754 interpretation)

- [x] **Task #11**: Create float_class executor for FCLASS.S instruction
  - Folder: `extensions/floats/circuit/src/float_class/`
  - Files to create: `mod.rs`, `core.rs`, `execution.rs`
  - Imports: Same base imports as Task #7
  - Implements:
    - `FloatClassPreCompute`: rd (int reg), rs1 (float reg)
    - `execute_e12_impl()`:
      1. Read float value from rs1
      2. Reconstruct RISC-V instruction with funct7=0x70, rs2=0, funct3=1
      3. JALR to handler
      4. Handler examines value, writes 10-bit classification to integer register rd
  - RISC-V instruction reconstruction:
    ```rust
    let riscv_inst = (0x70 << 25) | (0 << 20) | // rs2=0
                     ((pre_compute.rs1 as u32) << 15) | (1 << 12) | // funct3=1
                     ((pre_compute.rd as u32) << 7) | 0x53;
    ```
  - Classification bits (exactly one set):
    - Bit 0: -∞
    - Bit 1: negative normal
    - Bit 2: negative subnormal
    - Bit 3: -0
    - Bit 4: +0
    - Bit 5: positive subnormal
    - Bit 6: positive normal
    - Bit 7: +∞
    - Bit 8: signaling NaN
    - Bit 9: quiet NaN
  - Exports: FloatClassExecutor
  - Integration: Diagnostic instruction for examining float values
  - Note: Handler performs bitwise analysis, no exception flags set

---

#### Group C: Integration (Execute after Group B)

- [x] **Task #12**: Register all new executors in circuit module
  - File: `extensions/floats/circuit/src/lib.rs`
  - Changes:
    - Add module declarations:
      ```rust
      pub mod float_fma;
      pub mod float_convert;
      pub mod float_compare;
      pub mod float_move;
      pub mod float_class;
      ```
    - Add to public exports:
      ```rust
      pub use float_fma::FloatFmaExecutor;
      pub use float_convert::FloatConvertExecutor;
      pub use float_compare::FloatCompareExecutor;
      pub use float_move::FloatMoveExecutor;
      pub use float_class::FloatClassExecutor;
      ```
  - File: `extensions/floats/circuit/Cargo.toml`
  - No changes needed (all dependencies already present)
  - Integration: Makes executors available to VM runtime
  - Context: Follows existing pattern from float_load, float_store, float_alu

- [x] **Task #13**: Wire executors to VM runtime and opcode mapping
  - File: Location TBD (wherever VM executor registration happens - needs exploration)
  - Implements: Register new executors for OpenVM opcodes 0x306-0x312
  - Opcode mapping:
    ```
    0x306 → FloatAluExecutor (FSQRT)
    0x307 → FloatAluExecutor (FMIN/FMAX)
    0x308 → FloatAluExecutor (FSGNJ*)
    0x309 → FloatFmaExecutor (FMADD.S)
    0x30A → FloatFmaExecutor (FMSUB.S)
    0x30B → FloatFmaExecutor (FNMSUB.S)
    0x30C → FloatFmaExecutor (FNMADD.S)
    0x30D → FloatConvertExecutor (FCVT.W/WU.S)
    0x30E → FloatConvertExecutor (FCVT.S.W/WU)
    0x30F → FloatCompareExecutor (FEQ/FLT/FLE.S)
    0x310 → FloatMoveExecutor (FMV.X.W)
    0x311 → FloatMoveExecutor (FMV.W.X)
    0x312 → FloatClassExecutor (FCLASS.S)
    ```
  - Integration: Ensures runtime can dispatch to correct executor for each opcode
  - Context: May require exploration to find correct registration point

---

#### Group D: Comprehensive Testing (Can start after Group A, run in parallel with Groups B/C)

- [x] **Task #14**: Add basic arithmetic tests to test.S
  - File: `examples/floats/test.S`
  - Tests to add:
    - FSQRT.S: `sqrt(4.0) = 2.0`, `sqrt(2.0) ≈ 1.414213562`
    - FMIN.S: `min(1.0, 2.0) = 1.0`, `min(-0.0, +0.0) = -0.0`
    - FMAX.S: `max(1.0, 2.0) = 2.0`, `max(-0.0, +0.0) = +0.0`
    - FSGNJ.S: magnitude of 1.5 with sign of -2.0 = -1.5
    - FSGNJN.S: magnitude of 1.5 with negated sign of -2.0 = 1.5
    - FSGNJX.S: XOR signs (test with ±1.0, ±2.0)
  - Test pattern:
    ```asm
    # Test FSQRT.S: sqrt(4.0) = 2.0
    li      t0, 0x40800000    # 4.0 in IEEE 754
    sw      t0, 0(sp)
    flw     f0, 0(sp)
    fsqrt.s f1, f0
    fsw     f1, 0(sp)
    lw      t1, 0(sp)
    li      t2, 0x40000000    # Expected: 2.0
    bne     t1, t2, fail
    ```
  - Integration: Extends existing test.S infrastructure
  - Validation: Each test checks exact IEEE 754 bit pattern

- [x] **Task #15**: Add fused multiply-add tests to test.S
  - File: `examples/floats/test.S`
  - Tests to add:
    - FMADD.S: `(2.0 * 3.0) + 4.0 = 10.0`
    - FMSUB.S: `(2.0 * 3.0) - 4.0 = 2.0`
    - FNMSUB.S: `-(2.0 * 3.0) + 4.0 = -2.0`
    - FNMADD.S: `-(2.0 * 3.0) - 4.0 = -10.0`
    - Rounding test: `(1.0e20 * 1.0) + 1.0` should round correctly
  - Test pattern:
    ```asm
    # Test FMADD.S: (2.0 * 3.0) + 4.0 = 10.0
    li      t0, 0x40000000    # 2.0
    li      t1, 0x40400000    # 3.0
    li      t2, 0x40800000    # 4.0
    sw      t0, 0(sp)
    sw      t1, 4(sp)
    sw      t2, 8(sp)
    flw     f0, 0(sp)
    flw     f1, 4(sp)
    flw     f2, 8(sp)
    fmadd.s f3, f0, f1, f2
    fsw     f3, 12(sp)
    lw      t3, 12(sp)
    li      t4, 0x41200000    # Expected: 10.0
    bne     t3, t4, fail
    ```
  - Integration: Tests R4-type instruction handling
  - Validation: Verifies single rounding (not double rounding of separate mul+add)

- [ ] **Task #16**: Add conversion and comparison tests to test.S
  - File: `examples/floats/test.S`
  - Tests to add:
    - FCVT.W.S: `(int)3.7 = 3`, `(int)-3.7 = -4` (with rounding modes)
    - FCVT.WU.S: `(uint)3.7 = 3`, verify overflow handling for negatives
    - FCVT.S.W: `(float)42 = 42.0`
    - FCVT.S.WU: `(float)0xFFFFFFFF = 4294967296.0`
    - FEQ.S: `1.0 == 1.0 → 1`, `1.0 == 2.0 → 0`, `NaN == NaN → 0`
    - FLT.S: `1.0 < 2.0 → 1`, `2.0 < 1.0 → 0`, `1.0 < NaN → 0`
    - FLE.S: `1.0 <= 1.0 → 1`, `2.0 <= 1.0 → 0`
    - FMV.X.W: Move 3.14159 bits to integer register, verify exact pattern
    - FMV.W.X: Move integer bit pattern to float register
    - FCLASS.S: Test all 10 classes (±∞, ±normal, ±subnormal, ±0, qNaN, sNaN)
  - Test pattern for conversions:
    ```asm
    # Test FCVT.W.S: (int)3.7 = 3 with RTZ (round toward zero)
    li      t0, 0x406CCCCD    # 3.7 in IEEE 754
    sw      t0, 0(sp)
    flw     f0, 0(sp)
    fcvt.w.s x10, f0, rtz    # Convert to int with RTZ mode
    li      t1, 3
    bne     x10, t1, fail
    ```
  - Integration: Tests register file crossings and IEEE 754 edge cases
  - Validation: Verify correct rounding modes and exception flags

- [ ] **Task #17**: Add edge case tests (NaN, ±∞, ±0, subnormals)
  - File: `examples/floats/test.S`
  - Tests to add:
    - **NaN propagation**:
      - `NaN + 1.0 = NaN`
      - `sqrt(-1.0) = NaN`
      - `0.0 / 0.0 = NaN`
      - Verify quiet vs signaling NaN handling
    - **Infinity arithmetic**:
      - `∞ + 1.0 = ∞`
      - `∞ + ∞ = ∞`
      - `∞ - ∞ = NaN`
      - `∞ * 0.0 = NaN`
      - `1.0 / 0.0 = ∞`
    - **Signed zero**:
      - `+0.0 + +0.0 = +0.0`
      - `+0.0 + -0.0 = +0.0`
      - `-0.0 + -0.0 = -0.0`
      - `min(+0.0, -0.0) = -0.0`
      - `max(+0.0, -0.0) = +0.0`
    - **Subnormal numbers**:
      - Smallest positive normal: `0x00800000`
      - Largest subnormal: `0x007FFFFF`
      - Operations producing subnormals (underflow)
    - **Rounding modes** (test with each mode):
      - RNE (round to nearest, ties to even)
      - RTZ (round toward zero)
      - RDN (round down)
      - RUP (round up)
      - RMM (round to nearest, ties to max magnitude)
  - Test pattern for edge cases:
    ```asm
    # Test NaN propagation: sqrt(-1.0) = NaN
    li      t0, 0xBF800000    # -1.0
    sw      t0, 0(sp)
    flw     f0, 0(sp)
    fsqrt.s f1, f0
    fsw     f1, 0(sp)
    lw      t1, 0(sp)
    # Check if result is NaN (exponent=0xFF, mantissa≠0)
    li      t2, 0x7F800000
    and     t3, t1, t2
    bne     t3, t2, fail     # Exponent must be 0xFF
    li      t2, 0x007FFFFF
    and     t3, t1, t2
    beqz    t3, fail         # Mantissa must be non-zero
    ```
  - Integration: Validates IEEE 754 compliance
  - Validation: Handler must produce correct canonical NaN and ±∞ representations

- [ ] **Task #18**: Create comprehensive test validation script
  - File: `examples/floats/validate_tests.sh`
  - Implements:
    - Runs `openvm run` on test.elf
    - Captures execution log
    - Verifies all test cases pass
    - Reports which instructions were tested
    - Counts total instructions executed
    - Checks for any unexpected exceptions
  - Script pattern:
    ```bash
    #!/bin/bash
    set -e

    echo "Building tests..."
    ./build.sh

    echo "Running OpenVM with float tests..."
    cargo run --release --bin cargo-openvm -- openvm run \
        --exe build/test.elf \
        --config openvm.toml \
        > test_output.log 2>&1

    echo "Validating results..."
    # Check for success exit code
    grep "Exit code: 0" test_output.log || (echo "FAIL: Non-zero exit"; exit 1)

    # Count instructions tested
    echo "Instructions tested:"
    grep -o "FLW\|FSW\|FADD\|FSUB\|FMUL\|FDIV\|FSQRT\|FMADD\|..." test_output.log | sort | uniq -c

    echo "All tests passed!"
    ```
  - Integration: Automated testing for CI/CD
  - Validation: Ensures implementation correctness

---

## Implementation Workflow

This plan file serves as the authoritative checklist for implementation. When implementing:

### Required Process
1. **Load Plan**: Read this entire plan file before starting
2. **Sync Tasks**: Create TodoWrite tasks matching the checkboxes above
3. **Execute & Update**: For each task:
   - Mark TodoWrite as `in_progress` when starting
   - Update checkbox `[ ]` to `[x]` when completing in this file
   - Mark TodoWrite as `completed` when done
4. **Maintain Sync**: Keep this file and TodoWrite synchronized throughout

### Critical Rules
- This plan file is the source of truth for progress
- Update checkboxes in real-time as work progresses
- Never lose synchronization between plan file and TodoWrite
- Mark tasks complete only when fully implemented (no placeholders)
- **Tasks should be run in parallel using subtasks** to maximize efficiency and avoid context bloat
  - Group A: All 7 transpiler tasks can run in parallel
  - Group B: All 5 executor tasks can run in parallel
  - Group D: All 5 testing tasks can run in parallel with Groups B/C

### Parallelization Strategy
- **Maximum parallelism**: Use subtasks for all tasks within a group
- **Group A (Transpiler)**: 7 parallel subtasks adding decoders
- **Group B (Executors)**: 5 parallel subtasks creating new executors
- **Group C (Integration)**: 2 sequential tasks (must complete after Group B)
- **Group D (Testing)**: 5 parallel subtasks adding test cases (can start after Group A)

### Progress Tracking
The checkboxes above represent the authoritative status of each task. Keep them updated as you work.

---

## Success Criteria

Implementation is complete when:
- [ ] All 32 RV32F instructions are recognized by transpiler
- [ ] All instructions have working executors that JALR to handler
- [ ] test.S includes comprehensive tests for all instruction categories
- [ ] All tests pass with correct IEEE 754 results
- [ ] No compilation warnings or errors
- [ ] Edge cases (NaN, ±∞, ±0) handled correctly
- [ ] Documentation updated (README, code comments)

## Future Extensions

After completing RV32F:
- **RV32D**: Double-precision support (handler already supports it!)
- **RV64F/RV64D**: 64-bit variants (add FCVT.L.S, FCVT.S.L, etc.)
- **Zfinx**: Float in integer registers (different register mapping)
- **Optimization**: Direct SoftFloat calls without JALR overhead
