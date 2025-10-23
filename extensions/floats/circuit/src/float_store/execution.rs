use std::{
    borrow::{Borrow, BorrowMut},
    mem::size_of,
};

use openvm_circuit::{arch::*, system::memory::online::GuestMemory};
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;

use super::core::FloatStoreExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
struct FloatStorePreCompute {
    rs2: u8,          // Float source register (0-31)
    rs1: u8,          // Base address register
    _padding: [u8; 2],
    imm: i32,         // Signed offset
}

impl FloatStoreExecutor {
    /// Return true (always enabled for FSW).
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut FloatStorePreCompute,
    ) -> Result<bool, StaticProgramError> {
        // Note: inst.b contains rs1 * RV32_REGISTER_NUM_LIMBS (from transpiler)
        // Reconstruct signed immediate from fields c and g (same pattern as rv32im):
        // - Field c contains lower 16 bits
        // - Field g contains sign bit
        let imm_lower = inst.c.as_canonical_u32();
        let imm_sign = inst.g.as_canonical_u32();
        let imm = imm_lower + imm_sign * 0xffff0000;

        *data = FloatStorePreCompute {
            rs2: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8, // This is rs1 * 4, used directly as address
            _padding: [0; 2],
            imm: imm as i32,
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

impl<F> Executor<F> for FloatStoreExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatStorePreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatStorePreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatStoreExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatStorePreCompute>>()
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
        let data: &mut E2PreCompute<FloatStorePreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &FloatStorePreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // 1. Read from float register memory
    let float_addr = float_reg_addr(pre_compute.rs2);
    let word_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, float_addr);

    // 2. Read base address from rs1 register (use register number directly)
    let base_bytes = exec_state.vm_read::<u8, 4>(
        RV32_REGISTER_AS,
        pre_compute.rs1 as u32,
    );
    let base_addr = u32::from_le_bytes(base_bytes);

    // 3. Calculate effective address
    let addr = base_addr.wrapping_add(pre_compute.imm as u32);

    // 4. Store word to heap memory
    exec_state.vm_write(FLOAT_MEM_AS, addr, &word_bytes);

    // 5. Update PC and instruction counter
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
    let pre_compute: &FloatStorePreCompute = pre_compute.borrow();
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
    let pre_compute: &E2PreCompute<FloatStorePreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
