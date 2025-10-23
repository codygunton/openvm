use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;

use super::core::FloatMoveExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct FloatMovePreCompute {
    rd: u8,        // Destination register
    rs1: u8,       // Source register
    direction: u8, // 0=float→int (FMV.X.W), 1=int→float (FMV.W.X)
}

impl FloatMoveExecutor {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut FloatMovePreCompute,
    ) -> Result<bool, StaticProgramError> {
        // The opcode is in field `d` and tells us which FMV instruction this is
        // FMVXW (0x10) = float→int, direction=0
        // FMVWX (0x11) = int→float, direction=1
        let opcode = inst.d.as_canonical_u32() as u8;
        let direction = if opcode == 0x10 { 0 } else { 1 };

        *data = FloatMovePreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            direction,
        };
        Ok(true)
    }
}

macro_rules! dispatch {
    ($execute_impl:ident, $enabled:ident) => {
        if $enabled {
            Ok($execute_impl::<_, _, true>)
        } else {
            Ok($execute_impl::<_, _, false>)
        }
    };
}

impl<F> Executor<F> for FloatMoveExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatMovePreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatMovePreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatMoveExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatMovePreCompute>>()
    }

    fn metered_pre_compute<Ctx>(
        &self,
        chip_idx: usize,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError>
    where
        Ctx: MeteredExecutionCtxTrait,
    {
        let data: &mut E2PreCompute<FloatMovePreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &FloatMovePreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    eprintln!(
        "[FMOVE-ENTRY] PC=0x{:08x}, instret={}, direction={}, ENABLED={}",
        *pc, *instret, pre_compute.direction, ENABLED
    );

    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // Implement FMV directly without using the handler
    // FMV.X.W (direction=0): Move float register to integer register (bitwise copy)
    // FMV.W.X (direction=1): Move integer register to float register (bitwise copy)

    if pre_compute.direction == 0 {
        // FMV.X.W: float -> integer (bitwise copy)
        let f_addr = float_reg_addr(pre_compute.rs1);
        let f_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f_addr);
        exec_state.vm_write(RV32_REGISTER_AS, pre_compute.rd as u32 * 4, &f_bytes);
        let bits = u32::from_le_bytes(f_bytes);
        eprintln!("[FMV.X.W] f{} (0x{:08x}) -> x{}", pre_compute.rs1, bits, pre_compute.rd);
    } else {
        // FMV.W.X: integer -> float (bitwise copy)
        let i_bytes = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, pre_compute.rs1 as u32 * 4);
        let f_addr = float_reg_addr(pre_compute.rd);
        exec_state.vm_write(FLOAT_MEM_AS, f_addr, &i_bytes);
        let bits = u32::from_le_bytes(i_bytes);
        eprintln!("[FMV.W.X] x{} (0x{:08x}) -> f{}", pre_compute.rs1, bits, pre_compute.rd);
    }

    *pc += DEFAULT_PC_STEP;
    *instret += 1;
}

#[create_handler]
#[inline(always)]
unsafe fn execute_e1_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &[u8],
    instret: &mut u64,
    pc: &mut u32,
    _instret_end: u64,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let pre_compute: &FloatMovePreCompute = pre_compute.borrow();
    execute_e12_impl::<F, CTX, ENABLED>(pre_compute, instret, pc, exec_state);
}

#[create_handler]
#[inline(always)]
unsafe fn execute_e2_impl<F: PrimeField32, CTX: MeteredExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &[u8],
    instret: &mut u64,
    pc: &mut u32,
    _arg: u64,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let pre_compute: &E2PreCompute<FloatMovePreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
