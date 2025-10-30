use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;
use crate::handler_executor::operation::FloatOperation;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct AluPreCompute {
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub opcode: u8,    // 2-8: FADD/FSUB/FMUL/FDIV/FSQRT/FMINMAX/FSGNJ
    pub variant: u8,   // rm or funct3 variant
}

#[derive(Clone, Copy)]
pub struct AluOp;

impl FloatOperation for AluOp {
    type PreCompute = AluPreCompute;

    fn extract_fields<F: PrimeField32>(
        inst: &Instruction<F>,
        data: &mut Self::PreCompute,
    ) -> Result<bool, StaticProgramError> {
        *data = AluPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            rs2: inst.c.as_canonical_u32() as u8,
            opcode: inst.d.as_canonical_u32() as u8,
            variant: inst.e.as_canonical_u32() as u8,
        };
        Ok(true)
    }

    fn reconstruct_riscv_instruction(data: &Self::PreCompute) -> u32 {
        let (funct7, funct3) = match data.opcode {
            2 => (0x00, data.variant),  // FADD.S
            3 => (0x04, data.variant),  // FSUB.S
            4 => (0x08, data.variant),  // FMUL.S
            5 => (0x0C, data.variant),  // FDIV.S
            6 => (0x2C, data.variant),  // FSQRT.S
            7 => (0x14, data.variant),  // FMIN.S/FMAX.S
            8 => (0x10, data.variant),  // FSGNJ.S/FSGNJN.S/FSGNJX.S
            _ => (0x00, 0),
        };

        // FSQRT uses rs2=0, all others use data.rs2
        let rs2_val = if data.opcode == 6 { 0 } else { data.rs2 };

        // R-type: funct7[31:25] | rs2[24:20] | rs1[19:15] | funct3[14:12] | rd[11:7] | opcode[6:0]
        ((funct7 as u32) << 25) |
        ((rs2_val as u32) << 20) |
        ((data.rs1 as u32) << 15) |
        ((funct3 as u32) << 12) |
        ((data.rd as u32) << 7) |
        0x53
    }
}
