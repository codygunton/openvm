use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, riscv::RV32_REGISTER_AS};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;
use crate::handler_executor::operation::FloatOperation;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct ConvertPreCompute {
    pub rd: u8,
    pub rs1: u8,
    pub unsigned_flag: u8,  // 0=signed, 1=unsigned
    pub rm: u8,
    pub direction: u8,      // 0=float→int, 1=int→float
}

#[derive(Clone, Copy)]
pub struct ConvertOp;

impl FloatOperation for ConvertOp {
    type PreCompute = ConvertPreCompute;

    fn extract_fields<F: PrimeField32>(
        inst: &Instruction<F>,
        data: &mut Self::PreCompute,
    ) -> Result<bool, StaticProgramError> {
        let opcode_idx = inst.opcode.local_opcode_idx(0x300) as u8;
        let direction = if opcode_idx == 0x0D { 0 } else { 1 }; // 0x0D (FCVTWS) = float→int, 0x0E (FCVTSW) = int→float

        *data = ConvertPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            unsigned_flag: inst.c.as_canonical_u32() as u8,
            rm: inst.d.as_canonical_u32() as u8,
            direction,
        };
        Ok(true)
    }

    fn reconstruct_riscv_instruction(data: &Self::PreCompute) -> u32 {
        let funct7 = if data.direction == 0 { 0x60 } else { 0x68 };
        let rs2 = data.unsigned_flag;  // Encodes signed/unsigned in rs2 field

        // R-type: funct7[31:25] | rs2[24:20] | rs1[19:15] | rm[14:12] | rd[11:7] | opcode[6:0]
        (funct7 << 25) |
        ((rs2 as u32) << 20) |
        ((data.rs1 as u32) << 15) |
        ((data.rm as u32) << 12) |
        ((data.rd as u32) << 7) |
        0x53
    }

    unsafe fn prepare_for_handler<F: PrimeField32, CTX: ExecutionCtxTrait>(
        data: &Self::PreCompute,
        exec_state: &mut VmExecState<F, GuestMemory, CTX>,
    ) {
        // For int→float conversion, copy integer source to FLOAT_X0_BACKUP
        if data.direction == 1 {
            let src_int_value = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, data.rs1 as u32 * 4);
            let backup_addr = FLOAT_X0_BACKUP + (data.rs1 as u32 * 8);
            exec_state.vm_write(FLOAT_MEM_AS, backup_addr, &src_int_value);
        }
    }
}
