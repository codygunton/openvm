use std::{
    borrow::{Borrow, BorrowMut},
    mem::size_of,
};

use openvm_circuit::{arch::*, system::memory::online::GuestMemory};
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{
    instruction::Instruction,
    program::{DEFAULT_PC_STEP, PC_BITS},
    riscv::RV32_REGISTER_AS,
};
use openvm_stark_backend::p3_field::PrimeField32;

use super::core::Rv32JalrExecutor;

#[derive(AlignedBytesBorrow, Clone)]
#[repr(C)]
struct JalrPreCompute {
    imm_extended: u32,
    a: u8,
    b: u8,
}

impl<A> Rv32JalrExecutor<A> {
    /// Return true if enabled.
    fn pre_compute_impl<F: PrimeField32>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut JalrPreCompute,
    ) -> Result<bool, StaticProgramError> {
        let imm_extended = inst.c.as_canonical_u32() + inst.g.as_canonical_u32() * 0xffff0000;
        if inst.d.as_canonical_u32() != RV32_REGISTER_AS {
            return Err(StaticProgramError::InvalidInstruction(pc));
        }
        *data = JalrPreCompute {
            imm_extended,
            a: inst.a.as_canonical_u32() as u8,
            b: inst.b.as_canonical_u32() as u8,
        };
        let enabled = !inst.f.is_zero();
        Ok(enabled)
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

impl<F, A> Executor<F> for Rv32JalrExecutor<A>
where
    F: PrimeField32,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<JalrPreCompute>()
    }
    #[cfg(not(feature = "tco"))]
    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data: &mut JalrPreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }

    #[cfg(feature = "tco")]
    fn handler<Ctx>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<Handler<F, Ctx>, StaticProgramError>
    where
        Ctx: ExecutionCtxTrait,
    {
        let data: &mut JalrPreCompute = data.borrow_mut();
        let enabled = self.pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F, A> MeteredExecutor<F> for Rv32JalrExecutor<A>
where
    F: PrimeField32,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<JalrPreCompute>>()
    }

    #[cfg(not(feature = "tco"))]
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
        let data: &mut E2PreCompute<JalrPreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }

    #[cfg(feature = "tco")]
    fn metered_handler<Ctx>(
        &self,
        chip_idx: usize,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<Handler<F, Ctx>, StaticProgramError>
    where
        Ctx: MeteredExecutionCtxTrait,
    {
        let data: &mut E2PreCompute<JalrPreCompute> = data.borrow_mut();
        data.chip_idx = chip_idx as u32;
        let enabled = self.pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<F: PrimeField32, CTX: ExecutionCtxTrait, const ENABLED: bool>(
    pre_compute: &JalrPreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let rs1 = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, pre_compute.b as u32);
    let rs1 = u32::from_le_bytes(rs1);
    let to_pc = rs1.wrapping_add(pre_compute.imm_extended);
    let to_pc = to_pc - (to_pc & 1);

    // Check if we're jumping to the float trampoline BEFORE the assertion
    // (trampoline PC is > PC_BITS and would fail the assertion)
    const FLOAT_TRAMPOLINE_PC: u32 = 0xF0000000;
    const FLOAT_MEM_AS: u32 = 2;
    const FLOAT_SAVED_X1: u32 = 0x1F001200;
    const FLOAT_RETURN_ADDR: u32 = 0x1F001204;
    const FLOAT_SAVED_REGS_BASE: u32 = 0x1F001210;  // Save area for caller-saved registers
    const FLOAT_INST_ADDR: u32 = 0x00201108;   // FREG_FIRST (0x00201000) + 33*8 = 0x108 offset
    const FLOAT_X0_BACKUP: u32 = 0x00201118;   // FREG_FIRST (0x00201000) + 35*8 = 0x118 offset

    if to_pc == FLOAT_TRAMPOLINE_PC {
        // Restore x1 from saved location
        let saved_x1 = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_SAVED_X1);
        exec_state.vm_write(RV32_REGISTER_AS, 1 * 4, &saved_x1);

        // Read the float instruction to check if it writes to an integer register
        let inst_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_INST_ADDR);
        let inst = u32::from_le_bytes(inst_bytes);

        // Extract fields from instruction
        let opcode = inst & 0x7F;
        let rd = (inst >> 7) & 0x1F;
        let funct7 = (inst >> 25) & 0x7F;

        eprintln!("[TRAMPOLINE] inst=0x{:08x}, opcode=0x{:02x}, rd={}, funct7=0x{:02x}", inst, opcode, rd, funct7);

        // Check if this operation will write to an integer register
        // Float operations that write to integer registers:
        // - funct7=0x50: FLE.S, FLT.S, FEQ.S (comparisons)
        // - funct7=0x60: FCVT.W.S, FCVT.WU.S (float→int conversion)
        // - funct7=0x70: FCLASS.S, FMV.X.W (classification, move)
        let writes_to_int_reg = opcode == 0x53 &&
            (funct7 == 0x50 || funct7 == 0x60 || funct7 == 0x70) && rd != 0;

        // Restore caller-saved registers: t0-t2 (x5-x7), a0-a7 (x10-x17), t3-t6 (x28-x31)
        // IMPORTANT: Skip restoring rd if it will receive an integer result,
        // otherwise we'd write twice (restore old value, then copy new result)
        let saved_regs = [5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17, 28, 29, 30, 31];
        for (i, &reg) in saved_regs.iter().enumerate() {
            // Skip restoring this register if it's the destination of an int result operation
            if writes_to_int_reg && reg as u32 == rd {
                eprintln!("[TRAMPOLINE] Skipping restore of x{} (will receive int result)", reg);
                continue;
            }
            let reg_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_SAVED_REGS_BASE + (i as u32 * 4));
            exec_state.vm_write(RV32_REGISTER_AS, reg as u32 * 4, &reg_bytes);
        }


        // Copy integer result if this instruction writes to an integer register:
        // Only R-type float operations (opcode 0x53) can write to integer registers
        // FMA operations (opcodes 0x43, 0x47, 0x4B, 0x4F) always write to float registers
        // - Float comparisons (FLE, FLT, FEQ): opcode=0x53, funct7=0x50
        // - FCLASS: opcode=0x53, funct7=0x70 (funct3=1)
        // - FMV.X.W: opcode=0x53, funct7=0x70 (funct3=0)
        if writes_to_int_reg {
            // Copy result from integer register backup to actual integer register file
            // Handler writes to FLOAT_X0_BACKUP + (rd * 8) as 8-byte aligned storage
            let backup_addr = FLOAT_X0_BACKUP + (rd * 8);
            let result_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, backup_addr);
            let result = u32::from_le_bytes(result_bytes);

            // Also read the upper 32 bits to see what the handler wrote
            let upper_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, backup_addr + 4);
            let upper = u32::from_le_bytes(upper_bytes);

            eprintln!("[TRAMPOLINE] Copying int result: x{} = 0x{:08x} (from backup addr 0x{:08x}, upper=0x{:08x})",
                      rd, result, backup_addr, upper);
            exec_state.vm_write(RV32_REGISTER_AS, rd * 4, &result_bytes);
        }

        // Read actual return address
        let actual_return = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_RETURN_ADDR);
        let actual_return_pc = u32::from_le_bytes(actual_return);

        // Note: The handler returns via 'ret' which is 'jalr x0, 0(x1)', so pre_compute.a is 0
        // We skip writing to x0 since it's the zero register and should never be modified
        // (The earlier check for comparison results already handles not writing to x0)

        *pc = actual_return_pc;
        *instret += 1;
        return;
    }

    debug_assert!(to_pc < (1 << PC_BITS));
    let rd = (*pc + DEFAULT_PC_STEP).to_le_bytes();

    if ENABLED {
        exec_state.vm_write(RV32_REGISTER_AS, pre_compute.a as u32, &rd);
    }

    *pc = to_pc;
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
    let pre_compute: &JalrPreCompute = pre_compute.borrow();
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
    let pre_compute: &E2PreCompute<JalrPreCompute> = pre_compute.borrow();
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
