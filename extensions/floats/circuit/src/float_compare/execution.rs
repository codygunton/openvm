use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;

use super::core::FloatCompareExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct FloatComparePreCompute {
    rd: u8,        // Destination integer register
    rs1: u8,       // Source float register 1
    rs2: u8,       // Source float register 2
    comp_type: u8, // Comparison type: 0=FLE, 1=FLT, 2=FEQ
}

impl FloatCompareExecutor {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut FloatComparePreCompute,
    ) -> Result<bool, StaticProgramError> {
        *data = FloatComparePreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            rs2: inst.c.as_canonical_u32() as u8,
            comp_type: inst.d.as_canonical_u32() as u8,
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

impl<F> Executor<F> for FloatCompareExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatComparePreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatComparePreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatCompareExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatComparePreCompute>>()
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
        let data: &mut E2PreCompute<FloatComparePreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &FloatComparePreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // Implement compare directly without using the handler
    // This is needed because the handler writes to a backup location that doesn't
    // get copied back to the actual integer register file

    let f1_addr = float_reg_addr(pre_compute.rs1);
    let f2_addr = float_reg_addr(pre_compute.rs2);
    let f1_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f1_addr);
    let f2_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f2_addr);
    let f1_val = f32::from_le_bytes(f1_bytes);
    let f2_val = f32::from_le_bytes(f2_bytes);

    let result = match pre_compute.comp_type {
        0 => (f1_val <= f2_val) as u32, // FLE.S
        1 => (f1_val < f2_val) as u32,  // FLT.S
        2 => (f1_val == f2_val) as u32, // FEQ.S
        _ => 0,
    };

    exec_state.vm_write(RV32_REGISTER_AS, pre_compute.rd as u32 * 4, &result.to_le_bytes());

    // Update FCSR flags based on comparison type
    // Per RISC-V spec:
    // - FLE/FLT (comp_type 0,1): Set invalid flag for ANY NaN
    // - FEQ (comp_type 2): Set invalid flag ONLY for signaling NaN
    let should_set_invalid = match pre_compute.comp_type {
        0 | 1 => {
            // FLE/FLT: Set flag for any NaN
            f1_val.is_nan() || f2_val.is_nan()
        }
        2 => {
            // FEQ: Set flag only for signaling NaN
            // A signaling NaN has the high bit of mantissa (bit 22) = 0
            let f1_bits = f1_val.to_bits();
            let f2_bits = f2_val.to_bits();
            let is_f1_snan = f1_val.is_nan() && ((f1_bits & 0x00400000) == 0);
            let is_f2_snan = f2_val.is_nan() && ((f2_bits & 0x00400000) == 0);
            is_f1_snan || is_f2_snan
        }
        _ => false,
    };

    if should_set_invalid {
        // Read current FCSR
        let fcsr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_CSR_FCSR);
        let mut fcsr = u32::from_le_bytes(fcsr_bytes);

        // Set NV (Invalid Operation) flag - bit 4 (0x10)
        fcsr |= 0x10;

        // Write back updated FCSR
        exec_state.vm_write(FLOAT_MEM_AS, FLOAT_CSR_FCSR, &fcsr.to_le_bytes());
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
    let pre_compute: &FloatComparePreCompute = pre_compute.borrow();
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
    let pre_compute: &E2PreCompute<FloatComparePreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
