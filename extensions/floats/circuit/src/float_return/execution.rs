use std::borrow::BorrowMut;
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{
    instruction::Instruction, program::DEFAULT_PC_STEP, riscv::RV32_REGISTER_AS,
};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;

use super::core::FloatReturnExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct FloatReturnPreCompute {
    // No pre-computed data needed - all restoration happens at execution time
}

impl FloatReturnExecutor {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        _inst: &Instruction<F>,
        _data: &mut FloatReturnPreCompute,
    ) -> Result<bool, StaticProgramError> {
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

impl<F> Executor<F> for FloatReturnExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatReturnPreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatReturnPreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatReturnExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatReturnPreCompute>>()
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
        let data: &mut E2PreCompute<FloatReturnPreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    _pre_compute: &FloatReturnPreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }
    // Restore ALL integer registers x1-x31 from FLOAT_X0_BACKUP
    // The handler may have written integer results there for int-writing ops
    for reg in 1..32 {
        let backup_addr = FLOAT_X0_BACKUP + (reg * 8); // 8-byte aligned storage
        let reg_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, backup_addr);
        exec_state.vm_write(RV32_REGISTER_AS, reg * 4, &reg_bytes);
    }

    // Read return address from saved location (x1 was clobbered by library)
    let return_addr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_RETURN_ADDR);
    let return_addr = u32::from_le_bytes(return_addr_bytes);

    eprintln!("  Returning to PC=0x{:08x}", return_addr);
    *pc = return_addr;
    *instret += 1;
}

#[create_handler]
#[inline(always)]
unsafe fn execute_e1_handler<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &[u8],
    instret: &mut u64,
    pc: &mut u32,
    _instret_end: u64,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let pre_compute = unsafe { &*(pre_compute.as_ptr() as *const FloatReturnPreCompute) };
    execute_e12_impl::<F, CTX, ENABLED>(pre_compute, instret, pc, exec_state);
}

#[create_handler]
#[inline(always)]
unsafe fn execute_e2_handler<
    F: PrimeField32,
    CTX: MeteredExecutionCtxTrait,
    const ENABLED: bool,
>(
    pre_compute: &[u8],
    instret: &mut u64,
    pc: &mut u32,
    _arg: u64,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let pre_compute =
        unsafe { &*(pre_compute.as_ptr() as *const E2PreCompute<FloatReturnPreCompute>) };
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
