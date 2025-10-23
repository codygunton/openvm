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

// Memory map (must match circuit constants)
// Placed at 2MB to provide space for test code while staying well within 512MB limit
pub const FLOAT_REGISTER_BASE: u32 = 0x00200000;
pub const FLOAT_INST_ADDR: u32 = 0x00200108;
pub const FLOAT_LIB_ENTRY_PTR: u32 = 0x00100000;

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

    fn handle_float_compare<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        // Decode RISC-V instruction format (R-type)
        let rd = ((inst >> 7) & 0x1F) as u8;
        let rs1 = ((inst >> 15) & 0x1F) as u8;
        let rs2 = ((inst >> 20) & 0x1F) as u8;
        let funct3 = ((inst >> 12) & 0x7) as u8;

        // Map funct3 to comparison type
        let comp_type = match funct3 {
            0 => 0,  // FLE.S
            1 => 1,  // FLT.S
            2 => 2,  // FEQ.S
            _ => return None,  // Invalid comparison funct3
        };

        // Emit float compare instruction
        let instruction = Instruction::from_isize(
            FloatOpcode::FCMP.global_opcode(),
            rd as isize,                  // a: destination integer register
            rs1 as isize,                 // b: source float register 1
            rs2 as isize,                 // c: source float register 2
            comp_type as isize,           // d: comparison type (0=FLE, 1=FLT, 2=FEQ)
            0,                            // e: unused
        );

        Some(TranspilerOutput {
            instructions: vec![Some(instruction)],
            used_u32s: 1,
        })
    }

    fn handle_float_convert<F: PrimeField32>(&self, inst: u32, funct7: u8) -> Option<TranspilerOutput<F>> {
        // Decode RISC-V instruction format (R-type with rs2 as conversion type)
        let rd = ((inst >> 7) & 0x1F) as u8;
        let rm = ((inst >> 12) & 0x7) as u8;  // Rounding mode
        let rs1 = ((inst >> 15) & 0x1F) as u8;
        let rs2 = ((inst >> 20) & 0x1F) as u8;

        // rs2 encodes whether conversion is signed (0) or unsigned (1)
        let unsigned_flag = rs2 & 1;

        // Determine direction and opcode based on funct7
        let opcode = match funct7 {
            0x60 => FloatOpcode::FCVTWS,  // Float to Int (FCVT.W.S or FCVT.WU.S)
            0x68 => FloatOpcode::FCVTSW,  // Int to Float (FCVT.S.W or FCVT.S.WU)
            _ => return None,
        };

        // Emit float conversion instruction
        // Operand encoding: a=rd, b=rs1, c=unsigned_flag, d=rm
        let instruction = Instruction::from_isize(
            opcode.global_opcode(),
            rd as isize,                  // a: destination register
            rs1 as isize,                 // b: source register
            unsigned_flag as isize,       // c: unsigned flag (0=signed, 1=unsigned)
            rm as isize,                  // d: rounding mode
            0,                            // e: unused
        );

        Some(TranspilerOutput {
            instructions: vec![Some(instruction)],
            used_u32s: 1,
        })
    }

    fn handle_float_fma<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        // Decode RISC-V R4-type instruction format
        // R4-type: rs3[31:27], funct2[26:25], rs2[24:20], rs1[19:15], rm[14:12], rd[11:7], opcode[6:0]
        let opcode = (inst & 0x7F) as u8;
        let rd = ((inst >> 7) & 0x1F) as u8;
        let rm = ((inst >> 12) & 0x7) as u8;
        let rs1 = ((inst >> 15) & 0x1F) as u8;
        let rs2 = ((inst >> 20) & 0x1F) as u8;
        let funct2 = ((inst >> 25) & 0x3) as u8;
        let rs3 = ((inst >> 27) & 0x1F) as u8;

        // Verify funct2=0 for single precision (.S suffix)
        if funct2 != 0 {
            return None;  // Not single precision
        }

        // Map RISC-V opcode to FloatOpcode
        let float_opcode = match opcode {
            0x43 => FloatOpcode::FMADD,   // FMADD.S: (rs1 * rs2) + rs3
            0x47 => FloatOpcode::FMSUB,   // FMSUB.S: (rs1 * rs2) - rs3
            0x4B => FloatOpcode::FNMSUB,  // FNMSUB.S: -(rs1 * rs2) + rs3
            0x4F => FloatOpcode::FNMADD,  // FNMADD.S: -(rs1 * rs2) - rs3
            _ => return None,  // Invalid R4-type opcode
        };

        // Emit R4-type float FMA instruction
        // Operand encoding: a=rd, b=rs1, c=rs2, d=rs3, e=rm
        let instruction = Instruction::from_isize(
            float_opcode.global_opcode(),
            rd as isize,                  // a: destination float register
            rs1 as isize,                 // b: source float register 1 (multiplicand)
            rs2 as isize,                 // c: source float register 2 (multiplier)
            rs3 as isize,                 // d: source float register 3 (addend)
            rm as isize,                  // e: rounding mode
        );

        Some(TranspilerOutput {
            instructions: vec![Some(instruction)],
            used_u32s: 1,
        })
    }

    fn handle_float_alu<F: PrimeField32>(&self, inst: u32) -> Option<TranspilerOutput<F>> {
        // Decode RISC-V instruction format (R-type)
        let rd = ((inst >> 7) & 0x1F) as u8;
        let funct3 = ((inst >> 12) & 0x7) as u8;
        let rs1 = ((inst >> 15) & 0x1F) as u8;
        let rs2 = ((inst >> 20) & 0x1F) as u8;
        let funct7 = ((inst >> 25) & 0x7F) as u8;

        // Check if this is a comparison instruction
        if funct7 == 0x50 {
            return self.handle_float_compare(inst);
        }

        // Check if this is a conversion instruction
        if funct7 == 0x60 || funct7 == 0x68 {
            return self.handle_float_convert(inst, funct7);
        }

        // Map funct7 to FloatOpcode
        let opcode = match funct7 {
            0x00 => FloatOpcode::FADD,   // FADD.S
            0x04 => FloatOpcode::FSUB,   // FSUB.S
            0x08 => FloatOpcode::FMUL,   // FMUL.S
            0x0C => FloatOpcode::FDIV,   // FDIV.S
            0x2C => FloatOpcode::FSQRT,  // FSQRT.S (rs2 must be 0)
            0x10 => FloatOpcode::FSGNJ,  // FSGNJ.S/FSGNJN.S/FSGNJX.S (funct3: 0/1/2)
            0x14 => FloatOpcode::FMINMAX, // FMIN.S/FMAX.S (funct3: 0/1)
            0x70 => {
                // funct7=0x70 is shared between FMV.X.W and FCLASS.S
                // Distinguish by funct3: 0=FMV.X.W, 1=FCLASS.S
                match funct3 {
                    0 => FloatOpcode::FMVXW,   // FMV.X.W (float to int)
                    1 => FloatOpcode::FCLASS,  // FCLASS.S
                    _ => return None,  // Invalid funct3 for funct7=0x70
                }
            }
            0x78 => FloatOpcode::FMVWX,  // FMV.W.X (int to float)
            _ => return None,  // Unsupported operation
        };

        // Emit single float ALU instruction
        let instruction = Instruction::from_isize(
            opcode.global_opcode(),
            rd as isize,                  // a: destination float register
            rs1 as isize,                 // b: source float register 1
            rs2 as isize,                 // c: source float register 2
            opcode as usize as isize,     // d: opcode for executor dispatch
            funct3 as isize,              // e: funct3 for variant discrimination
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
                // Fused multiply-add variants (R4-type)
                self.handle_float_fma(inst)
            }
            _ => None, // Not a float instruction
        }
    }
}
