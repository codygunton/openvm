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

    // Reconstruct RISC-V instruction encoding for FCLASS.S
    // R-type: funct7 | rs2 | rs1 | funct3 | rd | opcode
    // FCLASS.S: funct7=0x70, rs2=0, funct3=1
    let funct7 = 0x70;
    let rs2 = 0;
    let funct3 = 1;

    let riscv_inst = (funct7 << 25) | ((rs2 as u32) << 20) |
                     ((pre_compute.rs1 as u32) << 15) | ((funct3 as u32) << 12) |
                     ((pre_compute.rd as u32) << 7) | 0x53; // FP_OPCODE

    // Store instruction to FLOAT_INST_ADDR (0x1F001108)
    let inst_bytes = riscv_inst.to_le_bytes();
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR, &inst_bytes);

    // Store 0 to FLOAT_INST_ADDR + 4
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR + 4, &[0u8; 4]);

    // Load handler address from FLOAT_LIB_ENTRY_PTR (0x0001EC60)
    let handler_ptr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_LIB_ENTRY_PTR);
    let handler_addr = u32::from_le_bytes(handler_ptr_bytes);

    eprintln!(
        "[FCLASS] Calling handler at 0x{:08x}, instruction=0x{:08x}",
        handler_addr, riscv_inst
    );

    // Store return address in x1 (ra)
    let return_addr = *pc + DEFAULT_PC_STEP;
    exec_state.vm_write(RV32_REGISTER_AS, 1 * 4, &return_addr.to_le_bytes()); // x1 = ra

    // JALR: jump to handler (with RISC-V compliant address rounding - clear LSB)
    let target_addr = handler_addr & !1;
    *pc = target_addr;
    *instret += 1;

    eprintln!(
        "[FCLASS-EXIT] Jumped to handler: 0x{:08x}, return_addr=0x{:08x}",
        handler_addr, return_addr
    );
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
