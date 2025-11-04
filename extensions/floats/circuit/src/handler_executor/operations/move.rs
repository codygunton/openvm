use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, riscv::RV32_REGISTER_AS};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;
use crate::handler_executor::operation::FloatOperation;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct MovePreCompute {
    pub rd: u8,
    pub rs1: u8,
    pub direction: u8,  // 0=float→int (FMV.X.W), 1=int→float (FMV.W.X)
}

#[derive(Clone, Copy)]
pub struct MoveOp;

impl FloatOperation for MoveOp {
    type PreCompute = MovePreCompute;

    fn extract_fields<F: PrimeField32>(
        inst: &Instruction<F>,
        data: &mut Self::PreCompute,
    ) -> Result<bool, StaticProgramError> {
        // The opcode is in field `d` and tells us which FMV instruction this is
        // FMVXW (0x10) = float→int, direction=0
        // FMVWX (0x11) = int→float, direction=1
        let opcode = inst.d.as_canonical_u32() as u8;
        let direction = if opcode == 0x10 { 0 } else { 1 };

        *data = MovePreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            direction,
        };
        Ok(true)
    }

    fn reconstruct_riscv_instruction(data: &Self::PreCompute) -> u32 {
        let funct7 = if data.direction == 0 { 0x70 } else { 0x78 };

        // R-type: funct7[31:25] | 0[24:20] | rs1[19:15] | 0[14:12] | rd[11:7] | opcode[6:0]
        (funct7 << 25) |
        ((data.rs1 as u32) << 15) |
        ((data.rd as u32) << 7) |
        0x53
    }

    unsafe fn prepare_for_handler<F: PrimeField32, CTX: ExecutionCtxTrait>(
        data: &Self::PreCompute,
        exec_state: &mut VmExecState<F, GuestMemory, CTX>,
    ) {
        // For int→float move, copy integer source to FLOAT_X0_BACKUP
        if data.direction == 1 {
            let src_int_value = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, data.rs1 as u32 * 4);
            let backup_addr = FLOAT_X0_BACKUP + (data.rs1 as u32 * 8);
            exec_state.vm_write(FLOAT_MEM_AS, backup_addr, &src_int_value);
        }
    }
}
