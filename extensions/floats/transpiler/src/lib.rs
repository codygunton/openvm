use openvm_instructions::{instruction::Instruction, riscv::RV32_REGISTER_NUM_LIMBS, LocalOpcode};
use openvm_stark_backend::p3_field::PrimeField32;
use openvm_transpiler::{TranspilerExtension, TranspilerOutput};
use rrs_lib::instruction_formats::{IType, SType};

mod opcodes;
pub use opcodes::FloatOpcode;

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

#[derive(Default, Clone, Copy)]
pub struct FloatsTranspilerExtension;

impl FloatsTranspilerExtension {
    pub fn new() -> Self {
        Self
    }

    fn handle_flw<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let dec = IType::new(inst);

        // Extract operands
        let rd = dec.rd;      // Float destination register
        let rs1 = dec.rs1;    // Base address register
        let imm = dec.imm;    // Offset

        // Emit single FLW instruction
        let instruction = Instruction::from_isize(
            FloatOpcode::FLW.global_opcode(),
            rd as isize,                               // a: float register index
            (rs1 as isize) * (RV32_REGISTER_NUM_LIMBS as isize), // b: base register index * 4
            imm as isize,                              // c: immediate offset
            0,                                         // d: unused
            0,                                         // e: unused
        );

        Some(TranspilerOutput {
            instructions: vec![Some(instruction)],
            used_u32s: 1,
        })
    }

    fn handle_fsw<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        let dec = SType::new(inst);

        // Extract operands
        let rs2 = dec.rs2;    // Float source register
        let rs1 = dec.rs1;    // Base address register
        let imm = dec.imm;    // Offset

        // Emit single FSW instruction
        let instruction = Instruction::from_isize(
            FloatOpcode::FSW.global_opcode(),
            rs2 as isize,                              // a: float register index
            (rs1 as isize) * (RV32_REGISTER_NUM_LIMBS as isize), // b: base register index * 4
            imm as isize,                              // c: immediate offset
            0,                                         // d: unused
            0,                                         // e: unused
        );

        Some(TranspilerOutput {
            instructions: vec![Some(instruction)],
            used_u32s: 1,
        })
    }

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
        );

        Some(TranspilerOutput {
            instructions: vec![Some(instruction)],
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

        match opcode {
            FLOAD_OPCODE => {
                // FLW - Float Load Word
                self.handle_flw(inst)
            }
            FSTORE_OPCODE => {
                // FSW - Float Store Word
                self.handle_fsw(inst)
            }
            FP_OPCODE => {
                // FADD.S, FSUB.S, FMUL.S, FDIV.S, etc.
                // Check fmt field (bits 25-26) - should be 00 for single precision
                let fmt = (inst >> 25) & 0x3;
                if fmt != 0 {
                    return None; // Not single precision
                }

                self.handle_float_alu(inst)
            }
            FMADD_OPCODE | FMSUB_OPCODE | FNMSUB_OPCODE | FNMADD_OPCODE => {
                // Fused multiply-add variants
                self.handle_float_alu(inst)
            }
            _ => None, // Not a float instruction
        }
    }
}
