use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP, riscv::RV32_REGISTER_AS};
use openvm_stark_backend::p3_field::PrimeField32;

use crate::constants::{
    FLOAT_INST_ADDR, FLOAT_LIB_ENTRY_PTR, FLOAT_MEM_AS, FLOAT_RETURN_ADDR,
    FLOAT_SAVED_REGS_BASE, FLOAT_SAVED_X1, FLOAT_TRAMPOLINE_PC,
};

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
    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // Step 1: Build RISC-V instruction encoding
    // Determine funct7 based on direction and signedness:
    // FCVT.W.S  (float→int signed):     funct7=0x60, rs2=0x00
    // FCVT.WU.S (float→int unsigned):   funct7=0x60, rs2=0x01
    // FCVT.S.W  (int→float signed):     funct7=0x68, rs2=0x00
    // FCVT.S.WU (int→float unsigned):   funct7=0x68, rs2=0x01
    let funct7 = if pre_compute.direction == 0 { 0x60 } else { 0x68 };
    let rs2 = pre_compute.unsigned_flag; // 0 or 1

    let riscv_inst = (funct7 << 25) | ((rs2 as u32) << 20) |
                     ((pre_compute.rs1 as u32) << 15) |
                     ((pre_compute.rm as u32) << 12) |
                     ((pre_compute.rd as u32) << 7) | 0x53;

    // Step 2: Store instruction for handler
    let inst_bytes = riscv_inst.to_le_bytes();
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR, &inst_bytes);
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR + 4, &[0u8; 4]);

    // Step 2.5: For int→float conversion (direction=1), copy source integer value
    // to FLOAT_X0_BACKUP so the handler can read it (handler can't access RV32_REGISTER_AS)
    if pre_compute.direction == 1 {
        let src_int_value = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, pre_compute.rs1 as u32 * 4);
        // Write to same backup location handler uses for int results, indexed by register number
        // FLOAT_X0_BACKUP is base address for integer register backup (8-byte aligned per register)
        let backup_addr = crate::constants::FLOAT_X0_BACKUP + (pre_compute.rs1 as u32 * 8);
        exec_state.vm_write(FLOAT_MEM_AS, backup_addr, &src_int_value);
    }

    // Step 3: Save register context
    let saved_x1 = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, 1 * 4);
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_SAVED_X1, &saved_x1);

    // Save caller-saved registers in indexed format that trampoline expects
    // The trampoline restores: x5-x7, x10-x17, x28-x31 (15 registers) at offsets 0-56
    let saved_regs = [5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17, 28, 29, 30, 31];
    for (i, &reg) in saved_regs.iter().enumerate() {
        let reg_bytes = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, reg * 4);
        exec_state.vm_write(FLOAT_MEM_AS, FLOAT_SAVED_REGS_BASE + (i as u32 * 4), &reg_bytes);
    }

    // Step 4: Setup trampoline return
    let actual_return_addr = *pc + DEFAULT_PC_STEP;
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_RETURN_ADDR, &actual_return_addr.to_le_bytes());
    exec_state.vm_write(RV32_REGISTER_AS, 1 * 4, &FLOAT_TRAMPOLINE_PC.to_le_bytes());

    // Step 5: Jump to handler
    let handler_ptr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_LIB_ENTRY_PTR);
    let handler_addr = u32::from_le_bytes(handler_ptr_bytes);
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
