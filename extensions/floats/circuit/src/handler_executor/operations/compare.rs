use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

use crate::handler_executor::operation::FloatOperation;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct ComparePreCompute {
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub comp_type: u8,  // 0=FLE, 1=FLT, 2=FEQ
}

#[derive(Clone, Copy)]
pub struct CompareOp;

impl FloatOperation for CompareOp {
    type PreCompute = ComparePreCompute;

    fn extract_fields<F: PrimeField32>(
        inst: &Instruction<F>,
        data: &mut Self::PreCompute,
    ) -> Result<bool, StaticProgramError> {
        *data = ComparePreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            rs2: inst.c.as_canonical_u32() as u8,
            comp_type: inst.d.as_canonical_u32() as u8,
        };
        Ok(true)
    }

    fn reconstruct_riscv_instruction(data: &Self::PreCompute) -> u32 {
        let funct7 = 0x50;
        let funct3 = data.comp_type;  // 0=FLE, 1=FLT, 2=FEQ

        // R-type: funct7[31:25] | rs2[24:20] | rs1[19:15] | funct3[14:12] | rd[11:7] | opcode[6:0]
        ((funct7 as u32) << 25) |
        ((data.rs2 as u32) << 20) |
        ((data.rs1 as u32) << 15) |
        ((funct3 as u32) << 12) |
        ((data.rd as u32) << 7) |
        0x53
    }
}
