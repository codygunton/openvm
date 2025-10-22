use openvm_instructions::{instruction::Instruction, riscv::RV32_REGISTER_NUM_LIMBS, LocalOpcode};
use openvm_rv32im_transpiler::{
    BaseAluOpcode, Rv32JalLuiOpcode, Rv32JalrOpcode, Rv32LoadStoreOpcode,
};
use openvm_stark_backend::p3_field::PrimeField32;
use openvm_transpiler::{TranspilerExtension, TranspilerOutput};
use rrs_lib::instruction_formats::{IType, SType};

// F extension opcode values (from RISC-V spec)
pub const FLOAD_OPCODE: u8 = 0x07; // FLW
pub const FSTORE_OPCODE: u8 = 0x27; // FSW
pub const FMADD_OPCODE: u8 = 0x43; // FMADD.S
pub const FMSUB_OPCODE: u8 = 0x47; // FMSUB.S
pub const FNMSUB_OPCODE: u8 = 0x4B; // FNMSUB.S
pub const FNMADD_OPCODE: u8 = 0x4F; // FNMADD.S
pub const FP_OPCODE: u8 = 0x53; // FADD.S, FMUL.S, etc.

// Memory map (must match float.h values)
pub const FLOAT_REGISTER_BASE: u32 = 0x1F001000;
pub const FLOAT_INST_ADDR: u32 = 0x1F001108;
pub const FLOAT_LIB_ENTRY_PTR: u32 = 0x0001_EC60;

// OpenVM addressing constants
pub const RV32_MEMORY_AS: u32 = 2; // Heap memory address space

#[derive(Default)]
pub struct FloatsTranspilerExtension;

impl FloatsTranspilerExtension {
    pub fn new() -> Self {
        Self
    }

    /// Convert float register index (0-31) to memory address
    fn float_reg_addr(&self, freg: usize) -> u32 {
        FLOAT_REGISTER_BASE + (freg as u32 * 4) // Each f32 is 4 bytes
    }

    fn handle_flw<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let dec = IType::new(inst);

        // FLW frd, imm(rs1)
        // Load word from memory[rs1 + imm] into float register frd
        // Float register frd is stored at memory address float_reg_addr(frd)
        //
        // Since STOREW requires a base register (not absolute address), we need to:
        // 1. Load from memory[rs1 + imm] into temp register x5
        // 2. Load the float register address into temp register x6 (using LUI + ADDI)
        // 3. Store from x5 to memory[x6 + 0]

        let mut instructions = Vec::new();

        // 1. LOADW x5, imm(rs1)
        instructions.push(Instruction::large_from_isize(
            Rv32LoadStoreOpcode::LOADW.global_opcode(),
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x5
            (RV32_REGISTER_NUM_LIMBS * dec.rs1) as isize, // b: base register rs1
            ((dec.imm as u32) & 0xffff) as isize,   // c: offset immediate (16-bit masked)
            1,                                      // d: base is register
            RV32_MEMORY_AS as isize,                // e: load from heap
            1,                                      // f: write enabled
            (dec.imm < 0) as isize,                 // g: sign flag
        ));

        let float_addr = self.float_reg_addr(dec.rd);

        // 2a. LUI x6, upper_20_bits(float_addr)
        let upper_20 = (float_addr >> 12) & 0xFFFFF;
        instructions.push(Instruction::from_isize(
            Rv32JalLuiOpcode::LUI.global_opcode(),
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x6
            upper_20 as isize,                      // b: upper immediate (20 bits)
            0,                                      // c: unused
            0,                                      // d: unused
            0,                                      // e: unused
        ));

        // 2b. ADDI x6, x6, lower_12_bits(float_addr)
        let lower_12 = ((float_addr & 0xFFF) as i32) << 20 >> 20; // sign extend
        let lower_12_masked = (lower_12 as u32) & 0xFFFFFF;
        instructions.push(Instruction::large_from_isize(
            BaseAluOpcode::ADD.global_opcode(),
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x6
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // b: source register x6
            lower_12_masked as isize,               // c: immediate (24-bit max)
            1,                                      // d: source is register
            0,                                      // e: imm address space (0=immediate)
            0,                                      // f: unused
            (lower_12 < 0) as isize,                // g: sign flag
        ));

        // 3. STOREW x5, 0(x6)
        instructions.push(Instruction::large_from_isize(
            Rv32LoadStoreOpcode::STOREW.global_opcode(),
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // a: source register x5
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // b: base register x6
            0,                                      // c: offset
            1,                                      // d: source is register
            RV32_MEMORY_AS as isize,                // e: dest is heap memory
            1,                                      // f: store enabled
            0,                                      // g: sign flag
        ));

        Some(TranspilerOutput {
            instructions: instructions.into_iter().map(Some).collect(),
            used_u32s: 1,
        })
    }

    fn handle_fsw<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let dec = SType::new(inst);

        // FSW frs2, imm(rs1)
        // Store float register frs2 to memory[rs1 + imm]
        // Float register frs2 is stored at memory address float_reg_addr(frs2)
        //
        // Since LOADW can only load from (register + offset), not from an absolute address,
        // we need to:
        // 1. Load the float register address into temp register x6 (using LUI + ADDI)
        // 2. Load from memory[x6 + offset] into temp register x5
        // 3. Store from x5 to memory[rs1 + imm]

        let mut instructions = Vec::new();
        let float_addr = self.float_reg_addr(dec.rs2);

        // 1a. LUI x6, upper_20_bits(float_addr)
        let upper_20 = (float_addr >> 12) & 0xFFFFF;
        instructions.push(Instruction::from_isize(
            Rv32JalLuiOpcode::LUI.global_opcode(),
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x6
            upper_20 as isize,                      // b: upper immediate (20 bits)
            0,                                      // c: unused
            0,                                      // d: unused
            0,                                      // e: unused
        ));

        // 1b. ADDI x6, x6, lower_12_bits(float_addr)
        let lower_12 = ((float_addr & 0xFFF) as i32) << 20 >> 20; // sign extend
        let lower_12_masked = (lower_12 as u32) & 0xFFFFFF;
        instructions.push(Instruction::large_from_isize(
            BaseAluOpcode::ADD.global_opcode(),
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x6
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // b: source register x6
            lower_12_masked as isize,               // c: immediate (24-bit max)
            1,                                      // d: source is register
            0,                                      // e: imm address space (0=immediate)
            0,                                      // f: unused
            (lower_12 < 0) as isize,                // g: sign flag
        ));

        // 2. LOADW x5, 0(x6)
        instructions.push(Instruction::large_from_isize(
            Rv32LoadStoreOpcode::LOADW.global_opcode(),
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x5
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // b: base register x6
            0,                                      // c: offset
            1,                                      // d: base is register
            RV32_MEMORY_AS as isize,                // e: load from heap memory
            1,                                      // f: write enabled
            0,                                      // g: sign flag
        ));

        // 3. STOREW x5, rs1, imm
        instructions.push(Instruction::large_from_isize(
            Rv32LoadStoreOpcode::STOREW.global_opcode(),
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // a: source register x5
            (RV32_REGISTER_NUM_LIMBS * dec.rs1) as isize, // b: base register rs1
            ((dec.imm as u32) & 0xffff) as isize,   // c: offset immediate (16-bit masked)
            1,                                      // d: source is register
            RV32_MEMORY_AS as isize,                // e: dest is heap memory
            1,                                      // f: store enabled
            (dec.imm < 0) as isize,                 // g: sign flag
        ));

        Some(TranspilerOutput {
            instructions: instructions.into_iter().map(Some).collect(),
            used_u32s: 1,
        })
    }

    fn handle_float_alu<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        // These operations need to call the float library
        // Generate sequence:
        // 1. Load raw instruction into temporary register x5
        // 2. Store from x5 to FLOAT_INST_ADDR for library to decode
        // 3. Load library function pointer from high-memory location
        // 4. Call _zisk_float() via JALR (which auto-saves return address to x1)

        // Compute LUI/ADDI split accounting for sign extension
        // When bit 11 of lower_12 is set, ADDI will sign-extend it as negative,
        // so we must add 0x1000 to the LUI value to compensate
        let lower_12_raw = inst & 0xFFF;
        let needs_compensation = (lower_12_raw & 0x800) != 0;
        let upper_20 = if needs_compensation {
            ((inst >> 12) + 1) & 0xFFFFF
        } else {
            (inst >> 12) & 0xFFFFF
        };
        let lower_12 = if needs_compensation {
            (lower_12_raw as i32) - 0x1000
        } else {
            lower_12_raw as i32
        };
        let lower_12_masked = (lower_12 as u32) & 0xFFFFFF;

        eprintln!(
            "[FLOAT_TRANSPILER] Transpiling float instruction: 0x{:08x}",
            inst
        );
        eprintln!("[FLOAT_TRANSPILER]   LUI x5, 0x{:x} (upper_20)", upper_20);
        eprintln!("[FLOAT_TRANSPILER]   ADDI x5, x5, {} (lower_12)", lower_12);

        let mut instructions = Vec::new();

        // 1. Load instruction bits into x5 using LUI + ADDI
        // LUI x5, upper_20_bits
        instructions.push(Instruction::large_from_isize(
            Rv32JalLuiOpcode::LUI.global_opcode(),
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x5
            0,                                      // b: unused
            upper_20 as isize, // c: immediate (20 bits) - LUI reads from field c!
            0,                 // d: unused
            0,                 // e: unused
            1,                 // f: write enabled
            0,                 // g: unused
        ));

        // ADDI x5, x5, lower_12_bits (sign-extended)
        // Mask to 24 bits for OpenVM immediate format (rv32im adapters expect <=24-bit immediates)
        instructions.push(Instruction::large_from_isize(
            BaseAluOpcode::ADD.global_opcode(),
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x5
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // b: source register x5
            lower_12_masked as isize,               // c: immediate (24-bit max)
            1,                                      // d: source is register
            0,                                      // e: imm address space (0=immediate)
            0,                                      // f: unused
            (lower_12 < 0) as isize,                // g: sign flag
        ));

        // 2. Load FLOAT_INST_ADDR into x6 and store from x5
        // LUI x6, upper_20_bits(FLOAT_INST_ADDR)
        let upper_20 = (FLOAT_INST_ADDR >> 12) & 0xFFFFF;
        instructions.push(Instruction::large_from_isize(
            Rv32JalLuiOpcode::LUI.global_opcode(),
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x6
            0,                                      // b: unused
            upper_20 as isize, // c: immediate (20 bits) - LUI reads from field c!
            0,                 // d: unused
            0,                 // e: unused
            1,                 // f: write enabled
            0,                 // g: unused
        ));

        // ADDI x6, x6, lower_12_bits(FLOAT_INST_ADDR)
        let lower_12 = ((FLOAT_INST_ADDR & 0xFFF) as i32) << 20 >> 20; // sign extend
        let lower_12_masked = (lower_12 as u32) & 0xFFFFFF;
        instructions.push(Instruction::large_from_isize(
            BaseAluOpcode::ADD.global_opcode(),
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x6
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // b: source register x6
            lower_12_masked as isize,               // c: immediate (24-bit max)
            1,                                      // d: source is register
            0,                                      // e: imm address space (0=immediate)
            0,                                      // f: unused
            (lower_12 < 0) as isize,                // g: sign flag
        ));

        // STOREW x5, 0(x6) - Store lower 32 bits of instruction
        instructions.push(Instruction::large_from_isize(
            Rv32LoadStoreOpcode::STOREW.global_opcode(),
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // a: source register x5
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // b: base register x6
            0,                                      // c: offset
            1,                                      // d: source is register
            RV32_MEMORY_AS as isize,                // e: dest is heap memory
            1,                                      // f: write enabled
            0,                                      // g: sign flag
        ));

        // STOREW x0, 4(x6) - Store 0 to upper 32 bits (library expects uint64_t)
        instructions.push(Instruction::large_from_isize(
            Rv32LoadStoreOpcode::STOREW.global_opcode(),
            (0 * RV32_REGISTER_NUM_LIMBS) as isize, // a: source register x0 (always 0)
            (6 * RV32_REGISTER_NUM_LIMBS) as isize, // b: base register x6
            4,                                      // c: offset (+4 bytes)
            1,                                      // d: source is register
            RV32_MEMORY_AS as isize,                // e: dest is heap memory
            1,                                      // f: write enabled
            0,                                      // g: sign flag
        ));

        // Note: We don't store return address to FLOAT_RETURN_ADDR because:
        // - JALR instruction saves PC+4 to x1 automatically
        // - Library returns via: jalr x0, x1, 0 (jump to address in x1)
        // - This is simpler than trying to calculate PC (which we don't have in transpiler)

        // 2. Load function pointer from FLOAT_LIB_ENTRY_PTR (0x1ec60)
        // Upper 20 bits: 0x1ec60 >> 12 = 0x1E
        // LUI x5, 0x1E (loads 0x1E000 into x5)
        instructions.push(Instruction::large_from_isize(
            Rv32JalLuiOpcode::LUI.global_opcode(),
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // a: dest register x5
            0,                                      // b: unused
            0x1E, // c: immediate (20 bits) - LUI reads from field c!
            0,    // d: unused
            0,    // e: unused
            1,    // f: write enabled
            0,    // g: unused
        ));

        // LW x5, 0xc60(x5) (load pointer from 0x1ec60)
        instructions.push(Instruction::large_from_isize(
            Rv32LoadStoreOpcode::LOADW.global_opcode(),
            (RV32_REGISTER_NUM_LIMBS * 5) as isize, // a: dest register x5
            (RV32_REGISTER_NUM_LIMBS * 5) as isize, // b: base register x5
            0xC60,                                  // c: offset (lower 12 bits of 0x1ec60)
            1,                                      // d: base is register
            RV32_MEMORY_AS as isize,                // e: load from heap
            1,                                      // f: write enabled
            0,                                      // g: sign flag
        ));

        // 3. JALR x1, x5, 0 (call library, save return in x1)
        // The JALR automatically saves PC+4 to x1, library returns via: jalr x0, x1, 0
        // JALR rd, rs1, imm: rd = PC+4; PC = rs1 + imm
        instructions.push(Instruction::large_from_isize(
            Rv32JalrOpcode::JALR.global_opcode(),
            (1 * RV32_REGISTER_NUM_LIMBS) as isize, // a: rd = x1 (save return addr)
            (5 * RV32_REGISTER_NUM_LIMBS) as isize, // b: rs1 = x5 (base containing target)
            0,                                      // c: immediate offset
            1,                                      // d: rs1 is register
            0,                                      // e: unused
            1,                                      // f: write enabled (rd != 0)
            0,                                      // g: sign bit (imm >= 0)
        ));

        // We generate 9 OpenVM instructions from 1 RISC-V instruction:
        // 1. LUI x5, upper_20_bits(inst)                 - Load upper bits of instruction
        // 2. ADDI x5, x5, lower_12_bits(inst)            - Complete instruction in x5
        // 3. LUI x6, upper_20_bits(FLOAT_INST_ADDR)      - Load upper bits of instruction address
        // 4. ADDI x6, x6, lower_12_bits(FLOAT_INST_ADDR) - Complete address in x6
        // 5. STOREW x5, 0(x6)                             - Store instruction (lower 32 bits)
        // 6. STOREW x0, 4(x6)                             - Store 0 (upper 32 bits, library expects uint64_t)
        // 7. LUI x5, 0x1E                                 - Load upper bits of library pointer address (0x1ec60)
        // 8. LW x5, 0(x5)                                 - Load library function pointer
        // 9. JALR x1, x5, 0                               - Call library function
        //
        // IMPORTANT: We return used_u32s=1 which means we only consume 1 RISC-V instruction,
        // but we generate 9 OpenVM instructions in its place. The VM will execute these
        // 9 instructions sequentially, but the transpiler will continue from the next
        // RISC-V instruction after this float op.
        Some(TranspilerOutput {
            instructions: instructions.into_iter().map(Some).collect(),
            used_u32s: 1,
        })
    }
}

impl<F: PrimeField32> TranspilerExtension<F> for FloatsTranspilerExtension {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<TranspilerOutput<F>> {
        if instruction_stream.is_empty() {
            return None;
        }

        let inst = instruction_stream[0];
        let opcode = (inst & 0x7f) as u8;

        let result = match opcode {
            FLOAD_OPCODE => {
                // FLW - Float Load Word
                eprintln!(
                    "[RV32F_TRANSPILER] Processing FLOAD instruction: 0x{:08x}",
                    inst
                );
                let output = self.handle_flw(inst);
                eprintln!("[RV32F_TRANSPILER] FLW returning: {:?}", output.is_some());
                if let Some(ref out) = output {
                    eprintln!(
                        "[RV32F_TRANSPILER] FLW generated {} OpenVM instructions",
                        out.instructions.len()
                    );
                }
                output
            }
            FSTORE_OPCODE => {
                // FSW - Float Store Word
                eprintln!(
                    "[RV32F_TRANSPILER] Processing FSTORE instruction: 0x{:08x}",
                    inst
                );
                let output = self.handle_fsw(inst);
                eprintln!("[RV32F_TRANSPILER] FSW returning: {:?}", output.is_some());
                if let Some(ref out) = output {
                    eprintln!(
                        "[RV32F_TRANSPILER] FSW generated {} OpenVM instructions",
                        out.instructions.len()
                    );
                }
                output
            }
            FP_OPCODE => {
                // FADD.S, FSUB.S, FMUL.S, FDIV.S, etc.
                // Check fmt field (bits 25-26) - should be 00 for single precision
                let fmt = (inst >> 25) & 0x3;
                if fmt != 0 {
                    return None; // Not single precision
                }

                eprintln!(
                    "[RV32F_TRANSPILER] Processing FLOAT ALU instruction: 0x{:08x}",
                    inst
                );
                let output = self.handle_float_alu(inst);
                eprintln!(
                    "[RV32F_TRANSPILER] FLOAT ALU returning: {:?}",
                    output.is_some()
                );
                if let Some(ref out) = output {
                    eprintln!(
                        "[RV32F_TRANSPILER] FLOAT ALU generated {} OpenVM instructions",
                        out.instructions.len()
                    );
                }
                output
            }
            FMADD_OPCODE | FMSUB_OPCODE | FNMSUB_OPCODE | FNMADD_OPCODE => {
                // Fused multiply-add variants
                eprintln!(
                    "[RV32F_TRANSPILER] Processing FLOAT fused instruction: 0x{:08x}",
                    inst
                );
                let output = self.handle_float_alu(inst);
                eprintln!(
                    "[RV32F_TRANSPILER] FLOAT fused returning: {:?}",
                    output.is_some()
                );
                if let Some(ref out) = output {
                    eprintln!(
                        "[RV32F_TRANSPILER] FLOAT fused generated {} OpenVM instructions",
                        out.instructions.len()
                    );
                }
                output
            }
            _ => None, // Not a float instruction
        };

        if result.is_none()
            && (opcode == FLOAD_OPCODE || opcode == FSTORE_OPCODE || opcode == FP_OPCODE)
        {
            eprintln!(
                "[RV32F_TRANSPILER] WARNING: Float opcode 0x{:02x} inst 0x{:08x} returned None!",
                opcode, inst
            );
        }

        result
    }
}
