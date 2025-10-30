use openvm_circuit::arch::*;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::instruction::Instruction;
use openvm_stark_backend::p3_field::PrimeField32;

use crate::handler_executor::operation::FloatOperation;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct ClassPreCompute {
    pub rd: u8,
    pub rs1: u8,
}

#[derive(Clone, Copy)]
pub struct ClassOp;

impl FloatOperation for ClassOp {
    type PreCompute = ClassPreCompute;

    fn extract_fields<F: PrimeField32>(
        inst: &Instruction<F>,
        data: &mut Self::PreCompute,
    ) -> Result<bool, StaticProgramError> {
        *data = ClassPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
        };
        Ok(true)
    }

    fn reconstruct_riscv_instruction(data: &Self::PreCompute) -> u32 {
        // R-type: funct7[31:25] | 0[24:20] | rs1[19:15] | funct3[14:12] | rd[11:7] | opcode[6:0]
        (0x70 << 25) |
        ((data.rs1 as u32) << 15) |
        (0x01 << 12) |  // funct3=1 for FCLASS
        ((data.rd as u32) << 7) |
        0x53
    }
}
