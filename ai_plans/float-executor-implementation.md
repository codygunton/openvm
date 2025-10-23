# Float Executor Implementation Plan

## Executive Summary

### Problem Statement
The current float implementation at tag `start-executor-approach` uses a transpiler-based approach where each RISC-V float instruction (FLW, FADD.S, etc.) is transpiled into sequences of 9 OpenVM instructions that call a SoftFloat software library via JALR. While functional, this approach has significant overhead:
- **9 VM instructions per float operation** (vs 1 with native executors)
- **Larger proof size** due to expanded instruction traces
- **Complex transpiler logic** to generate JALR calling sequences
- **Cannot execute** because the transpiler extension is not registered in the SDK

### Proposed Solution
Implement the **VmOpcode executor approach** as documented in `VMOPCODE_ARCHITECTURE.md`. This shifts complexity from the transpiler to dedicated executors:
- **1 RISC-V instruction → 1 VM instruction** (simple 1:1 transpilation)
- **Native f32 execution** on the host using Rust's built-in IEEE 754 operations
- **Dedicated executors** for FLW, FSW, and float ALU operations (FADD, FSUB, FMUL, FDIV)
- **Standard OpenVM extension pattern** matching other extensions like JALR, Keccak256, SHA256

### Technical Approach
1. **Define float opcodes** at offset `0x300` (16 opcodes available: 0x300-0x30F)
2. **Create three executor types:**
   - `FloatLoadExecutor` - Handles FLW (float load word)
   - `FloatStoreExecutor` - Handles FSW (float store word)
   - `FloatAluExecutor` - Handles FADD.S, FSUB.S, FMUL.S, FDIV.S
3. **Simplify transpiler** to emit single VM instructions instead of 9-instruction sequences
4. **Register executors** via `VmExecutionExtension` trait
5. **Integrate with SDK** to enable float extension via `[app_vm_config.rv32f]`

### Expected Outcomes
- **Test execution:** `elf_build_run.sh` successfully runs `examples/float-minimal-test/test.S`
- **Correct float computation:** FADD.S correctly computes `1.0 + 2.0 = 3.0` and `2.0 + 3.0 = 5.0`
- **JALR/RET functionality:** Successfully calls and returns from `simple_func`
- **Performance improvement:** 1 instruction trace per float op (vs 9 previously)
- **Foundation for proving:** Execution layer ready (AIR circuits deferred to future work)

## Goals & Objectives

### Primary Goals
- **Enable float instruction execution** via native VM opcodes with dedicated executors
- **Verify correct IEEE 754 semantics** using host f32 operations
- **Successfully execute test suite** at `examples/float-minimal-test/test.S` via `elf_build_run.sh`
- **Reduce instruction overhead** from 9 instructions per float op to 1

### Secondary Objectives
- **Establish executor architecture pattern** for future float operations (FSQRT, FMA, conversions, comparisons)
- **Maintain compatibility** with existing memory layout (`FLOAT_REGISTER_BASE = 0x1F001000`)
- **Clean integration** with SDK configuration system
- **Foundation for future work:** AIR circuits, proving, and RISC-V compliance testing via riscof

## Solution Overview

### Approach
Move float operation logic from transpiler (which generates instruction sequences) to executors (which perform operations directly). The transpiler becomes a simple 1:1 decoder that maps RISC-V float instructions to OpenVM float opcodes, while executors handle the actual IEEE 754 computation using native Rust f32 operations.

### Key Components

1. **Opcode Definitions** (`extensions/floats/transpiler/src/opcodes.rs`)
   - Define `FloatOpcode` enum with variants for FLW, FSW, FADD, FSUB, FMUL, FDIV
   - Use offset `0x300` to avoid collisions with existing opcodes
   - Implement `LocalOpcode` trait for global opcode resolution

2. **Executor Implementations** (`extensions/floats/circuit/src/`)
   - `FloatLoadExecutor`: Reads from heap memory, writes to float register memory
   - `FloatStoreExecutor`: Reads from float register memory, writes to heap memory
   - `FloatAluExecutor`: Performs native f32 operations (add, sub, mul, div)
   - All follow standard executor pattern: `Executor<F>`, `MeteredExecutor<F>`, `PreflightExecutor<F>`

3. **Transpiler Simplification** (`extensions/floats/transpiler/src/lib.rs`)
   - Replace `handle_float_alu()` (9 instructions) with simple opcode emission
   - Replace `handle_flw()` (3 instructions) with single FLW opcode
   - Replace `handle_fsw()` (3 instructions) with single FSW opcode

4. **Extension Registration** (`extensions/floats/circuit/src/extension/mod.rs`)
   - Implement `VmExecutionExtension<F>` trait
   - Register executors in `ExecutorInventoryBuilder`
   - Map opcodes to their corresponding executors

5. **SDK Integration** (`crates/sdk/src/config/global.rs`)
   - Add `rv32f` field to `SdkVmConfig`
   - Register `FloatsTranspilerExtension` in transpiler config
   - Enable extension via `[app_vm_config.rv32f]` in `openvm.toml`

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    RISC-V Program (test.S)                      │
│  flw f0, 0(a0)  │  fadd.s f2, f0, f1  │  jalr ra, t0, 0       │
└────────┬────────────────────┬────────────────────┬──────────────┘
         │                    │                    │
         │ Transpilation (1:1 mapping)             │
         ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                    OpenVM Instructions                          │
│   FLW opcode      │   FADD opcode     │   JALR opcode          │
│   (0x300)         │   (0x301)         │   (existing)           │
└────────┬────────────────────┬────────────────────┬──────────────┘
         │                    │                    │
         │ Executor Dispatch  │                    │
         ▼                    ▼                    ▼
┌──────────────────┐ ┌──────────────────┐ ┌─────────────────────┐
│ FloatLoadExecutor│ │ FloatAluExecutor │ │ Rv32JalrExecutor    │
│ ─────────────────│ │ ─────────────────│ │ ──────────────────  │
│ 1. Read from     │ │ 1. Read f0, f1   │ │ 1. Read target addr │
│    heap[a0+imm]  │ │    from float    │ │ 2. Save return PC   │
│ 2. Write to      │ │    registers     │ │ 3. Jump to target   │
│    float_regs[f0]│ │ 2. f32::add()    │ └─────────────────────┘
└──────────────────┘ │ 3. Write f2      │
                     └──────────────────┘
```

### Data Flow

**Float Register Memory Layout:**
```
Heap Memory (AS=2)
├── 0x1F001000: f0  (float register 0) ─┐
├── 0x1F001004: f1  (float register 1)  │ 32 registers
├── 0x1F001008: f2  (float register 2)  │ × 4 bytes each
├── ...                                  │ = 128 bytes
└── 0x1F00107C: f31 (float register 31)─┘
```

**FLW Execution Flow:**
```
1. RISC-V: flw f0, 0(a0)
   ↓
2. Transpiler: Emit FLW(rd=0, rs1=10, imm=0)
   ↓
3. FloatLoadExecutor:
   - Read address from x10 (a0 register)
   - Load word from heap[address + 0]
   - Write to float_regs[0] (0x1F001000)
```

**FADD Execution Flow:**
```
1. RISC-V: fadd.s f2, f0, f1
   ↓
2. Transpiler: Emit FADD(rd=2, rs1=0, rs2=1)
   ↓
3. FloatAluExecutor:
   - Read f0 bits from 0x1F001000
   - Read f1 bits from 0x1F001004
   - f32::from_bits() + f32::from_bits()
   - Write result to 0x1F001008 (f2)
```

### Expected Outcomes
- **Successful test execution:** `BTEST=1 BBIN=1 ./elf_build_run.sh` completes without errors
- **Correct float results:** f2 = 3.0 (1.0 + 2.0), f0 = 5.0 (2.0 + 3.0)
- **Clean termination:** TERMINATE instruction exits with code 0
- **JALR/RET verified:** Function call and return mechanism works
- **Reduced trace size:** 3 float instructions instead of 27 (3 × 9)

## Implementation Tasks

### CRITICAL IMPLEMENTATION RULES
1. **NO PLACEHOLDER CODE**: Every implementation must be production-ready with complete error handling
2. **CROSS-DIRECTORY TASKS**: Related changes across transpiler/circuit/sdk are grouped in single tasks
3. **COMPLETE IMPLEMENTATIONS**: Each task fully implements its feature including all integration points
4. **DETAILED SPECIFICATIONS**: Follow the exact function signatures and types specified below
5. **BREAKING CHANGES EXPECTED**: This is a major architectural shift - embrace breaking changes

### Visual Dependency Tree

```
extensions/floats/
├── transpiler/src/
│   ├── opcodes.rs (Task #0: Define all float opcodes with offset 0x300)
│   ├── lib.rs (Task #2: Simplify to emit single opcodes instead of sequences)
│   └── Cargo.toml (Task #0: No changes needed, already has dependencies)
│
├── circuit/src/
│   ├── float_load/
│   │   ├── mod.rs (Task #1: FLW executor exports)
│   │   ├── core.rs (Task #1: PreflightExecutor for FLW)
│   │   └── execution.rs (Task #1: Executor/MeteredExecutor for FLW)
│   │
│   ├── float_store/
│   │   ├── mod.rs (Task #1: FSW executor exports)
│   │   ├── core.rs (Task #1: PreflightExecutor for FSW)
│   │   └── execution.rs (Task #1: Executor/MeteredExecutor for FSW)
│   │
│   ├── float_alu/
│   │   ├── mod.rs (Task #1: FADD/FSUB/FMUL/FDIV executor exports)
│   │   ├── core.rs (Task #1: PreflightExecutor for float ALU)
│   │   └── execution.rs (Task #1: Executor/MeteredExecutor for float ALU)
│   │
│   ├── extension/
│   │   └── mod.rs (Task #2: Register all executors in VmExecutionExtension)
│   │
│   └── lib.rs (Task #2: Export executor modules)
│
crates/sdk/src/config/
└── global.rs (Task #2: Add rv32f to SdkVmConfig and register transpiler)

examples/float-minimal-test/
└── test.S (Task #3: Verify test runs - already exists, no changes)

elf_build_run.sh (Task #3: Verify script works - already exists, no changes)
```

### Execution Plan

#### Group A: Foundation (Execute all in parallel)

- [x] **Task #0A**: Define float opcode enum
  - **Folder:** `extensions/floats/transpiler/src/`
  - **File:** `opcodes.rs` (new file)
  - **Imports:**
    ```rust
    use openvm_instructions::LocalOpcode;
    use openvm_instructions_derive::LocalOpcode;
    use serde::{Deserialize, Serialize};
    use strum::{EnumCount, EnumIter, FromRepr};
    ```
  - **Implements:**
    ```rust
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
             EnumCount, EnumIter, FromRepr, LocalOpcode, Serialize, Deserialize)]
    #[opcode_offset = 0x300]
    #[repr(usize)]
    pub enum FloatOpcode {
        FLW = 0,     // Float Load Word (0x300)
        FSW = 1,     // Float Store Word (0x301)
        FADD = 2,    // Float Add (0x302)
        FSUB = 3,    // Float Subtract (0x303)
        FMUL = 4,    // Float Multiply (0x304)
        FDIV = 5,    // Float Divide (0x305)
    }
    ```
  - **Exports:** `pub enum FloatOpcode` and all its methods via LocalOpcode derive
  - **Integration:** Used by transpiler to emit opcodes and by executors for opcode matching
  - **Context:** LocalOpcode trait provides `global_opcode()` method that adds CLASS_OFFSET (0x300)

- [x] **Task #0B**: Create float memory constants module
  - **Folder:** `extensions/floats/circuit/src/`
  - **File:** `constants.rs` (new file)
  - **Imports:**
    ```rust
    // None - pure constants
    ```
  - **Implements:**
    ```rust
    /// Base address for float register file in heap memory
    pub const FLOAT_REGISTER_BASE: u32 = 0x1F001000;

    /// Heap memory address space ID
    pub const FLOAT_MEM_AS: u32 = 2;

    /// RISC-V register address space ID
    pub const RV32_REGISTER_AS: u32 = 1;

    /// Convert float register index (0-31) to memory address
    #[inline]
    pub const fn float_reg_addr(freg: u8) -> u32 {
        FLOAT_REGISTER_BASE + (freg as u32) * 4
    }

    /// Number of limbs per RV32 register
    pub const RV32_REGISTER_NUM_LIMBS: usize = 1;
    ```
  - **Exports:** All constants and `float_reg_addr()` helper
  - **Integration:** Shared by all executors for consistent memory layout
  - **Context:** Must match layout used by current transpiler (already at 0x1F001000)

- [x] **Task #0C**: Update transpiler lib.rs exports
  - **Folder:** `extensions/floats/transpiler/src/`
  - **File:** `lib.rs` (modify existing)
  - **Add to top of file:**
    ```rust
    mod opcodes;
    pub use opcodes::FloatOpcode;
    ```
  - **Exports:** `FloatOpcode` publicly available
  - **Integration:** Allows circuit crate to import opcodes for executor registration
  - **Note:** Don't modify the transpiler logic yet - that's Task #2

#### Group B: Executors (Execute all in parallel after Group A completes)

- [x] **Task #1A**: Implement FloatLoadExecutor (FLW)
  - **Folder:** `extensions/floats/circuit/src/float_load/`
  - **Files:** Create `mod.rs`, `core.rs`, `execution.rs`
  - **Pattern:** Follow `extensions/rv32im/circuit/src/loadstore/` structure
  - **Imports (execution.rs):**
    ```rust
    use std::borrow::BorrowMut;
    use std::mem::size_of;
    use openvm_circuit::arch::*;
    use openvm_circuit::system::memory::online::GuestMemory;
    use openvm_circuit_primitives_derive::AlignedBytesBorrow;
    use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
    use openvm_stark_backend::p3_field::PrimeField32;
    use crate::constants::*;
    use super::core::FloatLoadExecutor;
    ```
  - **PreCompute struct:**
    ```rust
    #[derive(AlignedBytesBorrow, Clone)]
    #[repr(C)]
    struct FloatLoadPreCompute {
        rd: u8,           // Float destination register (0-31)
        rs1: u8,          // Base address register
        _padding: [u8; 2],
        imm: i32,         // Signed offset
    }
    ```
  - **Executor struct (in core.rs):**
    ```rust
    pub struct FloatLoadExecutor;

    impl FloatLoadExecutor {
        pub fn new() -> Self {
            Self
        }
    }
    ```
  - **Core execution logic:**
    ```rust
    unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait>(
        pre_compute: &FloatLoadPreCompute,
        instret: &mut u64,
        pc: &mut u32,
        exec_state: &mut VmExecState<F, GuestMemory, CTX>,
    ) {
        // 1. Read base address from rs1 register
        let base_bytes = exec_state.vm_read::<u8, 4>(
            RV32_REGISTER_AS,
            pre_compute.rs1 as u32 * RV32_REGISTER_NUM_LIMBS as u32
        );
        let base_addr = u32::from_le_bytes(base_bytes);

        // 2. Calculate effective address
        let addr = base_addr.wrapping_add(pre_compute.imm as u32);

        // 3. Load word from heap memory
        let word_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, addr);

        // 4. Write to float register memory
        let float_addr = float_reg_addr(pre_compute.rd);
        exec_state.vm_write(FLOAT_MEM_AS, float_addr, &word_bytes);

        // 5. Update PC and instruction counter
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
    }
    ```
  - **Trait implementations:**
    - `Executor<F>` with `pre_compute()` extracting rd, rs1, imm from instruction
    - `MeteredExecutor<F>` wrapping execution with chip_idx tracking
    - `PreflightExecutor<F, RA>` for trace generation
  - **Exports (mod.rs):**
    ```rust
    mod core;
    mod execution;
    pub use core::FloatLoadExecutor;
    ```
  - **Integration:** Registered in extension for FLW opcode (0x300)
  - **Context:** Reads from heap (user data), writes to float register file

- [x] **Task #1B**: Implement FloatStoreExecutor (FSW)
  - **Folder:** `extensions/floats/circuit/src/float_store/`
  - **Files:** Create `mod.rs`, `core.rs`, `execution.rs`
  - **Pattern:** Mirror FloatLoadExecutor but reverse data flow
  - **Imports (execution.rs):**
    ```rust
    use std::borrow::BorrowMut;
    use std::mem::size_of;
    use openvm_circuit::arch::*;
    use openvm_circuit::system::memory::online::GuestMemory;
    use openvm_circuit_primitives_derive::AlignedBytesBorrow;
    use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
    use openvm_stark_backend::p3_field::PrimeField32;
    use crate::constants::*;
    use super::core::FloatStoreExecutor;
    ```
  - **PreCompute struct:**
    ```rust
    #[derive(AlignedBytesBorrow, Clone)]
    #[repr(C)]
    struct FloatStorePreCompute {
        rs2: u8,          // Float source register (0-31)
        rs1: u8,          // Base address register
        _padding: [u8; 2],
        imm: i32,         // Signed offset
    }
    ```
  - **Core execution logic:**
    ```rust
    unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait>(
        pre_compute: &FloatStorePreCompute,
        instret: &mut u64,
        pc: &mut u32,
        exec_state: &mut VmExecState<F, GuestMemory, CTX>,
    ) {
        // 1. Read from float register memory
        let float_addr = float_reg_addr(pre_compute.rs2);
        let word_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, float_addr);

        // 2. Read base address from rs1 register
        let base_bytes = exec_state.vm_read::<u8, 4>(
            RV32_REGISTER_AS,
            pre_compute.rs1 as u32 * RV32_REGISTER_NUM_LIMBS as u32
        );
        let base_addr = u32::from_le_bytes(base_bytes);

        // 3. Calculate effective address
        let addr = base_addr.wrapping_add(pre_compute.imm as u32);

        // 4. Store word to heap memory
        exec_state.vm_write(FLOAT_MEM_AS, addr, &word_bytes);

        // 5. Update PC and instruction counter
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
    }
    ```
  - **Trait implementations:** Same pattern as FloatLoadExecutor
  - **Exports (mod.rs):**
    ```rust
    mod core;
    mod execution;
    pub use core::FloatStoreExecutor;
    ```
  - **Integration:** Registered in extension for FSW opcode (0x301)
  - **Context:** Reads from float register file, writes to heap (user data)

- [x] **Task #1C**: Implement FloatAluExecutor (FADD, FSUB, FMUL, FDIV)
  - **Folder:** `extensions/floats/circuit/src/float_alu/`
  - **Files:** Create `mod.rs`, `core.rs`, `execution.rs`
  - **Pattern:** Single executor handles all binary float operations
  - **Imports (execution.rs):**
    ```rust
    use std::borrow::BorrowMut;
    use std::mem::size_of;
    use openvm_circuit::arch::*;
    use openvm_circuit::system::memory::online::GuestMemory;
    use openvm_circuit_primitives_derive::AlignedBytesBorrow;
    use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
    use openvm_stark_backend::p3_field::PrimeField32;
    use openvm_floats_transpiler::FloatOpcode;
    use crate::constants::*;
    use super::core::FloatAluExecutor;
    ```
  - **PreCompute struct:**
    ```rust
    #[derive(AlignedBytesBorrow, Clone)]
    #[repr(C)]
    struct FloatAluPreCompute {
        rd: u8,       // Destination float register
        rs1: u8,      // Source float register 1
        rs2: u8,      // Source float register 2
        opcode: u8,   // Which operation (FADD=2, FSUB=3, FMUL=4, FDIV=5)
    }
    ```
  - **Core execution logic:**
    ```rust
    unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait>(
        pre_compute: &FloatAluPreCompute,
        instret: &mut u64,
        pc: &mut u32,
        exec_state: &mut VmExecState<F, GuestMemory, CTX>,
    ) {
        // 1. Read operands from float register memory
        let rs1_addr = float_reg_addr(pre_compute.rs1);
        let rs2_addr = float_reg_addr(pre_compute.rs2);
        let rd_addr = float_reg_addr(pre_compute.rd);

        let rs1_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, rs1_addr);
        let rs2_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, rs2_addr);

        // 2. Convert to f32
        let rs1_f32 = f32::from_bits(u32::from_le_bytes(rs1_bytes));
        let rs2_f32 = f32::from_bits(u32::from_le_bytes(rs2_bytes));

        // 3. Perform operation
        let result_f32 = match pre_compute.opcode {
            2 => rs1_f32 + rs2_f32,  // FADD
            3 => rs1_f32 - rs2_f32,  // FSUB
            4 => rs1_f32 * rs2_f32,  // FMUL
            5 => rs1_f32 / rs2_f32,  // FDIV
            _ => panic!("Invalid float ALU opcode: {}", pre_compute.opcode),
        };

        // 4. Write result back
        let result_bytes = result_f32.to_bits().to_le_bytes();
        exec_state.vm_write(FLOAT_MEM_AS, rd_addr, &result_bytes);

        // 5. Update PC and instruction counter
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
    }
    ```
  - **Trait implementations:**
    - `Executor<F>` extracting rd, rs1, rs2, and opcode from instruction
    - `MeteredExecutor<F>` with chip tracking
    - `PreflightExecutor<F, RA>` for trace generation
  - **Exports (mod.rs):**
    ```rust
    mod core;
    mod execution;
    pub use core::FloatAluExecutor;
    ```
  - **Integration:** Registered in extension for FADD/FSUB/FMUL/FDIV opcodes (0x302-0x305)
  - **Context:** Uses native Rust f32 operations for IEEE 754 arithmetic

#### Group C: Integration (Execute all in parallel after Group B completes)

- [x] **Task #2A**: Simplify transpiler to emit single opcodes
  - **Folder:** `extensions/floats/transpiler/src/`
  - **File:** `lib.rs` (modify existing)
  - **Imports to add:**
    ```rust
    use crate::opcodes::FloatOpcode;
    use openvm_instructions::LocalOpcode;
    ```
  - **Replace `handle_flw()` (lines 39-108) with:**
    ```rust
    fn handle_flw<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let dec = IType::new(inst);

        // Extract operands
        let rd = dec.rd;      // Float destination register
        let rs1 = dec.rs1;    // Base address register
        let imm = dec.imm;    // Offset

        // Emit single FLW instruction
        let instruction = Instruction::from_isize(
            FloatOpcode::FLW.global_opcode(),
            rd as isize,                  // a: float register index
            rs1 as isize,                 // b: base register index
            imm as isize,                 // c: immediate offset
            0,                            // d: unused
            0,                            // e: unused
            0,                            // f: unused
            0,                            // g: unused
        );

        Some(TranspilerOutput {
            instructions: vec![Some(instruction)],
            used_u32s: 1,
        })
    }
    ```
  - **Replace `handle_fsw()` (lines 110-179) with:**
    ```rust
    fn handle_fsw<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let dec = SType::new(inst);

        // Extract operands
        let rs2 = dec.rs2;    // Float source register
        let rs1 = dec.rs1;    // Base address register
        let imm = dec.imm;    // Offset

        // Emit single FSW instruction
        let instruction = Instruction::from_isize(
            FloatOpcode::FSW.global_opcode(),
            rs2 as isize,                 // a: float register index
            rs1 as isize,                 // b: base register index
            imm as isize,                 // c: immediate offset
            0,                            // d: unused
            0,                            // e: unused
            0,                            // f: unused
            0,                            // g: unused
        );

        Some(TranspilerOutput {
            instructions: vec![Some(instruction)],
            used_u32s: 1,
        })
    }
    ```
  - **Replace `handle_float_alu()` (lines 181-357) with:**
    ```rust
    fn handle_float_alu<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        // Decode RISC-V instruction format (R-type)
        let rd = ((inst >> 7) & 0x1F) as u8;
        let rs1 = ((inst >> 15) & 0x1F) as u8;
        let rs2 = ((inst >> 20) & 0x1F) as u8;
        let funct7 = ((inst >> 25) & 0x7F) as u8;

        // Map funct7 to FloatOpcode
        let opcode = match funct7 {
            0x00 => FloatOpcode::FADD,   // FADD.S
            0x04 => FloatOpcode::FSUB,   // FSUB.S
            0x08 => FloatOpcode::FMUL,   // FMUL.S
            0x0C => FloatOpcode::FDIV,   // FDIV.S
            _ => return None,  // Unsupported operation
        };

        // Emit single float ALU instruction
        let instruction = Instruction::from_isize(
            opcode.global_opcode(),
            rd as isize,                  // a: destination float register
            rs1 as isize,                 // b: source float register 1
            rs2 as isize,                 // c: source float register 2
            opcode as usize as isize,     // d: opcode for executor dispatch
            0,                            // e: unused
            0,                            // f: unused
            0,                            // g: unused
        );

        Some(TranspilerOutput {
            instructions: vec![Some(instruction)],
            used_u32s: 1,
        })
    }
    ```
  - **Remove debug eprintln!() statements** throughout the file
  - **Integration:** Transpiler now emits 1 instruction per RISC-V float op instead of 9
  - **Context:** This is the key simplification - complexity moves to executors

- [x] **Task #2B**: Register executors in VmExecutionExtension
  - **Folder:** `extensions/floats/circuit/src/extension/`
  - **File:** `mod.rs` (replace existing stub)
  - **Imports:**
    ```rust
    use derive_more::From;
    use openvm_circuit::arch::*;
    use openvm_circuit_derive::{AnyEnum, Executor, MeteredExecutor, PreflightExecutor};
    use openvm_stark_backend::p3_field::PrimeField32;
    use openvm_floats_transpiler::FloatOpcode;
    use strum::IntoEnumIterator;

    use crate::float_load::FloatLoadExecutor;
    use crate::float_store::FloatStoreExecutor;
    use crate::float_alu::FloatAluExecutor;
    ```
  - **Executor enum:**
    ```rust
    #[derive(Clone, From, AnyEnum, Executor, MeteredExecutor, PreflightExecutor)]
    pub enum Rv32FExecutor {
        FloatLoad(FloatLoadExecutor),
        FloatStore(FloatStoreExecutor),
        FloatAlu(FloatAluExecutor),
    }
    ```
  - **Extension struct:**
    ```rust
    #[derive(Clone, Debug, Default)]
    pub struct Rv32F;
    ```
  - **VmExecutionExtension implementation:**
    ```rust
    impl<F: PrimeField32> VmExecutionExtension<F> for Rv32F {
        type Executor = Rv32FExecutor;

        fn extend_execution(
            &self,
            inventory: &mut ExecutorInventoryBuilder<F, Rv32FExecutor>,
        ) -> Result<(), ExecutorInventoryError> {
            // Register FLW executor
            inventory.add_executor(
                FloatLoadExecutor::new(),
                [FloatOpcode::FLW.global_opcode()],
            )?;

            // Register FSW executor
            inventory.add_executor(
                FloatStoreExecutor::new(),
                [FloatOpcode::FSW.global_opcode()],
            )?;

            // Register float ALU executor for all arithmetic ops
            inventory.add_executor(
                FloatAluExecutor::new(),
                [
                    FloatOpcode::FADD.global_opcode(),
                    FloatOpcode::FSUB.global_opcode(),
                    FloatOpcode::FMUL.global_opcode(),
                    FloatOpcode::FDIV.global_opcode(),
                ],
            )?;

            Ok(())
        }
    }
    ```
  - **Exports:** `pub use Rv32F`, `pub use Rv32FExecutor`
  - **Integration:** Called by SDK to register executors in inventory
  - **Context:** Maps opcodes to executors for runtime dispatch

- [x] **Task #2C**: Update circuit lib.rs exports
  - **Folder:** `extensions/floats/circuit/src/`
  - **File:** `lib.rs` (modify existing)
  - **Add module declarations:**
    ```rust
    mod constants;
    mod float_load;
    mod float_store;
    mod float_alu;
    pub mod extension;

    pub use constants::*;
    pub use extension::{Rv32F, Rv32FExecutor};
    ```
  - **Exports:** Make executors and extension publicly available
  - **Integration:** Allows SDK to import and use Rv32F extension
  - **Context:** Standard pattern for OpenVM extensions

- [x] **Task #2D**: Integrate with SDK configuration
  - **Folder:** `crates/sdk/src/config/`
  - **File:** `global.rs` (modify existing)
  - **Step 1: Add imports (around line 20):**
    ```rust
    use openvm_floats_circuit::Rv32F;
    use openvm_floats_transpiler::FloatsTranspilerExtension;
    ```
  - **Step 2: Add rv32f field to SdkVmConfig (around line 90):**
    ```rust
    #[derive(Builder, Clone, Debug, Serialize, Deserialize)]
    #[builder(setter(strip_option))]
    pub struct SdkVmConfig {
        pub system: SdkSystemConfig,
        pub rv32i: Option<UnitStruct>,
        pub rv32m: Option<Rv32M>,
        pub rv32f: Option<UnitStruct>,  // ADD THIS LINE
        // ... rest of fields
    }
    ```
  - **Step 3: Register transpiler extension (around line 210):**
    ```rust
    impl<F: PrimeField32> TranspilerConfig<F> for SdkVmConfig {
        fn transpiler(&self) -> Transpiler<F> {
            let mut transpiler = Transpiler::default();
            if self.rv32i.is_some() {
                transpiler = transpiler.with_extension(Rv32ITranspilerExtension);
            }
            if self.rv32m.is_some() {
                transpiler = transpiler.with_extension(Rv32MTranspilerExtension);
            }
            if self.rv32f.is_some() {  // ADD THIS BLOCK
                transpiler = transpiler.with_extension(FloatsTranspilerExtension);
            }
            // ... rest of extensions
            transpiler
        }
    }
    ```
  - **Step 4: Add to SdkVmConfigInner (around line 320):**
    ```rust
    #[derive(Clone, Debug, VmConfig, Serialize, Deserialize)]
    pub struct SdkVmConfigInner {
        #[extension(executor = "Rv32IExecutor")]
        pub rv32i: Option<Rv32I>,
        #[extension(executor = "Rv32MExecutor")]
        pub rv32m: Option<Rv32M>,
        #[extension(executor = "Rv32FExecutor")]  // ADD THIS LINE
        pub rv32f: Option<Rv32F>,
        // ... rest of extensions
    }
    ```
  - **Step 5: Add to SdkVmConfig::deduce_from (around line 380):**
    ```rust
    fn deduce_from<T>(config: &T) -> Self
    where
        T: VmConfig<VmInventory>,
    {
        let rv32i = config.extension::<Rv32I>();
        let rv32m = config.extension::<Rv32M>();
        let rv32f = config.extension::<Rv32F>();  // ADD THIS LINE
        // ... rest of extensions
        Self {
            rv32i: rv32i.cloned(),
            rv32m: rv32m.cloned(),
            rv32f: rv32f.cloned(),  // ADD THIS LINE
            // ... rest of fields
        }
    }
    ```
  - **Integration:** Enables `[app_vm_config.rv32f]` in openvm.toml
  - **Context:** Standard SDK integration pattern for extensions

#### Group D: Testing (Execute sequentially after Group C completes)

- [x] **Task #3**: Verify test execution with elf_build_run.sh
  - **Folder:** Project root
  - **File:** `elf_build_run.sh` (no changes - verify only)
  - **Prerequisites:** All previous tasks completed
  - **Steps to execute:**
    1. Build the test binary:
       ```bash
       BTEST=1 ./elf_build_run.sh
       ```
    2. Build cargo-openvm:
       ```bash
       BBIN=1 ./elf_build_run.sh
       ```
    3. Run the test:
       ```bash
       ./elf_build_run.sh
       ```
    4. Check the log file:
       ```bash
       cat examples/floats/elf.log
       ```
  - **Expected output:**
    - No transpiler errors
    - FLW instructions execute successfully
    - FADD instructions execute successfully
    - JALR and RET work correctly
    - Program terminates with exit code 0
  - **Success criteria:**
    - Log shows: "Program exited with code 0"
    - No "couldn't parse instruction" errors
    - No panic or execution errors
    - Float register f2 contains 3.0 (0x40400000 in IEEE 754)
    - Float register f0 contains 5.0 (0x40A00000 in IEEE 754)
  - **Debugging:** If errors occur:
    - Check transpiler registered: grep for FloatsTranspilerExtension in logs
    - Check executors registered: grep for "FloatLoad" in logs
    - Verify opcode offset: should see 0x300-0x305 in instruction stream
  - **Integration:** End-to-end verification that executor approach works
  - **Context:** This validates the entire implementation stack

---

## Implementation Workflow

This plan file serves as the authoritative checklist for implementation. When implementing:

### Required Process
1. **Load Plan**: Read this entire plan file before starting
2. **Sync Tasks**: Create TodoWrite tasks matching the checkboxes above
3. **Execute & Update**: For each task:
   - Mark TodoWrite as `in_progress` when starting
   - Update checkbox `[ ]` to `[x]` when completing
   - Mark TodoWrite as `completed` when done
4. **Maintain Sync**: Keep this file and TodoWrite synchronized throughout

### Critical Rules
- This plan file is the source of truth for progress
- Update checkboxes in real-time as work progresses
- Never lose synchronization between plan file and TodoWrite
- Mark tasks complete only when fully implemented (no placeholders)
- Tasks within the same group should be run in parallel using subtasks to avoid context bloat

### Parallelization Strategy
- **Group A** (3 tasks): All run in parallel - pure definitions, no dependencies
- **Group B** (3 tasks): All run in parallel after Group A - executors are independent
- **Group C** (4 tasks): All run in parallel after Group B - integration points are orthogonal
- **Group D** (1 task): Sequential after Group C - end-to-end verification

### Progress Tracking
The checkboxes above represent the authoritative status of each task. Keep them updated as you work.

---

## Notes and Considerations

### Future Work (Out of Scope)
- **AIR circuits:** Currently execution-only, no proving capability
- **Additional float ops:** FSQRT, FCMP, FCVT, FMA, etc.
- **RISC-V compliance:** Full riscof test suite verification
- **Optimization:** SIMD, CUDA acceleration for proving

### Design Decisions
- **Native f32 operations:** Use Rust's built-in IEEE 754 instead of SoftFloat library
- **Single ALU executor:** One executor handles all binary ops (FADD/FSUB/FMUL/FDIV) via opcode dispatch
- **Memory layout preserved:** Keep existing 0x1F001000 base for compatibility
- **No AIR constraints yet:** Focus on execution layer first, defer proving infrastructure

### Testing Strategy
- **Phase 1:** Basic functionality (this plan) - elf_build_run.sh
- **Phase 2:** Extended testing (future) - riscof architecture tests
- **Phase 3:** Compliance (future) - Full RV32F compliance suite
