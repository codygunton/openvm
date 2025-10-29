use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP, riscv::RV32_REGISTER_AS};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::*;

use super::core::FloatFmaExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
pub struct FloatFmaPreCompute {
    rd: u8,      // Destination float register
    rs1: u8,     // Source float register 1
    rs2: u8,     // Source float register 2
    rs3: u8,     // Source float register 3
    rm: u8,      // Rounding mode
    opcode: u8,  // Which operation (FMADD=9, FMSUB=10, FNMSUB=11, FNMADD=12)
}

impl FloatFmaExecutor {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut FloatFmaPreCompute,
    ) -> Result<bool, StaticProgramError> {
        *data = FloatFmaPreCompute {
            rd: inst.a.as_canonical_u32() as u8,
            rs1: inst.b.as_canonical_u32() as u8,
            rs2: inst.c.as_canonical_u32() as u8,
            rs3: inst.d.as_canonical_u32() as u8,
            rm: inst.e.as_canonical_u32() as u8,
            opcode: inst.opcode.local_opcode_idx(0x300) as u8,
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

impl<F> Executor<F> for FloatFmaExecutor
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<FloatFmaPreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut FloatFmaPreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F> MeteredExecutor<F> for FloatFmaExecutor
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<FloatFmaPreCompute>>()
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
        let data: &mut E2PreCompute<FloatFmaPreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &FloatFmaPreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    eprintln!(
        "[FMA-ENTRY] PC=0x{:08x}, instret={}, ENABLED={}",
        *pc, *instret, ENABLED
    );

    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // Reconstruct RISC-V R4-type instruction encoding
    // R4-type: rs3[31:27] | funct2[26:25] | rs2[24:20] | rs1[19:15] | rm[14:12] | rd[11:7] | opcode[6:0]
    let opcode = match pre_compute.opcode {
        9 => 0x43,  // FMADD.S
        10 => 0x47, // FMSUB.S
        11 => 0x4B, // FNMSUB.S
        12 => 0x4F, // FNMADD.S
        _ => 0x43,  // Default to FMADD.S
    };

    let riscv_inst = ((pre_compute.rs3 as u32) << 27) | (0 << 25) | // funct2=0 for single precision
                     ((pre_compute.rs2 as u32) << 20) |
                     ((pre_compute.rs1 as u32) << 15) |
                     ((pre_compute.rm as u32) << 12) |
                     ((pre_compute.rd as u32) << 7) |
                     opcode;

    // Store instruction to FLOAT_INST_ADDR (0x1F001108)
    let inst_bytes = riscv_inst.to_le_bytes();
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR, &inst_bytes);

    // Store 0 to FLOAT_INST_ADDR + 4
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR + 4, &[0u8; 4]);

    // Load handler address
    let handler_ptr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_LIB_ENTRY_PTR);
    let handler_addr = u32::from_le_bytes(handler_ptr_bytes);

    // Save x1 before clobbering it with return address
    // The test code may be using x1 to store important values (e.g., signature base address)
    let saved_x1 = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, 1 * 4);
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_SAVED_X1, &saved_x1);

    // Save all caller-saved registers (x5-x7, x10-x17, x28-x31) before calling C handler
    // The C float handler follows RISC-V calling convention and may clobber these
    let caller_saved_regs = [5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17, 28, 29, 30, 31];
    for (i, &reg) in caller_saved_regs.iter().enumerate() {
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
    let pre_compute: &FloatFmaPreCompute = pre_compute.borrow();
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
    let pre_compute: &E2PreCompute<FloatFmaPreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
