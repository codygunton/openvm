use std::borrow::BorrowMut;
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP, riscv::RV32_REGISTER_AS};
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

    // Read the float instruction to check if it writes to an integer register
    let inst_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_INST_ADDR);
    let inst = u32::from_le_bytes(inst_bytes);

    // Extract fields from instruction
    let opcode = inst & 0x7F;
    let rd = (inst >> 7) & 0x1F;
    let rs1 = (inst >> 15) & 0x1F;
    let rs2 = (inst >> 20) & 0x1F;
    let funct7 = (inst >> 25) & 0x7F;


    // Check if this operation writes to an integer register
    // Float operations that write to integer registers:
    // - funct7=0x50: FLE.S, FLT.S, FEQ.S (comparisons)
    // - funct7=0x60: FCVT.W.S, FCVT.WU.S (float→int conversion)
    // - funct7=0x70: FCLASS.S, FMV.X.W (classification, move)
    let writes_to_int_reg =
        opcode == 0x53 && (funct7 == 0x50 || funct7 == 0x60 || funct7 == 0x70) && rd != 0;

    // Restore x1 (return address register) first
    // x1 may be used as the signature base register by tests, so we must restore it
    let saved_x1 = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_SAVED_X1);
    exec_state.vm_write(RV32_REGISTER_AS, 1 * 4, &saved_x1);

    // Restore caller-saved registers: t0-t2 (x5-x7), a0-a7 (x10-x17), t3-t6 (x28-x31)
    // IMPORTANT: Skip restoring rd if it will receive an integer result,
    // otherwise we'd write twice (restore old value, then copy new result)
    let saved_regs = [5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17, 28, 29, 30, 31];
    for (i, &reg) in saved_regs.iter().enumerate() {
        // Skip restoring this register if it's the destination of an int result operation
        if writes_to_int_reg && reg as u32 == rd {
            continue;
        }
        let reg_bytes =
            exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_SAVED_REGS_BASE + (i as u32 * 4));
        exec_state.vm_write(RV32_REGISTER_AS, reg as u32 * 4, &reg_bytes);
    }

    // Copy integer result if this instruction writes to an integer register
    if writes_to_int_reg {
        // Copy result from integer register backup to actual integer register file
        // Handler writes to FLOAT_X0_BACKUP + (rd * 8) as 8-byte aligned storage
        let backup_addr = FLOAT_X0_BACKUP + (rd * 8);
        let result_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, backup_addr);
        exec_state.vm_write(RV32_REGISTER_AS, rd * 4, &result_bytes);
    }

    // Read return address from saved location (x1 was clobbered by library)
    let return_addr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_RETURN_ADDR);
    let return_addr = u32::from_le_bytes(return_addr_bytes);

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
unsafe fn execute_e2_handler<F: PrimeField32, CTX: MeteredExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &[u8],
    instret: &mut u64,
    pc: &mut u32,
    _arg: u64,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let pre_compute = unsafe { &*(pre_compute.as_ptr() as *const E2PreCompute<FloatReturnPreCompute>) };
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
