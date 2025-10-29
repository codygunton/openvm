use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP, riscv::RV32_REGISTER_AS};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;

use super::core::FloatAluExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct FloatAluPreCompute {
    rd: u8,      // Destination float register
    rs1: u8,     // Source float register 1
    rs2: u8,     // Source float register 2
    opcode: u8,  // Which operation (FADD=2, FSUB=3, FMUL=4, FDIV=5, FSQRT=6, FMIN/FMAX=7, FSGNJ*=8)
    variant: u8, // funct3 for operations that need it (FMIN/FMAX: 0/1, FSGNJ*: 0/1/2)
}

impl FloatAluExecutor {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut FloatAluPreCompute,
    ) -> Result<bool, StaticProgramError> {
        *data = FloatAluPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            rs2: inst.c.as_canonical_u32() as u8,
            opcode: inst.d.as_canonical_u32() as u8,
            variant: inst.e.as_canonical_u32() as u8,
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

impl<F> Executor<F> for FloatAluExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatAluPreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatAluPreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatAluExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatAluPreCompute>>()
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
        let data: &mut E2PreCompute<FloatAluPreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &FloatAluPreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // Reconstruct RISC-V instruction encoding
    // R-type: funct7 | rs2 | rs1 | funct3 | rd | opcode
    let (funct7, funct3) = match pre_compute.opcode {
        2 => (0x00, pre_compute.variant), // FADD - preserve rounding mode
        3 => (0x04, pre_compute.variant), // FSUB - preserve rounding mode
        4 => (0x08, pre_compute.variant), // FMUL - preserve rounding mode
        5 => (0x0C, pre_compute.variant), // FDIV - preserve rounding mode
        6 => (0x2C, pre_compute.variant), // FSQRT - preserve rounding mode
        7 => (0x14, pre_compute.variant), // FMIN (variant=0) / FMAX (variant=1)
        8 => (0x10, pre_compute.variant), // FSGNJ (0) / FSGNJN (1) / FSGNJX (2)
        _ => (0x00, 0),
    };

    // FSQRT uses rs2=0, all others use pre_compute.rs2
    let rs2_val = if pre_compute.opcode == 6 { 0 } else { pre_compute.rs2 };

    let riscv_inst = (funct7 << 25) | ((rs2_val as u32) << 20) |
                     ((pre_compute.rs1 as u32) << 15) | ((funct3 as u32) << 12) |
                     ((pre_compute.rd as u32) << 7) | 0x53; // FP_OPCODE

    // Store instruction to FLOAT_INST_ADDR
    let inst_bytes = riscv_inst.to_le_bytes();
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR, &inst_bytes);
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR + 4, &[0u8; 4]);

    // Load handler address
    let handler_ptr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_LIB_ENTRY_PTR);
    let handler_addr = u32::from_le_bytes(handler_ptr_bytes);

    // Save x1 before clobbering it with return address
    // The test code may be using x1 to store important values (e.g., signature base address)
    let saved_x1 = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, 1 * 4);
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_SAVED_X1, &saved_x1);

    // Save temporary registers plus x14 (a4) which tests use for data pointers
    // t0-t2 (x5-x7), a4 (x14), t3-t6 (x28-x31)
    let saved_regs = [5, 6, 7, 14, 28, 29, 30, 31];
    for (i, &reg) in saved_regs.iter().enumerate() {
        let reg_bytes = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, reg * 4);
        exec_state.vm_write(FLOAT_MEM_AS, FLOAT_SAVED_REGS_BASE + (i as u32 * 4), &reg_bytes);
    }

    // Save actual return address and write trampoline to x1
    let actual_return_addr = *pc + DEFAULT_PC_STEP;
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_RETURN_ADDR, &actual_return_addr.to_le_bytes());
    exec_state.vm_write(RV32_REGISTER_AS, 1 * 4, &FLOAT_TRAMPOLINE_PC.to_le_bytes());

    // Jump to handler
    let target_addr = handler_addr & !1;
    *pc = target_addr;
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
    let pre_compute: &FloatAluPreCompute = pre_compute.borrow();
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
    let pre_compute: &E2PreCompute<FloatAluPreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
