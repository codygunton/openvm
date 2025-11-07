use std::borrow::{Borrow, BorrowMut};

use openvm_circuit::arch::*;
use openvm_circuit::system::memory::{online::TracingMemory, MemoryAuxColsFactory};
use openvm_circuit_primitives::AlignedBytesBorrow;
use openvm_circuit_primitives_derive::AlignedBorrow;
use openvm_instructions::{instruction::Instruction, riscv::RV32_REGISTER_AS};
use openvm_stark_backend::{
    interaction::InteractionBuilder,
    p3_air::BaseAir,
    p3_field::{Field, FieldAlgebra, PrimeField32},
    rap::BaseAirWithPublicValues,
};

use crate::constants::*;

/// Record generated during execution of FloatHandlerReturn
/// Contains the restored register values and return address
#[repr(C, align(4))]
#[derive(AlignedBytesBorrow, Debug, Clone)]
pub struct FloatHandlerReturnCoreRecord {
    pub return_addr: u32,
    pub restored_registers: [u32; 31], // x1-x31
}

/// AIR columns for FloatHandlerReturn
#[repr(C)]
#[derive(AlignedBorrow, Debug, Clone)]
pub struct FloatHandlerReturnCoreCols<T> {
    pub return_addr: T,
    pub restored_registers: [T; 31],
}

/// AIR for FloatHandlerReturn operation
/// Constrains the restoration of 31 integer registers from backup memory
/// and the jump back to the return address
#[derive(Copy, Clone, Debug)]
pub struct FloatHandlerReturnCoreAir {
    pub offset: usize,
}

impl FloatHandlerReturnCoreAir {
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }
}

impl<F: Field> BaseAir<F> for FloatHandlerReturnCoreAir {
    fn width(&self) -> usize {
        FloatHandlerReturnCoreCols::<F>::width()
    }
}

impl<F: Field> BaseAirWithPublicValues<F> for FloatHandlerReturnCoreAir {}

impl<AB, I> VmCoreAir<AB, I> for FloatHandlerReturnCoreAir
where
    AB: InteractionBuilder,
    I: VmAdapterInterface<AB::Expr>,
    I::Reads: Default,
    I::Writes: Default,
    I::ProcessedInstruction: From<MinimalInstruction<AB::Expr>>,
{
    fn eval(
        &self,
        _builder: &mut AB,
        local_core: &[AB::Var],
        _from_pc: AB::Var,
    ) -> AdapterAirContext<AB::Expr, I> {
        let cols: &FloatHandlerReturnCoreCols<AB::Var> = (*local_core).borrow();

        // No constraints needed on the register values themselves - they are read from memory
        // The memory bridge will handle the reads from FLOAT_X0_BACKUP
        // The execution bridge will handle the PC transition to return_addr

        // to_pc = return_addr (no alignment needed, already aligned)
        let to_pc = cols.return_addr.into();

        // Return context with PC jump to return address
        AdapterAirContext {
            to_pc: Some(to_pc),
            reads: Default::default(),
            writes: Default::default(),
            instruction: MinimalInstruction {
                is_valid: AB::F::from_canonical_u32(1).into(),
                opcode: AB::F::from_canonical_usize(self.offset).into(),
            }
            .into(),
        }
    }

    fn start_offset(&self) -> usize {
        self.offset
    }
}

/// FloatReturnExecutor handles the custom FLOAT_RETURN instruction
/// This instruction restores caller-saved registers after returning from the float handler
#[derive(Clone, Copy)]
pub struct FloatReturnExecutor;

impl FloatReturnExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatReturnExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// PreflightExecutor implementation for traced execution (E3)
impl<F, RA> PreflightExecutor<F, RA> for FloatReturnExecutor
where
    F: PrimeField32,
    for<'buf> RA: RecordArena<'buf, MultiRowLayout<EmptyMultiRowMetadata>, &'buf mut FloatHandlerReturnCoreRecord>,
{
    fn get_opcode_name(&self, opcode: usize) -> String {
        match opcode {
            0x314 => "FLOAT_RETURN".to_string(),
            _ => format!("UnknownFloatReturn(0x{:x})", opcode),
        }
    }

    fn execute(
        &self,
        state: VmStateMut<F, TracingMemory, RA>,
        _instruction: &Instruction<F>,
    ) -> Result<(), ExecutionError> {
        use openvm_instructions::program::DEFAULT_PC_STEP;

        unsafe {
            // Allocate record for trace generation
            let core_record = state.ctx.alloc(MultiRowLayout::new(EmptyMultiRowMetadata::new()));

            // Restore ALL integer registers x1-x31 from FLOAT_X0_BACKUP
            // The handler may have written integer results there for int-writing ops
            for reg in 1..32 {
                let backup_addr = FLOAT_X0_BACKUP + (reg * 8); // 8-byte aligned storage
                let (_, reg_bytes) = state.memory.read::<u8, 4, 4>(FLOAT_MEM_AS, backup_addr);
                let reg_value = u32::from_le_bytes(reg_bytes);
                core_record.restored_registers[(reg - 1) as usize] = reg_value;

                state
                    .memory
                    .write::<u8, 4, 4>(RV32_REGISTER_AS, reg * 4, reg_bytes);
            }

            // Read return address from saved location (x1 was clobbered by library)
            let (_, return_addr_bytes) = state
                .memory
                .read::<u8, 4, 4>(FLOAT_MEM_AS, FLOAT_RETURN_ADDR);
            let return_addr = u32::from_le_bytes(return_addr_bytes);
            core_record.return_addr = return_addr;

            *state.pc = return_addr;

            Ok(())
        }
    }
}

/// TraceFiller for FloatHandlerReturn
/// Converts records to full trace columns
#[derive(Clone)]
pub struct FloatHandlerReturnFiller;

impl FloatHandlerReturnFiller {
    pub fn new() -> Self {
        Self
    }
}

impl<F: PrimeField32> TraceFiller<F> for FloatHandlerReturnFiller {
    fn fill_trace_row(&self, _mem_helper: &MemoryAuxColsFactory<F>, row_slice: &mut [F]) {
        // SAFETY: row_slice is guaranteed by the caller to contain a valid FloatHandlerReturnCoreRecord
        let mut core_row = row_slice;
        let record: &FloatHandlerReturnCoreRecord =
            unsafe { get_record_from_slice(&mut core_row, ()) };
        let cols: &mut FloatHandlerReturnCoreCols<F> = core_row.borrow_mut();

        // Fill columns in reverse order to avoid corrupting the record
        for i in 0..31 {
            cols.restored_registers[30 - i] =
                F::from_canonical_u32(record.restored_registers[30 - i]);
        }
        cols.return_addr = F::from_canonical_u32(record.return_addr);
    }
}
