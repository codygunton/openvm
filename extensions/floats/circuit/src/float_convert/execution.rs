use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;

use super::core::FloatConvertExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct FloatConvertPreCompute {
    rd: u8,            // Destination register
    rs1: u8,           // Source register 1
    unsigned_flag: u8, // 0=signed, 1=unsigned
    rm: u8,            // Rounding mode
    direction: u8,     // 0=float→int (0x30D), 1=int→float (0x30E)
}

impl FloatConvertExecutor {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut FloatConvertPreCompute,
    ) -> Result<bool, StaticProgramError> {
        let opcode_idx = inst.opcode.local_opcode_idx(0x300) as u8;
        let direction = if opcode_idx == 0x0D { 0 } else { 1 }; // 0x0D (FCVTWS) = float→int, 0x0E (FCVTSW) = int→float

        *data = FloatConvertPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            unsigned_flag: inst.c.as_canonical_u32() as u8,
            rm: inst.d.as_canonical_u32() as u8,
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

impl<F> Executor<F> for FloatConvertExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatConvertPreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatConvertPreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatConvertExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatConvertPreCompute>>()
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
        let data: &mut E2PreCompute<FloatConvertPreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &FloatConvertPreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    eprintln!(
        "[FCVT-ENTRY] PC=0x{:08x}, instret={}, direction={}, unsigned={}, ENABLED={}",
        *pc, *instret, pre_compute.direction, pre_compute.unsigned_flag, ENABLED
    );

    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // Implement FCVT directly without using the handler
    // This is needed because the handler writes to a backup location that doesn't
    // get copied back to the actual integer register file

    match (pre_compute.direction, pre_compute.unsigned_flag) {
        (0, 0) => { // FCVT.W.S (float to signed int)
            let f_addr = float_reg_addr(pre_compute.rs1);
            let f_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f_addr);
            let f_val = f32::from_le_bytes(f_bytes);
            let i_val = f_val as i32;
            exec_state.vm_write(RV32_REGISTER_AS, pre_compute.rd as u32 * 4, &i_val.to_le_bytes());
            eprintln!("[FCVT.W.S] f{} ({}=0x{:08x}) -> x{} ({}=0x{:08x})",
                      pre_compute.rs1, f_val, f_val.to_bits(), pre_compute.rd, i_val, i_val as u32);
        }
        (0, 1) => { // FCVT.WU.S (float to unsigned int)
            let f_addr = float_reg_addr(pre_compute.rs1);
            let f_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f_addr);
            let f_val = f32::from_le_bytes(f_bytes);
            let u_val = f_val as u32;
            exec_state.vm_write(RV32_REGISTER_AS, pre_compute.rd as u32 * 4, &u_val.to_le_bytes());
            eprintln!("[FCVT.WU.S] f{} ({}=0x{:08x}) -> x{} ({})",
                      pre_compute.rs1, f_val, f_val.to_bits(), pre_compute.rd, u_val);
        }
        (1, 0) => { // FCVT.S.W (signed int to float)
            let i_bytes = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, pre_compute.rs1 as u32 * 4);
            let i_val = i32::from_le_bytes(i_bytes);
            let f_val = i_val as f32;
            let f_addr = float_reg_addr(pre_compute.rd);
            exec_state.vm_write(FLOAT_MEM_AS, f_addr, &f_val.to_le_bytes());
            eprintln!("[FCVT.S.W] x{} ({}) -> f{} ({}=0x{:08x})",
                      pre_compute.rs1, i_val, pre_compute.rd, f_val, f_val.to_bits());
        }
        (1, 1) => { // FCVT.S.WU (unsigned int to float)
            let u_bytes = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, pre_compute.rs1 as u32 * 4);
            let u_val = u32::from_le_bytes(u_bytes);
            let f_val = u_val as f32;
            let f_addr = float_reg_addr(pre_compute.rd);
            exec_state.vm_write(FLOAT_MEM_AS, f_addr, &f_val.to_le_bytes());
            eprintln!("[FCVT.S.WU] x{} ({}) -> f{} ({}=0x{:08x})",
                      pre_compute.rs1, u_val, pre_compute.rd, f_val, f_val.to_bits());
        }
        _ => {}
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
    let pre_compute: &FloatConvertPreCompute = pre_compute.borrow();
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
    let pre_compute: &E2PreCompute<FloatConvertPreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
