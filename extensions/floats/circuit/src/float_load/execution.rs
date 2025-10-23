use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP, riscv::RV32_REGISTER_NUM_LIMBS};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;
use super::core::FloatLoadExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
struct FloatLoadPreCompute {
    rd: u8,           // Float destination register (0-31)
    rs1: u8,          // Base address register
    _padding: [u8; 2],
    imm: i32,         // Signed offset
}

impl FloatLoadExecutor {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut FloatLoadPreCompute,
    ) -> Result<bool, StaticProgramError> {
        // Note: inst.b contains rs1 * RV32_REGISTER_NUM_LIMBS (from transpiler)
        // Reconstruct signed immediate from fields c and g (same pattern as rv32im):
        // - Field c contains lower 16 bits
        // - Field g contains sign bit
        let imm_lower = inst.c.as_canonical_u32();
        let imm_sign = inst.g.as_canonical_u32();
        let imm = imm_lower + imm_sign * 0xffff0000;

        *data = FloatLoadPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
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

impl<F> Executor<F> for FloatLoadExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatLoadPreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatLoadPreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatLoadExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatLoadPreCompute>>()
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
        let data: &mut E2PreCompute<FloatLoadPreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &FloatLoadPreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    eprintln!("[FLW-ENTRY] PC=0x{:08x}, instret={}, ENABLED={}", *pc, *instret, ENABLED);

    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // 1. Read base address from rs1 register (use register number directly like JALR does)
    let base_bytes = exec_state.vm_read::<u8, 4>(
        RV32_REGISTER_AS,
        pre_compute.rs1 as u32,
    );
    let base_addr = u32::from_le_bytes(base_bytes);

    eprintln!("[FLW] PC=0x{:08x}, Reading x{} = 0x{:08x}, imm={}",
              *pc, pre_compute.rs1 / RV32_REGISTER_NUM_LIMBS as u8, base_addr, pre_compute.imm);

    // 2. Calculate effective address
    let addr = base_addr.wrapping_add(pre_compute.imm as u32);

    // 3. Load word from heap memory
    let word_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, addr);

    // DEBUG: Show what's being loaded
    let value = u32::from_le_bytes(word_bytes);
    let value_f32 = f32::from_bits(value);
    eprintln!("[FLW] f{} <- mem[0x{:08x}] = 0x{:08x} ({})",
              pre_compute.rd, addr, value, value_f32);

    // 4. Write to float register memory
    let float_addr = float_reg_addr(pre_compute.rd);
    exec_state.vm_write(FLOAT_MEM_AS, float_addr, &word_bytes);

    // 5. Update PC and instruction counter
    let old_pc = *pc;
    *pc += DEFAULT_PC_STEP;
    *instret += 1;
    eprintln!("[FLW-EXIT] PC: 0x{:08x} -> 0x{:08x}, instret={}", old_pc, *pc, *instret);
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
    let pre_compute: &FloatLoadPreCompute = pre_compute.borrow();
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
    let pre_compute: &E2PreCompute<FloatLoadPreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
