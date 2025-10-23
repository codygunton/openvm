use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::online::GuestMemory;
use openvm_circuit_primitives_derive::AlignedBytesBorrow;
use openvm_instructions::{instruction::Instruction, program::DEFAULT_PC_STEP};
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

    // Load handler address from FLOAT_LIB_ENTRY_PTR (0x0001EC60)
    let handler_ptr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_LIB_ENTRY_PTR);
    let handler_addr = u32::from_le_bytes(handler_ptr_bytes);

    // Read the float register values to debug
    let f_rs1_addr = FLOAT_REGISTER_BASE + (pre_compute.rs1 as u32) * 4;
    let f_rs2_addr = FLOAT_REGISTER_BASE + (pre_compute.rs2 as u32) * 4;
    let f_rs3_addr = FLOAT_REGISTER_BASE + (pre_compute.rs3 as u32) * 4;
    let f_rd_addr = FLOAT_REGISTER_BASE + (pre_compute.rd as u32) * 4;

    let rs1_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f_rs1_addr);
    let rs2_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f_rs2_addr);
    let rs3_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, f_rs3_addr);

    let rs1_val = u32::from_le_bytes(rs1_bytes);
    let rs2_val = u32::from_le_bytes(rs2_bytes);
    let rs3_val = u32::from_le_bytes(rs3_bytes);

    eprintln!(
        "[FMA] Calling handler at 0x{:08x}, instruction=0x{:08x}",
        handler_addr, riscv_inst
    );
    eprintln!(
        "[FMA] f{}=0x{:08x}, f{}=0x{:08x}, f{}=0x{:08x} -> f{}",
        pre_compute.rs1, rs1_val,
        pre_compute.rs2, rs2_val,
        pre_compute.rs3, rs3_val,
        pre_compute.rd
    );

    // Store return address in x1 (ra)
    let return_addr = *pc + DEFAULT_PC_STEP;
    exec_state.vm_write(RV32_REGISTER_AS, 1 * 4, &return_addr.to_le_bytes()); // x1 = ra

    // JALR: jump to handler (with RISC-V compliant address rounding - clear LSB)
    let target_addr = handler_addr & !1;
    *pc = target_addr;
    *instret += 1;

    eprintln!(
        "[FMA-EXIT] Jumped to handler: 0x{:08x}, return_addr=0x{:08x}",
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
