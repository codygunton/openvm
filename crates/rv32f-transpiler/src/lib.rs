// OpenVM RV32F Transpiler Extension - Memory-Mapped Approach
// Adds F extension (single-precision floating-point) to RV32IM base
// Uses memory-mapped I/O: writes instructions to FREG_INST and calls unified handler

use openvm_instructions::{instruction::Instruction, riscv::RV32_REGISTER_NUM_LIMBS, LocalOpcode};
use openvm_rv32im_transpiler::{
    BaseAluOpcode, Rv32JalLuiOpcode, Rv32JalrOpcode, Rv32LoadStoreOpcode,
};
use openvm_stark_backend::p3_field::PrimeField32;
use openvm_transpiler::{TranspilerExtension, TranspilerOutput};

/// Transpiler extension for the F extension (single-precision floating-point)
///
/// Adds RV32F support to OpenVM's RV32IM base, creating full RV32IMF.
/// Uses memory-mapped I/O approach where each F instruction is written to a
/// memory-mapped register (FREG_INST) and a single handler function is called.
///
/// For each F extension instruction like FADD.S f3, f1, f2:
/// 1-2. li t0, FREG_INST_ADDR   - Load instruction register address
/// 3-4. li t1, <inst>            - Load original instruction encoding
/// 5.   sw t1, 0(t0)             - Write instruction to FREG_INST
/// 6-7. li t0, handler_addr      - Load handler address
/// 8.   jalr ra, t0              - Call _openvm_float handler
///
/// This approach reduces transpilation overhead from ~10 instructions to ~5-7 instructions
/// per F operation and simplifies the runtime by having a single entry point.
pub struct Rv32FArchATranspilerExtension;

// Memory-mapped register addresses (must match runtime header)
// Float register base address - must be within VM memory (< 512MB)
const FLOAT_REG_BASE: u32 = 0x18000000; // 384MB - safe within 512MB limit
const FCSR_ADDR: u32 = 0x18000080; // FCSR control/status register
const FREG_INST_ADDR: u32 = 0x18000088; // Instruction register for memory-mapped approach
const HANDLER_PC_ADDR: u32 = 0x18000090; // Stores the OpenVM PC of the float handler (set by SDK)

// No need to redefine - imported from openvm_instructions::riscv

// RISC-V opcodes
const OPCODE_LOAD_FP: u8 = 0x07; // FLW (Float Load Word)
const OPCODE_STORE_FP: u8 = 0x27; // FSW (Float Store Word)
const OPCODE_OP_FP: u8 = 0x53; // Floating-point arithmetic (FADD, FSUB, etc.)
const OPCODE_MADD: u8 = 0x43; // FMADD
const OPCODE_MSUB: u8 = 0x47; // FMSUB
const OPCODE_NMSUB: u8 = 0x4B; // FNMSUB
const OPCODE_NMADD: u8 = 0x4F; // FNMADD

impl<F: PrimeField32> TranspilerExtension<F> for Rv32FArchATranspilerExtension {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        if instruction_stream.is_empty() {
            return None;
        }

        let inst = instruction_stream[0];
        let opcode = (inst & 0x7F) as u8;

        match opcode {
            OPCODE_LOAD_FP => {
                eprintln!("[RV32F] FLW 0x{:08x}", inst);
                self.process_flw(inst)
            }
            OPCODE_STORE_FP => {
                eprintln!("[RV32F] FSW 0x{:08x}", inst);
                self.process_fsw(inst)
            }
            OPCODE_OP_FP => {
                eprintln!("[RV32F] F-arith 0x{:08x}", inst);
                self.process_op_fp(inst)
            }
            OPCODE_MADD => {
                eprintln!("[RV32F] FMADD 0x{:08x}", inst);
                self.process_fused_op(inst, 0)
            }
            OPCODE_MSUB => {
                eprintln!("[RV32F] FMSUB 0x{:08x}", inst);
                self.process_fused_op(inst, 0)
            }
            OPCODE_NMSUB => {
                eprintln!("[RV32F] FNMSUB 0x{:08x}", inst);
                self.process_fused_op(inst, 0)
            }
            OPCODE_NMADD => {
                eprintln!("[RV32F] FNMADD 0x{:08x}", inst);
                self.process_fused_op(inst, 0)
            }
            _ => None,
        }
    }
}

impl Rv32FArchATranspilerExtension {
    /// Process FLW (Float Load Word) - I-type
    /// FLW fd, offset(rs1) loads 32-bit float from memory into float register
    fn process_flw<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let rd = (inst >> 7) & 0x1F; // Float destination register
        let rs1 = (inst >> 15) & 0x1F; // Base address register
        let imm = (inst as i32) >> 20; // Sign-extended 12-bit immediate

        // FLW fd, offset(rs1) translates to:
        // 1. li t0, FLOAT_REG_BASE + fd*4   # Calculate float reg address (may be LUI + ADDI)
        // 2. lw t1, offset(rs1)              # Load from program memory
        // 3. sw t1, 0(t0)                    # Store to float reg space

        let mut instructions = Vec::new();
        let float_reg_addr = FLOAT_REG_BASE + (rd * 4);

        // t0 (x5) = address of float register fd (may emit multiple instructions)
        for insn in li::<F>(5, float_reg_addr as i32) {
            instructions.push(Some(insn));
        }

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
        let rs2 = (inst >> 20) & 0x1F; // Float source register
        let rs1 = (inst >> 15) & 0x1F; // Base address register
        let imm11_5 = (inst >> 25) & 0x7F;
        let imm4_0 = (inst >> 7) & 0x1F;
        let imm = (((imm11_5 << 5) | imm4_0) as i32) << 20 >> 20; // Sign-extend

        // FSW rs2, offset(rs1) translates to:
        // 1. li t0, FLOAT_REG_BASE + rs2*4  # Calculate float reg address (may be LUI + ADDI)
        // 2. lw t1, 0(t0)                   # Load from float reg space
        // 3. sw t1, offset(rs1)             # Store to program memory

        let mut instructions = Vec::new();
        let float_reg_addr = FLOAT_REG_BASE + (rs2 * 4);

        // t0 (x5) = address of float register rs2 (may emit multiple instructions)
        for insn in li::<F>(5, float_reg_addr as i32) {
            instructions.push(Some(insn));
        }

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
    /// All operations now use the memory-mapped approach via emit_float_call
    fn process_op_fp<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        // Validate that this is a supported F instruction
        let funct7 = (inst >> 25) & 0x7F;
        let rs2 = (inst >> 20) & 0x1F;
        let rm = (inst >> 12) & 0x7;

        // Quick validation to ensure this is a valid F instruction
        let _valid = match funct7 {
            0x00 | 0x04 | 0x08 | 0x0C => true, // FADD, FSUB, FMUL, FDIV
            0x2C if rs2 == 0 => true,          // FSQRT
            0x10 if rm <= 2 => true,           // FSGNJ, FSGNJN, FSGNJX
            0x14 if rm <= 1 => true,           // FMIN, FMAX
            0x50 if rm <= 2 => true,           // FLE, FLT, FEQ
            0x60 if rs2 <= 1 => true,          // FCVT.W.S, FCVT.WU.S
            0x68 if rs2 <= 1 => true,          // FCVT.S.W, FCVT.S.WU
            0x70 if rs2 == 0 && (rm == 0 || rm == 1) => true, // FMV.X.W, FCLASS.S
            0x78 if rs2 == 0 && rm == 0 => true, // FMV.W.X
            _ => return None,
        };

        // All F instructions now use the same memory-mapped calling pattern
        Some(self.emit_float_call(inst))
    }

    /// Process fused multiply-add/sub opcodes (R4-type)
    /// All operations now use the memory-mapped approach via emit_float_call
    fn process_fused_op<F: PrimeField32>(
        &self,
        inst: u32,
        _op_index: u32,
    ) -> Option<TranspilerOutput<F>> {
        // No need to decode or validate - just pass the instruction to the handler
        Some(self.emit_float_call(inst))
    }

    /// Emit calling sequence for any float operation using memory-mapped approach
    ///
    /// Writes instruction to FREG_INST and calls _openvm_float handler via trampoline.
    ///
    /// Emits approximately 6-8 instructions:
    /// 1-2. li t0, FREG_INST_ADDR    (may be LUI + ADDI for large address)
    /// 3-4. li t1, inst               (load original instruction encoding)
    /// 5.   sw t1, 0(t0)              (write instruction to FREG_INST)
    /// 6-7. li t0, HANDLER_PC_ADDR    (load trampoline address)
    /// 8.   lw t1, 0(t0)              (read handler's OpenVM PC from memory)
    /// 9.   jalr ra, t1, 0            (call handler at the OpenVM PC)
    fn emit_float_call<F: PrimeField32>(&self, inst: u32) -> TranspilerOutput<F> {
        let mut instructions = Vec::new();

        // Load FREG_INST address into t0 (x5)
        for insn in li::<F>(5, FREG_INST_ADDR as i32) {
            instructions.push(Some(insn));
        }

        // Load original instruction into t1 (x6)
        for insn in li::<F>(6, inst as i32) {
            instructions.push(Some(insn));
        }

        // Store instruction to FREG_INST: sw t1, 0(t0)
        instructions.push(Some(sw::<F>(6, 5, 0)));

        // Load HANDLER_PC_ADDR into t0 (x5)
        for insn in li::<F>(5, HANDLER_PC_ADDR as i32) {
            instructions.push(Some(insn));
        }

        // Load the handler's OpenVM PC from memory: lw t1, 0(t0)
        instructions.push(Some(lw::<F>(6, 5, 0)));

        // Indirect call via JALR: jalr ra, t1, 0
        instructions.push(Some(jalr::<F>(1, 6, 0)));

        TranspilerOutput {
            instructions,
            used_u32s: 1, // Consumed one F instruction
        }
    }
}

// Helper functions to construct OpenVM instructions

/// li rd, imm - Load immediate (may need LUI + ADDI for large values)
/// Returns a vector of instructions needed to load the immediate
fn li<F: PrimeField32>(rd: u32, imm: i32) -> Vec<Instruction<F>> {
    // Check if immediate fits in 12 bits (ADDI range: -2048 to 2047)
    if imm >= -2048 && imm <= 2047 {
        // Simple case: just use ADDI
        vec![Instruction::from_isize(
            BaseAluOpcode::ADD.global_opcode(),
            ((rd as usize) * RV32_REGISTER_NUM_LIMBS) as isize,
            0, // x0
            imm as isize,
            1, // rd is register
            0, // rs2 is immediate
        )]
    } else {
        // Large immediate: use LUI + ADDI
        let imm_u = imm as u32;
        let upper = (imm_u >> 12) & 0xfffff; // bits 31:12
        let lower = (imm_u & 0xfff) as i32; // bits 11:0

        // Adjust for sign extension: if lower is negative when sign-extended,
        // we need to add 1 to upper and use negative lower
        let (upper_adjusted, lower_adjusted) = if lower >= 2048 {
            (upper + 1, lower - 4096)
        } else {
            (upper, lower)
        };

        let mut result = Vec::new();

        // LUI rd, upper
        let mut lui_insn = Instruction::new(
            Rv32JalLuiOpcode::LUI.global_opcode(),
            F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rd as usize),
            F::ZERO,
            F::from_canonical_u32(upper_adjusted),
            F::ONE, // rd is register
            F::ZERO,
            F::ZERO,
            F::ZERO,
        );
        lui_insn.f = F::ONE; // f=1 for LUI (shares chip with JAL)
        result.push(lui_insn);

        // ADDI rd, rd, lower (if lower != 0)
        if lower_adjusted != 0 {
            result.push(Instruction::from_isize(
                BaseAluOpcode::ADD.global_opcode(),
                ((rd as usize) * RV32_REGISTER_NUM_LIMBS) as isize,
                ((rd as usize) * RV32_REGISTER_NUM_LIMBS) as isize,
                lower_adjusted as isize,
                1, // rd is register
                0, // rs2 is immediate
            ));
        }

        result
    }
}

/// lw rd, offset(rs1)
fn lw<F: PrimeField32>(rd: u32, rs1: u32, offset: i32) -> Instruction<F> {
    Instruction::new(
        Rv32LoadStoreOpcode::LOADW.global_opcode(),
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rd as usize),
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rs1 as usize),
        F::from_canonical_u32((offset as u32) & 0xffff),
        F::ONE,                   // d: rd is register
        F::TWO,                   // e: load from memory (CRITICAL - was F::ZERO!)
        F::from_bool(rd != 0),    // f: flag for operation (was offset < 0!)
        F::from_bool(offset < 0), // g: flag for sign extension
    )
}

/// sw rs2, offset(rs1)
fn sw<F: PrimeField32>(rs2: u32, rs1: u32, offset: i32) -> Instruction<F> {
    Instruction::new(
        Rv32LoadStoreOpcode::STOREW.global_opcode(),
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rs2 as usize), // a: rs2 (data)
        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * rs1 as usize), // b: rs1 (address base)
        F::from_canonical_u32((offset as u32) & 0xffff),                 // c: offset
        F::ONE,                                                          // d: register flag
        F::TWO,                   // e: store to memory (was F::ONE!)
        F::ONE,                   // f: operation flag (was offset < 0!)
        F::from_bool(offset < 0), // g: sign extension flag
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
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FADD.S");

        assert_eq!(output.used_u32s, 1, "Should consume 1 instruction");
        // Memory-mapped approach: ~8 instructions (2x li, sw, 2x li, jalr)
        assert!(
            output.instructions.len() >= 5 && output.instructions.len() <= 10,
            "Should emit 5-10 instructions, got {}",
            output.instructions.len()
        );

        // Verify all instructions are Some (not None)
        assert!(output.instructions.iter().all(|i| i.is_some()));
    }

    // All F operation tests now expect memory-mapped approach (5-10 instructions)

    #[test]
    fn test_fsub_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 5, 0, 4, 3, 0x04);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FSUB.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fmul_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 10, 0, 11, 12, 0x08);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FMUL.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fdiv_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 7, 0, 8, 9, 0x0C);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FDIV.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fsqrt_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 6, 0, 5, 0, 0x2C);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FSQRT.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fmin_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 0, 2, 3, 0x14);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FMIN.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fmax_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 1, 2, 3, 0x14);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FMAX.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fsgnj_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 0, 2, 3, 0x10);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FSGNJ.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fsgnjn_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 1, 2, 3, 0x10);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FSGNJN.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fsgnjx_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 2, 2, 3, 0x10);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FSGNJX.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_feq_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 2, 2, 3, 0x50);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FEQ.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_flt_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 1, 2, 3, 0x50);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FLT.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fle_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 0, 2, 3, 0x50);
        let output: TranspilerOutput<F> = ext.process_custom(&[inst]).expect("Should decode FLE.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fcvt_w_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 0, 2, 0, 0x60);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FCVT.W.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fcvt_wu_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 0, 2, 1, 0x60);
        let output: TranspilerOutput<F> = ext
            .process_custom(&[inst])
            .expect("Should decode FCVT.WU.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fcvt_s_w_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 0, 2, 0, 0x68);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FCVT.S.W");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fcvt_s_wu_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 0, 2, 1, 0x68);
        let output: TranspilerOutput<F> = ext
            .process_custom(&[inst])
            .expect("Should decode FCVT.S.WU");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fmv_x_w_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 0, 2, 0, 0x70);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FMV.X.W");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fmv_w_x_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 0, 2, 0, 0x78);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FMV.W.X");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fclass_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r_type(0x53, 1, 1, 2, 0, 0x70);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FCLASS.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fmadd_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r4_type(0x43, 1, 0, 2, 3, 4);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FMADD.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fmsub_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r4_type(0x47, 1, 0, 2, 3, 4);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FMSUB.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fnmadd_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r4_type(0x4B, 1, 0, 2, 3, 4);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FNMADD.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
    }

    #[test]
    fn test_fnmsub_s_decoding() {
        let ext = Rv32FArchATranspilerExtension;
        let inst = encode_r4_type(0x4F, 1, 0, 2, 3, 4);
        let output: TranspilerOutput<F> =
            ext.process_custom(&[inst]).expect("Should decode FNMSUB.S");
        assert_eq!(output.used_u32s, 1);
        assert!(output.instructions.len() >= 5 && output.instructions.len() <= 10);
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
    fn test_handler_address_constant() {
        // Verify the handler address constant
        assert_eq!(OPENVM_FLOAT_HANDLER_ADDR, 0x10000000);
    }

    #[test]
    fn test_freg_inst_address_constant() {
        // Verify the FREG_INST address constant
        assert_eq!(FREG_INST_ADDR, 0x18000088);
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
        let output: TranspilerOutput<F> = ext
            .process_custom(&[inst])
            .expect("Should decode FLW with negative offset");

        assert_eq!(output.used_u32s, 1);
        assert_eq!(output.instructions.len(), 3);
    }

    #[test]
    fn test_fsw_negative_offset() {
        let ext = Rv32FArchATranspilerExtension;
        // FSW f7, -8(x12)
        let inst = encode_s_type(0x27, -8, 7, 12, 0b010);
        let output: TranspilerOutput<F> = ext
            .process_custom(&[inst])
            .expect("Should decode FSW with negative offset");

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

        assert_eq!(
            instructions[0].opcode, add_opcode,
            "First should be li (ADD)"
        );
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

        assert_eq!(
            instructions[0].opcode, add_opcode,
            "First should be li (ADD)"
        );
        assert_eq!(instructions[1].opcode, lw_opcode, "Second should be lw");
        assert_eq!(instructions[2].opcode, sw_opcode, "Third should be sw");
    }

    #[test]
    fn test_float_reg_base_constant() {
        // Verify float register base address
        assert_eq!(FLOAT_REG_BASE, 0x18000000);
    }
}
