use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;

use super::core::FloatClassExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct FloatClassPreCompute {
    rd: u8,   // Destination integer register
    rs1: u8,  // Source float register
}

impl FloatClassExecutor {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut FloatClassPreCompute,
    ) -> Result<bool, StaticProgramError> {
        *data = FloatClassPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
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

impl<F> Executor<F> for FloatClassExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatClassPreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatClassPreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatClassExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatClassPreCompute>>()
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
        let data: &mut E2PreCompute<FloatClassPreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &FloatClassPreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    eprintln!(
        "[FCLASS-ENTRY] PC=0x{:08x}, instret={}, ENABLED={}",
        *pc, *instret, ENABLED
    );

    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // Implement FCLASS directly without using handler (handler writes to backup location)
    // FCLASS.S returns a 10-bit mask indicating the class of the floating-point value:
    // Bit 0: Negative infinity
    // Bit 1: Negative normal number
    // Bit 2: Negative subnormal number
    // Bit 3: Negative zero
    // Bit 4: Positive zero
    // Bit 5: Positive subnormal number
    // Bit 6: Positive normal number
    // Bit 7: Positive infinity
    // Bit 8: Signaling NaN
    // Bit 9: Quiet NaN

    let f_addr = float_reg_addr(pre_compute.rs1);
    let f_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f_addr);
    let f_val = f32::from_le_bytes(f_bytes);
    let bits = u32::from_le_bytes(f_bytes);

    let result: u32 = if f_val.is_nan() {
        // Check if signaling NaN (bit 22 is 0 for signaling)
        if (bits & 0x00400000) == 0 {
            1u32 << 8  // Signaling NaN
        } else {
            1u32 << 9  // Quiet NaN
        }
    } else if f_val.is_infinite() {
        if f_val.is_sign_negative() {
            1u32 << 0  // Negative infinity
        } else {
            1u32 << 7  // Positive infinity
        }
    } else if f_val == 0.0 {
        if f_val.is_sign_negative() {
            1u32 << 3  // Negative zero
        } else {
            1u32 << 4  // Positive zero
        }
    } else {
        // Check if subnormal (exponent is 0)
        let exponent = (bits >> 23) & 0xFF;
        if exponent == 0 {
            if f_val.is_sign_negative() {
                1u32 << 2  // Negative subnormal
            } else {
                1u32 << 5  // Positive subnormal
            }
        } else {
            // Normal number
            if f_val.is_sign_negative() {
                1u32 << 1  // Negative normal
            } else {
                1u32 << 6  // Positive normal
            }
        }
    };

    exec_state.vm_write(RV32_REGISTER_AS, pre_compute.rd as u32 * 4, &result.to_le_bytes());

    eprintln!("[FCLASS.S] f{} (val={}, bits=0x{:08x}) -> x{} (class=0x{:x})",
              pre_compute.rs1, f_val, bits, pre_compute.rd, result);

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
    let pre_compute: &FloatClassPreCompute = pre_compute.borrow();
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
    let pre_compute: &E2PreCompute<FloatClassPreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
