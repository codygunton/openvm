use openvm_circuit::arch::*;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

use crate::handler_executor::operation::FloatOperation;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct FmaPreCompute {
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub rs3: u8,
    pub rm: u8,
    pub opcode: u8,  // 9=FMADD, 10=FMSUB, 11=FNMSUB, 12=FNMADD
}

#[derive(Clone, Copy)]
pub struct FmaOp;

impl FloatOperation for FmaOp {
    type PreCompute = FmaPreCompute;

    fn extract_fields<F: PrimeField32>(
        inst: &Instruction<F>,
        data: &mut Self::PreCompute,
    ) -> Result<bool, StaticProgramError> {
        *data = FmaPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            rs2: inst.c.as_canonical_u32() as u8,
            rs3: inst.d.as_canonical_u32() as u8,
            rm: inst.e.as_canonical_u32() as u8,
            opcode: inst.opcode.local_opcode_idx(0x300) as u8,
        };
        Ok(true)
    }

    fn reconstruct_riscv_instruction(data: &Self::PreCompute) -> u32 {
        let opcode = match data.opcode {
            9 => 0x43,   // FMADD.S
            10 => 0x47,  // FMSUB.S
            11 => 0x4B,  // FNMSUB.S
            12 => 0x4F,  // FNMADD.S
            _ => 0x43,
        };

        // R4-type: rs3[31:27] | funct2[26:25] | rs2[24:20] | rs1[19:15] | rm[14:12] | rd[11:7] | opcode[6:0]
        ((data.rs3 as u32) << 27) |
        (0 << 25) |  // funct2=0 for single precision
        ((data.rs2 as u32) << 20) |
        ((data.rs1 as u32) << 15) |
        ((data.rm as u32) << 12) |
        ((data.rd as u32) << 7) |
        opcode
    }
}
