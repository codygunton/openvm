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

    // Read the float values being compared for debugging
    let f1_addr = FLOAT_REGISTER_BASE + (pre_compute.rs1 as u32 * 8);
    let f2_addr = FLOAT_REGISTER_BASE + (pre_compute.rs2 as u32 * 8);
    let f1_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f1_addr);
    let f2_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f2_addr);
    let f1_val = u32::from_le_bytes(f1_bytes);
    let f2_val = u32::from_le_bytes(f2_bytes);

    eprintln!("[FLOAT_CMP] PC=0x{:08x}, rd={}, rs1={}(f{}=0x{:08x}), rs2={}(f{}=0x{:08x}), comp_type={}",
              *pc, pre_compute.rd, pre_compute.rs1, pre_compute.rs1, f1_val,
              pre_compute.rs2, pre_compute.rs2, f2_val, pre_compute.comp_type);

    // Reconstruct RISC-V instruction encoding to pass to handler
    // R-type: funct7 | rs2 | rs1 | funct3 | rd | opcode
    // All float comparisons use funct7=0x50 (0b1010000, 7-bit value)
    // FLE.S: funct3=0, FLT.S: funct3=1, FEQ.S: funct3=2
    let funct7 = 0x50;  // Corrected from 0xA0 - funct7 is only 7 bits!
    let funct3 = pre_compute.comp_type; // 0=FLE, 1=FLT, 2=FEQ

    let riscv_inst = (funct7 << 25) | ((pre_compute.rs2 as u32) << 20) |
                     ((pre_compute.rs1 as u32) << 15) | ((funct3 as u32) << 12) |
                     ((pre_compute.rd as u32) << 7) | 0x53; // FP_OPCODE

    eprintln!("[FLOAT_CMP] Reconstructed inst=0x{:08x}, handler will be called", riscv_inst);

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

    // Save caller-saved registers: t0-t2 (x5-x7), a0-a7 (x10-x17), t3-t6 (x28-x31)
    // These may be clobbered by the C handler per RISC-V calling convention
    let saved_regs = [5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17, 28, 29, 30, 31];
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
