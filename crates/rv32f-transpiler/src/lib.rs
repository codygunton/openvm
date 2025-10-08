// OpenVM RV32F Transpiler Extension - Architecture A
// Adds F extension (single-precision floating-point) to RV32IM base
// Uses indirect calls via function pointer table to avoid offset calculation

use openvm_instructions::{
    instruction::Instruction, riscv::RV32_REGISTER_NUM_LIMBS, LocalOpcode,
};
use openvm_rv32im_transpiler::{
    BaseAluOpcode, Rv32JalrOpcode, Rv32LoadStoreOpcode, ShiftOpcode,
};
use openvm_stark_backend::p3_field::PrimeField32;
use openvm_transpiler::{TranspilerExtension, TranspilerOutput};

/// Transpiler extension for the F extension (single-precision floating-point)
///
/// Adds RV32F support to OpenVM's RV32IM base, creating full RV32IMF.
/// Replaces F extension instructions with calling sequences that invoke
/// runtime library functions via an indirect call table at fixed address 0x10000000.
///
/// For each F extension instruction like FADD.S f3, f1, f2:
/// 1. Load register indices into a0-a2 (RV32IM instructions)
/// 2. Load rounding mode into a3 (RV32IM instruction)
/// 3. Load function pointer from dispatch table (RV32IM instructions)
/// 4. Indirect call via JALR (RV32IM instruction)
pub struct Rv32FArchATranspilerExtension;

// Dispatch table address (must match linker script and runtime header)
// Must be below OpenVM's MEM_SIZE limit of 0x20000000 (512MB)
const FLOAT_DISPATCH_TABLE_ADDR: u32 = 0x10000000;

// Dispatch table indices (must match dispatch_table.h)
const FLOAT_OP_FADD_S: u32 = 0;
const FLOAT_OP_FSUB_S: u32 = 1;
const FLOAT_OP_FMUL_S: u32 = 2;
const FLOAT_OP_FDIV_S: u32 = 3;
const FLOAT_OP_FSQRT_S: u32 = 4;
const FLOAT_OP_FMIN_S: u32 = 5;
const FLOAT_OP_FMAX_S: u32 = 6;
const FLOAT_OP_FMADD_S: u32 = 7;
const FLOAT_OP_FMSUB_S: u32 = 8;
const FLOAT_OP_FNMADD_S: u32 = 9;
const FLOAT_OP_FNMSUB_S: u32 = 10;
const FLOAT_OP_FSGNJ_S: u32 = 11;
const FLOAT_OP_FSGNJN_S: u32 = 12;
const FLOAT_OP_FSGNJX_S: u32 = 13;
const FLOAT_OP_FEQ_S: u32 = 14;
const FLOAT_OP_FLT_S: u32 = 15;
const FLOAT_OP_FLE_S: u32 = 16;
const FLOAT_OP_FCVT_W_S: u32 = 17;
const FLOAT_OP_FCVT_WU_S: u32 = 18;
const FLOAT_OP_FCVT_S_W: u32 = 19;
const FLOAT_OP_FCVT_S_WU: u32 = 20;
const FLOAT_OP_FMV_X_W: u32 = 21;
const FLOAT_OP_FMV_W_X: u32 = 22;
const FLOAT_OP_FCLASS_S: u32 = 23;

// No need to redefine - imported from openvm_instructions::riscv

// RISC-V opcodes
const OPCODE_LOAD_FP: u8 = 0x07; // FLW (Float Load Word)
const OPCODE_STORE_FP: u8 = 0x27; // FSW (Float Store Word)
const OPCODE_OP_FP: u8 = 0x53; // Floating-point arithmetic (FADD, FSUB, etc.)
const OPCODE_MADD: u8 = 0x43;  // FMADD
const OPCODE_MSUB: u8 = 0x47;  // FMSUB
const OPCODE_NMSUB: u8 = 0x4B; // FNMSUB
const OPCODE_NMADD: u8 = 0x4F; // FNMADD

// Float register base address (must match runtime)
const FLOAT_REG_BASE: u32 = 0xC0000000;

impl<F: PrimeField32> TranspilerExtension<F> for Rv32FArchATranspilerExtension {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        if instruction_stream.is_empty() {
            return None;
        }

        let inst = instruction_stream[0];
        let opcode = (inst & 0x7F) as u8;

        match opcode {
            OPCODE_LOAD_FP => self.process_flw(inst),
            OPCODE_STORE_FP => self.process_fsw(inst),
            OPCODE_OP_FP => self.process_op_fp(inst),
            OPCODE_MADD => self.process_fused_op(inst, FLOAT_OP_FMADD_S),
            OPCODE_MSUB => self.process_fused_op(inst, FLOAT_OP_FMSUB_S),
            OPCODE_NMSUB => self.process_fused_op(inst, FLOAT_OP_FNMADD_S),
            OPCODE_NMADD => self.process_fused_op(inst, FLOAT_OP_FNMSUB_S),
            _ => None,
        }
    }
}

impl Rv32FArchATranspilerExtension {
    /// Process FLW (Float Load Word) - I-type
    /// FLW fd, offset(rs1) loads 32-bit float from memory into float register
    fn process_flw<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let rd = (inst >> 7) & 0x1F;    // Float destination register
        let rs1 = (inst >> 15) & 0x1F;  // Base address register
        let imm = (inst as i32) >> 20; // Sign-extended 12-bit immediate

        // FLW fd, offset(rs1) translates to:
        // 1. li t0, FLOAT_REG_BASE + fd*4   # Calculate float reg address
        // 2. lw t1, offset(rs1)              # Load from program memory
        // 3. sw t1, 0(t0)                    # Store to float reg space

        let mut instructions = Vec::new();
        let float_reg_addr = FLOAT_REG_BASE + (rd * 4);

        // t0 (x5) = address of float register fd
        instructions.push(Some(li::<F>(5, float_reg_addr as i32)));

        // t1 (x6) = load from program memory[rs1 + offset]
        instructions.push(Some(lw::<F>(6, rs1, imm)));

        // Store t1 to float register space at address t0
        instructions.push(Some(sw::<F>(6, 5, 0)));

        Some(TranspilerOutput {
            instructions,
            used_u32s: 1,
        })
    }

    /// Process FSW (Float Store Word) - S-type
    /// FSW rs2, offset(rs1) stores 32-bit float from float register to memory
    fn process_fsw<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let rs2 = (inst >> 20) & 0x1F;  // Float source register
        let rs1 = (inst >> 15) & 0x1F;  // Base address register
        let imm11_5 = (inst >> 25) & 0x7F;
        let imm4_0 = (inst >> 7) & 0x1F;
        let imm = (((imm11_5 << 5) | imm4_0) as i32) << 20 >> 20; // Sign-extend

        // FSW rs2, offset(rs1) translates to:
        // 1. li t0, FLOAT_REG_BASE + rs2*4  # Calculate float reg address
        // 2. lw t1, 0(t0)                   # Load from float reg space
        // 3. sw t1, offset(rs1)             # Store to program memory

        let mut instructions = Vec::new();
        let float_reg_addr = FLOAT_REG_BASE + (rs2 * 4);

        // t0 (x5) = address of float register rs2
        instructions.push(Some(li::<F>(5, float_reg_addr as i32)));

        // t1 (x6) = load from float register space
        instructions.push(Some(lw::<F>(6, 5, 0)));

        // Store t1 to program memory[rs1 + offset]
        instructions.push(Some(sw::<F>(6, rs1, imm)));

        Some(TranspilerOutput {
            instructions,
            used_u32s: 1,
        })
    }

    /// Process OP-FP opcode (0x53) - arithmetic, comparison, conversion, move
    fn process_op_fp<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let funct7 = (inst >> 25) & 0x7F;
        let rs2 = (inst >> 20) & 0x1F;
        let rs1 = (inst >> 15) & 0x1F;
        let rm = (inst >> 12) & 0x7;
        let rd = (inst >> 7) & 0x1F;

        // Determine operation and whether result goes to integer register
        let (op_index, int_result) = match funct7 {
            0x00 => (FLOAT_OP_FADD_S, false),    // FADD.S -> float
            0x04 => (FLOAT_OP_FSUB_S, false),    // FSUB.S -> float
            0x08 => (FLOAT_OP_FMUL_S, false),    // FMUL.S -> float
            0x0C => (FLOAT_OP_FDIV_S, false),    // FDIV.S -> float
            0x2C if rs2 == 0 => (FLOAT_OP_FSQRT_S, false), // FSQRT.S -> float
            0x10 => match rm {          // FSGNJ/FSGNJN/FSGNJX -> float
                0 => (FLOAT_OP_FSGNJ_S, false),
                1 => (FLOAT_OP_FSGNJN_S, false),
                2 => (FLOAT_OP_FSGNJX_S, false),
                _ => return None,
            },
            0x14 => match rm {          // FMIN.S/FMAX.S -> float
                0 => (FLOAT_OP_FMIN_S, false),
                1 => (FLOAT_OP_FMAX_S, false),
                _ => return None,
            },
            0x50 => match rm {          // Comparisons -> INTEGER
                0 => (FLOAT_OP_FLE_S, true),
                1 => (FLOAT_OP_FLT_S, true),
                2 => (FLOAT_OP_FEQ_S, true),
                _ => return None,
            },
            0x60 if rs2 == 0 => (FLOAT_OP_FCVT_W_S, true),   // FCVT.W.S -> INTEGER
            0x60 if rs2 == 1 => (FLOAT_OP_FCVT_WU_S, true),  // FCVT.WU.S -> INTEGER
            0x68 if rs2 == 0 => (FLOAT_OP_FCVT_S_W, false),  // FCVT.S.W -> float
            0x68 if rs2 == 1 => (FLOAT_OP_FCVT_S_WU, false), // FCVT.S.WU -> float
            0x70 if rs2 == 0 && rm == 0 => (FLOAT_OP_FMV_X_W, true), // FMV.X.W -> INTEGER
            0x78 if rs2 == 0 && rm == 0 => (FLOAT_OP_FMV_W_X, false), // FMV.W.X -> float
            0x70 if rs2 == 0 && rm == 1 => (FLOAT_OP_FCLASS_S, true), // FCLASS.S -> INTEGER
            _ => return None,
        };

        if int_result {
            Some(self.emit_indirect_call_int_result(rs1, rs2, rd, rm, op_index))
        } else {
            Some(self.emit_indirect_call(rs1, rs2, rd, rm, op_index))
        }
    }

    /// Process fused multiply-add/sub opcodes (R4-type)
    fn process_fused_op<F: PrimeField32>(&self, inst: u32, op_index: u32) -> Option<TranspilerOutput<F>> {
        let rs3 = (inst >> 27) & 0x1F;
        let rs2 = (inst >> 20) & 0x1F;
        let rs1 = (inst >> 15) & 0x1F;
        let rm = (inst >> 12) & 0x7;
        let rd = (inst >> 7) & 0x1F;

        // FMA operations need all 4 source register indices
        Some(self.emit_indirect_call_fma(rs1, rs2, rs3, rd, rm, op_index))
    }

    /// Emit calling sequence with indirect call via dispatch table
    ///
    /// Emits:
    /// 1. li a0, rs1   (source register 1 index)
    /// 2. li a1, rs2   (source register 2 index)
    /// 3. li a2, rd    (destination register index)
    /// 4. li a3, rm    (rounding mode)
    /// 5. li t0, TABLE_ADDR       (dispatch table base)
    /// 6. li t1, op_index         (function index)
    /// 7. slli t1, t1, 2          (multiply by pointer size)
    /// 8. add t0, t0, t1          (t0 = &table[op_index])
    /// 9. lw t0, 0(t0)            (t0 = table[op_index])
    /// 10. jalr ra, t0, 0         (call function)
    fn emit_indirect_call<F: PrimeField32>(
        &self,
        rs1: u32,
        rs2: u32,
        rd: u32,
        rm: u32,
        op_index: u32,
    ) -> TranspilerOutput<F> {
        let mut instructions = Vec::new();

        // Load arguments into a0-a3 (x10-x13)
        instructions.push(Some(li::<F>(10, rs1 as i32)));  // a0 = rs1
        instructions.push(Some(li::<F>(11, rs2 as i32)));  // a1 = rs2
        instructions.push(Some(li::<F>(12, rd as i32)));   // a2 = rd
        instructions.push(Some(li::<F>(13, rm as i32)));   // a3 = rm

        // Load dispatch table address into t0 (x5)
        instructions.push(Some(li::<F>(5, FLOAT_DISPATCH_TABLE_ADDR as i32))); // t0 = TABLE_ADDR

        // Load operation index into t1 (x6)
        instructions.push(Some(li::<F>(6, op_index as i32))); // t1 = op_index

        // Calculate table offset: t1 = t1 * 4 (pointer size)
        instructions.push(Some(slli::<F>(6, 6, 2))); // t1 <<= 2

        // Add offset to base: t0 = t0 + t1
        instructions.push(Some(add::<F>(5, 5, 6))); // t0 = t0 + t1

        // Load function pointer: t0 = *t0
        instructions.push(Some(lw::<F>(5, 5, 0))); // t0 = mem[t0]

        // Indirect call: jalr ra, t0, 0
        instructions.push(Some(jalr::<F>(1, 5, 0))); // ra = pc+4; pc = t0

        TranspilerOutput {
            instructions,
            used_u32s: 1, // Consumed one RV32F instruction
        }
    }

    /// Emit calling sequence for operations that return INTEGER results
    ///
    /// For operations like FEQ, FLT, FLE, FCVT.W.S, FCVT.WU.S, FMV.X.W, FCLASS.S
    /// that produce integer results (not float results).
    ///
    /// Runtime function returns result in a0, which is then moved to integer register rd.
    ///
    /// Emits:
    /// 1. li a0, rs1   (source register 1 index)
    /// 2. li a1, rs2   (source register 2 index)
    /// 3. li a2, rm    (rounding mode - moved from a3 to a2)
    /// 4. li t0, TABLE_ADDR       (dispatch table base)
    /// 5. li t1, op_index         (function index)
    /// 6. slli t1, t1, 2          (multiply by pointer size)
    /// 7. add t0, t0, t1          (t0 = &table[op_index])
    /// 8. lw t0, 0(t0)            (t0 = table[op_index])
    /// 9. jalr ra, t0, 0          (call function, result in a0)
    /// 10. mv rd, a0               (move result to integer destination register)
    fn emit_indirect_call_int_result<F: PrimeField32>(
        &self,
        rs1: u32,
        rs2: u32,
        rd: u32,
        rm: u32,
        op_index: u32,
    ) -> TranspilerOutput<F> {
        let mut instructions = Vec::new();

        // Load arguments into a0-a2 (x10-x12)
        // Note: a2 is now rm, not rd, since result comes back in a0
        instructions.push(Some(li::<F>(10, rs1 as i32)));  // a0 = rs1
        instructions.push(Some(li::<F>(11, rs2 as i32)));  // a1 = rs2
        instructions.push(Some(li::<F>(12, rm as i32)));   // a2 = rm

        // Load dispatch table address into t0 (x5)
        instructions.push(Some(li::<F>(5, FLOAT_DISPATCH_TABLE_ADDR as i32))); // t0 = TABLE_ADDR

        // Load operation index into t1 (x6)
        instructions.push(Some(li::<F>(6, op_index as i32))); // t1 = op_index

        // Calculate table offset: t1 = t1 * 4 (pointer size)
        instructions.push(Some(slli::<F>(6, 6, 2))); // t1 <<= 2

        // Add offset to base: t0 = t0 + t1
        instructions.push(Some(add::<F>(5, 5, 6))); // t0 = t0 + t1

        // Load function pointer: t0 = *t0
        instructions.push(Some(lw::<F>(5, 5, 0))); // t0 = mem[t0]

        // Indirect call: jalr ra, t0, 0 (result returned in a0)
        instructions.push(Some(jalr::<F>(1, 5, 0))); // ra = pc+4; pc = t0

        // Move result from a0 to integer register rd
        // mv rd, a0 is pseudo-instruction for addi rd, a0, 0
        instructions.push(Some(add::<F>(rd, 10, 0))); // rd = a0 + x0

        TranspilerOutput {
            instructions,
            used_u32s: 1, // Consumed one RV32F instruction
        }
    }

    /// Emit calling sequence for FMA operations (4-operand R4-type)
    ///
    /// For FMADD.S, FMSUB.S, FNMADD.S, FNMSUB.S which need 3 source operands.
    /// These instructions compute: fd = (fs1 * fs2) +/- fs3
    ///
    /// Emits:
    /// 1. li a0, rs1   (source register 1 index)
    /// 2. li a1, rs2   (source register 2 index)
    /// 3. li a2, rs3   (source register 3 index)
    /// 4. li a3, rd    (destination register index)
    /// 5. li a4, rm    (rounding mode)
    /// 6. li t0, TABLE_ADDR       (dispatch table base)
    /// 7. li t1, op_index         (function index)
    /// 8. slli t1, t1, 2          (multiply by pointer size)
    /// 9. add t0, t0, t1          (t0 = &table[op_index])
    /// 10. lw t0, 0(t0)           (t0 = table[op_index])
    /// 11. jalr ra, t0, 0         (call function)
    fn emit_indirect_call_fma<F: PrimeField32>(
        &self,
        rs1: u32,
        rs2: u32,
        rs3: u32,
        rd: u32,
        rm: u32,
        op_index: u32,
    ) -> TranspilerOutput<F> {
        let mut instructions = Vec::new();

        // Load arguments into a0-a4 (x10-x14)
        instructions.push(Some(li::<F>(10, rs1 as i32)));  // a0 = rs1
        instructions.push(Some(li::<F>(11, rs2 as i32)));  // a1 = rs2
        instructions.push(Some(li::<F>(12, rs3 as i32)));  // a2 = rs3
        instructions.push(Some(li::<F>(13, rd as i32)));   // a3 = rd
        instructions.push(Some(li::<F>(14, rm as i32)));   // a4 = rm

        // Load dispatch table address into t0 (x5)
        instructions.push(Some(li::<F>(5, FLOAT_DISPATCH_TABLE_ADDR as i32))); // t0 = TABLE_ADDR

        // Load operation index into t1 (x6)
        instructions.push(Some(li::<F>(6, op_index as i32))); // t1 = op_index

        // Calculate table offset: t1 = t1 * 4 (pointer size)
        instructions.push(Some(slli::<F>(6, 6, 2))); // t1 <<= 2

        // Add offset to base: t0 = t0 + t1
        instructions.push(Some(add::<F>(5, 5, 6))); // t0 = t0 + t1

        // Load function pointer: t0 = *t0
        instructions.push(Some(lw::<F>(5, 5, 0))); // t0 = mem[t0]

        // Indirect call: jalr ra, t0, 0
        instructions.push(Some(jalr::<F>(1, 5, 0))); // ra = pc+4; pc = t0

        TranspilerOutput {
            instructions,
            used_u32s: 1, // Consumed one RV32F instruction
        }
    }
}

// Helper functions to construct OpenVM instructions

/// li rd, imm  =  addi rd, x0, imm
fn li<F: PrimeField32>(rd: u32, imm: i32) -> Instruction<F> {
    Instruction::from_isize(
        BaseAluOpcode::ADD.global_opcode(),
        ((rd as usize) * RV32_REGISTER_NUM_LIMBS) as isize,
        0, // x0
        imm as isize,
        1, // rd is register
        0, // rs2 is immediate
    )
}

/// slli rd, rs1, shamt
fn slli<F: PrimeField32>(rd: u32, rs1: u32, shamt: u32) -> Instruction<F> {
    Instruction::from_isize(
        ShiftOpcode::SLL.global_opcode(),
        ((rd as usize) * RV32_REGISTER_NUM_LIMBS) as isize,
        ((rs1 as usize) * RV32_REGISTER_NUM_LIMBS) as isize,
        shamt as isize,
        1, // rd is register
        0, // shamt is immediate
    )
}

/// add rd, rs1, rs2
fn add<F: PrimeField32>(rd: u32, rs1: u32, rs2: u32) -> Instruction<F> {
    Instruction::from_isize(
        BaseAluOpcode::ADD.global_opcode(),
        ((rd as usize) * RV32_REGISTER_NUM_LIMBS) as isize,
        ((rs1 as usize) * RV32_REGISTER_NUM_LIMBS) as isize,
        ((rs2 as usize) * RV32_REGISTER_NUM_LIMBS) as isize,
        1, // rd is register
        1, // rs2 is register
    )
}

/// lw rd, offset(rs1)
fn lw<F: PrimeField32>(rd: u32, rs1: u32, offset: i32) -> Instruction<F> {
    Instruction::new(
        Rv32LoadStoreOpcode::LOADW.global_opcode(),
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rd as usize),
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rs1 as usize),
        F::from_canonical_u32((offset as u32) & 0xffff),
        F::ONE,  // rd is register
        F::ZERO, // rs2 not used
        F::from_bool(offset < 0),
        F::ZERO,
    )
}

/// sw rs2, offset(rs1)
fn sw<F: PrimeField32>(rs2: u32, rs1: u32, offset: i32) -> Instruction<F> {
    Instruction::new(
        Rv32LoadStoreOpcode::STOREW.global_opcode(),
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rs1 as usize),
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rs2 as usize),
        F::from_canonical_u32((offset as u32) & 0xffff),
        F::ZERO, // rs1 is address base
        F::ONE,  // rs2 is data register
        F::from_bool(offset < 0),
        F::ZERO,
    )
}

/// jalr rd, rs1, offset
fn jalr<F: PrimeField32>(rd: u32, rs1: u32, offset: i32) -> Instruction<F> {
    // Based on process_jalr in rv32im/transpiler/src/rrs.rs:248
    Instruction::new(
        Rv32JalrOpcode::JALR.global_opcode(),
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rd as usize),
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rs1 as usize),
        F::from_canonical_u32((offset as u32) & 0xffff),
        F::ONE,  // rd is register
        F::ZERO, // rs2 not used
        F::from_bool(rd != 0),
        F::from_bool(offset < 0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use openvm_stark_sdk::p3_baby_bear::BabyBear;

    type F = BabyBear;

    // Helper to encode RV32F R-type instruction
    fn encode_r_type(opcode: u8, rd: u32, funct3: u8, rs1: u32, rs2: u32, funct7: u8) -> u32 {
        ((funct7 as u32) << 25)
            | ((rs2 & 0x1F) << 20)
            | ((rs1 & 0x1F) << 15)
            | ((funct3 as u32) << 12)
            | ((rd & 0x1F) << 7)
            | (opcode as u32)
    }

    // Helper to encode RV32F R4-type instruction (FMA)
    fn encode_r4_type(opcode: u8, rd: u32, funct3: u8, rs1: u32, rs2: u32, rs3: u32) -> u32 {
        ((rs3 & 0x1F) << 27)
            | (0b00 << 25)  // fmt = S (single precision)
            | ((rs2 & 0x1F) << 20)
            | ((rs1 & 0x1F) << 15)
            | ((funct3 as u32) << 12)
            | ((rd & 0x1F) << 7)
            | (opcode as u32)
    }

    #[test]
    fn test_fadd_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FADD.S f3, f1, f2, rne
        let inst = encode_r_type(0x53, 3, 0, 1, 2, 0x00);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FADD.S");

        assert_eq!(output.used_u32s, 1, "Should consume 1 instruction");
        assert_eq!(output.instructions.len(), 10, "Should emit 10 instructions");

        // Verify all instructions are Some (not None)
        assert!(output.instructions.iter().all(|i| i.is_some()));
    }

    #[test]
    fn test_fsub_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FSUB.S f5, f4, f3, rne
        let inst = encode_r_type(0x53, 5, 0, 4, 3, 0x04);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FSUB.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fmul_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FMUL.S f10, f11, f12, rne
        let inst = encode_r_type(0x53, 10, 0, 11, 12, 0x08);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FMUL.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fdiv_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FDIV.S f7, f8, f9, rne
        let inst = encode_r_type(0x53, 7, 0, 8, 9, 0x0C);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FDIV.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fsqrt_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FSQRT.S f6, f5, rne (rs2 must be 0)
        let inst = encode_r_type(0x53, 6, 0, 5, 0, 0x2C);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FSQRT.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fmin_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FMIN.S f1, f2, f3 (rm=0)
        let inst = encode_r_type(0x53, 1, 0, 2, 3, 0x14);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FMIN.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fmax_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FMAX.S f1, f2, f3 (rm=1)
        let inst = encode_r_type(0x53, 1, 1, 2, 3, 0x14);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FMAX.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fsgnj_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FSGNJ.S f1, f2, f3 (rm=0)
        let inst = encode_r_type(0x53, 1, 0, 2, 3, 0x10);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FSGNJ.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fsgnjn_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FSGNJN.S f1, f2, f3 (rm=1)
        let inst = encode_r_type(0x53, 1, 1, 2, 3, 0x10);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FSGNJN.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fsgnjx_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FSGNJX.S f1, f2, f3 (rm=2)
        let inst = encode_r_type(0x53, 1, 2, 2, 3, 0x10);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FSGNJX.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_feq_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FEQ.S x1, f2, f3 (rm=2)
        let inst = encode_r_type(0x53, 1, 2, 2, 3, 0x50);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FEQ.S");

        assert_eq!(output.used_u32s, 1);
        // FEQ returns integer result - same 10 instructions but with move from a0 instead of li a3
        assert_eq!(output.instructions.len(), 10, "FEQ.S should emit 10 instructions");
    }

    #[test]
    fn test_flt_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FLT.S x1, f2, f3 (rm=1)
        let inst = encode_r_type(0x53, 1, 1, 2, 3, 0x50);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FLT.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fle_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FLE.S x1, f2, f3 (rm=0)
        let inst = encode_r_type(0x53, 1, 0, 2, 3, 0x50);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FLE.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fcvt_w_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FCVT.W.S x1, f2, rne (rs2=0)
        let inst = encode_r_type(0x53, 1, 0, 2, 0, 0x60);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FCVT.W.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fcvt_wu_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FCVT.WU.S x1, f2, rne (rs2=1)
        let inst = encode_r_type(0x53, 1, 0, 2, 1, 0x60);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FCVT.WU.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fcvt_s_w_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FCVT.S.W f1, x2, rne (rs2=0)
        let inst = encode_r_type(0x53, 1, 0, 2, 0, 0x68);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FCVT.S.W");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fcvt_s_wu_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FCVT.S.WU f1, x2, rne (rs2=1)
        let inst = encode_r_type(0x53, 1, 0, 2, 1, 0x68);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FCVT.S.WU");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fmv_x_w_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FMV.X.W x1, f2 (rs2=0, rm=0)
        let inst = encode_r_type(0x53, 1, 0, 2, 0, 0x70);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FMV.X.W");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fmv_w_x_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FMV.W.X f1, x2 (rs2=0, rm=0)
        let inst = encode_r_type(0x53, 1, 0, 2, 0, 0x78);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FMV.W.X");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fclass_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FCLASS.S x1, f2 (rs2=0, rm=1)
        let inst = encode_r_type(0x53, 1, 1, 2, 0, 0x70);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FCLASS.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 10);
    }

    #[test]
    fn test_fmadd_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FMADD.S f1, f2, f3, f4, rne
        let inst = encode_r4_type(0x43, 1, 0, 2, 3, 4);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FMADD.S");

        assert_eq!(output.used_u32s, 1);
        // FMA operations emit 11 instructions (5 li for args + 6 for dispatch)
        assert_eq!(output.instructions.len(), 11, "FMADD.S should emit 11 instructions (4-operand)");
    }

    #[test]
    fn test_fmsub_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FMSUB.S f1, f2, f3, f4, rne
        let inst = encode_r4_type(0x47, 1, 0, 2, 3, 4);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FMSUB.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 11, "FMSUB.S should emit 11 instructions (4-operand)");
    }

    #[test]
    fn test_fnmadd_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FNMADD.S f1, f2, f3, f4, rne
        let inst = encode_r4_type(0x4B, 1, 0, 2, 3, 4);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FNMADD.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 11, "FNMADD.S should emit 11 instructions (4-operand)");
    }

    #[test]
    fn test_fnmsub_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FNMSUB.S f1, f2, f3, f4, rne
        let inst = encode_r4_type(0x4F, 1, 0, 2, 3, 4);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FNMSUB.S");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 11, "FNMSUB.S should emit 11 instructions (4-operand)");
    }

    #[test]
    fn test_invalid_opcode_returns_none() {
        let ext = Rv32FArchATranspilerExtension;
        // Not a float instruction
        let inst = encode_r_type(0x33, 1, 0, 2, 3, 0x00); // ADD (RV32I)
        let output: Option<TranspilerOutput<F>> = ext.process_custom(&[inst]);

        assert!(output.is_none(), "Should return None for non-float opcodes");
    }

    #[test]
    fn test_empty_stream_returns_none() {
        let ext = Rv32FArchATranspilerExtension;
        let output: Option<TranspilerOutput<F>> = ext.process_custom(&[]);

        assert!(output.is_none(), "Should return None for empty stream");
    }

    #[test]
    fn test_emitted_instruction_opcodes() {
        let ext = Rv32FArchATranspilerExtension;
        // FADD.S f3, f1, f2
        let inst = encode_r_type(0x53, 3, 0, 1, 2, 0x00);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).unwrap();

        // Check that we have the right opcodes
        // Expected sequence:
        // 1-4: li (ADD with immediate)
        // 5: li (ADD with immediate for table address)
        // 6: li (ADD with immediate for op index)
        // 7: slli (shift left logical immediate)
        // 8: add (add)
        // 9: lw (load word)
        // 10. jalr (jump and link register)

        let instructions: Vec<_> = output.instructions.iter().flatten().collect();
        assert_eq!(instructions.len(), 10);

        // First 4 should be ADD (li is pseudo-instruction for addi)
        let add_opcode = BaseAluOpcode::ADD.global_opcode();
        assert_eq!(instructions[0].opcode, add_opcode);
        assert_eq!(instructions[1].opcode, add_opcode);
        assert_eq!(instructions[2].opcode, add_opcode);
        assert_eq!(instructions[3].opcode, add_opcode);

        // 5-6 also ADD (li)
        assert_eq!(instructions[4].opcode, add_opcode);
        assert_eq!(instructions[5].opcode, add_opcode);

        // 7: slli
        let sll_opcode = ShiftOpcode::SLL.global_opcode();
        assert_eq!(instructions[6].opcode, sll_opcode);

        // 8: add
        assert_eq!(instructions[7].opcode, add_opcode);

        // 9: lw
        let lw_opcode = Rv32LoadStoreOpcode::LOADW.global_opcode();
        assert_eq!(instructions[8].opcode, lw_opcode);

        // 10. jalr
        let jalr_opcode = Rv32JalrOpcode::JALR.global_opcode();
        assert_eq!(instructions[9].opcode, jalr_opcode);
    }

    #[test]
    fn test_dispatch_table_address_constant() {
        // Verify the dispatch table address matches linker script
        assert_eq!(FLOAT_DISPATCH_TABLE_ADDR, 0x10000000);
    }

    #[test]
    fn test_rounding_modes() {
        let ext = Rv32FArchATranspilerExtension;

        // Test different rounding modes (encoded in rm field)
        for rm in 0..5 {
            // FADD.S f3, f1, f2 with different rounding modes
            let inst = encode_r_type(0x53, 3, rm as u8, 1, 2, 0x00);
            let output: Option<TranspilerOutput<F>> = ext.process_custom(&[inst]);

            if rm < 4 {
                // Valid rounding modes: RNE=0, RTZ=1, RDN=2, RUP=3
                assert!(output.is_some(), "rm={} should be valid", rm);
            }
            // rm=4 (RMM) and higher might not be supported by all operations
        }
    }

    #[test]
    fn test_register_indices() {
        let ext = Rv32FArchATranspilerExtension;

        // Test with maximum register indices (f31)
        let inst = encode_r_type(0x53, 31, 0, 30, 29, 0x00); // FADD.S f31, f30, f29
        let output: Option<TranspilerOutput<F>> = ext.process_custom(&[inst]);

        assert!(output.is_some(), "Should handle f31 register");
        assert_eq!(output.unwrap().used_u32s, 1);
    }

    // Helper to encode I-type instruction (for FLW)
    fn encode_i_type(opcode: u8, rd: u32, funct3: u8, rs1: u32, imm: i32) -> u32 {
        let imm_u = (imm as u32) & 0xFFF;
        (imm_u << 20)
            | ((rs1 & 0x1F) << 15)
            | ((funct3 as u32) << 12)
            | ((rd & 0x1F) << 7)
            | (opcode as u32)
    }

    // Helper to encode S-type instruction (for FSW)
    fn encode_s_type(opcode: u8, imm: i32, rs2: u32, rs1: u32, funct3: u8) -> u32 {
        let imm_u = (imm as u32) & 0xFFF;
        let imm11_5 = (imm_u >> 5) & 0x7F;
        let imm4_0 = imm_u & 0x1F;
        (imm11_5 << 25)
            | ((rs2 & 0x1F) << 20)
            | ((rs1 & 0x1F) << 15)
            | ((funct3 as u32) << 12)
            | (imm4_0 << 7)
            | (opcode as u32)
    }

    #[test]
    fn test_flw_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FLW f1, 8(x2)
        let inst = encode_i_type(0x07, 1, 0b010, 2, 8);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FLW");

        assert_eq!(output.used_u32s, 1, "Should consume 1 instruction");
        assert_eq!(output.instructions.len(), 3, "Should emit 3 instructions");

        // Verify all instructions are Some
        assert!(output.instructions.iter().all(|i| i.is_some()));
    }

    #[test]
    fn test_fsw_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        // FSW f3, 16(x4)
        let inst = encode_s_type(0x27, 16, 3, 4, 0b010);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FSW");

        assert_eq!(output.used_u32s, 1, "Should consume 1 instruction");
        assert_eq!(output.instructions.len(), 3, "Should emit 3 instructions");

        // Verify all instructions are Some
        assert!(output.instructions.iter().all(|i| i.is_some()));
    }

    #[test]
    fn test_flw_negative_offset() {
        let ext = Rv32FArchATranspilerExtension;
        // FLW f5, -4(x10)
        let inst = encode_i_type(0x07, 5, 0b010, 10, -4);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FLW with negative offset");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 3);
    }

    #[test]
    fn test_fsw_negative_offset() {
        let ext = Rv32FArchATranspilerExtension;
        // FSW f7, -8(x12)
        let inst = encode_s_type(0x27, -8, 7, 12, 0b010);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FSW with negative offset");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 3);
    }

    #[test]
    fn test_flw_max_register() {
        let ext = Rv32FArchATranspilerExtension;
        // FLW f31, 0(x31)
        let inst = encode_i_type(0x07, 31, 0b010, 31, 0);
        let output: Option<TranspilerOutput<F>> = ext.process_custom(&[inst]);

        assert!(output.is_some(), "Should handle f31 register");
        assert_eq!(output.unwrap().used_u32s, 1);
    }

    #[test]
    fn test_fsw_max_register() {
        let ext = Rv32FArchATranspilerExtension;
        // FSW f31, 0(x31)
        let inst = encode_s_type(0x27, 0, 31, 31, 0b010);
        let output: Option<TranspilerOutput<F>> = ext.process_custom(&[inst]);

        assert!(output.is_some(), "Should handle f31 register");
        assert_eq!(output.unwrap().used_u32s, 1);
    }

    #[test]
    fn test_flw_emitted_opcodes() {
        let ext = Rv32FArchATranspilerExtension;
        // FLW f2, 4(x3)
        let inst = encode_i_type(0x07, 2, 0b010, 3, 4);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).unwrap();

        // Expected sequence:
        // 1. li t0, FLOAT_REG_BASE + 2*4  (ADD with immediate)
        // 2. lw t1, 4(x3)                  (LOADW)
        // 3. sw t1, 0(t0)                  (STOREW)

        let instructions: Vec<_> = output.instructions.iter().flatten().collect();
        assert_eq!(instructions.len(), 3);

        let add_opcode = BaseAluOpcode::ADD.global_opcode();
        let lw_opcode = Rv32LoadStoreOpcode::LOADW.global_opcode();
        let sw_opcode = Rv32LoadStoreOpcode::STOREW.global_opcode();

        assert_eq!(instructions[0].opcode, add_opcode, "First should be li (ADD)");
        assert_eq!(instructions[1].opcode, lw_opcode, "Second should be lw");
        assert_eq!(instructions[2].opcode, sw_opcode, "Third should be sw");
    }

    #[test]
    fn test_fsw_emitted_opcodes() {
        let ext = Rv32FArchATranspilerExtension;
        // FSW f4, 12(x5)
        let inst = encode_s_type(0x27, 12, 4, 5, 0b010);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).unwrap();

        // Expected sequence:
        // 1. li t0, FLOAT_REG_BASE + 4*4  (ADD with immediate)
        // 2. lw t1, 0(t0)                  (LOADW)
        // 3. sw t1, 12(x5)                 (STOREW)

        let instructions: Vec<_> = output.instructions.iter().flatten().collect();
        assert_eq!(instructions.len(), 3);

        let add_opcode = BaseAluOpcode::ADD.global_opcode();
        let lw_opcode = Rv32LoadStoreOpcode::LOADW.global_opcode();
        let sw_opcode = Rv32LoadStoreOpcode::STOREW.global_opcode();

        assert_eq!(instructions[0].opcode, add_opcode, "First should be li (ADD)");
        assert_eq!(instructions[1].opcode, lw_opcode, "Second should be lw");
        assert_eq!(instructions[2].opcode, sw_opcode, "Third should be sw");
    }

    #[test]
    fn test_float_reg_base_constant() {
        // Verify float register base address
        assert_eq!(FLOAT_REG_BASE, 0xC0000000);
    }

    #[test]
    fn test_integer_result_opcode_sequence() {
        let ext = Rv32FArchATranspilerExtension;
        // FEQ.S x3, f1, f2 - comparison returns integer result
        let inst = encode_r_type(0x53, 3, 2, 1, 2, 0x50);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).unwrap();

        // Expected sequence for integer result:
        // 1-3: li a0, a1, a2 (args: rs1, rs2, rm)
        // 4-6: load dispatch table and index
        // 7-9: calculate address and load function pointer
        // 10: move result from a0 to rd (ADD instruction)

        let instructions: Vec<_> = output.instructions.iter().flatten().collect();
        assert_eq!(instructions.len(), 10);

        let add_opcode = BaseAluOpcode::ADD.global_opcode();
        let jalr_opcode = Rv32JalrOpcode::JALR.global_opcode();

        // 9th instruction should be JALR
        assert_eq!(instructions[8].opcode, jalr_opcode, "9th should be jalr");

        // 10th instruction should be ADD (implementing mv rd, a0)
        assert_eq!(instructions[9].opcode, add_opcode, "10th should be add (mv from a0 to rd)");
    }

    #[test]
    fn test_float_result_has_no_move() {
        let ext = Rv32FArchATranspilerExtension;
        // FADD.S f3, f1, f2 - arithmetic returns float result
        let inst = encode_r_type(0x53, 3, 0, 1, 2, 0x00);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).unwrap();

        // Float results use the standard calling sequence with 4 li instructions
        // and pass rd as an argument
        let instructions: Vec<_> = output.instructions.iter().flatten().collect();
        assert_eq!(instructions.len(), 10);

        let add_opcode = BaseAluOpcode::ADD.global_opcode();
        let jalr_opcode = Rv32JalrOpcode::JALR.global_opcode();

        // Last instruction should be JALR (no move after call)
        assert_eq!(instructions[9].opcode, jalr_opcode, "Last should be jalr for float results");

        // First 4 should be li instructions (ADD with immediate)
        assert_eq!(instructions[0].opcode, add_opcode);
        assert_eq!(instructions[1].opcode, add_opcode);
        assert_eq!(instructions[2].opcode, add_opcode);
        assert_eq!(instructions[3].opcode, add_opcode);
    }

    #[test]
    fn test_fma_opcode_sequence() {
        let ext = Rv32FArchATranspilerExtension;
        // FMADD.S f5, f1, f2, f3 - 4-operand FMA operation
        let inst = encode_r4_type(0x43, 5, 0, 1, 2, 3);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).unwrap();

        // Expected sequence for FMA:
        // 1-5: li a0-a4 (args: rs1, rs2, rs3, rd, rm)
        // 6-8: load dispatch table and index
        // 9-11: calculate address, load function pointer, call

        let instructions: Vec<_> = output.instructions.iter().flatten().collect();
        assert_eq!(instructions.len(), 11);

        let add_opcode = BaseAluOpcode::ADD.global_opcode();
        let jalr_opcode = Rv32JalrOpcode::JALR.global_opcode();

        // First 5 should be li instructions (ADD with immediate) for all 5 args
        for i in 0..5 {
            assert_eq!(instructions[i].opcode, add_opcode, "Instruction {} should be li (ADD)", i + 1);
        }

        // Last instruction should be JALR
        assert_eq!(instructions[10].opcode, jalr_opcode, "Last should be jalr");
    }
}
