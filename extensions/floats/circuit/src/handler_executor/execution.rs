use std::{marker::PhantomData, mem::size_of};

use openvm_circuit::{
    arch::*,
    system::memory::online::{GuestMemory, TracingMemory},
};
use openvm_instructions::{
    instruction::Instruction, program::DEFAULT_PC_STEP, riscv::RV32_REGISTER_AS,
};
use openvm_stark_backend::p3_field::PrimeField32;

use super::operation::FloatOperation;
use crate::constants::*;

/// Generic executor for float operations that call the native handler.
/// Type parameter OP determines which operation this executor handles.
#[derive(Clone, Copy)]
pub struct FloatHandlerExecutor<OP: FloatOperation> {
    _phantom: PhantomData<OP>,
}

impl<OP: FloatOperation> FloatHandlerExecutor<OP> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn pre_compute_impl<F: PrimeField32>(
        _pc: u32,
        inst: &Instruction<F>,
        data: &mut OP::PreCompute,
    ) -> Result<bool, StaticProgramError> {
        OP::extract_fields(inst, data)
    }
}

impl<OP: FloatOperation> Default for FloatHandlerExecutor<OP> {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! dispatch {
    ($execute_impl:ident, $enabled:ident) => {
        if $enabled {
            Ok($execute_impl::<_, _, OP, true>)
        } else {
            Ok($execute_impl::<_, _, OP, false>)
        }
    };
}

impl<F, OP> Executor<F> for FloatHandlerExecutor<OP>
where
    F: PrimeField32,
    OP: FloatOperation,
{
    #[inline(always)]
    fn pre_compute_size(&self) -> usize {
        size_of::<OP::PreCompute>()
    }

    #[inline(always)]
    fn pre_compute<Ctx: ExecutionCtxTrait>(
        &self,
        pc: u32,
        inst: &Instruction<F>,
        data: &mut [u8],
    ) -> Result<ExecuteFunc<F, Ctx>, StaticProgramError> {
        let data = unsafe { &mut *(data.as_mut_ptr() as *mut OP::PreCompute) };
        let enabled = Self::pre_compute_impl(pc, inst, data)?;
        dispatch!(execute_e1_handler, enabled)
    }
}

impl<F, RA, OP> PreflightExecutor<F, RA> for FloatHandlerExecutor<OP>
where
    F: PrimeField32,
    OP: FloatOperation,
    RA: Arena,
{
    fn get_opcode_name(&self, _opcode: usize) -> String {
        "FloatHandlerOp".to_string()
    }

    fn execute(
        &self,
        _state: VmStateMut<F, TracingMemory, RA>,
        _instruction: &Instruction<F>,
    ) -> Result<(), ExecutionError> {
        panic!("Float handler operations should use Executor trait, not PreflightExecutor");
    }
}

impl<F, OP> MeteredExecutor<F> for FloatHandlerExecutor<OP>
where
    F: PrimeField32,
    OP: FloatOperation,
{
    fn metered_pre_compute_size(&self) -> usize {
        size_of::<E2PreCompute<OP::PreCompute>>()
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
        let data = unsafe { &mut *(data.as_mut_ptr() as *mut E2PreCompute<OP::PreCompute>) };
        data.chip_idx = chip_idx as u32;
        let enabled = Self::pre_compute_impl(pc, inst, &mut data.data)?;
        dispatch!(execute_e2_handler, enabled)
    }
}

#[inline(always)]
unsafe fn execute_e12_impl<
    F: PrimeField32,
    CTX: ExecutionCtxTrait,
    OP: FloatOperation,
    const ENABLED: bool,
>(
    pre_compute: &OP::PreCompute,
    instret: &mut u64,
    pc: &mut u32,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    if !ENABLED {
        *pc += DEFAULT_PC_STEP;
        *instret += 1;
        return;
    }

    // Reconstruct RISC-V instruction
    let riscv_inst = OP::reconstruct_riscv_instruction(pre_compute);

    // Store instruction to FLOAT_INST_ADDR for handler to read
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR, &riscv_inst.to_le_bytes());
    exec_state.vm_write(FLOAT_MEM_AS, FLOAT_INST_ADDR + 4, &[0u8; 4]);

    // Load handler entry point
    let handler_ptr_bytes = exec_state.vm_read::<u8, 4>(FLOAT_MEM_AS, FLOAT_LIB_ENTRY_PTR);
    let handler_addr = u32::from_le_bytes(handler_ptr_bytes);

    // Save ALL integer registers x1-x31 to FLOAT_X0_BACKUP
    // Zisk's restore code expects to find them there
    for reg in 1..32 {
        let reg_bytes = exec_state.vm_read::<u8, 4>(RV32_REGISTER_AS, reg * 4);
        let backup_addr = FLOAT_X0_BACKUP + (reg * 8); // 8-byte aligned storage
        exec_state.vm_write(FLOAT_MEM_AS, backup_addr, &reg_bytes);
    }

    // Prepare operation-specific data (e.g., copy int→float for FCVT.S.W)
    OP::prepare_for_handler(pre_compute, exec_state);

    // Save actual return address to memory (library will clobber x1)
    let actual_return_addr = *pc + DEFAULT_PC_STEP;
    exec_state.vm_write(
        FLOAT_MEM_AS,
        FLOAT_RETURN_ADDR,
        &actual_return_addr.to_le_bytes(),
    );

    // Write return address to x1 as well (for library to use if needed)
    exec_state.vm_write(RV32_REGISTER_AS, 1 * 4, &actual_return_addr.to_le_bytes());

    // Jump to handler (clear LSB for alignment)
    let target_addr = handler_addr & !1;
    *pc = target_addr;
    *instret += 1;
}

#[create_handler]
#[inline(always)]
unsafe fn execute_e1_impl<
    F: PrimeField32,
    CTX: ExecutionCtxTrait,
    OP: FloatOperation,
    const ENABLED: bool,
>(
    pre_compute: &[u8],
    instret: &mut u64,
    pc: &mut u32,
    _instret_end: u64,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let pre_compute = unsafe { &*(pre_compute.as_ptr() as *const OP::PreCompute) };
    execute_e12_impl::<F, CTX, OP, ENABLED>(pre_compute, instret, pc, exec_state);
}

#[create_handler]
#[inline(always)]
unsafe fn execute_e2_impl<
    F: PrimeField32,
    CTX: MeteredExecutionCtxTrait,
    OP: FloatOperation,
    const ENABLED: bool,
>(
    pre_compute: &[u8],
    instret: &mut u64,
    pc: &mut u32,
    _arg: u64,
    exec_state: &mut VmExecState<F, GuestMemory, CTX>,
) {
    let pre_compute = unsafe { &*(pre_compute.as_ptr() as *const E2PreCompute<OP::PreCompute>) };
    exec_state
        .ctx
        .on_height_change(pre_compute.chip_idx as usize, 1);
    execute_e12_impl::<F, CTX, OP, ENABLED>(&pre_compute.data, instret, pc, exec_state);
}
